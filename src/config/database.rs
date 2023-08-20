use crate::{
    dtos::{ChampionshipStatements, EventDataStatements, PreparedStatementsKey, UserStatements},
    error::AppResult,
};
use ahash::AHashMap;
use dotenvy::var;
use redis::{aio::Connection, Client};
use scylla::{prepared_statement::PreparedStatement, Session, SessionBuilder};
use std::sync::Arc;
use tokio::fs;
use tracing::info;

const USER_INSERT: &str = "INSERT INTO users (id, username, password, email, active, role, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)";
const USER_MAIL_BY_EMAIL: &str = "SELECT email FROM users_by_email WHERE email = ?";
const USER_BY_ID: &str = "SELECT * FROM users where id = ?";
const USER_BY_EMAIL: &str = "SELECT * FROM users_by_email WHERE email = ?";
const USER_DELETE: &str = "DELETE FROM users WHERE id = ?";
const USER_ACTIVE: &str = "UPDATE users SET active = true WHERE id = ?";
const USER_DEACTIVATE: &str = "UPDATE users SET active = false WHERE id = ?";

const CHAMPIONSHIP_INSERT: &str = "INSERT INTO championships (id, name, port, user_id, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)";
const CHAMPIONSHIP_BY_ID: &str = "SELECT * FROM championships where id = ?";
const CHAMPIONSHIP_BY_NAME: &str = "SELECT name FROM championships_by_name where name = ?";
const CHAMPIONSHIP_PORTS: &str = "SELECT port FROM championships";
const CHAMPIONSHIP_DELETE: &str = "DELETE FROM championships WHERE id = ?";
const CHAMPIONSHIP_BY_USER_ID: &str = "SELECT * FROM championships where user_id = ?";

const EVENT_DATA_SELECT: &str =
    "SELECT * FROM event_data WHERE session_id = ? AND string_code = ?;";
const EVENT_DATA_INSERT: &str =
    "INSERT INTO event_data (session_id, string_code, events) VALUES (?,?,?);";
const EVENT_DATA_UPDATE: &str =
    "UPDATE event_data SET events = events + ? WHERE session_id = ? AND string_code = ?;";
const EVENT_DATA_INFO: &str = "SELECT * FROM event_data WHERE session_id = ?;";

type DbStatements = AHashMap<PreparedStatementsKey, PreparedStatement>;

pub struct Database {
    redis: Client,
    pub scylla: Arc<Session>,
    pub statements: Arc<DbStatements>,
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
        let statements = Self::prepared_statements(&scylla).await.unwrap();

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

    async fn prepared_statements(session: &Session) -> AppResult<DbStatements> {
        let mut statements: DbStatements = AHashMap::new();

        Self::user_statements(session, &mut statements).await?;
        Self::championship_statements(session, &mut statements).await?;
        Self::event_data_statements(session, &mut statements).await?;

        Ok(statements)
    }

    async fn user_statements(session: &Session, statements: &mut DbStatements) -> AppResult<()> {
        statements.insert(
            PreparedStatementsKey::User(UserStatements::Insert),
            session.prepare(USER_INSERT).await?,
        );

        statements.insert(
            PreparedStatementsKey::User(UserStatements::EmailByEmail),
            session.prepare(USER_MAIL_BY_EMAIL).await?,
        );

        statements.insert(
            PreparedStatementsKey::User(UserStatements::ById),
            session.prepare(USER_BY_ID).await?,
        );

        statements.insert(
            PreparedStatementsKey::User(UserStatements::ByEmail),
            session.prepare(USER_BY_EMAIL).await?,
        );

        statements.insert(
            PreparedStatementsKey::User(UserStatements::Delete),
            session.prepare(USER_DELETE).await?,
        );

        statements.insert(
            PreparedStatementsKey::User(UserStatements::Activate),
            session.prepare(USER_ACTIVE).await?,
        );

        statements.insert(
            PreparedStatementsKey::User(UserStatements::Deactivate),
            session.prepare(USER_DEACTIVATE).await?,
        );

        Ok(())
    }

    async fn championship_statements(
        session: &Session,
        statements: &mut DbStatements,
    ) -> AppResult<()> {
        statements.insert(
            PreparedStatementsKey::Championship(ChampionshipStatements::Insert),
            session.prepare(CHAMPIONSHIP_INSERT).await?,
        );

        statements.insert(
            PreparedStatementsKey::Championship(ChampionshipStatements::Ports),
            session.prepare(CHAMPIONSHIP_PORTS).await?,
        );

        statements.insert(
            PreparedStatementsKey::Championship(ChampionshipStatements::ById),
            session.prepare(CHAMPIONSHIP_BY_ID).await?,
        );

        statements.insert(
            PreparedStatementsKey::Championship(ChampionshipStatements::ByUser),
            session.prepare(CHAMPIONSHIP_BY_USER_ID).await?,
        );

        statements.insert(
            PreparedStatementsKey::Championship(ChampionshipStatements::Delete),
            session.prepare(CHAMPIONSHIP_DELETE).await?,
        );

        statements.insert(
            PreparedStatementsKey::Championship(ChampionshipStatements::NameByName),
            session.prepare(CHAMPIONSHIP_BY_NAME).await?,
        );

        Ok(())
    }

    async fn event_data_statements(
        session: &Session,
        statements: &mut DbStatements,
    ) -> AppResult<()> {
        statements.insert(
            PreparedStatementsKey::EventData(EventDataStatements::Select),
            session.prepare(EVENT_DATA_SELECT).await?,
        );

        statements.insert(
            PreparedStatementsKey::EventData(EventDataStatements::Insert),
            session.prepare(EVENT_DATA_INSERT).await?,
        );

        statements.insert(
            PreparedStatementsKey::EventData(EventDataStatements::Update),
            session.prepare(EVENT_DATA_UPDATE).await?,
        );

        statements.insert(
            PreparedStatementsKey::EventData(EventDataStatements::Info),
            session.prepare(EVENT_DATA_INFO).await?,
        );

        Ok(())
    }

    pub fn get_redis(&self) -> redis::Connection {
        self.redis.get_connection().unwrap()
    }

    pub async fn get_redis_async(&self) -> Connection {
        self.redis.get_async_connection().await.unwrap()
    }
}
