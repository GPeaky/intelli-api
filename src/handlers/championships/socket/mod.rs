use std::{future::ready, io};

use garde::Validate;
use ntex::{
    chain,
    channel::oneshot,
    fn_service, rt,
    service::{fn_factory_with_config, fn_shutdown, map_config},
    util::{select, Bytes, Either},
    web::{
        self,
        types::{Path, State},
        HttpRequest, HttpResponse,
    },
    ws::{self, Message},
    Service,
};
use tokio::sync::broadcast::Receiver;

use crate::{
    error::{AppResult, ChampionshipError, CommonError, F123ServiceError},
    states::AppState,
    structs::ChampionshipIdPath,
};

use self::counter::{decrement, increment};

pub(super) mod counter;

#[inline(always)]
pub async fn session_socket(
    req: HttpRequest,
    state: State<AppState>,
    path: Path<ChampionshipIdPath>,
) -> AppResult<HttpResponse> {
    if path.validate(&()).is_err() {
        Err(CommonError::ValidationFailed)?
    }

    let Some(championship) = state.championship_repository.find(path.id).await? else {
        Err(ChampionshipError::NotFound)?
    };

    if !state.f123_service.service_active(championship.id).await {
        Err(F123ServiceError::NotActive)?
    }

    web::ws::start(
        req,
        map_config(fn_factory_with_config(web_socket), move |cfg| {
            (cfg, state.clone(), path.id)
        }),
    )
    .await
}

#[inline(always)]
async fn web_socket(
    (sink, state, championship_id): (ws::WsSink, State<AppState>, i32),
) -> AppResult<impl Service<ws::Frame, Response = Option<Message>, Error = io::Error>> {
    let (tx, close_rx) = oneshot::channel();

    {
        // todo: Use this cache data to send to the client
        let _cache = state
            .f123_repository
            .get_cache_data(championship_id)
            .await?;

        // if let Some(data) = cache {
        //     if sink.send(Message::Binary(Bytes::from(data))).await.is_err() {
        //         return Err(SocketError::FailedToSendMessage.into());
        //     };
        // }
    }

    let Some(rx) = state.f123_service.subscribe(championship_id).await else {
        return Err(F123ServiceError::NotFound.into());
    };

    increment(championship_id);
    rt::spawn(send_data(sink, rx, close_rx));

    let service = fn_service(move |_| ready(Ok(None)));

    let on_shutdown = fn_shutdown(move || {
        decrement(&championship_id);
        let _ = tx.send(());
    });

    Ok(chain(service).and_then(on_shutdown))
}

#[inline(always)]
async fn send_data(sink: ws::WsSink, mut rx: Receiver<Bytes>, mut close_rx: oneshot::Receiver<()>) {
    while let Either::Left(Ok(data)) = select(rx.recv(), &mut close_rx).await {
        if sink.send(Message::Binary(data)).await.is_err() {
            break;
        }
    }
}
