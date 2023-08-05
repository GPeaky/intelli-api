use crate::{config::Database, entity::EventData, error::AppResult};
use scylla::transport::session::TypedRowIter;
use std::sync::Arc;

#[derive(Clone)]
pub struct F123Repository {
    database: Arc<Database>,
}

impl F123Repository {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            database: db_conn.clone(),
        }
    }

    pub async fn events_data(&self, id: i64) -> AppResult<TypedRowIter<EventData>> {
        let events_dat = self
            .database
            .get_scylla()
            .execute(self.database.statements.get("events_data").unwrap(), (id,))
            .await?
            .rows_typed::<EventData>()?;

        Ok(events_dat)
    }
}
