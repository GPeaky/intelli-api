use crate::{
    config::Database, dtos::CreateChampionshipDto, entity::Championship, error::AppResult,
    repositories::ChampionshipRepository,
};
use rand::{rngs::StdRng as Rand, Rng, SeedableRng};
use std::sync::Arc;
// use redis::Commands;
use tokio::sync::RwLock;
use tracing::info;

#[derive(Clone)]
// TODO: Fix this service to change things without begin mutable
pub struct ChampionshipService {
    db: Arc<Database>,
    ports: Arc<RwLock<Vec<i16>>>,
    #[allow(unused)]
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
        // todo: restrict port to receive only one connection, and release it when the connection is closed
        let mut rng = Rand::from_entropy();
        let port = self.get_port().await?;
        let id = rng.gen::<i32>();

        sqlx::query(
            r#"
                INSERT INTO championship (id, port, name, user_id)
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

    pub async fn delete_championship(&self, id: &i32) -> AppResult<()> {
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

    pub async fn user_championships(&self, user_id: &i32) -> AppResult<Vec<Championship>> {
        let championships = sqlx::query_as::<_, Championship>(
            r#"
                SELECT
                    c.*
                FROM
                    championship c
                JOIN
                    user_championships uc ON c.id = uc.championship_id
                WHERE
                    uc.user_id = ?
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.db.mysql)
        .await?;

        Ok(championships)
    }

    async fn available_ports(
        championship_repository: &ChampionshipRepository,
    ) -> AppResult<Arc<RwLock<Vec<i16>>>> {
        let mut ports: Vec<i16> = (20777..=20899).collect();
        let ports_in_use = championship_repository.ports_in_use().await?;

        for port in ports_in_use {
            let port_index = ports.iter().position(|&p| p.eq(&port.0)).unwrap();

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
