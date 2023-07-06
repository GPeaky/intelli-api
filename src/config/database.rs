use dotenvy::var;
use scylla::{Session, SessionBuilder};
use tokio::fs;
use tracing::info;

pub struct Database {
    scylla: Session,
}

impl Database {
    pub async fn default() -> Self {
        info!("Connecting Databases...");

        let db = Self {
            scylla: SessionBuilder::new()
                .known_node(var("DB_URL").unwrap())
                .build()
                .await
                .unwrap(),
        };

        info!("Connected To Database! Parsing Schema...");
        db.parse_schema().await;

        info!("Schema Parsed! Database Ready!");
        db
    }

    async fn parse_schema(&self) {
        let schema = fs::read_to_string("src/config/schema.cql").await.unwrap();

        for q in schema.split(';') {
            let query = q.to_owned() + ";";

            if query.len() > 1 {
                self.scylla
                    .query(query, &[])
                    .await
                    .expect("Error Creating Schema");
            }
        }
    }

    pub fn get_scylla(&self) -> &Session {
        &self.scylla
    }
}
