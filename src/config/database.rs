use ahash::AHashMap;
use dotenvy::var;
use redis::{aio::Connection, Client};
use scylla::{prepared_statement::PreparedStatement, Session, SessionBuilder};
use std::sync::Arc;
use tokio::{fs, try_join};
use tracing::info;

pub struct Database {
    redis: Client,
    pub scylla: Arc<Session>,
    pub statements: Arc<AHashMap<String, PreparedStatement>>,
}

impl Database {
    pub async fn default() -> Self {
        info!("Connecting Databases...");
        let scylla = SessionBuilder::new()
            .known_node(var("SCYLLA_URI").unwrap())
            // .user(var("SCYLLA_USER").unwrap(), var("SCYLLA_PASS").unwrap())
            .use_keyspace(var("SCYLLA_KEYSPACE").unwrap(), true)
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
            scylla: Arc::new(scylla),
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

        //* User Tasks
        let user_insert_task = session
            .prepare("INSERT INTO users (id, username, password, email, active, role, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)");

        let user_email_by_email_task =
            session.prepare("SELECT email FROM users where email = ? ALLOW FILTERING");

        let user_by_id_task = session.prepare("SELECT * FROM users where id = ? ALLOW FILTERING");

        let user_by_email_task =
            session.prepare("SELECT * FROM users where email = ? ALLOW FILTERING");

        let user_delete_task = session.prepare("DELETE FROM users WHERE id = ?");

        let user_activate_task = session.prepare("UPDATE users SET active = true WHERE id = ?");

        let user_deactivate_task = session.prepare("UPDATE users SET active = false WHERE id = ?");

        //* Championships Tasks
        let championship_insert_task = session
            .prepare(
                "INSERT INTO championships (id, name, port, user_id, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
            );

        let championship_by_user_id_task =
            session.prepare("SELECT * FROM championships where user_id = ? ALLOW FILTERING");

        let championship_by_id_task = session.prepare("SELECT * FROM championships where id = ?");

        let championship_by_name_task =
            session.prepare("SELECT name FROM championships where name = ? ALLOW FILTERING");

        let championships_ports_task = session.prepare("SELECT port FROM championships");

        let championship_delete_task = session.prepare("DELETE FROM championships WHERE id = ?");

        //* Event Data Tasks
        let event_data_select_task =
            session.prepare("SELECT * FROM event_data WHERE session_id = ? AND string_code = ?;");

        let event_data_insert_task = session
            .prepare("INSERT INTO event_data (session_id, string_code, events) VALUES (?,?,?);");

        let event_data_update_task = session.prepare(
            "UPDATE event_data SET events = events + ? WHERE session_id = ? AND string_code = ?;",
        );

        let event_data_info = session.prepare("SELECT * FROM event_data WHERE session_id = ?;");

        //* Other Tasks
        let lap_data_insert_task =
            session.prepare("INSERT INTO lap_data (session_id, lap) VALUES (?,?);");

        let game_session_insert_task =
            session.prepare("INSERT INTO game_sessions (id, data) VALUES (?,?);");

        let participant_data_insert_task = session
            .prepare("INSERT INTO participants_data (session_id, participants) VALUES (?,?);");

        let final_classification_insert_task = session.prepare(
            "INSERT INTO final_classification_data (session_id, classification) VALUES (?,?);",
        );

        let (
            user_insert,
            user_email_by_email,
            user_by_id,
            user_by_email,
            user_delete,
            user_activate,
            user_deactivate,
            championship_insert,
            championship_by_name,
            championships_ports,
            championship_by_id,
            game_session_insert,
            lap_data_insert,
            event_data_select,
            event_data_insert,
            event_data_update,
            participant_data_insert,
            final_classification_insert,
            event_data_info,
            championship_by_user_id,
            championship_delete,
        ) = try_join!(
            user_insert_task,
            user_email_by_email_task,
            user_by_id_task,
            user_by_email_task,
            user_delete_task,
            user_activate_task,
            user_deactivate_task,
            championship_insert_task,
            championship_by_name_task,
            championships_ports_task,
            championship_by_id_task,
            game_session_insert_task,
            lap_data_insert_task,
            event_data_select_task,
            event_data_insert_task,
            event_data_update_task,
            participant_data_insert_task,
            final_classification_insert_task,
            event_data_info,
            championship_by_user_id_task,
            championship_delete_task
        )
        .unwrap();

        //* User Statements
        statements.insert("user.insert".to_string(), user_insert);
        statements.insert("user.email_by_email".to_string(), user_email_by_email);
        statements.insert("user.by_id".to_string(), user_by_id);
        statements.insert("user.by_email".to_string(), user_by_email);
        statements.insert("user.delete".to_string(), user_delete);
        statements.insert("user.activate".to_string(), user_activate);
        statements.insert("user.deactivate".to_string(), user_deactivate);

        //* Championship Statements
        statements.insert("championship.insert".to_owned(), championship_insert);
        statements.insert("championship.ports".to_string(), championships_ports);
        statements.insert("championship.by_id".to_string(), championship_by_id);
        statements.insert("championship.by_user".to_string(), championship_by_user_id);
        statements.insert("championship.delete".to_string(), championship_delete);
        statements.insert(
            "championship.name_by_name".to_string(),
            championship_by_name,
        );

        //* Event Data Statements
        statements.insert("event_data.get".to_string(), event_data_select);
        statements.insert("event_data.insert".to_string(), event_data_insert);
        statements.insert("event_data.update".to_string(), event_data_update);
        statements.insert("event_data.events_by_id".to_string(), event_data_info);

        // TODO: Update this to use the new statements
        statements.insert("game_session.insert".to_string(), game_session_insert);
        statements.insert("lap_data.insert".to_string(), lap_data_insert);
        statements.insert(
            "participant_data.insert".to_string(),
            participant_data_insert,
        );
        statements.insert(
            "final_classification.insert".to_string(),
            final_classification_insert,
        );

        statements
    }

    pub fn get_redis(&self) -> redis::Connection {
        self.redis.get_connection().unwrap()
    }

    pub async fn get_redis_async(&self) -> Connection {
        self.redis.get_async_connection().await.unwrap()
    }
}
