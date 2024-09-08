use chrono::{Duration, Utc};
use postgres_types::ToSql;
use tracing::info;

use crate::{
    cache::{EntityCache, ServiceCache},
    config::Database,
    entity::{Provider, SharedUser},
    error::{AppResult, UserError},
    repositories::UserRepository,
    structs::{TokenPurpose, UserRegistrationData, UserUpdateData},
    utils::{slice_iter, IdsGenerator},
};

use super::TokenService;

/// Manages user account operations, integrating database operations with caching.
pub struct UserService {
    cache: &'static ServiceCache,
    db: &'static Database,
    user_repo: &'static UserRepository,
    token_svc: &'static TokenService,
    ids_generator: IdsGenerator,
}

impl UserService {
    /// Constructs a new UserService instance.
    ///
    /// # Arguments
    /// - `db`: Database connection.
    /// - `cache`: Redis cache.
    /// - `user_repo`: User repository for database operations.
    /// - `token_svc`: Token service for authentication.
    ///
    /// # Returns
    /// A new UserService instance.
    pub async fn new(
        db: &'static Database,
        cache: &'static ServiceCache,
        user_repo: &'static UserRepository,
        token_svc: &'static TokenService,
    ) -> Self {
        let ids_generator = {
            let used_ids = user_repo.used_ids().await.unwrap();
            IdsGenerator::new(600000000..699999999, used_ids)
        };

        Self {
            cache,
            db,
            token_svc,
            user_repo,
            ids_generator,
        }
    }

    /// Creates a new user account.
    ///
    /// # Arguments
    /// - `register`: User registration data.
    ///
    /// # Returns
    /// The ID of the newly created user.
    pub async fn create(&self, registration_data: UserRegistrationData) -> AppResult<i32> {
        if self.user_repo.user_exists(&registration_data.email).await? {
            return Err(UserError::AlreadyExists)?;
        }

        let id = self.ids_generator.next();

        let hashed_password = if let Some(pwd) = registration_data.password {
            Some(self.user_repo.hash_password(pwd).await?)
        } else {
            None
        };

        let avatar = registration_data.avatar.unwrap_or_else(|| {
            format!(
                "https://ui-avatars.com/api/?name={}",
                &registration_data.username
            )
        });

        let provider = registration_data.provider.unwrap_or(Provider::Local);
        let active = provider != Provider::Local;

        let conn = self.db.pg.get().await?;
        let create_user_stmt = conn.prepare_cached(
            r#"
                INSERT INTO users (id, email, username, password, avatar, provider, discord_id, active)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        ).await?;

        conn.execute_raw(
            &create_user_stmt,
            slice_iter(&[
                &id,
                &registration_data.email,
                &registration_data.username,
                &hashed_password,
                &avatar,
                &provider,
                &registration_data.discord_id,
                &active,
            ]),
        )
        .await?;

        Ok(id)
    }

    /// Updates an existing user account.
    ///
    /// # Arguments
    /// - `user`: Current user data.
    /// - `form`: Updated user data.
    ///
    /// # Returns
    /// Result indicating success or failure.
    pub async fn update(&self, user: SharedUser, form: &UserUpdateData) -> AppResult<()> {
        if let Some(last_update) = user.updated_at {
            if Utc::now().signed_duration_since(last_update) <= Duration::days(7) {
                return Err(UserError::UpdateLimitExceeded)?;
            }
        }

        let (query, params) = {
            let mut params_counter = 1u8;
            let mut clauses = Vec::with_capacity(2);
            let mut params: Vec<&(dyn ToSql + Sync)> = Vec::with_capacity(3);

            if let Some(username) = &form.username {
                clauses.push(format!("username = ${}", params_counter));
                params.push(username);
                params_counter += 1;
            }

            if let Some(avatar) = &form.avatar {
                clauses.push(format!("avatar = ${}", params_counter));
                params.push(avatar);
                params_counter += 1;
            }

            if clauses.is_empty() {
                return Err(UserError::InvalidUpdate)?;
            }

            clauses.push("updated_at = CURRENT_TIMESTAMP".to_owned());

            let set_clause = clauses.join(", ");
            let query = format!(
                "UPDATE users SET {} WHERE id = ${}",
                set_clause, params_counter
            );
            params.push(&user.id);

            (query, params)
        };

        let conn = self.db.pg.get().await?;
        conn.execute_raw(&query, slice_iter(&params[..])).await?;
        self.cache.user.delete(user.id);

        Ok(())
    }

    /// Deletes a user account.
    ///
    /// # Arguments
    /// - `id`: User ID to delete.
    ///
    /// # Returns
    /// Result indicating success or failure.
    // TODO: Create a column "deleted" in users table and update it instead of delete
    pub async fn delete(&self, id: i32) -> AppResult<()> {
        let conn = self.db.pg.get().await?;

        let delete_users_relations_stmt_fut = conn.prepare_cached(
            r#"
                DELETE FROM championship_users
                WHERE user_id = $1
            "#,
        );

        let delete_user_stmt_fut = conn.prepare_cached(
            r#"
                DELETE FROM users
                WHERE id = $1
            "#,
        );

        let (delete_users_relations_stmt, delete_user_stmt) =
            tokio::try_join!(delete_users_relations_stmt_fut, delete_user_stmt_fut)?;

        conn.execute_raw(&delete_users_relations_stmt, &[&id])
            .await?;

        conn.execute_raw(&delete_user_stmt, &[&id]).await?;
        self.cache.user.delete(id);
        self.cache.championship.delete_by_user(id);

        info!("User deleted with success: {}", id);

        Ok(())
    }

    /// Resets user password using a token.
    ///
    /// # Arguments
    /// - `token`: Password reset token.
    /// - `password`: New password.
    ///
    /// # Returns
    /// ID of the user whose password was reset.
    pub async fn reset_password_with_token(
        &self,
        token: String,
        password: String,
    ) -> AppResult<i32> {
        self.cache
            .token
            .get_token(token.clone(), TokenPurpose::PasswordReset);

        let user_id = {
            let token_data = self.token_svc.validate(&token)?;
            token_data.claims.subject_id
        };

        self.reset_password(user_id, password).await?;
        self.cache
            .token
            .remove_token(token, TokenPurpose::PasswordReset);

        Ok(user_id)
    }

    /// Activates a user account.
    ///
    /// # Arguments
    /// - `id`: User ID to activate.
    ///
    /// # Returns
    /// Result indicating success or failure.
    pub async fn activate(&self, id: i32) -> AppResult<()> {
        let conn = self.db.pg.get().await?;

        let activate_user_stmt = conn
            .prepare_cached(
                r#"
                    UPDATE users
                    SET active = true, updated_at = CURRENT_TIMESTAMP
                    WHERE id = $1
                "#,
            )
            .await?;

        conn.execute_raw(&activate_user_stmt, &[&id]).await?;
        self.cache.user.delete(id);

        info!("User activated with success: {}", id);

        Ok(())
    }

    /// Activates a user account using a token.
    ///
    /// # Arguments
    /// - `token`: Activation token.
    ///
    /// # Returns
    /// ID of the activated user.
    pub async fn activate_with_token(&self, token: String) -> AppResult<i32> {
        self.cache
            .token
            .get_token(token.clone(), TokenPurpose::EmailVerification);

        let user_id = {
            let token_data = self.token_svc.validate(&token)?;
            token_data.claims.subject_id
        };

        self.activate(user_id).await?;
        self.cache
            .token
            .remove_token(token, TokenPurpose::EmailVerification);

        Ok(user_id)
    }

    /// Deactivates a user account.
    ///
    /// # Arguments
    /// - `id`: User ID to deactivate.
    ///
    /// # Returns
    /// Result indicating success or failure.
    pub async fn deactivate(&self, id: i32) -> AppResult<()> {
        let conn = self.db.pg.get().await?;
        let deactivate_user_stmt = conn
            .prepare_cached(
                r#"
                    UPDATE users
                    SET active = false, updated_at = CURRENT_TIMESTAMP
                    WHERE id = $1
                "#,
            )
            .await?;

        conn.execute_raw(&deactivate_user_stmt, &[&id]).await?;
        self.cache.user.delete(id);

        info!("User activated with success: {}", id);
        Ok(())
    }

    /// Resets a user's password.
    ///
    /// # Arguments
    /// - `id`: User ID.
    /// - `password`: New password.
    ///
    /// # Returns
    /// Result indicating success or failure.
    async fn reset_password(&self, id: i32, password: String) -> AppResult<()> {
        let Some(user) = self.user_repo.find(id).await? else {
            Err(UserError::NotFound)?
        };

        if let Some(last_update) = user.updated_at {
            if Utc::now().signed_duration_since(last_update) > Duration::minutes(15) {
                return Err(UserError::UpdateLimitExceeded)?;
            }
        }

        let conn = self.db.pg.get().await?;
        let reset_password_stmt = conn
            .prepare_cached(
                r#"
                    UPDATE users
                    SET password = $1, updated_at = CURRENT_TIMESTAMP
                    WHERE id = $2
                "#,
            )
            .await?;

        let hashed_password = self.user_repo.hash_password(password).await?;
        conn.execute_raw(&reset_password_stmt, slice_iter(&[&hashed_password, &id]))
            .await?;

        self.cache.user.delete(id);

        info!("User password reseated with success: {}", id);
        Ok(())
    }
}
