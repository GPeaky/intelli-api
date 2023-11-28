use super::{TokenService, TokenServiceTrait};
use crate::{
    cache::{EntityCache, RedisCache},
    config::Database,
    dtos::{RegisterUserDto, TokenType},
    entity::Provider,
    error::{AppResult, TokenError, UserError},
    repositories::{UserRepository, UserRepositoryTrait},
};
use async_trait::async_trait;
use bcrypt::{hash, DEFAULT_COST};
use log::{error, info};
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::sync::Arc;

pub struct UserService {
    #[allow(unused)]
    cache: Arc<RedisCache>,
    db_conn: Arc<Database>,
    user_repo: UserRepository,
    token_service: TokenService,
}

#[async_trait]
pub trait UserServiceTrait {
    fn new(db_conn: &Arc<Database>, cache: &Arc<RedisCache>) -> Self;
    async fn create(&self, register: &RegisterUserDto) -> AppResult<i32>;
    async fn delete(&self, id: &i32) -> AppResult<()>;
    async fn reset_password(&self, id: &i32, password: &str) -> AppResult<()>;
    async fn reset_password_with_token(&self, token: &str, password: &str) -> AppResult<i32>;
    async fn activate(&self, id: &i32) -> AppResult<()>;
    async fn activate_with_token(&self, token: &str) -> AppResult<()>;
    async fn deactivate(&self, id: &i32) -> AppResult<()>;
}

#[async_trait]
impl UserServiceTrait for UserService {
    fn new(db_conn: &Arc<Database>, cache: &Arc<RedisCache>) -> Self {
        Self {
            cache: cache.clone(),
            db_conn: db_conn.clone(),
            user_repo: UserRepository::new(db_conn, cache),
            token_service: TokenService::new(cache),
        }
    }

    async fn create(&self, register: &RegisterUserDto) -> AppResult<i32> {
        let user_exists = self.user_repo.user_exists(&register.email).await?;

        if user_exists {
            Err(UserError::AlreadyExists)?
        }

        let id = {
            let mut rand = StdRng::from_entropy();
            rand.gen_range(600000000..700000000)
        };

        match &register.provider {
            Some(provider) if provider.eq(&Provider::Google) => {
                let conn = self.db_conn.pg.get().await?;

                let cached_statement = conn
                    .prepare_cached(
                        r#"
                        INSERT INTO users (id, email, username, avatar, provider, active)
                        VALUES ($1,$2,$3,$4,$5, true)
                    "#,
                    )
                    .await?;

                conn.execute(
                    &cached_statement,
                    &[
                        &id,
                        &register.email,
                        &register.username,
                        &register.avatar,
                        &provider,
                    ],
                )
                .await?;
            }

            None => {
                let hashed_password = hash(register.password.clone().unwrap(), DEFAULT_COST)?;
                let conn = self.db_conn.pg.get().await?;

                let cached_statement = conn
                    .prepare_cached(
                        r#"
                        INSERT INTO users (id, email, username, password, avatar, active)
                        VALUES ($1,$2,$3,$4,$5, false)
                    "#,
                    )
                    .await?;

                conn.execute(
                    &cached_statement,
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

    async fn delete(&self, id: &i32) -> AppResult<()> {
        {
            let conn = self.db_conn.pg.get().await?;

            let cached_statement = conn
                .prepare_cached(
                    r#"
                    DELETE FROM user_championships
                    WHERE user_id = $1
                "#,
                )
                .await?;

            let cached2 = conn
                .prepare_cached(
                    r#"
                    DELETE FROM user
                    WHERE id = $1
                "#,
                )
                .await?;

            conn.execute(&cached_statement, &[id]).await?;
            conn.execute(&cached2, &[id]).await?;
        }

        self.cache.user.delete(id).await?;
        info!("User deleted with success: {}", id);

        Ok(())
    }

    async fn reset_password_with_token(&self, token: &str, password: &str) -> AppResult<i32> {
        self.cache
            .token
            .get_token(token, &TokenType::ResetPassword)
            .await?;

        let user_id = {
            let token_data = self.token_service.validate(token)?;
            if token_data.claims.token_type.ne(&TokenType::ResetPassword) {
                error!("Token type is not ResetPassword");
                Err(TokenError::InvalidToken)?
            }

            token_data.claims.sub
        };

        self.reset_password(&user_id, password).await?;
        self.cache
            .token
            .remove_token(token, &TokenType::ResetPassword)
            .await?;

        Ok(user_id)
    }

    async fn reset_password(&self, id: &i32, password: &str) -> AppResult<()> {
        // TODO: Check if updated_at is less than 5 minutes & if the updated_at is being updated
        {
            let hashed_password = hash(password, DEFAULT_COST)?;
            let conn = self.db_conn.pg.get().await?;

            let cached_statement = conn
                .prepare_cached(
                    r#"
                        UPDATE users
                        SET password = $1
                        WHERE id = $2
                    "#,
                )
                .await?;

            conn.execute(&cached_statement, &[&hashed_password, id])
                .await?;
        }

        self.cache.user.delete(id).await?;
        info!("User password reseated with success: {}", id);

        Ok(())
    }

    async fn activate_with_token(&self, token: &str) -> AppResult<()> {
        self.cache.token.get_token(token, &TokenType::Email).await?;

        let user_id = {
            let token_data = self.token_service.validate(token)?;
            if token_data.claims.token_type.ne(&TokenType::Email) {
                Err(TokenError::InvalidToken)?
            }

            token_data.claims.sub
        };

        self.activate(&user_id).await?;

        self.cache
            .token
            .remove_token(token, &TokenType::Email)
            .await?;

        Ok(())
    }

    async fn activate(&self, id: &i32) -> AppResult<()> {
        {
            let conn = self.db_conn.pg.get().await?;

            let cached_statement = conn
                .prepare_cached(
                    r#"
                        UPDATE users
                        SET active = true
                        WHERE id = $1
                    "#,
                )
                .await?;

            conn.execute(&cached_statement, &[id]).await?;
        }

        self.cache.user.delete(id).await?;
        info!("User activated with success: {}", id);

        Ok(())
    }

    async fn deactivate(&self, id: &i32) -> AppResult<()> {
        {
            let conn = self.db_conn.pg.get().await?;

            let cached_statement = conn
                .prepare_cached(
                    r#"
                    UPDATE user
                    SET active = false
                    WHERE id = $1
                "#,
                )
                .await?;

            conn.execute(&cached_statement, &[id]).await?;
        }

        self.cache.user.delete(id).await?;
        info!("User activated with success: {}", id);

        Ok(())
    }
}
