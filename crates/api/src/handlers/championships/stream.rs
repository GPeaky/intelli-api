use std::pin::Pin;
use std::task::{Context, Poll};

use garde::Validate;
use ntex::{
    http::header::HeaderValue,
    web::{
        types::{Path, State},
        HttpRequest, HttpResponse,
    },
};
use tokio_stream::{wrappers::BroadcastStream, Stream, StreamExt};

use entities::{ChampionshipRole, UserExtension};
use error::{AppResult, ChampionshipError, CommonError, F1ServiceError};
use structs::ChampionshipId;

use crate::states::AppState;

enum StreamType {
    Normal,
    Engineer(u8),
}

struct CleanupStream<S> {
    inner: S,
    state: State<AppState>,
    championship_id: i32,
    stream_type: StreamType,
}

impl<S: Stream + Unpin> Stream for CleanupStream<S> {
    type Item = S::Item;

    #[inline]
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}

impl<S> Drop for CleanupStream<S> {
    fn drop(&mut self) {
        match &self.stream_type {
            StreamType::Normal => {
                self.state.f1_svc.unsubscribe(&self.championship_id);
            }

            StreamType::Engineer(team_id) => {
                self.state
                    .f1_svc
                    .unsubscribe_team(&self.championship_id, *team_id);
            }
        }
    }
}

#[inline]
pub async fn stream_live_session(
    state: State<AppState>,
    path: Path<ChampionshipId>,
) -> AppResult<HttpResponse> {
    if path.validate().is_err() {
        Err(CommonError::ValidationFailed)?
    }

    let Some((cached_data, rx)) = state.f1_svc.cache_and_subscribe(&path.0) else {
        Err(F1ServiceError::NotActive)?
    };

    let stream = CleanupStream {
        inner: BroadcastStream::new(rx),
        state: state.clone(),
        championship_id: path.0,
        stream_type: StreamType::Normal,
    };

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

pub async fn stream_telemetry_session(
    req: HttpRequest,
    state: State<AppState>,
    path: Path<ChampionshipId>,
) -> AppResult<HttpResponse> {
    if path.validate().is_err() {
        Err(CommonError::ValidationFailed)?
    }

    let user_id = req.user_id()?;

    let relation = state
        .championship_repo
        .user_relation(path.0, user_id)
        .await?;

    match relation {
        Some(relation) => {
            if relation.role != ChampionshipRole::Engineer || relation.team_id.is_none() {
                Err(ChampionshipError::NotAnEngineer)?
            }

            let team_id = relation.team_id.unwrap() as u8;

            let Some(rx) = state.f1_svc.subscribe_team(&path.0, team_id) else {
                Err(F1ServiceError::NotActive)?
            };

            let stream = CleanupStream {
                inner: BroadcastStream::new(rx),
                state: state.clone(),
                championship_id: path.0,
                stream_type: StreamType::Engineer(team_id),
            };

            let mut response = HttpResponse::Ok();
            response.content_type(HeaderValue::from_static("application/octet-stream"));

            Ok(response.streaming(stream))
        }

        None => Err(ChampionshipError::InvalidTeamId)?,
    }
}
