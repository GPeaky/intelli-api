use crate::{config::Database, entity::Championship, error::AppResult};
use scylla::transport::session::TypedRowIter;
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
            .scylla
            .execute(
                self.database.statements.get("championship.ports").unwrap(),
                &[],
            )
            .await?
            .rows_typed_or_empty::<(i16,)>();

        Ok(ports_in_use)
    }

    pub async fn find(&self, id: &i32) -> AppResult<Championship> {
        let championship = self
            .database
            .scylla
            .execute(
                self.database.statements.get("championship.by_id").unwrap(),
                (id,),
            )
            .await?
            .single_row_typed::<Championship>()?;

        Ok(championship)
    }

    pub async fn find_all(&self, user_id: &i32) -> AppResult<TypedRowIter<Championship>> {
        let championships = self
            .database
            .scylla
            .execute(
                self.database
                    .statements
                    .get("championships_by_user_id")
                    .unwrap(),
                (user_id,),
            )
            .await?
            .rows_typed::<Championship>()?;

        Ok(championships)
    }

    pub async fn championships_exists(&self, name: &str) -> AppResult<bool> {
        let rows = self
            .database
            .scylla
            .execute(
                self.database
                    .statements
                    .get("championship.name_by_name")
                    .unwrap(),
                (name,),
            )
            .await?
            .rows_num()?;

        Ok(rows > 0)
    }
}
