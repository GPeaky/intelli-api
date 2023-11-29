use crate::{
    cache::{EntityCache, RedisCache},
    config::Database,
    dtos::{CreateChampionshipDto, UpdateChampionship},
    error::{AppResult, ChampionshipError, CommonError, UserError},
    repositories::{ChampionshipRepository, UserRepository, UserRepositoryTrait},
};
use log::info;
use parking_lot::RwLock;
use postgres_types::ToSql;
use rand::{rngs::StdRng, Rng, SeedableRng};
use rustc_hash::FxHashSet;
use std::sync::Arc;

pub struct ChampionshipService {
    db: Arc<Database>,
    cache: Arc<RedisCache>,
    ports: Arc<RwLock<FxHashSet<i32>>>,
    user_repository: UserRepository,
    championship_repository: ChampionshipRepository,
}

impl ChampionshipService {
    pub async fn new(db_conn: &Arc<Database>, cache: &Arc<RedisCache>) -> Self {
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
        let id = {
            let mut rand = StdRng::from_entropy();
            rand.gen_range(600000000..700000000)
        };

        self.championship_repository
            .exist_by_name(&payload.name)
            .await?;

        {
            let conn = self.db.pg.get().await?;

            let cached_statement = conn
                .prepare_cached(
                    r#"
                    INSERT INTO championship (id, port, name, category, season, owner_id)
                    VALUES ($1,$2,$3,$4,$5,$6)
                "#,
                )
                .await?;

            conn.execute(
                &cached_statement,
                &[
                    &id,
                    &port,
                    &payload.name,
                    &payload.category,
                    &payload.season,
                    &user_id,
                ],
            )
            .await?;

            conn.execute(
                r#"
                    INSERT INTO user_championships (user_id, championship_id)
                    VALUES ($1,$2)
                "#,
                &[&user_id, &id],
            )
            .await?;
        }

        self.remove_port(port).await?;
        self.cache.championship.delete_by_user_id(user_id).await?;

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

        let del_task = self.cache.championship.delete(id);
        let del_by_id_task = self.cache.championship.delete_by_user_id(user_id);

        tokio::try_join!(del_task, del_by_id_task)?;

        Ok(())
    }

    // TODO: Check if we receive new_user id or email
    pub async fn add_user(&self, id: &i32, user_id: &i32, bind_user_email: &str) -> AppResult<()> {
        {
            let Some(championship) = self.championship_repository.find(id).await? else {
                Err(ChampionshipError::NotFound)?
            };

            if championship.owner_id != *user_id {
                Err(ChampionshipError::NotOwner)?
            }
        }

        let Some(bind_user) = self.user_repository.find_by_email(bind_user_email).await? else {
            Err(UserError::NotFound)?
        };

        {
            let conn = self.db.pg.get().await?;

            let cached_statement = conn
                .prepare_cached(
                    r#"
                    INSERT INTO user_championships (user_id, championship_id)
                    VALUES ($1,$2)
                "#,
                )
                .await?;

            conn.execute(&cached_statement, &[&bind_user.id, &id])
                .await?;
        }

        self.cache.championship.delete(id).await?;
        let del_by_user_id_task = self.cache.championship.delete_by_user_id(user_id);
        let del_by_new_user_id_task = self.cache.championship.delete_by_user_id(&bind_user.id);

        tokio::try_join!(del_by_user_id_task, del_by_new_user_id_task)?;

        Ok(())
    }

    pub async fn delete(&self, id: &i32) -> AppResult<()> {
        {
            let conn = self.db.pg.get().await?;

            let cached_statement = conn
                .prepare_cached(
                    r#"
                    DELETE FROM championship WHERE id = $1
                "#,
                )
                .await?;

            let cached_2 = conn
                .prepare_cached(
                    r#"
                    DELETE FROM user_championships WHERE championship_id = $1
                "#,
                )
                .await?;

            conn.execute(&cached_statement, &[&id]).await?;
            conn.execute(&cached_2, &[&id]).await?;
        }

        self.cache.championship.delete(id).await?;
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
