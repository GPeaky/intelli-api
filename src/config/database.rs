use dotenvy::var;
use scylla::{prepared_statement::PreparedStatement, Session, SessionBuilder};
use std::{collections::HashMap, sync::Arc};
use tokio::fs;
use tracing::info;

pub struct Database {
    scylla: Session,
    pub statements: Arc<HashMap<String, PreparedStatement>>,
}

impl Database {
    pub async fn default() -> Self {
        info!("Connecting Databases...");

        let session = SessionBuilder::new()
            .known_node(var("DB_URL").unwrap())
            .build()
            .await
            .unwrap();

        info!("Connected To Database! Parsing Schema...");
        Self::parse_schema(&session).await;

        info!("Schema Parsed!, Saving Prepared Statements...");
        let statements = Self::prepared_statements(&session).await;

        info!("Prepared Statements Saved!, Returning Database Instance");
        Self {
            scylla: session,
            statements: Arc::new(statements),
        }
    }

    async fn parse_schema(session: &Session) {
        let schema = fs::read_to_string("src/config/schema.cql").await.unwrap();

        for q in schema.split(';') {
            let query = q.to_owned() + ";";

            if query.len() > 1 {
                session
                    .query(query, &[])
                    .await
                    .expect("Error Creating Schema");
            }
        }
    }

    async fn prepared_statements(session: &Session) -> HashMap<String, PreparedStatement> {
        let mut statements: HashMap<String, PreparedStatement> = HashMap::new();

        let insert_user = session
            .prepare("INSERT INTO intelli_api.users (id, username, password, email, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)")
            .await
            .unwrap();

        statements.insert("insert_user".to_string(), insert_user);

        statements
    }

    pub fn get_scylla(&self) -> &Session {
        &self.scylla
    }
}
