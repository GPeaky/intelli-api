use ahash::AHashMap;
use dotenvy::var;
use redis::{aio::Connection, Client};
use scylla::{prepared_statement::PreparedStatement, Session, SessionBuilder};
use std::sync::Arc;
use tokio::{fs, try_join};
use tracing::info;

pub struct Database {
    redis: Client,
    scylla: Session,
    pub statements: Arc<AHashMap<String, PreparedStatement>>,
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
        Self::parse_schema(&scylla).await;
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

    async fn prepared_statements(session: &Session) -> AHashMap<String, PreparedStatement> {
        let mut statements: AHashMap<String, PreparedStatement> = AHashMap::new();

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

        let championship_by_id_task =
            session.prepare("SELECT * FROM intelli_api.championships where id = ? ALLOW FILTERING");

        let championship_by_name_task = session
            .prepare("SELECT name FROM intelli_api.championships where name = ? ALLOW FILTERING");

        let championships_ports_task =
            session.prepare("SELECT port FROM intelli_api.championships");

        let insert_game_session_task =
            session.prepare("INSERT INTO intelli_api.game_sessions (id, data) VALUES (?,?);");

        let insert_car_motion_task =
            session.prepare("INSERT INTO intelli_api.car_motion (session_id, data) VALUES (?,?);");

        let insert_lap_data_task =
            session.prepare("INSERT INTO intelli_api.lap_data (session_id, lap) VALUES (?,?);");

        let insert_event_data_task = session.prepare(
            "INSERT INTO intelli_api.event_data (session_id, string_code, event) VALUES (?,?,?);",
        );

        let insert_participant_data_task = session.prepare(
            "INSERT INTO intelli_api.participants_data (session_id, participants) VALUES (?,?);",
        );

        let insert_final_classification_data_task = session
        .prepare("INSERT INTO intelli_api.final_classification_data (session_id, classification) VALUES (?,?);");

        let (
            insert_user,
            user_email_by_email,
            user_by_id,
            user_by_email,
            activate_user,
            insert_championships,
            championship_by_name,
            championships_ports,
            championship_by_id,
            insert_game_session,
            insert_car_motion,
            insert_lap_data,
            insert_event_data,
            insert_participant_data,
            insert_final_classification_data,
        ) = try_join!(
            insert_user_task,
            user_email_by_email_task,
            user_by_id_task,
            user_by_email_task,
            activate_user_task,
            insert_championships_task,
            championship_by_name_task,
            championships_ports_task,
            championship_by_id_task,
            insert_game_session_task,
            insert_car_motion_task,
            insert_lap_data_task,
            insert_event_data_task,
            insert_participant_data_task,
            insert_final_classification_data_task
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
        statements.insert("championship_by_id".to_string(), championship_by_id);
        statements.insert("insert_game_session".to_string(), insert_game_session);
        statements.insert("insert_car_motion".to_string(), insert_car_motion);
        statements.insert("insert_lap_data".to_string(), insert_lap_data);
        statements.insert("insert_event_data".to_string(), insert_event_data);
        statements.insert(
            "insert_participant_data".to_string(),
            insert_participant_data,
        );
        statements.insert(
            "insert_final_classification_data".to_string(),
            insert_final_classification_data,
        );

        statements
    }

    pub fn get_scylla(&self) -> &Session {
        &self.scylla
    }

    pub async fn get_redis(&self) -> Connection {
        self.redis.get_async_connection().await.unwrap()
    }
}
