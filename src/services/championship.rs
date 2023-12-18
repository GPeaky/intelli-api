use crate::{
    cache::RedisCache,
    config::Database,
    dtos::{CreateChampionshipDto, UpdateChampionship},
    error::{AppResult, ChampionshipError, CommonError, UserError},
    repositories::{ChampionshipRepository, UserRepository, UserRepositoryTrait},
};
use chrono::{Duration, Utc};
use log::info;
use parking_lot::RwLock;
use postgres_types::ToSql;
use rustc_hash::FxHashSet;
use std::sync::Arc;

#[derive(Clone)]
pub struct ChampionshipService {
    db: Database,
    cache: RedisCache,
    ports: Arc<RwLock<FxHashSet<i32>>>,
    user_repository: UserRepository,
    championship_repository: ChampionshipRepository,
}

impl ChampionshipService {
    pub async fn new(db_conn: &Database, cache: &RedisCache) -> Self {
        let championship_repository = ChampionshipRepository::new(db_conn, cache).await;
        let user_repository = UserRepository::new(db_conn, cache);

        let ports = Self::available_ports(&championship_repository)
            .await
            .unwrap();

        Self {
            db: db_conn.clone(),
            cache: cache.clone(),
            user_repository,
            championship_repository,
            ports: Arc::new(RwLock::new(ports)),
        }
    }

    pub async fn create(&self, payload: CreateChampionshipDto, user_id: &i32) -> AppResult<()> {
        let port = self.get_port().await?;
        let id = fastrand::i32(700000000..799999999);

        if self
            .championship_repository
            .find_by_name(&payload.name)
            .await?
            .is_some()
        {
            Err(ChampionshipError::AlreadyExists)?
        };

        {
            let conn = self.db.pg.get().await?;

            let cached_task = conn.prepare_cached(
                r#"
                    INSERT INTO championship (id, port, name, category, season, owner_id)
                    VALUES ($1,$2,$3,$4,$5,$6)
                "#,
            );

            let cached2_task = conn.prepare_cached(
                r#"
                    INSERT INTO user_championships (user_id, championship_id)
                    VALUES ($1,$2)
                "#,
            );

            let (cached, cached2) = tokio::try_join!(cached_task, cached2_task)?;

            conn.execute(
                &cached,
                &[
                    &id,
                    &port,
                    &payload.name,
                    &payload.category,
                    &payload.season,
                    user_id,
                ],
            )
            .await?;

            conn.execute(&cached2, &[user_id, &id]).await?;
        }

        let remove_port_task = self.remove_port(port);
        let delete_by_user_id_task = self.cache.championship.delete_by_user_id(user_id);

        tokio::try_join!(remove_port_task, delete_by_user_id_task)?;

        Ok(())
    }

    pub async fn update(
        &self,
        id: &i32,
        user_id: &i32,
        form: &UpdateChampionship,
    ) -> AppResult<()> {
        {
            let Some(championship) = self.championship_repository.find(id).await? else {
                Err(ChampionshipError::NotFound)?
            };

            if Utc::now().signed_duration_since(championship.updated_at) <= Duration::days(7) {
                Err(ChampionshipError::IntervalNotReached)?
            };

            if championship.owner_id != *user_id {
                Err(ChampionshipError::NotOwner)?
            }
        }

        let (query, params) = {
            let mut counter = 1;
            let mut query = String::from("UPDATE championship SET ");
            let mut params: Vec<&(dyn ToSql + Sync)> = Vec::new();

            if let Some(name) = &form.name {
                if counter > 1 {
                    query.push(',');
                }

                query.push_str(&format!(" name = ${}", counter));
                params.push(name);
                counter += 1;
            }

            if let Some(category) = &form.category {
                if counter > 1 {
                    query.push(',');
                }

                query.push_str(&format!(" category = ${}", counter));
                params.push(category);
                counter += 1;
            }

            if let Some(season) = &form.season {
                if counter > 1 {
                    query.push(',');
                }

                query.push_str(&format!(" season = ${}", counter));
                params.push(season);
                counter += 1;
            }

            if counter == 1 {
                Err(CommonError::NotValidUpdate)?
            }

            query.push_str(&format!(" WHERE id = ${}", counter));
            params.push(id);

            // Check if owner_id is the same as user_id
            query.push_str(&format!(" AND owner_id = ${}", counter + 1));
            params.push(user_id);

            (query, params)
        };

        {
            let conn = self.db.pg.get().await?;
            let cached_statement = conn.prepare_cached(&query).await?;
            conn.execute(&cached_statement, &params).await?;
        }

        let users = self.championship_repository.users(id).await?;
        // Todo: Check if this is working as expected
        self.cache.championship.delete_all(id, users).await?;

        Ok(())
    }

    pub async fn add_user(&self, id: &i32, user_id: &i32, bind_user_email: &str) -> AppResult<()> {
        {
            let Some(championship) = self.championship_repository.find(id).await? else {
                Err(ChampionshipError::NotFound)?
            };

            if championship.owner_id != *user_id {
                Err(ChampionshipError::NotOwner)?
            }
        }

        let new_user_id = {
            let Some(bind_user) = self.user_repository.find_by_email(bind_user_email).await? else {
                Err(UserError::NotFound)?
            };

            bind_user.id
        };

        let conn = self.db.pg.get().await?;

        let cached_statement = conn
            .prepare_cached(
                r#"
                    INSERT INTO user_championships (user_id, championship_id)
                    VALUES ($1,$2)
                "#,
            )
            .await?;

        let bindings: [&(dyn ToSql + Sync); 2] = [&new_user_id, id];
        let add_user_future = conn.execute(&cached_statement, &bindings);

        // Delete New User Championship Cache
        let delete_user_cache_future = self.cache.championship.delete_by_user_id(&new_user_id);

        let (add_user_res, delete_user_cache_res) =
            tokio::join!(add_user_future, delete_user_cache_future);

        add_user_res?;
        delete_user_cache_res?;

        Ok(())
    }

    pub async fn remove_user(
        &self,
        id: &i32,
        user_id: &i32,
        remove_user_id: &i32,
    ) -> AppResult<()> {
        {
            let Some(championship) = self.championship_repository.find(id).await? else {
                Err(ChampionshipError::NotFound)?
            };

            if championship.owner_id != *user_id {
                Err(ChampionshipError::NotOwner)?
            }

            if championship.owner_id == *remove_user_id {
                Err(ChampionshipError::CannotRemoveOwner)?
            }
        }

        if self.user_repository.find(remove_user_id).await?.is_none() {
            Err(UserError::NotFound)?
        };

        let conn = self.db.pg.get().await?;

        let cached_statement = conn
            .prepare_cached(
                r#"
                    DELETE FROM user_championships WHERE user_id = $1 AND championship_id = $2
                "#,
            )
            .await?;

        let bindings: [&(dyn ToSql + Sync); 2] = [remove_user_id, id];
        let remove_user_future = conn.execute(&cached_statement, &bindings);
        // Delete Removed User Championship Cache
        let remove_user_cache_future = self.cache.championship.delete_by_user_id(remove_user_id);

        let (remove_user, remove_user_cache) =
            tokio::join!(remove_user_future, remove_user_cache_future);

        remove_user?;
        remove_user_cache?;

        Ok(())
    }

    pub async fn delete(&self, id: &i32) -> AppResult<()> {
        let conn = self.db.pg.get().await?;

        let cached_task = conn.prepare_cached(
            r#"
                    DELETE FROM user_championships WHERE championship_id = $1
                "#,
        );

        let cached2_task = conn.prepare_cached(
            r#"
                    DELETE FROM championship WHERE id = $1
                "#,
        );

        let bindings: [&(dyn ToSql + Sync); 1] = [id];
        let (cached, cached_2) = tokio::try_join!(cached_task, cached2_task)?;

        let users = self.championship_repository.users(id).await?;
        let remove_championship_users_fut = conn.execute(&cached, &bindings);
        let delete_champ_cache_fut = self.cache.championship.delete_all(id, users);

        let (remove_championship_users_fut, delete_champ_cache) =
            tokio::join!(remove_championship_users_fut, delete_champ_cache_fut);

        remove_championship_users_fut?;
        delete_champ_cache?;

        conn.execute(&cached_2, &bindings).await?;
        info!("Championship deleted with success: {id}");

        Ok(())
    }

    async fn available_ports(
        championship_repository: &ChampionshipRepository,
    ) -> AppResult<FxHashSet<i32>> {
        let mut all_ports: FxHashSet<i32> = (20777..=20850).collect();
        let ports_in_use = championship_repository.ports_in_use().await?;

        for port in ports_in_use {
            all_ports.remove(&port);
        }

        info!("Available ports: {:?}", all_ports);
        Ok(all_ports)
    }

    async fn get_port(&self) -> AppResult<i32> {
        let ports = self.ports.read();

        if let Some(port) = ports.iter().next() {
            Ok(*port)
        } else {
            Err(CommonError::NotPortsAvailable)?
        }
    }

    async fn remove_port(&self, port: i32) -> AppResult<()> {
        let mut ports = self.ports.write();
        ports.remove(&port);

        Ok(())
    }
}
