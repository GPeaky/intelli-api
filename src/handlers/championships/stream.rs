use garde::Validate;
use ntex::web::{
    types::{Path, State},
    HttpResponse,
};
use reqwest::header::HeaderValue;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};

use crate::{
    error::{AppResult, CommonError, F1ServiceError},
    states::AppState,
    structs::ChampionshipIdPath,
};

#[inline(always)]
pub async fn handle_stream(
    state: State<AppState>,
    path: Path<ChampionshipIdPath>,
) -> AppResult<HttpResponse> {
    if path.validate().is_err() {
        Err(CommonError::ValidationFailed)?
    }

    if !state.f1_svc.service(&path.0) {
        Err(F1ServiceError::NotActive)?
    }

    let cached_data = state.f1_svc.cache(&path.0).await?;

    let Some(rx) = state.f1_svc.subscribe(&path.0) else {
        Err(F1ServiceError::NotActive)?
    };

    let stream = BroadcastStream::new(rx);
    let mut response = HttpResponse::Ok();

    response.content_type(HeaderValue::from_static("application/octet-stream"));

    match cached_data {
        None => Ok(response.streaming(stream)),

        Some(data) => {
            let cache_steam = tokio_stream::once(Ok(data));
            let combined_stream = cache_steam.chain(stream);

            Ok(response.streaming(combined_stream))
        }
    }
}
