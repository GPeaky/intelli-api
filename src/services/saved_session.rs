use crate::{
    cache::ServiceCache, config::Database, error::AppResult, repositories::SavedSessionRepository,
    utils::IdsGenerator,
};

#[derive(Clone)]
pub struct SavedSessionService {
    #[allow(unused)]
    cache: &'static ServiceCache,
    #[allow(unused)]
    db: &'static Database,
    #[allow(unused)]
    saved_session_repo: SavedSessionRepository,
    ids_generator: IdsGenerator<SavedSessionRepository>,
}

impl SavedSessionService {
    pub async fn new(db: &'static Database, cache: &'static ServiceCache) -> Self {
        let saved_session_repo = SavedSessionRepository::new(db, cache);
        let ids_generator =
            IdsGenerator::new(800000000..900000000, saved_session_repo.clone()).await;

        Self {
            cache,
            db,
            saved_session_repo,
            ids_generator,
        }
    }

    #[allow(unused)]
    pub async fn create(&self) -> AppResult<()> {
        let id = self.ids_generator.next().await;

        let conn = self.db.pg.get().await?;
        let save_session_stmt = conn
            .prepare_cached(
                r#"
                    INSERT INTO saved_session (id)
                    VALUES ($1)
                "#,
            )
            .await?;

        conn.execute(&save_session_stmt, &[&id]).await?;

        Ok(())
    }
}
