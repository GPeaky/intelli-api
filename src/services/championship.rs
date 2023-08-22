use crate::{
    config::Database,
    dtos::{ChampionshipStatements, CreateChampionshipDto, PreparedStatementsKey},
    entity::Championship,
    error::{AppResult, ChampionshipError},
    repositories::ChampionshipRepository,
};
use chrono::Utc;
use rand::{rngs::StdRng as Rand, Rng, SeedableRng};
use scylla::transport::session::TypedRowIter;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

#[derive(Clone)]
// TODO: Fix this service to change things without begin mutable
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
        &self,
        payload: CreateChampionshipDto,
        user_id: &i32,
    ) -> AppResult<()> {
        let mut rng = Rand::from_entropy();
        let championship_exist = self
            .championship_repository
            .championships_exists(&payload.name)
            .await?;

        if championship_exist {
            return Err(ChampionshipError::AlreadyExists)?;
        }

        // todo: restrict port to receive only one connection, and release it when the connection is closed
        let time = Utc::now();
        let port = self.get_port().await?;

        self.db
            .scylla
            .execute(
                self.db
                    .statements
                    .get(&PreparedStatementsKey::Championship(
                        ChampionshipStatements::Insert,
                    ))
                    .unwrap(),
                (rng.gen::<i32>(), payload.name, port, user_id, time, time),
            )
            .await?;

        self.remove_port(port).await?;

        Ok(())
    }

    pub async fn delete_championship(&self, id: &i32) -> AppResult<()> {
        self.db
            .scylla
            .execute(
                self.db
                    .statements
                    .get(&PreparedStatementsKey::Championship(
                        ChampionshipStatements::Delete,
                    ))
                    .unwrap(),
                (id,),
            )
            .await?;

        info!("Championship deleted with success: {id}");

        Ok(())
    }

    pub async fn user_championships(&self, user_id: &i32) -> AppResult<TypedRowIter<Championship>> {
        let championships = self
            .db
            .scylla
            .execute(
                self.db
                    .statements
                    .get(&PreparedStatementsKey::Championship(
                        ChampionshipStatements::ByUser,
                    ))
                    .unwrap(),
                (user_id,),
            )
            .await?
            .rows_typed::<Championship>()?;

        Ok(championships)
    }

    async fn available_ports(
        championship_repository: &ChampionshipRepository,
    ) -> AppResult<Arc<RwLock<Vec<i16>>>> {
        let mut ports: Vec<i16> = (20777..=20899).collect();
        let ports_in_use = championship_repository.ports_in_use().await?;

        for port in ports_in_use {
            let port_index = ports
                .iter()
                .position(|&p| p.eq(&port.clone().unwrap().0))
                .unwrap();

            ports.remove(port_index);
        }

        info!("Available ports: {:?}", ports);
        Ok(Arc::new(RwLock::new(ports)))
    }

    async fn get_port(&self) -> AppResult<i16> {
        let ports = self.ports.read().await;
        Ok(*ports.first().unwrap())
    }

    async fn remove_port(&self, port: i16) -> AppResult<()> {
        let mut ports = self.ports.write().await;
        let port_index = ports.iter().position(|&p| p.eq(&port)).unwrap();

        ports.remove(port_index);
        Ok(())
    }
}
