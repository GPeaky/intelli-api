pub(crate) mod admin;
pub(crate) mod service;
pub(crate) mod stream;

pub(crate) mod core {
    use garde::Validate;
    use ntex::web::{
        types::{Json, Path, State},
        HttpRequest, HttpResponse,
    };

    use entities::{Role, UserExtension};
    use error::{AppResult, ChampionshipError, CommonError};
    use intelli_core::services::ChampionshipServiceOperations;
    use structs::{
        ChampionshipAndUserId, ChampionshipCreationData, ChampionshipData, ChampionshipId,
        ChampionshipUpdateData, ChampionshipUserAddForm,
    };

    use crate::states::AppState;

    #[inline]
    #[tracing::instrument(skip(req, state))]
    pub async fn create(
        req: HttpRequest,
        state: State<AppState>,
        Json(championship_creation): Json<ChampionshipCreationData>,
    ) -> AppResult<HttpResponse> {
        if championship_creation.validate().is_err() {
            return Err(CommonError::ValidationFailed)?;
        }

        let user = req.user()?;
        let championships_len = state.user_repo.championship_len(user.id).await?;

        match user.role {
            Role::User => {
                if championships_len >= 1 {
                    Err(ChampionshipError::LimitReached)?
                }
            }

            Role::Premium => {
                if championships_len >= 3 {
                    Err(ChampionshipError::LimitReached)?
                }
            }

            Role::Admin => {}
        }

        state
            .championship_svc
            .create(championship_creation, user.id)
            .await?;
        Ok(HttpResponse::Created().finish())
    }

    #[inline]
    #[tracing::instrument(skip(req, state))]
    pub async fn update(
        req: HttpRequest,
        state: State<AppState>,
        championship_update: Json<ChampionshipUpdateData>,
        path: Path<ChampionshipId>,
    ) -> AppResult<HttpResponse> {
        if championship_update.validate().is_err() || path.validate().is_err() {
            Err(CommonError::ValidationFailed)?
        }

        let user_id = req.user_id()?;
        state
            .championship_svc
            .update(path.0, user_id, &championship_update)
            .await?;

        Ok(HttpResponse::Ok().finish())
    }

    #[inline]
    #[tracing::instrument(skip(req, state))]
    pub async fn add_user(
        req: HttpRequest,
        state: State<AppState>,
        Json(add_user): Json<ChampionshipUserAddForm>,
        path: Path<ChampionshipId>,
    ) -> AppResult<HttpResponse> {
        if add_user.validate().is_err() || path.validate().is_err() {
            Err(CommonError::ValidationFailed)?
        }

        let user_id = req.user_id()?;
        state
            .championship_svc
            .add_user(path.0, user_id, add_user)
            .await?;

        Ok(HttpResponse::Ok().finish())
    }

    #[inline]
    #[tracing::instrument(skip(req, state))]
    pub async fn remove_user(
        req: HttpRequest,
        state: State<AppState>,
        path: Path<ChampionshipAndUserId>,
    ) -> AppResult<HttpResponse> {
        if path.validate().is_err() {
            Err(CommonError::ValidationFailed)?
        }

        let user_id = req.user_id()?;
        state
            .championship_svc
            .remove_user(path.championship_id, user_id, path.user_id)
            .await?;

        Ok(HttpResponse::Ok().finish())
    }

    #[inline]
    #[tracing::instrument(skip(state))]
    pub async fn get(
        state: State<AppState>,
        path: Path<ChampionshipId>,
    ) -> AppResult<HttpResponse> {
        path.validate().map_err(|_| CommonError::ValidationFailed)?;

        let (championship, races) = tokio::try_join!(
            state.championship_repo.find(path.0),
            state.championship_repo.races(path.0)
        )?;

        Ok(HttpResponse::Ok().json(&ChampionshipData {
            championship: championship.ok_or(ChampionshipError::NotFound)?,
            races,
        }))
    }
}
