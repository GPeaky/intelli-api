use crate::{config::Database, structs::DatabasesStatus};

/// Manages access to server resources, including database connections.
///
/// This repository provides an interface for querying the status of various database pools
/// within the server, such as Redis and PostgreSQL. It's designed to facilitate the monitoring
/// and management of database connections and their operational status.
#[derive(Clone)]
pub struct ServerRepository {
    /// The database connection(s) managed by this repository.
    database: Database,
}

impl ServerRepository {
    /// Constructs a new `ServerRepository` instance.
    ///
    /// Initializes the repository with a given database connection. This allows the repository
    /// to perform operations related to the database, such as checking the status of connection pools.
    ///
    /// # Arguments
    ///
    /// * `db_conn` - A reference to an established `Database` connection.
    ///
    /// # Returns
    ///
    /// A new instance of `ServerRepository`.
    pub fn new(db_conn: &Database) -> Self {
        Self {
            database: db_conn.clone(),
        }
    }

    /// Retrieves the status of active database pools.
    ///
    /// Queries the status of both Redis and PostgreSQL connection pools managed by the server.
    /// This method can be used to monitor the health and availability of the database resources.
    ///
    /// # Returns
    ///
    /// A `DatabasesStatus` struct containing the status information for the Redis and PostgreSQL pools.
    pub fn active_pools(&self) -> DatabasesStatus {
        let redis_pool = self.database.redis.status();
        let pg_pool = self.database.pg.status();

        DatabasesStatus {
            redis: redis_pool.into(),
            pg: pg_pool.into(),
        }
    }
}
