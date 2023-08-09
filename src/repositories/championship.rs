use redis::AsyncCommands;
use scylla::transport::session::TypedRowIter;

use crate::{config::Database, entity::Championship, error::AppResult};
use std::sync::Arc;

#[derive(Clone)]
pub struct ChampionshipRepository {
    database: Arc<Database>,
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
            .execute(
                self.database.statements.get("championships_ports").unwrap(),
                &[],
            )
            .await?
            .rows_typed_or_empty::<(i16,)>();

        Ok(ports_in_use)
    }

    pub async fn find(&self, id: &i32) -> AppResult<Championship> {
        let championship = self
            .database
            .get_scylla()
            .execute(
                self.database.statements.get("championship_by_id").unwrap(),
                (id,),
            )
            .await?
            .single_row_typed::<Championship>()?;

        Ok(championship)
    }

    pub async fn session_exists(&self, id: &i32, session_id: i64) -> AppResult<bool> {
        let res: bool = self
            .database
            .get_redis_async()
            .await
            .exists(format!(
                "f123:championship:{id}:session:{session_id}:motion" // Motion is a key that is always present
            ))
            .await
            .unwrap();

        Ok(res)
    }

    pub async fn championships_exists(&self, name: &str) -> AppResult<bool> {
        let rows = self
            .database
            .get_scylla()
            .execute(
                self.database
                    .statements
                    .get("championship_name_by_name")
                    .unwrap(),
                (name,),
            )
            .await?
            .rows_num()?;

        Ok(rows > 0)
    }
}
