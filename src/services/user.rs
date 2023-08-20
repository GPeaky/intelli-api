use crate::{
    config::Database,
    dtos::{PreparedStatementsKey, RegisterUserDto, UserStatements},
    entity::Role,
    error::{AppResult, UserError},
    repositories::{UserRepository, UserRepositoryTrait},
};
use argon2::{self, Config};
use axum::async_trait;
use chrono::Utc;
use dotenvy::var;
use rand::{rngs::StdRng as Rand, Rng, SeedableRng};
use std::sync::Arc;
use tracing::trace;

#[derive(Clone)]
pub struct UserService {
    pass_salt: Vec<u8>,
    db_conn: Arc<Database>,
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
        let time = Utc::now();
        let mut rng = Rand::from_entropy();
        let user_exists = self.user_repo.user_exists(&register.email).await?;
        let id = rng.gen::<i32>();

        if user_exists {
            return Err(UserError::AlreadyExists)?;
        }

        // TODO: Check what is the result and if we can return the new user id
        self.db_conn
            .scylla
            .execute(
                self.db_conn
                    .statements
                    .get(&PreparedStatementsKey::User(UserStatements::Insert))
                    .unwrap(),
                (
                    id,
                    register.username.clone(),
                    argon2::hash_encoded(
                        register.password.as_bytes(),
                        &self.pass_salt,
                        &self.argon2_config,
                    )
                    .unwrap(),
                    register.email.clone(),
                    false,
                    Role::User as i16,
                    time,
                    time,
                ),
            )
            .await?;

        trace!("User created: {}", register.username);

        Ok(id)
    }

    async fn delete_user(&self, id: &i32) -> AppResult<()> {
        // TODO: Delete all the data from this user
        self.db_conn
            .scylla
            .execute(
                self.db_conn
                    .statements
                    .get(&PreparedStatementsKey::User(UserStatements::Delete))
                    .unwrap(),
                (id,),
            )
            .await?;

        trace!("User deleted with success: {}", id);

        Ok(())
    }

    async fn activate_user(&self, id: &i32) -> AppResult<()> {
        self.db_conn
            .scylla
            .execute(
                self.db_conn
                    .statements
                    .get(&PreparedStatementsKey::User(UserStatements::Activate))
                    .unwrap(),
                (id,),
            )
            .await?;

        trace!("User activated with success: {}", id);

        Ok(())
    }

    async fn deactivate_user(&self, id: &i32) -> AppResult<()> {
        self.db_conn
            .scylla
            .execute(
                self.db_conn
                    .statements
                    .get(&PreparedStatementsKey::User(UserStatements::Deactivate))
                    .unwrap(),
                (id,),
            )
            .await?;

        trace!("User deactivated with success: {}", id);

        Ok(())
    }
}
