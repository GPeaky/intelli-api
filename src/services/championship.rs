use crate::{
    cache::{EntityCache, RedisCache},
    config::Database,
    dtos::CreateChampionshipDto,
    error::{AppResult, CommonError},
    repositories::ChampionshipRepository,
};
use rand::{rngs::StdRng, Rng, SeedableRng};
use rustc_hash::FxHashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

pub struct ChampionshipService {
    db: Arc<Database>,
    ports: Arc<RwLock<FxHashSet<i32>>>,
    #[allow(unused)]
    cache: Arc<RedisCache>,
    championship_repository: ChampionshipRepository,
}

impl ChampionshipService {
    pub async fn new(db_conn: &Arc<Database>, cache: &Arc<RedisCache>) -> Self {
        let championship_repository: ChampionshipRepository =
            ChampionshipRepository::new(db_conn, cache).await;

        let ports = Self::available_ports(&championship_repository)
            .await
            .unwrap();

        Self {
            db: db_conn.clone(),
            cache: cache.clone(),
            championship_repository,
            ports: Arc::new(RwLock::new(ports)),
        }
    }

    pub async fn create_championship(
        &self,
        payload: CreateChampionshipDto,
        user_id: &i32,
    ) -> AppResult<()> {
        let port = self.get_port().await?;
        let mut rand = StdRng::from_entropy();
        let id: i32 = rand.gen_range(600000000..700000000);

        self.championship_repository
            .exist_by_name(&payload.name)
            .await?;

        sqlx::query(
            r#"
                INSERT INTO championship (id, port, name, category, season, owner_id)
                VALUES ($1,$2,$3,$4,$5,$6)
            "#,
        )
        .bind(id)
        .bind(port)
        .bind(payload.name)
        .bind(payload.category)
        .bind(payload.season)
        .bind(user_id)
        .execute(&self.db.pg)
        .await?;

        sqlx::query(
            r#"
                INSERT INTO user_championships (user_id, championship_id)
                VALUES ($1,$2)
            "#,
        )
        .bind(user_id)
        .bind(id)
        .execute(&self.db.pg)
        .await?;

        self.remove_port(port).await?;
        self.cache.championship.delete_by_user_id(user_id).await?;

        Ok(())
    }

    pub async fn delete_championship(&self, id: &i32) -> AppResult<()> {
        sqlx::query(
            r#"
                DELETE FROM championship WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.db.pg)
        .await?;

        self.cache.championship.delete(id).await?;
        info!("Championship deleted with success: {id}");

        Ok(())
    }

    async fn available_ports(
        championship_repository: &ChampionshipRepository,
    ) -> AppResult<FxHashSet<i32>> {
        let mut all_ports: FxHashSet<i32> = (20777..=20850).collect();
        let ports_in_use = championship_repository.ports_in_use().await?;

        for (port,) in ports_in_use {
            all_ports.remove(&port);
        }

        info!("Available ports: {:?}", all_ports);
        Ok(all_ports)
    }

    async fn get_port(&self) -> AppResult<i32> {
        let ports = self.ports.read().await;

        if let Some(port) = ports.iter().next() {
            Ok(*port)
        } else {
            Err(CommonError::NotPortsAvailable)?
        }
    }

    async fn remove_port(&self, port: i32) -> AppResult<()> {
        let mut ports = self.ports.write().await;
        ports.remove(&port);

        Ok(())
    }
}
