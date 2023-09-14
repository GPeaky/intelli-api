use crate::{
    config::Database,
    dtos::RegisterUserDto,
    error::AppResult,
    repositories::{UserRepository, UserRepositoryTrait},
};
use argon2::{self, Config};
use axum::async_trait;
use dotenvy::var;
use rand::{rngs::StdRng as Rand, Rng, SeedableRng};
use std::sync::Arc;
use tracing::info;

#[derive(Clone)]
pub struct UserService {
    pass_salt: Vec<u8>,
    db_conn: Arc<Database>,
    #[allow(unused)]
    user_repo: UserRepository,
    argon2_config: Config<'static>,
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
            argon2_config: Config::default(),
            pass_salt: var("PASS_SALT").unwrap().as_bytes().to_owned(),
        }
    }

    async fn new_user(&self, register: &RegisterUserDto) -> AppResult<i32> {
        let mut rng = Rand::from_entropy();
        let id = rng.gen::<i32>();

        let hashed_password = argon2::hash_encoded(
            register.password.as_bytes(),
            &self.pass_salt,
            &self.argon2_config,
        )
        .unwrap();

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
