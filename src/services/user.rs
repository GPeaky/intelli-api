use crate::entity::Provider;
use crate::error::UserError;
use crate::{
    config::Database,
    dtos::RegisterUserDto,
    error::AppResult,
    repositories::{UserRepository, UserRepositoryTrait},
};
use axum::async_trait;
use bcrypt::{hash, DEFAULT_COST};
use std::sync::Arc;
use tinyrand::{Rand, StdRand};
use tracing::info;

#[derive(Clone)]
pub struct UserService {
    db_conn: Arc<Database>,
    #[allow(unused)]
    user_repo: UserRepository,
}

#[async_trait]
pub trait UserServiceTrait {
    fn new(db_conn: &Arc<Database>) -> Self;
    async fn new_user(&self, register: &RegisterUserDto) -> AppResult<u32>;
    async fn delete_user(&self, id: &u32) -> AppResult<()>;
    async fn activate_user(&self, id: &u32) -> AppResult<()>;
    async fn deactivate_user(&self, id: &u32) -> AppResult<()>;
}

#[async_trait]
impl UserServiceTrait for UserService {
    fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            db_conn: db_conn.clone(),
            user_repo: UserRepository::new(db_conn),
        }
    }

    async fn new_user(&self, register: &RegisterUserDto) -> AppResult<u32> {
        let user_exists = self.user_repo.user_exists(&register.email).await?;

        if user_exists {
            Err(UserError::AlreadyExists)?
        }

        let mut rng = StdRand::default();
        let id = rng.next_u32();
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
