use crate::{
    config::Database,
    dtos::CreateChampionshipDto,
    error::{AppResult, ChampionshipError},
    repositories::ChampionshipRepository,
};
use chrono::Utc;
use frand::Rand;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct ChampionshipService {
    db: Arc<Database>,
    ports: Arc<RwLock<Vec<i16>>>,
    championship_repository: ChampionshipRepository,
}

impl ChampionshipService {
    pub async fn new(db_conn: &Arc<Database>) -> Self {
        let championship_repository: ChampionshipRepository = ChampionshipRepository::new(db_conn);
        let ports = Self::available_ports(&championship_repository)
            .await
            .unwrap();

        Self {
            ports,
            db: db_conn.clone(),
            championship_repository,
        }
    }

    pub async fn create_championship(
        &mut self,
        payload: CreateChampionshipDto,
        user_id: &i64,
    ) -> AppResult<()> {
        let mut rng = Rand::new();
        let championship_exist = self
            .championship_repository
            .championships_exists(&payload.name)
            .await?;

        if championship_exist {
            return Err(ChampionshipError::AlreadyExists)?;
        }

        // todo: restrict port to receive only one connection, and release it when the connection is closed
        let port = self.get_port().await?;
        let time = Utc::now().timestamp();

        self.db
            .get_scylla()
            .execute(
                self.db.statements.get("insert_championship").unwrap(),
                (rng.gen::<i64>(), payload.name, port, user_id, time, time),
            )
            .await?;

        self.remove_port(port).await?;

        Ok(())
    }

    async fn available_ports(
        championship_repository: &ChampionshipRepository,
    ) -> AppResult<Arc<RwLock<Vec<i16>>>> {
        let mut ports: Vec<i16> = (20777..=20899).collect();
        let ports_in_use = championship_repository.ports_in_use().await?;

        for port in ports_in_use {
            let port_index = ports
                .iter()
                .position(|&p| p == port.clone().unwrap().0)
                .unwrap();

            ports.remove(port_index);
        }

        Ok(Arc::new(RwLock::new(ports)))
    }

    async fn get_port(&self) -> AppResult<i16> {
        let ports = self.ports.read().await;
        Ok(*ports.first().unwrap())
    }

    async fn remove_port(&self, port: i16) -> AppResult<()> {
        let mut ports = self.ports.write().await;
        let port_index = ports.iter().position(|&p| p == port).unwrap();

        ports.remove(port_index);
        Ok(())
    }
}
