use scylla::transport::session::TypedRowIter;

use crate::{config::Database, error::AppResult};
use std::sync::Arc;

#[derive(Clone)]
pub struct ChampionshipRepository {
    pub database: Arc<Database>,
}

impl ChampionshipRepository {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            database: db_conn.clone(),
        }
    }

    pub async fn ports_in_use(&self) -> AppResult<TypedRowIter<(i16,)>> {
        let ports_in_use = self
            .database
            .get_scylla()
            .execute(self.database.statements.get("select_ports").unwrap(), &[])
            .await?
            .rows_typed_or_empty::<(i16,)>();

        Ok(ports_in_use)
    }

    pub async fn championships_exists(&self, name: &str) -> AppResult<bool> {
        let rows = self
            .database
            .get_scylla()
            .execute(
                self.database.statements.get("select_name_by_name").unwrap(),
                (name,),
            )
            .await?
            .rows_num()?;

        Ok(rows > 0)
    }
}
