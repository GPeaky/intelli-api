use super::{TokenService, TokenServiceTrait};
use crate::{
    cache::{EntityCache, RedisCache},
    config::Database,
    dtos::{RegisterUserDto, TokenType, UpdateUser},
    entity::{Provider, UserExtension},
    error::{AppResult, TokenError, UserError},
    repositories::{UserRepository, UserRepositoryTrait},
};
use async_trait::async_trait;
use bcrypt::{hash, DEFAULT_COST};
use chrono::{Duration, Utc};
use log::{error, info};
use postgres_types::ToSql;

#[derive(Clone)]
pub struct UserService {
    #[allow(unused)]
    cache: RedisCache,
    db_conn: Database,
    user_repo: UserRepository,
    token_service: TokenService,
}

#[async_trait]
pub trait UserServiceTrait {
    fn new(db_conn: &Database, cache: &RedisCache) -> Self;
    async fn create(&self, register: &RegisterUserDto) -> AppResult<i32>;
    async fn update(&self, user: &UserExtension, form: &UpdateUser) -> AppResult<()>;
    async fn delete(&self, id: &i32) -> AppResult<()>;
    async fn reset_password(&self, id: &i32, password: &str) -> AppResult<()>;
    async fn reset_password_with_token(&self, token: &str, password: &str) -> AppResult<i32>;
    async fn activate(&self, id: &i32) -> AppResult<()>;
    async fn activate_with_token(&self, token: &str) -> AppResult<i32>;
    async fn deactivate(&self, id: &i32) -> AppResult<()>;
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
            Some(provider) if provider.eq(&Provider::Google) => {
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
                        provider,
                    ],
                )
                .await?;
            }

            None => {
                let hashed_password = hash(register.password.clone().unwrap(), DEFAULT_COST)?;

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

    async fn update(&self, user: &UserExtension, form: &UpdateUser) -> AppResult<()> {
        if Utc::now().signed_duration_since(user.updated_at) <= Duration::days(7) {
            Err(UserError::UpdateLimitExceeded)?
        }

        let (query, params) = {
            let mut counter = 1;
            let mut query = String::from("UPDATE users SET");
            let mut params: Vec<&(dyn ToSql + Sync)> = Vec::new();

            if let Some(username) = &form.username {
                if counter > 1 {
                    query.push(',');
                }

                query.push_str(&format!(" username = ${counter}"));
                params.push(username);
                counter += 1;
            }

            if let Some(avatar) = &form.avatar {
                if counter > 1 {
                    query.push(',');
                }

                query.push_str(&format!(" avatar = ${counter}"));
                params.push(avatar);
                counter += 1;
            }

            if counter == 1 {
                Err(UserError::InvalidUpdate)?
            }

            query.push_str(&format!(" WHERE id = ${}", counter));
            params.push(&user.id);

            (query, params)
        };

        let conn = self.db_conn.pg.get().await?;
        let cached_statement = conn.prepare_cached(&query).await?;

        let delete_cache_fut = self.cache.user.delete(&user.id);
        let update_user_fut = async {
            conn.execute(&cached_statement, &params[..]).await?;
            Ok(())
        };

        tokio::try_join!(update_user_fut, delete_cache_fut);

        Ok(())
    }

    async fn delete(&self, id: &i32) -> AppResult<()> {
        let conn = self.db_conn.pg.get().await?;

        let cached_task = conn.prepare_cached(
            r#"
                    DELETE FROM user_championships
                    WHERE user_id = $1
                "#,
        );

        let cached2_task = conn.prepare_cached(
            r#"
                    DELETE FROM user
                    WHERE id = $1
                "#,
        );

        let (cached, cached2) = tokio::try_join!(cached_task, cached2_task)?;

        let binding: [&(dyn ToSql + Sync); 1] = [id];
        conn.execute(&cached, &binding).await?;

        let user_deletion_fut = async {
            conn.execute(&cached2, &binding).await?;
            Ok(())
        };
        let cache_del_fut = self.cache.user.delete(id);

        tokio::try_join!(user_deletion_fut, cache_del_fut);
        info!("User deleted with success: {}", id);

        Ok(())
    }

    async fn reset_password(&self, id: &i32, password: &str) -> AppResult<()> {
        let Some(user) = self.user_repo.find(id).await? else {
            Err(UserError::NotFound)?
        };

        if Utc::now().signed_duration_since(user.updated_at) <= Duration::minutes(15) {
            Err(UserError::UpdateLimitExceeded)?
        }

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

        let hashed_password = hash(password, DEFAULT_COST)?;
        let bindings: [&(dyn ToSql + Sync); 2] = [&hashed_password, id];
        let update_user_fut = async {
            conn.execute(&cached_statement, &bindings).await?;
            Ok(())
        };
        let remove_cache_fut = self.cache.user.delete(id);

        tokio::try_join!(update_user_fut, remove_cache_fut);
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

    async fn activate(&self, id: &i32) -> AppResult<()> {
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

        let bindings: [&(dyn ToSql + Sync); 1] = [id];
        let activate_user_fut = async {
            conn.execute(&cached_statement, &bindings).await?;
            Ok(())
        };
        let delete_cache_fut = self.cache.user.delete(id);

        tokio::try_join!(activate_user_fut, delete_cache_fut);

        info!("User activated with success: {}", id);

        Ok(())
    }

    async fn activate_with_token(&self, token: &str) -> AppResult<i32> {
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

        Ok(user_id)
    }

    async fn deactivate(&self, id: &i32) -> AppResult<()> {
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

        let bindings: [&(dyn ToSql + Sync); 1] = [id];
        let deactivate_user_fut = async {
            conn.execute(&cached_statement, &bindings).await?;
            Ok(())
        };
        let delete_cache_fut = self.cache.user.delete(id);

        tokio::try_join!(deactivate_user_fut, delete_cache_fut);

        info!("User activated with success: {}", id);

        Ok(())
    }
}
