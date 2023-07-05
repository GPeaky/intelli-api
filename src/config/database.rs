use dotenvy::var;
use scylla::{Session, SessionBuilder};
use tracing::info;

pub struct Database {
    scylla: Session,
}

impl Database {
    pub async fn default() -> Self {
        info!("Connecting Databases...");

        Self {
            scylla: SessionBuilder::new()
                .known_node(var("DB_URL").unwrap())
                .build()
                .await
                .unwrap(),
        }
    }

    pub fn get_scylla(&self) -> &Session {
        &self.scylla
    }
}
