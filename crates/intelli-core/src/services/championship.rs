use std::future::Future;

use chrono::{DateTime, Duration, Utc};
use postgres_types::ToSql;

use db::Database;
use error::{AppResult, ChampionshipError, CommonError, UserError};
use id_generator::IdsGenerator;
use structs::{ChampionshipCreationData, ChampionshipUpdateData, ChampionshipUserAddForm};
use utils::MachinePorts;

use crate::repositories::{ChampionshipRepository, UserRepository};

/// Defines the core operations for managing championships.
pub trait ChampionshipServiceOperations {
    /// Creates a new championship.
    ///
    /// # Arguments
    ///
    /// * `payload` - The data required to create a championship.
    /// * `user_id` - The ID of the user creating the championship.
    ///
    /// # Errors
    ///
    /// Returns an error if the championship name already exists or if there's a database error.
    fn create(
        &self,
        payload: ChampionshipCreationData,
        user_id: i32,
    ) -> impl Future<Output = AppResult<()>> + Send;

    /// Creates a new race for a championship.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the championship to which the race belongs.
    /// * `track_id` - The ID of the track where the race will take place.
    /// * `date` - The date and time when the race is scheduled.
    ///
    /// # Returns
    ///
    /// Returns the ID of the newly created race as an `i32`.
    ///
    /// # Errors
    ///
    /// Returns an error if the championship is not found, the track ID is invalid,
    /// or if there's a database error.
    fn create_race(
        &self,
        id: i32,
        track_id: i16,
        date: DateTime<Utc>,
    ) -> impl Future<Output = AppResult<i32>> + Send;

    /// Updates an existing championship.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the championship to update.
    /// * `user_id` - The ID of the user attempting the update.
    /// * `form` - The data to update the championship with.
    ///
    /// # Errors
    ///
    /// Returns an error if the championship is not found, the user is not the owner,
    /// or if the update interval hasn't been reached.
    fn update(
        &self,
        id: i32,
        user_id: i32,
        form: &ChampionshipUpdateData,
    ) -> impl Future<Output = AppResult<()>> + Send;

    /// Adds a user to a championship.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the championship.
    /// * `user_id` - The ID of the user adding another user.
    /// * `form` - The form containing the new user's details.
    ///
    /// # Errors
    ///
    /// Returns an error if the championship is not found, the user is not the owner,
    /// or if the user to be added doesn't exist.
    fn add_user(
        &self,
        id: i32,
        user_id: i32,
        form: ChampionshipUserAddForm,
    ) -> impl Future<Output = AppResult<()>> + Send;

    /// Adds a driver to a championship.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the championship.
    /// * `steam_name` - The steam_name of the new driver.
    /// * `team_id` - The team of the driver.
    /// * `number` - The racing number of the driver.
    ///
    /// # Errors
    ///
    /// Returns an error if the championship or driver is not found,
    fn add_driver(
        &self,
        id: i32,
        steam_name: &str,
        team_id: i16,
        number: i16,
    ) -> impl Future<Output = AppResult<()>> + Send;

    /// Adds a race result to a specific race in the championship.
    ///
    /// # Arguments
    ///
    /// * `race_id` - The ID of the race to which the result belongs.
    /// * `session_type` - The type of session (e.g., practice, qualifying, race).
    /// * `data` - The raw data of the race result.
    ///
    /// # Errors
    ///
    /// Returns an error if the race is not found, the session type is invalid,
    /// or if there's a database error while storing the result.
    #[allow(unused)]
    fn add_race_result(
        &self,
        race_id: i32,
        session_type: i16,
        data: &[u8],
    ) -> impl Future<Output = AppResult<()>> + Send;

    /// Removes a user from a championship.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the championship.
    /// * `user_id` - The ID of the user removing another user.
    /// * `remove_user_id` - The ID of the user to be removed.
    ///
    /// # Errors
    ///
    /// Returns an error if the championship is not found, the user is not the owner,
    /// or if attempting to remove the owner.
    fn remove_user(
        &self,
        id: i32,
        user_id: i32,
        remove_user_id: i32,
    ) -> impl Future<Output = AppResult<()>> + Send;

    /// Removes a driver from a championship.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the championship.
    /// * `steam_name` - The steam_name of the new driver.
    ///
    /// # Errors
    ///
    /// Returns an error if the championship is not found or driver is not found,
    #[allow(unused)]
    fn remove_driver(
        &self,
        id: i32,
        steam_name: &str,
    ) -> impl Future<Output = AppResult<()>> + Send;

    /// Deletes a championship.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the championship to delete.
    /// * `user_id` - The ID of the user attempting to delete the championship.
    ///
    /// # Errors
    ///
    /// Returns an error if the championship is not found or if the user is not the owner.
    #[allow(unused)]
    fn delete(&self, id: i32, user_id: i32) -> impl Future<Output = AppResult<()>> + Send;
}

/// Defines additional admin-level operations for managing championships.
pub trait ChampionshipAdminServiceOperations: ChampionshipServiceOperations {
    /// Allows an admin to delete a championship without ownership checks.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the championship to delete.
    ///
    /// # Errors
    ///
    /// Returns an error if the championship is not found or if there's a database error.
    fn admin_delete_championship(&self, id: i32) -> impl Future<Output = AppResult<()>> + Send;
}

/// Implements the championship service logic.
pub struct ChampionshipService {
    db: &'static Database,
    machine_ports: MachinePorts,
    user_repo: &'static UserRepository,
    championship_repo: &'static ChampionshipRepository,
    ids_generator: IdsGenerator,
}

impl ChampionshipService {
    /// Creates a new ChampionshipService instance.
    ///
    /// # Arguments
    ///
    /// * `db` - The database connection.
    /// * `user_repo` - The user repository.
    /// * `championship_repo` - The championship repository.
    ///
    /// # Errors
    ///
    /// Returns an error if there's an issue initializing the service components.
    pub async fn new(
        db: &'static Database,
        user_repo: &'static UserRepository,
        championship_repo: &'static ChampionshipRepository,
    ) -> AppResult<Self> {
        let machine_ports = {
            let used_ports = championship_repo._ports_in_use().await?;
            MachinePorts::new(used_ports).await?
        };

        let ids_generator = {
            let used_ids = championship_repo._used_ids().await?;
            IdsGenerator::new(700000000..799999999, used_ids)
        };

        Ok(Self {
            db,
            user_repo,
            championship_repo,
            machine_ports,
            ids_generator,
        })
    }

    /// Internal method to create a championship.
    #[inline]
    async fn _create(&self, payload: ChampionshipCreationData, user_id: i32) -> AppResult<()> {
        if self
            .championship_repo
            .find_by_name(&payload.name)
            .await?
            .is_some()
        {
            Err(ChampionshipError::AlreadyExists)?
        };

        let conn = self.db.pg.get().await?;

        let create_championship_stmt_fut = conn.prepare_cached(
            r#"
                INSERT INTO championships (id, port, name, category, owner_id)
                VALUES ($1,$2,$3,$4,$5)
            "#,
        );

        let relate_user_with_championship_stmt_fut = conn.prepare_cached(
            r#"
                INSERT INTO championship_users (user_id, championship_id, role)
                VALUES ($1,$2, 'Admin')
            "#,
        );

        let (create_championship_stmt, relate_user_with_championship_stmt) = tokio::try_join!(
            create_championship_stmt_fut,
            relate_user_with_championship_stmt_fut
        )?;

        let id = self.ids_generator.next();

        let port = self
            .machine_ports
            .next()
            .ok_or(ChampionshipError::NoPortsAvailable)?;

        let result = conn
            .execute(
                &create_championship_stmt,
                &[&id, &port, &payload.name, &payload.category, &user_id],
            )
            .await;

        if let Err(e) = result {
            self.machine_ports.return_port(port);
            return Err(e)?;
        }

        conn.execute_raw(&relate_user_with_championship_stmt, &[&user_id, &id])
            .await?;

        self.db.cache.championship.delete_by_user(&user_id);
        Ok(())
    }

    #[inline]
    async fn _create_race(&self, id: i32, track_id: i16, date: DateTime<Utc>) -> AppResult<i32> {
        let conn = self.db.pg.get().await?;

        let create_race_stmt = conn
            .prepare_cached(
                r#"
                    INSERT INTO races (championship_id, track_id, date)
                    VALUES ($1, $2, $3)
                    RETURNING id
                "#,
            )
            .await?;

        let id = conn
            .query_one(&create_race_stmt, &[&id, &track_id, &date])
            .await?
            .get(0);

        self.db.cache.championship.delete_races(&id);

        Ok(id)
    }

    /// Internal method to update a championship.
    #[inline]
    async fn _update(&self, id: i32, form: &ChampionshipUpdateData) -> AppResult<()> {
        let (query, params) = {
            let mut params_counter = 1u8;
            let mut clauses = Vec::with_capacity(3);
            let mut params: Vec<&(dyn ToSql + Sync)> = Vec::with_capacity(5);

            if let Some(name) = &form.name {
                clauses.push(format!("name = ${}", params_counter));
                params.push(name);
                params_counter += 1;
            }

            if let Some(category) = &form.category {
                clauses.push(format!("category = ${}", params_counter));
                params.push(category);
                params_counter += 1;
            }

            if clauses.is_empty() {
                Err(CommonError::NotValidUpdate)?
            }

            clauses.push("updated_at = CURRENT_TIMESTAMP".to_owned());

            let clause = clauses.join(", ");
            let query = format!(
                "UPDATE championships SET {} WHERE id = ${}",
                clause, params_counter,
            );

            params.push(&id);

            (query, params)
        };

        {
            let conn = self.db.pg.get().await?;
            conn.execute(&query, &params).await?;
        }

        let users = self.championship_repo.users(id).await?;
        self.db.cache.championship.prune(id, users);

        Ok(())
    }

    /// Internal method to add a user to a championship.
    #[inline]
    async fn _add_user(&self, id: i32, form: ChampionshipUserAddForm) -> AppResult<()> {
        let bind_user_id = {
            let Some(bind_user) = self.user_repo.find_by_email(&form.email).await? else {
                Err(UserError::NotFound)?
            };

            bind_user.id
        };

        let conn = self.db.pg.get().await?;

        let add_user_stmt = conn
            .prepare_cached(
                r#"
                    INSERT INTO championship_users (user_id, championship_id, role, team_id)
                    VALUES ($1,$2,$3,$4)
                "#,
            )
            .await?;

        conn.execute(
            &add_user_stmt,
            &[&bind_user_id, &id, &form.role, &form.team_id],
        )
        .await?;

        self.db.cache.championship.delete_by_user(&bind_user_id);

        Ok(())
    }

    async fn _add_driver(
        &self,
        id: i32,
        steam_name: &str,
        team_id: i16,
        number: i16,
    ) -> AppResult<()> {
        let conn = self.db.pg.get().await?;

        let add_driver_stmt = conn
            .prepare_cached(
                r#"
                    INSERT INTO championship_drivers (steam_name, championship_id, team_id, number)
                    VALUES ($1, $2, $3, $4)
                "#,
            )
            .await?;

        conn.execute(&add_driver_stmt, &[&steam_name, &id, &team_id, &number])
            .await?;

        Ok(())
    }

    async fn _add_race_result(
        &self,
        race_id: i32,
        session_type: i16,
        data: &[u8],
    ) -> AppResult<()> {
        let conn = self.db.pg.get().await?;

        let add_result_stmt = conn
            .prepare_cached(
                r#"
                    INSERT INTO results (race_id, session_type, data)
                    VALUES ($1, $2, $3)
                "#,
            )
            .await?;

        conn.execute(&add_result_stmt, &[&race_id, &session_type, &data])
            .await?;

        Ok(())
    }

    /// Internal method to remove a user from a championship.
    #[inline]
    async fn _remove_user(&self, id: i32, remove_user_id: i32) -> AppResult<()> {
        if self.user_repo.find(remove_user_id).await?.is_none() {
            Err(UserError::NotFound)?
        };

        let conn = self.db.pg.get().await?;

        let remove_user_stmt = conn
            .prepare_cached(
                r#"
                    DELETE FROM championship_users
                    WHERE user_id = $1 AND championship_id = $2
                "#,
            )
            .await?;

        conn.execute_raw(&remove_user_stmt, &[&remove_user_id, &id])
            .await?;

        self.db.cache.championship.delete_by_user(&remove_user_id);

        Ok(())
    }

    #[inline]
    async fn _remove_driver(&self, id: i32, steam_name: &str) -> AppResult<()> {
        let conn = self.db.pg.get().await?;

        let remove_driver_stmt = conn
            .prepare_cached(
                r#"
                    DELETE FROM championship_users
                    WHERE championship_id = $1 AND steam_name = $2
                "#,
            )
            .await?;

        conn.execute(&remove_driver_stmt, &[&id, &steam_name])
            .await?;

        Ok(())
    }

    /// Internal method to delete a championship.
    #[inline]
    async fn _delete(&self, id: i32) -> AppResult<()> {
        let conn = self.db.pg.get().await?;

        let delete_championship_relations_stmt_fut = conn.prepare_cached(
            r#"
                DELETE FROM championship_users WHERE championship_id = $1
            "#,
        );

        let delete_championship_stmt_fut = conn.prepare_cached(
            r#"
                DELETE FROM championships WHERE id = $1
            "#,
        );

        let (delete_championship_relations_stmt, delete_championship_stmt) = tokio::try_join!(
            delete_championship_relations_stmt_fut,
            delete_championship_stmt_fut
        )?;

        let users = self.championship_repo.users(id).await?;

        conn.execute_raw(&delete_championship_relations_stmt, &[&id])
            .await?;

        self.db.cache.championship.prune(id, users);

        conn.execute_raw(&delete_championship_stmt, &[&id]).await?;

        Ok(())
    }
}

impl ChampionshipServiceOperations for ChampionshipService {
    async fn create(&self, payload: ChampionshipCreationData, user_id: i32) -> AppResult<()> {
        self._create(payload, user_id).await
    }

    async fn create_race(&self, id: i32, track_id: i16, date: DateTime<Utc>) -> AppResult<i32> {
        // I don't know if this is necessary
        // if self.championship_repo.find(id).await?.is_none() {
        //     Err(ChampionshipError::NotFound)?
        // }

        self._create_race(id, track_id, date).await
    }

    async fn update(&self, id: i32, user_id: i32, form: &ChampionshipUpdateData) -> AppResult<()> {
        {
            let Some(championship) = self.championship_repo.find(id).await? else {
                Err(ChampionshipError::NotFound)?
            };

            if championship.owner_id != user_id {
                Err(ChampionshipError::NotOwner)?
            }

            if let Some(last_update) = championship.updated_at {
                if Utc::now().signed_duration_since(last_update) <= Duration::days(7) {
                    Err(CommonError::UpdateLimit)?
                };
            }
        }

        self._update(id, form).await
    }

    async fn add_user(
        &self,
        id: i32,
        user_id: i32,
        form: ChampionshipUserAddForm,
    ) -> AppResult<()> {
        {
            let Some(championship) = self.championship_repo.find(id).await? else {
                Err(ChampionshipError::NotFound)?
            };

            if championship.owner_id != user_id {
                Err(ChampionshipError::NotOwner)?
            }
        }

        self._add_user(id, form).await
    }

    async fn add_driver(
        &self,
        id: i32,
        steam_name: &str,
        team_id: i16,
        number: i16,
    ) -> AppResult<()> {
        // TODO: Add check if championship and driver exists
        self._add_driver(id, steam_name, team_id, number).await
    }

    async fn add_race_result(&self, race_id: i32, session_type: i16, data: &[u8]) -> AppResult<()> {
        // TODO: Maybe add checks for race_id
        self._add_race_result(race_id, session_type, data).await
    }

    async fn remove_user(&self, id: i32, user_id: i32, remove_user_id: i32) -> AppResult<()> {
        {
            let Some(championship) = self.championship_repo.find(id).await? else {
                Err(ChampionshipError::NotFound)?
            };

            if championship.owner_id != user_id {
                Err(ChampionshipError::NotOwner)?
            }

            if championship.owner_id == remove_user_id {
                Err(ChampionshipError::CannotRemoveOwner)?
            }
        }

        self._remove_user(id, remove_user_id).await
    }

    async fn remove_driver(&self, id: i32, steam_name: &str) -> AppResult<()> {
        // Maybe do some checks
        self._remove_driver(id, steam_name).await
    }

    async fn delete(&self, id: i32, user_id: i32) -> AppResult<()> {
        {
            let Some(championship) = self.championship_repo.find(id).await? else {
                Err(ChampionshipError::NotFound)?
            };

            if championship.owner_id != user_id {
                Err(ChampionshipError::NotOwner)?
            }
        }

        self._delete(id).await
    }
}

impl ChampionshipAdminServiceOperations for ChampionshipService {
    async fn admin_delete_championship(&self, id: i32) -> AppResult<()> {
        self._delete(id).await
    }
}
