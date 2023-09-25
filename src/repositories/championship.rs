use crate::{config::Database, entity::Championship, error::AppResult};
use std::sync::Arc;

#[derive(Clone)]
pub struct ChampionshipRepository {
    database: Arc<Database>,
}

impl ChampionshipRepository {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            database: db_conn.clone(),
        }
    }

    pub async fn ports_in_use(&self) -> AppResult<Vec<(u16,)>> {
        let ports_in_use = sqlx::query_as::<_, (u16,)>(
            r#"
                SELECT port FROM championship
            "#,
        )
        .fetch_all(&self.database.mysql)
        .await?;

        Ok(ports_in_use)
    }

    pub async fn find(&self, id: &u32) -> AppResult<Option<Championship>> {
        let championship = sqlx::query_as::<_, Championship>(
            r#"
                SELECT * FROM championship
                WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.database.mysql)
        .await?;

        Ok(championship)
    }

    pub async fn find_all(&self, user_id: &u32) -> AppResult<Vec<Championship>> {
        let championships = sqlx::query_as::<_, Championship>(
            r#"
                SELECT
                    c.*
                FROM
                    championship c
                JOIN
                    user_championships uc ON c.id = uc.championship_id
                WHERE
                    uc.user_id = ?
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.database.mysql)
        .await?;

        Ok(championships)
    }
}
