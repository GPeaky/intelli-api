use crate::{
    dtos::CreateChampionshipDto,
    entity::{Championship, Role, UserExtension},
    error::{AppResult, ChampionshipError, CommonError},
    states::UserState,
};
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Extension, Form, Json,
};
use garde::Validate;
use hyper::StatusCode;

pub(crate) use admin::*;
pub(crate) use sockets::*;
pub(crate) use web_socket::*;

mod admin;
mod sockets;
mod web_socket;

#[inline(always)]
pub async fn create_championship(
    Extension(user): Extension<UserExtension>,
    State(state): State<UserState>,
    Form(form): Form<CreateChampionshipDto>,
) -> AppResult<Response> {
    if form.validate(&()).is_err() {
        return Err(CommonError::FormValidationFailed)?;
    }

    {
        let championships_len = state
            .championship_repository
            .user_champions_len(&user.id)
            .await?;

        match user.role {
            Role::Free => {
                if championships_len >= 1 {
                    Err(ChampionshipError::LimitReached)?
                }
            }

            Role::Premium => {
                if championships_len >= 3 {
                    Err(ChampionshipError::LimitReached)?
                }
            }

            Role::Business => {
                if championships_len >= 14 {
                    Err(ChampionshipError::LimitReached)?
                }
            }

            Role::Admin => {}
        }
    }

    state
        .championship_service
        .create_championship(form, &user.id)
        .await?;

    Ok(StatusCode::CREATED.into_response())
}

#[inline(always)]
pub async fn get_championship(
    State(state): State<UserState>,
    Path(championship_id): Path<i32>,
) -> AppResult<Json<Championship>> {
    let Some(championship) = state.championship_repository.find(&championship_id).await? else {
        Err(ChampionshipError::NotFound)?
    };

    Ok(Json(championship))
}

#[inline(always)]
pub async fn all_championships(
    State(state): State<UserState>,
    Extension(user): Extension<UserExtension>,
) -> AppResult<Json<Vec<Championship>>> {
    let championships = state.championship_repository.find_all(&user.id).await?;

    Ok(Json(championships))
}
