use std::sync::Arc;

use ahash::AHashSet;
use chrono::{Duration, Utc};
use parking_lot::RwLock;
use postgres_types::ToSql;
use tracing::info;

use crate::{
    cache::RedisCache,
    config::Database,
    structs::{CreateChampionshipDto, UpdateChampionship},
    error::{AppResult, ChampionshipError, CommonError, UserError},
    repositories::{ChampionshipRepository, UserRepository, UserRepositoryTrait},
};

#[derive(Clone)]
pub struct ChampionshipService {
    db: Database,
    cache: RedisCache,
    ports: Arc<RwLock<AHashSet<i32>>>,
    user_repository: UserRepository,
    championship_repository: ChampionshipRepository,
}

impl ChampionshipService {
    pub async fn new(db_conn: &Database, cache: &RedisCache) -> Self {
        let user_repository = UserRepository::new(db_conn, cache);
        let championship_repository = ChampionshipRepository::new(db_conn, cache).await;

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

            let create_championship_stmt_fut = conn.prepare_cached(
                r#"
                    INSERT INTO championship (id, port, name, category, season, owner_id)
                    VALUES ($1,$2,$3,$4,$5,$6)
                "#,
            );

            let relate_user_with_championship_stmt_fut = conn.prepare_cached(
                r#"
                    INSERT INTO user_championships (user_id, championship_id)
                    VALUES ($1,$2)
                "#,
            );

            let (create_championship_stmt, relate_user_with_championship_stmt) = tokio::try_join!(
                create_championship_stmt_fut,
                relate_user_with_championship_stmt_fut
            )?;

            conn.execute(
                &create_championship_stmt,
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

            conn.execute(&relate_user_with_championship_stmt, &[user_id, &id])
                .await?;
        }

        let remove_port_fut = self.remove_port(port);
        let delete_by_user_id_fut = self.cache.championship.delete_by_user_id(user_id);

        tokio::try_join!(remove_port_fut, delete_by_user_id_fut)?;

        Ok(())
    }

    pub async fn update(
        &self,
        id: &i32,
        user_id: &i32,
        form: &UpdateChampionship,
    ) -> AppResult<()> {
        // Scope to check if championship exists and if user is owner
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
            let mut params: Vec<&(dyn ToSql + Sync)> = Vec::new();
            let mut query = String::from("UPDATE championship SET ");

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

        // Scope to update championship
        {
            let conn = self.db.pg.get().await?;
            let update_championship_stmt = conn.prepare_cached(&query).await?;
            conn.execute(&update_championship_stmt, &params).await?;
        }

        let users = self.championship_repository.users(id).await?;
        // Todo: Check if this is working as expected
        self.cache.championship.delete_all(id, users).await?;

        Ok(())
    }

    pub async fn add_user(&self, id: &i32, user_id: &i32, bind_user_email: &str) -> AppResult<()> {
        // Scope to check if championship exists and if user is owner
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

        let add_user_stmt = conn
            .prepare_cached(
                r#"
                    INSERT INTO user_championships (user_id, championship_id)
                    VALUES ($1,$2)
                "#,
            )
            .await?;

        let add_user_fut = async {
            let bindings: [&(dyn ToSql + Sync); 2] = [&new_user_id, id];
            conn.execute(&add_user_stmt, &bindings).await?;
            Ok(())
        };

        let delete_user_cache_fut = self.cache.championship.delete_by_user_id(&new_user_id);
        tokio::try_join!(add_user_fut, delete_user_cache_fut)?;
        Ok(())
    }

    pub async fn remove_user(
        &self,
        id: &i32,
        user_id: &i32,
        remove_user_id: &i32,
    ) -> AppResult<()> {
        // Scope to check if championship exists and if user is owner
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

        let remove_user_stmt = conn
            .prepare_cached(
                r#"
                    DELETE FROM user_championships WHERE user_id = $1 AND championship_id = $2
                "#,
            )
            .await?;

        let remove_user_fut = async {
            let bindings: [&(dyn ToSql + Sync); 2] = [remove_user_id, id];
            conn.execute(&remove_user_stmt, &bindings).await?;
            Ok(())
        };
        let remove_user_cache_fut = self.cache.championship.delete_by_user_id(remove_user_id);

        tokio::try_join!(remove_user_fut, remove_user_cache_fut)?;
        Ok(())
    }

    pub async fn delete(&self, id: &i32) -> AppResult<()> {
        let conn = self.db.pg.get().await?;

        let delete_championship_relations_stmt_fut = conn.prepare_cached(
            r#"
                DELETE FROM user_championships WHERE championship_id = $1
            "#,
        );

        let delete_championship_stmt_fut = conn.prepare_cached(
            r#"
                DELETE FROM championship WHERE id = $1
            "#,
        );

        let bindings: [&(dyn ToSql + Sync); 1] = [id];
        let (delete_championship_relations_stmt, delete_championship_stmt) = tokio::try_join!(
            delete_championship_relations_stmt_fut,
            delete_championship_stmt_fut
        )?;

        let users = self.championship_repository.users(id).await?;
        let remove_championship_users_fut = async {
            conn.execute(&delete_championship_relations_stmt, &bindings)
                .await?;
            Ok(())
        };
        let delete_champ_cache_fut = self.cache.championship.delete_all(id, users);

        tokio::try_join!(remove_championship_users_fut, delete_champ_cache_fut)?;
        conn.execute(&delete_championship_stmt, &bindings).await?;
        info!("Championship deleted with success: {id}");

        Ok(())
    }

    async fn available_ports(
        championship_repository: &ChampionshipRepository,
    ) -> AppResult<AHashSet<i32>> {
        let mut all_ports: AHashSet<i32> = (20777..=20850).collect();
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
