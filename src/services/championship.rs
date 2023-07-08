use crate::{
    config::Database,
    dtos::CreateChampionshipDto,
    error::{AppResult, ChampionshipError},
    repositories::ChampionshipRepository,
};
use chrono::Utc;
use rs_nanoid::standard;
use std::sync::Arc;

#[derive(Clone)]
pub struct ChampionshipService {
    pub db: Arc<Database>,
    pub championship_repository: ChampionshipRepository,
}

impl ChampionshipService {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            db: db_conn.clone(),
            championship_repository: ChampionshipRepository::new(db_conn),
        }
    }

    pub async fn create_championship(
        &self,
        payload: CreateChampionshipDto,
        user_id: &str,
    ) -> AppResult<()> {
        // todo: user port of static hashmap
        let port = 2770;
        let time = Utc::now().timestamp();
        let championship_exist = self
            .championship_repository
            .championships_exists(&payload.name)
            .await?;

        if championship_exist {
            return Err(ChampionshipError::AlreadyExists)?;
        }

        self.db
            .get_scylla()
            .execute(
                self.db.statements.get("insert_championship").unwrap(),
                (
                    standard::<16>().to_string(),
                    payload.name,
                    port,
                    user_id,
                    time,
                    time,
                ),
            )
            .await?;

        Ok(())
    }
}
