use tokio_stream::StreamExt;

use crate::{cache::ServiceCache, config::Database, error::AppResult, utils::slice_iter};

#[derive(Clone)]
pub struct SavedSessionRepository {
    db: &'static Database,
    #[allow(unused)]
    cache: &'static ServiceCache,
}

impl SavedSessionRepository {
    pub fn new(db: &'static Database, cache: &'static ServiceCache) -> Self {
        Self { db, cache }
    }

    /// This method should only be called once
    pub async fn used_ids(&self) -> AppResult<Vec<i32>> {
        let conn = self.db.pg.get().await?;

        let saved_session_ids_stmt = conn
            .prepare_cached(
                r#"
                    SELECT id FROM saved_sessions
                "#,
            )
            .await?;

        let stream = conn
            .query_raw(&saved_session_ids_stmt, slice_iter(&[]))
            .await?;

        tokio::pin!(stream);

        let mut saved_session_ids =
            Vec::with_capacity(stream.rows_affected().unwrap_or(0) as usize);

        while let Some(row) = stream.try_next().await? {
            saved_session_ids.push(row.get(0));
        }

        Ok(saved_session_ids)
    }
}
