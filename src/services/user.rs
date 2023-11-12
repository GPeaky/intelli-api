use super::{TokenService, TokenServiceTrait};
use crate::{
    config::Database,
    dtos::{RegisterUserDto, TokenType},
    entity::Provider,
    error::{AppResult, TokenError, UserError},
    repositories::{UserRepository, UserRepositoryTrait},
};
use axum::async_trait;
use bcrypt::{hash, DEFAULT_COST};
use rand::{rngs::StdRng, Rng, SeedableRng};
use redis::AsyncCommands;
use std::sync::Arc;
use tracing::{error, info};

pub struct UserService {
    db_conn: Arc<Database>,
    #[allow(unused)]
    user_repo: UserRepository,
    token_service: TokenService,
}

#[async_trait]
pub trait UserServiceTrait {
    fn new(db_conn: &Arc<Database>) -> Self;
    async fn new_user(&self, register: &RegisterUserDto) -> AppResult<u32>;
    async fn delete_user(&self, id: &u32) -> AppResult<()>;
    async fn reset_password_with_token(&self, token: &str, password: &str) -> AppResult<u32>;
    async fn reset_password(&self, id: &u32, password: &str) -> AppResult<()>;
    async fn activate_user(&self, id: &u32) -> AppResult<()>;
    async fn activate_user_with_token(&self, token: &str) -> AppResult<()>;
    async fn deactivate_user(&self, id: &u32) -> AppResult<()>;
}

#[async_trait]
impl UserServiceTrait for UserService {
    fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            db_conn: db_conn.clone(),
            user_repo: UserRepository::new(db_conn),
            token_service: TokenService::new(db_conn),
        }
    }

    async fn new_user(&self, register: &RegisterUserDto) -> AppResult<u32> {
        let user_exists = self.user_repo.user_exists(&register.email).await?;

        if user_exists {
            Err(UserError::AlreadyExists)?
        }

        let mut rand = StdRng::from_entropy();
        let id: u32 = rand.gen_range(600000000..700000000);

        let hashed_password = register
            .password
            .as_ref()
            .map(|password| hash(password, DEFAULT_COST).unwrap());

        match &register.provider {
            Some(provider) if provider.eq(&Provider::Google) => {
                sqlx::query(
                    r#"
                        INSERT INTO user (id, email, username, avatar, provider, active)
                        VALUES (?,?,?,?,?,1)
                    "#,
                )
                .bind(id)
                .bind(&register.email)
                .bind(&register.username)
                .bind(&register.avatar)
                .bind(&register.provider)
                .execute(&self.db_conn.mysql)
                .await?;
            }

            None => {
                sqlx::query(
                    r#"
                    INSERT INTO user (id, email, username, password, avatar)
                    VALUES (?,?,?,?,?)
                "#,
                )
                .bind(id)
                .bind(&register.email)
                .bind(&register.username)
                .bind(hashed_password)
                .bind(format!(
                    "https://ui-avatars.com/api/?name={}",
                    &register.username
                ))
                .execute(&self.db_conn.mysql)
                .await?;
            }

            _ => Err(UserError::InvalidProvider)?,
        }

        info!("User created: {}", register.username);

        Ok(id)
    }

    async fn delete_user(&self, id: &u32) -> AppResult<()> {
        sqlx::query(
            r#"
                DELETE FROM user_championships
                WHERE user_id = ?
            "#,
        )
        .bind(id)
        .execute(&self.db_conn.mysql)
        .await?;

        sqlx::query(
            r#"
                DELETE FROM user
                WHERE ID = ?
            "#,
        )
        .bind(id)
        .execute(&self.db_conn.mysql)
        .await?;

        info!("User deleted with success: {}", id);

        Ok(())
    }

    async fn reset_password_with_token(&self, token: &str, password: &str) -> AppResult<u32> {
        let user_id;
        let mut redis = self.db_conn.redis.aquire().await.unwrap();

        let Ok(_) = redis.get::<_, u8>(format!("reset:{}", token)).await else {
            error!("Token not found in redis");
            Err(TokenError::InvalidToken)?
        };

        {
            let token_data = self.token_service.validate(token)?;
            if token_data.claims.token_type.ne(&TokenType::ResetPassword) {
                error!("Token type is not ResetPassword");
                Err(TokenError::InvalidToken)?
            }

            user_id = token_data.claims.sub;
        }

        self.reset_password(&user_id, password).await?;

        redis
            .del::<_, u8>(format!("reset:{}", token))
            .await
            .unwrap();

        Ok(user_id)
    }

    async fn reset_password(&self, id: &u32, password: &str) -> AppResult<()> {
        // TODO: Check if updated_at is less than 5 minutes & if the updated_at is being updated
        sqlx::query(
            r#"
                UPDATE user
                SET password = ?
                WHERE id = ?
            "#,
        )
        .bind(hash(password, DEFAULT_COST).unwrap())
        .bind(id)
        .execute(&self.db_conn.mysql)
        .await?;

        info!("User password reseated with success: {}", id);

        Ok(())
    }

    async fn activate_user_with_token(&self, token: &str) -> AppResult<()> {
        let user_id;
        let mut redis = self.db_conn.redis.aquire().await.unwrap();

        let Ok(_) = redis.get::<_, u8>(format!("email:{}", token)).await else {
            Err(TokenError::InvalidToken)?
        };

        {
            let token_data = self.token_service.validate(token)?;
            if token_data.claims.token_type.ne(&TokenType::Email) {
                Err(TokenError::InvalidToken)?
            }

            user_id = token_data.claims.sub;
        }

        self.activate_user(&user_id).await?;

        redis
            .del::<_, u8>(format!("email:{}", token))
            .await
            .unwrap();

        Ok(())
    }

    async fn activate_user(&self, id: &u32) -> AppResult<()> {
        sqlx::query(
            r#"
                UPDATE user
                SET active = 1
                WHERE id = ?
            "#,
        )
        .bind(id)
        .execute(&self.db_conn.mysql)
        .await?;

        info!("User activated with success: {}", id);

        Ok(())
    }

    async fn deactivate_user(&self, id: &u32) -> AppResult<()> {
        sqlx::query(
            r#"
                UPDATE user
                SET active = 0
                WHERE id = ?
            "#,
        )
        .bind(id)
        .execute(&self.db_conn.mysql)
        .await?;

        info!("User deactivated with success: {}", id);

        Ok(())
    }
}
