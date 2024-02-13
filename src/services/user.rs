use async_trait::async_trait;
use bcrypt::{hash, DEFAULT_COST};
use chrono::{Duration, Utc};
use postgres_types::ToSql;
use tracing::{error, info};

use crate::{
    cache::{EntityCache, RedisCache},
    config::Database,
    entity::{Provider, UserExtension},
    error::{AppResult, TokenError, UserError},
    repositories::{UserRepository, UserRepositoryTrait},
    structs::{RegisterUserDto, TokenType, UpdateUser},
    utils::write,
};

use super::{TokenService, TokenServiceTrait};

/// A service for managing user accounts.
///
/// This service provides functionality to create, update, delete, and manage the state of user
/// accounts. It integrates database operations with caching for improved performance and
/// efficiency, leveraging both a direct database connection and a Redis cache.
#[derive(Clone)]
pub struct UserService {
    /// Redis cache for temporarily storing user data.
    cache: RedisCache,
    /// Database connection for persistent storage.
    db_conn: Database,
    /// Repository for user-specific database operations.
    user_repo: UserRepository,
    /// Service for managing authentication tokens.
    token_service: TokenService,
}

// TODO: Remove the `UserServiceTrait` and `UserService` and use the `UserService` directly. With the possibility of using a `EntityService` trait for common methods for all entities.
#[async_trait]
pub trait UserServiceTrait {
    /// Constructs a new instance of the user service.
    ///
    /// # Arguments
    /// - `db_conn`: A reference to the database connection.
    /// - `cache`: A reference to the Redis cache.
    ///
    /// # Returns
    /// A new `UserService` instance.
    fn new(db_conn: &Database, cache: &RedisCache) -> Self;

    /// Creates a new user account.
    ///
    /// # Arguments
    /// - `register`: Data transfer object containing user registration details.
    ///
    /// # Returns
    /// The ID of the newly created user if successful.
    async fn create(&self, register: &RegisterUserDto) -> AppResult<i32>;

    /// Updates an existing user account.
    ///
    /// # Arguments
    /// - `user`: Current state of the user to be updated.
    /// - `form`: Data transfer object containing updated user details.
    ///
    /// # Returns
    /// An empty result indicating success or failure.
    async fn update(&self, user: &UserExtension, form: &UpdateUser) -> AppResult<()>;

    /// Deletes a user account by ID.
    ///
    /// # Arguments
    /// - `id`: The ID of the user to delete.
    ///
    /// # Returns
    /// An empty result indicating success or failure.
    async fn delete(&self, id: i32) -> AppResult<()>;

    /// Resets the password for a user account.
    ///
    /// # Arguments
    /// - `id`: The ID of the user whose password is to be reset.
    /// - `password`: The new password.
    ///
    /// # Returns
    /// An empty result indicating success or failure.
    async fn reset_password(&self, id: i32, password: &str) -> AppResult<()>;

    /// Resets the password for a user account using a token.
    ///
    /// # Arguments
    /// - `token`: A token validating the reset request.
    /// - `password`: The new password.
    ///
    /// # Returns
    /// The ID of the user whose password was reset if successful.
    async fn reset_password_with_token(&self, token: &str, password: &str) -> AppResult<i32>;

    /// Activates a user account.
    ///
    /// # Arguments
    /// - `id`: The ID of the user to activate.
    ///
    /// # Returns
    /// An empty result indicating success or failure.
    async fn activate(&self, id: i32) -> AppResult<()>;

    /// Activates a user account using a token.
    ///
    /// # Arguments
    /// - `token`: A token validating the activation request.
    ///
    /// # Returns
    /// The ID of the user activated if successful.
    async fn activate_with_token(&self, token: &str) -> AppResult<i32>;

    /// Deactivates a user account.
    ///
    /// # Arguments
    /// - `id`: The ID of the user to deactivate.
    ///
    /// # Returns
    /// An empty result indicating success or failure.
    async fn deactivate(&self, id: i32) -> AppResult<()>;
}

#[async_trait]
impl UserServiceTrait for UserService {
    fn new(db_conn: &Database, cache: &RedisCache) -> Self {
        Self {
            cache: cache.clone(),
            db_conn: db_conn.clone(),
            user_repo: UserRepository::new(db_conn, cache),
            token_service: TokenService::new(cache),
        }
    }

    // TODO: Refactor this method
    async fn create(&self, register: &RegisterUserDto) -> AppResult<i32> {
        let user_exists = self.user_repo.user_exists(&register.email).await?;

        if user_exists {
            Err(UserError::AlreadyExists)?
        }

        let id = fastrand::i32(600000000..699999999);
        let conn = self.db_conn.pg.get().await?;

        match &register.provider {
            Some(provider) if provider == &Provider::Google => {
                let create_user_stmt = conn
                    .prepare_cached(
                        r#"
                            INSERT INTO users (id, email, username, avatar, provider, active)
                            VALUES ($1,$2,$3,$4,$5, true)
                        "#,
                    )
                    .await?;

                conn.execute(
                    &create_user_stmt,
                    &[
                        &id,
                        &register.email,
                        &register.username,
                        &register.avatar,
                        provider,
                    ],
                )
                .await?;
            }

            None => {
                let hashed_password = hash(register.password.clone().unwrap(), DEFAULT_COST)?;

                let create_google_user_stmt = conn
                    .prepare_cached(
                        r#"
                            INSERT INTO users (id, email, username, password, avatar, active)
                            VALUES ($1,$2,$3,$4,$5, false)
                        "#,
                    )
                    .await?;

                conn.execute(
                    &create_google_user_stmt,
                    &[
                        &id,
                        &register.email,
                        &register.username,
                        &hashed_password,
                        &format!("https://ui-avatars.com/api/?name={}", &register.username),
                    ],
                )
                .await?;
            }

            _ => Err(UserError::InvalidProvider)?,
        }

        info!("User created: {}", register.username);
        Ok(id)
    }

    async fn update(&self, user: &UserExtension, form: &UpdateUser) -> AppResult<()> {
        if Utc::now().signed_duration_since(user.updated_at) <= Duration::days(7) {
            Err(UserError::UpdateLimitExceeded)?
        }

        let (query, params) = {
            let mut counter = 1u8;
            let mut query = String::from("UPDATE users SET");
            let mut params: Vec<&(dyn ToSql + Sync)> = Vec::with_capacity(3);

            if let Some(username) = &form.username {
                write(&mut query, &mut counter, "username");
                params.push(username);
            }

            if let Some(avatar) = &form.avatar {
                write(&mut query, &mut counter, "avatar");
                params.push(avatar);
            }

            if counter == 1 {
                Err(UserError::InvalidUpdate)?
            }

            write(&mut query, &mut counter, "WHERE id");
            params.push(&user.id);

            (query, params)
        };

        let conn = self.db_conn.pg.get().await?;
        let update_user_stmt = conn.prepare_cached(&query).await?;

        let delete_cache_fut = self.cache.user.delete(user.id);
        let update_user_fut = async {
            conn.execute(&update_user_stmt, &params[..]).await?;
            Ok(())
        };

        tokio::try_join!(update_user_fut, delete_cache_fut)?;
        Ok(())
    }

    // Todo: Create a column "deleted" in users table and update it instead of delete
    async fn delete(&self, id: i32) -> AppResult<()> {
        let conn = self.db_conn.pg.get().await?;

        let delete_users_relations_stmt_fut = conn.prepare_cached(
            r#"
                DELETE FROM user_championships
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

        let binding: [&(dyn ToSql + Sync); 1] = [&id];
        conn.execute(&delete_users_relations_stmt, &binding).await?;

        let user_deletion_fut = async {
            conn.execute(&delete_user_stmt, &binding).await?;
            Ok(())
        };
        let cache_del_fut = self.cache.user.delete(id);

        tokio::try_join!(user_deletion_fut, cache_del_fut)?;
        info!("User deleted with success: {}", id);

        Ok(())
    }

    async fn reset_password(&self, id: i32, password: &str) -> AppResult<()> {
        let Some(user) = self.user_repo.find(id).await? else {
            Err(UserError::NotFound)?
        };

        if Utc::now().signed_duration_since(user.updated_at) <= Duration::minutes(15) {
            Err(UserError::UpdateLimitExceeded)?
        }

        let conn = self.db_conn.pg.get().await?;
        let reset_password_stmt = conn
            .prepare_cached(
                r#"
                    UPDATE users
                    SET password = $1
                    WHERE id = $2
                "#,
            )
            .await?;

        let hashed_password = hash(password, DEFAULT_COST)?;
        let bindings: [&(dyn ToSql + Sync); 2] = [&hashed_password, &id];
        let update_user_fut = async {
            conn.execute(&reset_password_stmt, &bindings).await?;
            Ok(())
        };
        let remove_cache_fut = self.cache.user.delete(id);

        tokio::try_join!(update_user_fut, remove_cache_fut)?;
        info!("User password reseated with success: {}", id);

        Ok(())
    }

    async fn reset_password_with_token(&self, token: &str, password: &str) -> AppResult<i32> {
        self.cache
            .token
            .get_token(token, &TokenType::ResetPassword)
            .await?;

        let user_id = {
            let token_data = self.token_service.validate(token)?;
            if token_data.claims.token_type != TokenType::ResetPassword {
                error!("Token type is not ResetPassword");
                Err(TokenError::InvalidToken)?
            }

            token_data.claims.sub
        };

        self.reset_password(user_id, password).await?;
        self.cache
            .token
            .remove_token(token, &TokenType::ResetPassword)
            .await?;

        Ok(user_id)
    }

    async fn activate(&self, id: i32) -> AppResult<()> {
        let conn = self.db_conn.pg.get().await?;

        let activate_user_stmt = conn
            .prepare_cached(
                r#"
                    UPDATE users
                    SET active = true
                    WHERE id = $1
                "#,
            )
            .await?;

        let bindings: [&(dyn ToSql + Sync); 1] = [&id];
        let activate_user_fut = async {
            conn.execute(&activate_user_stmt, &bindings).await?;
            Ok(())
        };
        let delete_cache_fut = self.cache.user.delete(id);

        tokio::try_join!(activate_user_fut, delete_cache_fut)?;

        info!("User activated with success: {}", id);

        Ok(())
    }

    async fn activate_with_token(&self, token: &str) -> AppResult<i32> {
        self.cache.token.get_token(token, &TokenType::Email).await?;
        let user_id = {
            let token_data = self.token_service.validate(token)?;
            if token_data.claims.token_type != TokenType::Email {
                Err(TokenError::InvalidToken)?
            }

            token_data.claims.sub
        };

        self.activate(user_id).await?;
        self.cache
            .token
            .remove_token(token, &TokenType::Email)
            .await?;

        Ok(user_id)
    }

    async fn deactivate(&self, id: i32) -> AppResult<()> {
        let conn = self.db_conn.pg.get().await?;
        let deactivate_user_stmt = conn
            .prepare_cached(
                r#"
                    UPDATE users
                    SET active = false
                    WHERE id = $1
                "#,
            )
            .await?;

        let bindings: [&(dyn ToSql + Sync); 1] = [&id];
        let delete_cache_fut = self.cache.user.delete(id);
        let deactivate_user_fut = async {
            conn.execute(&deactivate_user_stmt, &bindings).await?;
            Ok(())
        };

        tokio::try_join!(deactivate_user_fut, delete_cache_fut)?;
        info!("User activated with success: {}", id);
        Ok(())
    }
}
