use crate::{cache::RedisCache, config::Database, error::AppResult, utils::UsedIds};

#[derive(Clone)]
pub struct SavedSessionRepository {
    db: &'static Database,
    #[allow(unused)]
    cache: &'static RedisCache,
}

impl UsedIds for SavedSessionRepository {
    async fn used_ids(&self) -> AppResult<Vec<i32>> {
        let conn = self.db.pg.get().await?;

        let saved_session_ids_stmt = conn
            .prepare_cached(
                r#"
                    SELECT id FROM saved_session
                "#,
            )
            .await?;

        let rows = conn.query(&saved_session_ids_stmt, &[]).await?;

        let saved_session_ids = rows.iter().map(|row| row.get("id")).collect();

        Ok(saved_session_ids)
    }
}

impl SavedSessionRepository {
    pub fn new(db: &'static Database, cache: &'static RedisCache) -> Self {
        Self { db, cache }
    }
}
