use crate::{config::Database, structs::DatabasesStatus};

#[derive(Clone)]
pub struct ServerRepository {
    database: Database,
}

impl ServerRepository {
    pub fn new(db_conn: &Database) -> Self {
        Self {
            database: db_conn.clone(),
        }
    }

    pub fn active_pools(&self) -> DatabasesStatus {
        let redis_pool = self.database.redis.status();
        let pg_pool = self.database.pg.status();

        DatabasesStatus {
            redis: redis_pool.into(),
            pg: pg_pool.into(),
        }
    }
}
