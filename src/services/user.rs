use chrono::{Duration, Utc};
use postgres_types::ToSql;
use tracing::info;

use crate::{
    cache::{EntityCache, ServiceCache},
    config::Database,
    entity::{Provider, SharedUser},
    error::{AppResult, TokenError, UserError},
    repositories::UserRepository,
    structs::{TokenPurpose, UserRegistrationData, UserUpdateData},
    utils::{slice_iter, IdsGenerator},
};

use super::TokenService;

/// Defines the core operations for managing users.
pub trait UserServiceOperations {
    /// Creates a new user.
    ///
    /// # Arguments
    ///
    /// * `registration_data` - The data required to create a user.
    ///
    /// # Errors
    ///
    /// Returns an error if the user already exists or if there's a database error.
    async fn create(&self, registration_data: UserRegistrationData) -> AppResult<i32>;

    /// Updates an existing user.
    ///
    /// # Arguments
    ///
    /// * `user` - The user to update.
    /// * `form` - The data to update the user with.
    ///
    /// # Errors
    ///
    /// Returns an error if the update interval hasn't been reached or if there's a database error.
    async fn update(&self, user: SharedUser, form: &UserUpdateData) -> AppResult<()>;

    /// Deletes a user.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the user to delete.
    ///
    /// # Errors
    ///
    /// Returns an error if the user is not found or if there's a database error.
    async fn delete(&self, id: i32) -> AppResult<()>;

    /// Resets a user's password.
    ///
    /// # Arguments
    ///
    /// * `token` - The password reset token.
    /// * `password` - The new password.
    ///
    /// # Errors
    ///
    /// Returns an error if the token is invalid or if there's a database error.
    async fn reset_password(&self, token: String, password: String) -> AppResult<i32>;

    /// Activates a user's account.
    ///
    /// # Arguments
    ///
    /// * `token` - The activation token.
    ///
    /// # Errors
    ///
    /// Returns an error if the token is invalid or if there's a database error.
    async fn activate(&self, token: String) -> AppResult<i32>;
}

/// Defines additional admin-level operations for managing users.
pub trait UserAdminServiceOperations: UserServiceOperations {
    /// Allows an admin to activate a user's account.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the user to activate.
    ///
    /// # Errors
    ///
    /// Returns an error if the user is not found or if there's a database error.
    async fn admin_activate(&self, id: i32) -> AppResult<()>;

    /// Allows an admin to deactivate a user's account.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the user to deactivate.
    ///
    /// # Errors
    ///
    /// Returns an error if the user is not found or if there's a database error.
    async fn admin_deactivate(&self, id: i32) -> AppResult<()>;
}

/// Implements the user service logic.
pub struct UserService {
    cache: &'static ServiceCache,
    db: &'static Database,
    user_repo: &'static UserRepository,
    token_svc: &'static TokenService,
    ids_generator: IdsGenerator,
}

impl UserService {
    /// Creates a new UserService instance.
    ///
    /// # Arguments
    ///
    /// * `db` - The database connection.
    /// * `cache` - The service cache.
    /// * `user_repo` - The user repository.
    /// * `token_svc` - The token service.
    ///
    /// # Errors
    ///
    /// Returns an error if there's an issue initializing the service components.
    pub async fn new(
        db: &'static Database,
        cache: &'static ServiceCache,
        user_repo: &'static UserRepository,
        token_svc: &'static TokenService,
    ) -> Self {
        let ids_generator = {
            let used_ids = user_repo._used_ids().await.unwrap();
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

    /// Internal method to create a user.
    #[inline(always)]
    async fn _create(&self, registration_data: UserRegistrationData) -> AppResult<i32> {
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

    /// Internal method to update a user.
    #[inline(always)]
    async fn _update(&self, user: SharedUser, form: &UserUpdateData) -> AppResult<()> {
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

    /// Internal method to delete a user.
    #[inline(always)]
    async fn _delete(&self, id: i32) -> AppResult<()> {
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
        self.cache.championship.delete_by_user(&id);

        info!("User deleted with success: {}", id);

        Ok(())
    }

    /// Internal method to reset a user's password.
    #[inline(always)]
    async fn _reset_password(&self, id: i32, password: String) -> AppResult<()> {
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

    /// Internal method to activate a user's account.
    #[inline(always)]
    async fn _activate(&self, id: i32) -> AppResult<()> {
        let conn = self.db.pg.get().await?;

        let activate_user_stmt = conn
            .prepare_cached(
                r#"
                    UPDATE users
                    SET active = true
                    WHERE id = $1
                "#,
            )
            .await?;

        conn.execute_raw(&activate_user_stmt, &[&id]).await?;
        self.cache.user.delete(id);

        info!("User activated with success: {}", id);

        Ok(())
    }

    /// Internal method to deactivate a user's account.
    #[inline(always)]
    async fn _deactivate(&self, id: i32) -> AppResult<()> {
        let conn = self.db.pg.get().await?;
        let deactivate_user_stmt = conn
            .prepare_cached(
                r#"
                    UPDATE users
                    SET active = false
                    WHERE id = $1
                "#,
            )
            .await?;

        conn.execute_raw(&deactivate_user_stmt, &[&id]).await?;
        self.cache.user.delete(id);

        info!("User activated with success: {}", id);
        Ok(())
    }
}

impl UserServiceOperations for UserService {
    async fn create(&self, registration_data: UserRegistrationData) -> AppResult<i32> {
        self._create(registration_data).await
    }

    async fn update(&self, user: SharedUser, form: &UserUpdateData) -> AppResult<()> {
        if let Some(last_update) = user.updated_at {
            if Utc::now().signed_duration_since(last_update) <= Duration::days(7) {
                return Err(UserError::UpdateLimitExceeded)?;
            }
        }

        self._update(user, form).await
    }

    async fn delete(&self, id: i32) -> AppResult<()> {
        self._delete(id).await
    }

    async fn reset_password(&self, token: String, password: String) -> AppResult<i32> {
        if !self
            .cache
            .token
            .get_token(token.clone(), TokenPurpose::PasswordReset)
        {
            return Err(TokenError::InvalidToken)?;
        }

        let user_id = self.token_svc.subject_id(&token)?;

        self._reset_password(user_id, password).await?;

        self.cache
            .token
            .remove_token(token, TokenPurpose::PasswordReset);

        Ok(user_id)
    }

    async fn activate(&self, token: String) -> AppResult<i32> {
        if !self
            .cache
            .token
            .get_token(token.clone(), TokenPurpose::EmailVerification)
        {
            return Err(TokenError::InvalidToken)?;
        }

        let user_id = self.token_svc.subject_id(&token)?;

        self._activate(user_id).await?;

        self.cache
            .token
            .remove_token(token, TokenPurpose::EmailVerification);

        Ok(user_id)
    }
}

impl UserAdminServiceOperations for UserService {
    async fn admin_activate(&self, id: i32) -> AppResult<()> {
        self._activate(id).await
    }

    async fn admin_deactivate(&self, id: i32) -> AppResult<()> {
        self._deactivate(id).await
    }
}
