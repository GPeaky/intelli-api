use std::{
    net::{IpAddr, Ipv4Addr},
    str::FromStr,
    sync::Arc,
    time::{Duration, Instant},
};

use ahash::AHashMap;
use ntex::{
    service::{Middleware, Service, ServiceCtx},
    web::{Error, WebRequest, WebResponse},
};
use parking_lot::Mutex;
use tokio::time::interval;
use tracing::warn;

use crate::error::CommonError;

const RATE_LIMIT: u8 = 5;
const DEFAULT_SIZE: usize = 1000;
const RATE_LIMIT_DURATION: Duration = Duration::from_secs(120);

pub struct LoginLimit;

impl<S> Middleware<S> for LoginLimit {
    type Service = LoginLimitMiddleware<S>;

    fn create(&self, service: S) -> Self::Service {
        let visitors = Arc::from(Mutex::from(AHashMap::with_capacity(DEFAULT_SIZE)));

        tokio::spawn({
            let visitors = visitors.clone();
            async move {
                let mut interval = interval(Duration::from_hours(1));

                loop {
                    interval.tick().await;

                    let mut visitors = visitors.lock();
                    visitors
                        .retain(|_, &mut (_, ref instant)| instant.elapsed() < RATE_LIMIT_DURATION);

                    visitors.shrink_to(DEFAULT_SIZE)
                }
            }
        });

        LoginLimitMiddleware { service, visitors }
    }
}

pub struct LoginLimitMiddleware<S> {
    service: S,
    visitors: Arc<Mutex<AHashMap<IpAddr, (u8, Instant)>>>,
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
        let ip = req.headers().get("CF-Connecting-IP");

        // Only rate limit if the request is coming from the cloudflare proxy
        if let Some(ip) = ip {
            let now = Instant::now();
            let ip = {
                let ip_str = unsafe { std::str::from_utf8_unchecked(ip.as_ref()) };
                IpAddr::from_str(ip_str).unwrap()
            };

            let mut visitors = self.visitors.lock();
            let entry = visitors.entry(ip).or_insert((0, now + RATE_LIMIT_DURATION));

            if now > entry.1 {
                *entry = (0, now + RATE_LIMIT_DURATION);
            } else if entry.0 > RATE_LIMIT {
                return Err(CommonError::LoginRateLimited)?;
            }

            entry.0 += 1;
        } else {
            warn!("No CF-Connecting-IP header, not rate limiting");
        }

        let res = ctx.call(&self.service, req).await?;
        Ok(res)
    }
}
