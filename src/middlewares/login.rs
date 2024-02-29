use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use ahash::AHashMap;
use compact_str::CompactString;
use ntex::{
    service::{Middleware, Service, ServiceCtx},
    web::{Error, WebRequest, WebResponse},
};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use tracing::{info, warn};

use crate::{error::CommonError, utils::CachedTime};

const RATE_LIMIT: u8 = 5;
const RATE_LIMIT_DURATION: Duration = Duration::from_secs(120);
static CACHED_TIME: Lazy<Arc<CachedTime>> = Lazy::new(CachedTime::new);

pub struct LoginLimit;

impl<S> Middleware<S> for LoginLimit {
    type Service = LoginLimitMiddleware<S>;

    fn create(&self, service: S) -> Self::Service {
        let visitors = Arc::from(Mutex::from(AHashMap::with_capacity(100)));
        LoginLimitMiddleware { service, visitors }
    }
}

pub struct LoginLimitMiddleware<S> {
    service: S,
    visitors: Arc<Mutex<AHashMap<CompactString, (u8, Instant)>>>,
}

impl<S, Err> Service<WebRequest<Err>> for LoginLimitMiddleware<S>
where
    S: Service<WebRequest<Err>, Response = WebResponse, Error = Error>,
{
    type Response = WebResponse;
    type Error = Error;

    ntex::forward_poll_ready!(service);
    ntex::forward_poll_shutdown!(service);

    // Todo: Optimize this method to reduce the number of locks
    async fn call(
        &self,
        req: WebRequest<Err>,
        ctx: ServiceCtx<'_, Self>,
    ) -> Result<Self::Response, Self::Error> {
        let time = std::time::Instant::now();
        let ip = req.headers().get("CF-Connecting-IP");

        // Only rate limit if the request is coming from the cloudflare proxy
        if let Some(ip) = ip {
            let now = CACHED_TIME.instant();
            let ip = CompactString::from(ip.to_str().unwrap());
            let mut visitors = self.visitors.lock();
            let entry = visitors.entry(ip).or_insert((0, now + RATE_LIMIT_DURATION));

            if now > entry.1 {
                *entry = (1, now + RATE_LIMIT_DURATION);
            } else if entry.0 > RATE_LIMIT {
                return Err(CommonError::LoginRateLimited)?;
            } else {
                entry.0 += 1;
            }
        } else {
            warn!("No CF-Connecting-IP header, not rate limiting");
        }

        let time = time.elapsed();
        info!("Time taken: {:?}", time);

        let res = ctx.call(&self.service, req).await?;
        Ok(res)
    }
}
