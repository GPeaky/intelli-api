use ahash::AHashSet;

use crate::{cache::ServiceCache, config::Database, error::AppResult, utils::UsedIds};

#[derive(Clone)]
pub struct SavedSessionRepository {
    db: &'static Database,
    #[allow(unused)]
    cache: &'static ServiceCache,
}

impl UsedIds for SavedSessionRepository {
    async fn used_ids(&self) -> AppResult<AHashSet<i32>> {
        let conn = self.db.pg.get().await?;

        let saved_session_ids_stmt = conn
            .prepare_cached(
                r#"
                    SELECT id FROM saved_session
                "#,
            )
            .await?;

        let rows = conn.query(&saved_session_ids_stmt, &[]).await?;
        let mut saved_session_ids = AHashSet::with_capacity(rows.len());

        for row in rows {
            let id: i32 = row.get("id");
            saved_session_ids.insert(id);
        }

        Ok(saved_session_ids)
    }
}

impl SavedSessionRepository {
    pub fn new(db: &'static Database, cache: &'static ServiceCache) -> Self {
        Self { db, cache }
    }
}
