use crate::{config::Database, structs::ConnectionPoolStatus};

/// Manages access to server resources, primarily database connections.
#[derive(Clone)]
pub struct ServerRepository {
    db: &'static Database,
}

impl ServerRepository {
    /// Creates a new ServerRepository instance.
    ///
    /// # Arguments
    /// - `db`: A reference to the Database connection.
    ///
    /// # Returns
    /// A new ServerRepository instance.
    pub fn new(db: &'static Database) -> Self {
        Self { db }
    }

    /// Retrieves the status of active database pools.
    ///
    /// This method queries the status of the PostgresSQL connection pool
    /// managed by the server. It can be used to monitor the health and
    /// availability of the database resources.
    ///
    /// # Returns
    /// A ConnectionPoolStatus struct containing the status information
    /// for the PostgresSQL pool.
    pub fn active_pools(&self) -> ConnectionPoolStatus {
        let pg_pool = self.db.pg.status();
        pg_pool.into()
    }
}
