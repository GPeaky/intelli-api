use crate::{
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
    ports: Arc<RwLock<FxHashSet<u16>>>,
    #[allow(unused)]
    championship_repository: ChampionshipRepository,
}

impl ChampionshipService {
    pub async fn new(db_conn: &Arc<Database>) -> Self {
        let championship_repository: ChampionshipRepository =
            ChampionshipRepository::new(db_conn).await;

        let ports = Self::available_ports(&championship_repository)
            .await
            .unwrap();

        Self {
            ports: Arc::new(RwLock::new(ports)),
            db: db_conn.clone(),
            championship_repository,
        }
    }

    pub async fn create_championship(
        &self,
        payload: CreateChampionshipDto,
        user_id: &u32,
    ) -> AppResult<()> {
        let port = self.get_port().await?;
        let mut rand = StdRng::from_entropy();
        let id: u32 = rand.gen_range(600000000..700000000);

        sqlx::query(
            r#"
                INSERT INTO championship (id, port, name, owner_id)
                VALUES (?,?,?,?)
            "#,
        )
        .bind(id)
        .bind(port)
        .bind(payload.name)
        .bind(user_id)
        .execute(&self.db.mysql)
        .await?;

        sqlx::query(
            r#"
                INSERT INTO user_championships (user_id, championship_id)
                VALUES (?,?)
            "#,
        )
        .bind(user_id)
        .bind(id)
        .execute(&self.db.mysql)
        .await?;

        self.remove_port(port).await?;

        Ok(())
    }

    pub async fn delete_championship(&self, id: &u32) -> AppResult<()> {
        sqlx::query(
            r#"
                DELETE FROM championship WHERE id = ?
            "#,
        )
        .bind(id)
        .execute(&self.db.mysql)
        .await?;

        info!("Championship deleted with success: {id}");

        Ok(())
    }

    async fn available_ports(
        championship_repository: &ChampionshipRepository,
    ) -> AppResult<FxHashSet<u16>> {
        let mut all_ports: FxHashSet<u16> = (20777..=20850).collect();
        let ports_in_use = championship_repository.ports_in_use().await?;

        for (port,) in ports_in_use {
            all_ports.remove(&port);
        }

        info!("Available ports: {:?}", all_ports);
        Ok(all_ports)
    }

    async fn get_port(&self) -> AppResult<u16> {
        let ports = self.ports.read().await;

        if let Some(port) = ports.iter().next() {
            Ok(*port)
        } else {
            Err(CommonError::NotPortsAvailable)?
        }
    }

    async fn remove_port(&self, port: u16) -> AppResult<()> {
        let mut ports = self.ports.write().await;
        ports.remove(&port);

        Ok(())
    }
}
