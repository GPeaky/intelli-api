use crate::{config::Database, error::AppResult};
use std::sync::Arc;

pub struct F123Repository {
    #[allow(unused)]
    database: Arc<Database>,
}

impl F123Repository {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            database: db_conn.clone(),
        }
    }

    #[allow(unused)]
    pub async fn events_data(&self, id: i64) -> AppResult<()> {
        // // TODO: Parse all data into their codes
        // let event_data = sqlx::query_as::<_, EventData>(
        //     r#"
        //         SELECT * FROM event_data
        //         WHERE id = ?
        //     "#,
        // )
        // .bind(id)
        // .fetch_all(&self.database.pg)
        // .await?;

        // Ok(event_data)

        todo!()
    }
}
