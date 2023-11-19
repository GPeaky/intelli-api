use crate::{cache::RedisCache, config::Database, error::AppResult};
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::sync::Arc;

pub struct SavedSessionService {
    #[allow(unused)]
    cache: Arc<RedisCache>,
    #[allow(unused)]
    db_conn: Arc<Database>,
    // saved_session_repo: SavedSessionRepository,
}

impl SavedSessionService {
    pub fn new(db_conn: &Arc<Database>, cache: &Arc<RedisCache>) -> Self {
        Self {
            cache: cache.clone(),
            db_conn: db_conn.clone(),
            // saved_session_repo: SavedSessionRepository::new(db_conn, cache),
        }
    }

    #[allow(unused)]

    pub async fn create(&self) -> AppResult<()> {
        panic!("Not implemented");

        let id;

        {
            let mut rand = StdRng::from_entropy();
            id = rand.gen_range(600000000..700000000);
        }

        sqlx::query(
            r#"
                INSERT INTO saved_session (id, events, session_data, participants, session_history, final_classification, championship_id)
                VALUES($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(id)
        .execute(&self.db_conn.pg)
        .await?;

        Ok(())
    }
}
