use crate::config::Database;

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

    pub fn active_pools(&self) -> (usize, usize) {
        self.database.active_pools()
    }
}
