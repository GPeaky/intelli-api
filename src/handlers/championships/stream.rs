use garde::Validate;
use ntex::web::{
    types::{Path, State},
    HttpResponse, Responder,
};
use tokio_stream::wrappers::BroadcastStream;
use tracing::info;

use crate::{
    error::{AppResult, CommonError, F1ServiceError},
    states::AppState,
    structs::ChampionshipIdPath,
};

pub async fn handle_stream(
    state: State<AppState>,
    path: Path<ChampionshipIdPath>,
) -> AppResult<impl Responder> {
    if path.validate(&()).is_err() {
        Err(CommonError::ValidationFailed)?
    }

    if !state.f1_svc.service_active(path.0).await {
        Err(F1ServiceError::NotActive)?
    }

    let x = state.f1_svc.service_cache(path.0).await?;
    info!("Cache Size: {}", x.unwrap().len());

    // Todo - Get cache from redis and return it before sending real time data
    let Some(rx) = state.f1_svc.subscribe(path.0).await else {
        Err(F1ServiceError::NotActive)?
    };

    let stream = BroadcastStream::new(rx);

    Ok(HttpResponse::Ok()
        .content_type("application/octet-stream")
        .streaming(stream))
}
