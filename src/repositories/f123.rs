use crate::{
    config::Database,
    dtos::{EventDataStatements, PreparedStatementsKey},
    entity::EventData,
    error::AppResult,
};
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

    #[allow(unused)]
    pub async fn events_data(&self, id: i64) -> AppResult<TypedRowIter<EventData>> {
        let events_dat = self
            .database
            .scylla
            .execute(
                self.database
                    .statements
                    .get(&PreparedStatementsKey::EventData(EventDataStatements::Info))
                    .unwrap(),
                (id,),
            )
            .await?
            .rows_typed::<EventData>()?;

        Ok(events_dat)
    }
}
