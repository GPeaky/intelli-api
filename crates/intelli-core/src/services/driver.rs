use std::future::Future;

use db::Database;
use error::{AppResult, DriverError};

use crate::repositories::DriverRepository;

pub trait DriverServiceOperations {
    fn create(
        &self,
        steam_name: &str,
        nationality: i16,
        user_id: Option<i32>,
    ) -> impl Future<Output = AppResult<()>> + Send;

    // TODO: Implement driver update
    // async fn update(&self, form: DriverUpdateData) -> AppResult<()>;
}

pub trait DriverAdminServiceOperations: DriverServiceOperations {
    #[allow(unused)]
    fn admin_delete(&self, steam_name: &str) -> impl Future<Output = AppResult<()>> + Send;
}

pub struct DriverService {
    db: &'static Database,
    driver_repo: &'static DriverRepository,
}

impl DriverService {
    pub async fn new(db: &'static Database, driver_repo: &'static DriverRepository) -> Self {
        DriverService { db, driver_repo }
    }

    #[inline]
    #[tracing::instrument(skip_all)]
    async fn _create(
        &self,
        steam_name: &str,
        nationality: i16,
        user_id: Option<i32>,
    ) -> AppResult<()> {
        let conn = self.db.pg.get().await?;

        let create_driver_stmt = conn
            .prepare_cached(
                r#"
                    INSERT INTO drivers (steam_name, nationality, user_id)
                    VALUES ($1, $2, $3)
                "#,
            )
            .await?;

        conn.execute(&create_driver_stmt, &[&steam_name, &nationality, &user_id])
            .await?;

        Ok(())
    }

    async fn _delete(&self, steam_name: &str) -> AppResult<()> {
        let conn = self.db.pg.get().await?;

        let delete_driver_rel_stmt = conn.prepare_cached(
            r#"
                DELETE FROM championship_drivers
                WHERE steam_name = $1
            "#,
        );

        let delete_driver_stmt = conn.prepare_cached(
            r#"
                DELETE FROM drivers
                WHERE steam_name = $1
            "#,
        );

        let (delete_driver_rel, delete_driver) =
            tokio::try_join!(delete_driver_rel_stmt, delete_driver_stmt)?;

        conn.execute_raw(&delete_driver_rel, &[&steam_name]).await?;

        conn.execute_raw(&delete_driver, &[&steam_name]).await?;

        self.db.cache.driver.delete(steam_name);

        Ok(())
    }
}

impl DriverServiceOperations for DriverService {
    #[tracing::instrument(skip(self))]
    async fn create(
        &self,
        steam_name: &str,
        nationality: i16,
        user_id: Option<i32>,
    ) -> AppResult<()> {
        if self.driver_repo.find(steam_name).await?.is_some() {
            return Err(DriverError::AlreadyExists)?;
        }

        self._create(steam_name, nationality, user_id).await
    }
}

impl DriverAdminServiceOperations for DriverService {
    async fn admin_delete(&self, steam_name: &str) -> AppResult<()> {
        self._delete(steam_name).await
    }
}
