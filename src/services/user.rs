use crate::error::UserError;
use crate::{
    config::Database,
    dtos::RegisterUserDto,
    error::AppResult,
    repositories::{UserRepository, UserRepositoryTrait},
};
use axum::async_trait;
use bcrypt::{hash, DEFAULT_COST};
use rand::{rngs::StdRng as Rand, Rng, SeedableRng};
use std::sync::Arc;
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
    async fn new_user(&self, register: &RegisterUserDto) -> AppResult<i32>;
    async fn delete_user(&self, id: &i32) -> AppResult<()>;
    async fn activate_user(&self, id: &i32) -> AppResult<()>;
    async fn deactivate_user(&self, id: &i32) -> AppResult<()>;
}

#[async_trait]
impl UserServiceTrait for UserService {
    fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            db_conn: db_conn.clone(),
            user_repo: UserRepository::new(db_conn),
        }
    }

    async fn new_user(&self, register: &RegisterUserDto) -> AppResult<i32> {
        let user_exists = self.user_repo.user_exists(&register.email).await?;

        if user_exists {
            Err(UserError::AlreadyExists)?
        }

        let mut rng = Rand::from_entropy();
        let id = rng.gen::<i32>();
        let hashed_password = hash(&register.password, DEFAULT_COST).unwrap();

        // TODO: Check what is the result and if we can return the new user id
        sqlx::query(
            r#"
                INSERT INTO user (id, email, username, password)
                VALUES (?,?,?,?)
            "#,
        )
        .bind(id)
        .bind(&register.email)
        .bind(&register.username)
        .bind(hashed_password)
        .execute(&self.db_conn.mysql)
        .await?;

        info!("User created: {}", register.username);

        Ok(id)
    }

    async fn delete_user(&self, id: &i32) -> AppResult<()> {
        // TODO: Delete all the data from this user

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

    async fn activate_user(&self, id: &i32) -> AppResult<()> {
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

    async fn deactivate_user(&self, id: &i32) -> AppResult<()> {
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
