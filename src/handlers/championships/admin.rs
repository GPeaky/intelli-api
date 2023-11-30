use crate::{
    error::{AppResult, ChampionshipError},
    states::AppState,
};
use ntex::web;

#[inline(always)]
pub async fn user_championships(
    state: web::types::State<AppState>,
    user_id: web::types::Path<i32>,
) -> AppResult<impl web::Responder> {
    let championships = state.championship_repository.find_all(&user_id).await?;

    Ok(web::HttpResponse::Ok().json(&championships))
}

#[inline(always)]
pub async fn delete_championship(
    state: web::types::State<AppState>,
    id: web::types::Path<i32>,
) -> AppResult<impl web::Responder> {
    let Some(championship) = state.championship_repository.find(&id).await? else {
        Err(ChampionshipError::NotFound)?
    };

    state.championship_service.delete(&championship.id).await?;

    Ok(web::HttpResponse::Ok())
}
