use crate::{config::Database, entity::EventData, error::AppResult};
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
    pub async fn events_data(&self, id: i64) -> AppResult<Vec<EventData>> {
        // TODO: Parse all data into their codes
        let event_data = sqlx::query_as::<_, EventData>(
            r#"
                SELECT * FROM event_data
                WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_all(&self.database.mysql)
        .await?;

        Ok(event_data)
    }
}
