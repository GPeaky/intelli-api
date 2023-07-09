use dotenvy::var;
use redis::{aio::Connection, Client};
use scylla::{prepared_statement::PreparedStatement, Session, SessionBuilder};
use std::{collections::HashMap, sync::Arc};
use tokio::{fs, join, try_join};
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

        info!("Connected To Database! Parsing Schema & Saving Prepared Statements...");
        let schema_task = Self::parse_schema(&scylla);
        let statements_task = Self::prepared_statements(&scylla);

        let (_, statements) = join!(schema_task, statements_task);

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

        let insert_user_task = session
            .prepare("INSERT INTO intelli_api.users (id, username, password, email, active, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)");

        let user_email_by_email_task =
            session.prepare("SELECT email FROM intelli_api.users where email = ? ALLOW FILTERING");

        let user_by_id_task =
            session.prepare("SELECT * FROM intelli_api.users where id = ? ALLOW FILTERING");

        let user_by_email_task =
            session.prepare("SELECT * FROM intelli_api.users where email = ? ALLOW FILTERING");

        let activate_user_task = session
            .prepare("UPDATE intelli_api.users SET active = true WHERE id = ? AND email = ?");

        let insert_championships_task = session
            .prepare(
                "INSERT INTO intelli_api.championships (id, name, port, user_id, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
            );

        let championship_by_name_task = session
            .prepare("SELECT name FROM intelli_api.championships where name = ? ALLOW FILTERING");

        let championships_ports_task =
            session.prepare("SELECT port FROM intelli_api.championships");

        let (
            insert_user,
            user_email_by_email,
            user_by_id,
            user_by_email,
            activate_user,
            insert_championships,
            championship_by_name,
            championships_ports,
        ) = try_join!(
            insert_user_task,
            user_email_by_email_task,
            user_by_id_task,
            user_by_email_task,
            activate_user_task,
            insert_championships_task,
            championship_by_name_task,
            championships_ports_task
        )
        .unwrap();

        statements.insert("insert_user".to_string(), insert_user);
        statements.insert("user_email_by_email".to_string(), user_email_by_email);
        statements.insert("user_by_id".to_string(), user_by_id);
        statements.insert("user_by_email".to_string(), user_by_email);
        statements.insert("activate_user".to_string(), activate_user);
        statements.insert("insert_championship".to_owned(), insert_championships);
        statements.insert(
            "championship_name_by_name".to_string(),
            championship_by_name,
        );
        statements.insert("championships_ports".to_string(), championships_ports);

        statements
    }

    pub fn get_scylla(&self) -> &Session {
        &self.scylla
    }

    pub async fn get_redis(&self) -> Connection {
        self.redis.get_async_connection().await.unwrap()
    }
}
