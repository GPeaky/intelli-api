use dotenvy::var;
use redis::{aio::Connection, Client};
use scylla::{prepared_statement::PreparedStatement, Session, SessionBuilder};
use std::{collections::HashMap, sync::Arc};
use tokio::fs;
use tracing::info;

pub struct Database {
    redis: Client,
    scylla: Session,
    pub statements: Arc<HashMap<String, PreparedStatement>>,
}

impl Database {
    pub async fn default() -> Self {
        info!("Connecting Databases...");
        let scylla = SessionBuilder::new()
            .known_node(var("DB_URL").unwrap())
            .build()
            .await
            .unwrap();

        let redis = Client::open(var("REDIS_URL").unwrap()).unwrap();

        info!("Connected To Database! Parsing Schema...");
        Self::parse_schema(&scylla).await;

        info!("Schema Parsed!, Saving Prepared Statements...");
        let statements = Self::prepared_statements(&scylla).await;

        info!("Prepared Statements Saved!, Returning Database Instance");
        Self {
            redis,
            scylla,
            statements: Arc::new(statements),
        }
    }

    async fn parse_schema(session: &Session) {
        let schema = fs::read_to_string("src/config/schema.cql").await.unwrap();

        for q in schema.split(';') {
            let query = q.to_owned() + ";";

            if query.len() > 1 {
                session.query(query, &[]).await.unwrap();
            }
        }
    }

    async fn prepared_statements(session: &Session) -> HashMap<String, PreparedStatement> {
        let mut statements: HashMap<String, PreparedStatement> = HashMap::new();

        let insert_user = session
            .prepare("INSERT INTO intelli_api.users (id, username, password, email, active, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)")
            .await
            .unwrap();

        statements.insert("insert_user".to_string(), insert_user);

        let find_by_email = session
            .prepare("SELECT email FROM intelli_api.users where email = ? ALLOW FILTERING")
            .await
            .unwrap();

        statements.insert("select_email_by_email".to_string(), find_by_email);

        let select_user_by_id = session
            .prepare("SELECT * FROM intelli_api.users where id = ? ALLOW FILTERING")
            .await
            .unwrap();

        statements.insert("select_user_by_id".to_string(), select_user_by_id);

        let select_user_by_email = session
            .prepare("SELECT * FROM intelli_api.users where email = ? ALLOW FILTERING")
            .await
            .unwrap();

        statements.insert("select_user_by_email".to_string(), select_user_by_email);

        let activate_user = session
            .prepare("UPDATE intelli_api.users SET active = true WHERE id = ? AND email = ?")
            .await
            .unwrap();

        statements.insert("activate_user".to_string(), activate_user);

        statements
    }

    pub fn get_scylla(&self) -> &Session {
        &self.scylla
    }

    pub async fn get_redis(&self) -> Connection {
        self.redis.get_async_connection().await.unwrap()
    }
}
