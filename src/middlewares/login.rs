use std::{
    net::IpAddr,
    str::FromStr,
    time::{Duration, Instant},
};

use dashmap::DashMap;
use ntex::{
    service::{Middleware, Service, ServiceCtx},
    web::{Error, WebRequest, WebResponse},
};
use tracing::warn;

use crate::error::CommonError;

const RATE_LIMIT: u8 = 5;
const RATE_LIMIT_DURATION: Duration = Duration::from_secs(120);

pub type VisitorData = (u8, Instant);

pub struct LoginLimit {
    visitors: &'static DashMap<IpAddr, VisitorData>,
}

impl LoginLimit {
    pub fn new(visitors: &'static DashMap<IpAddr, VisitorData>) -> Self {
        LoginLimit { visitors }
    }
}

impl<S> Middleware<S> for LoginLimit {
    type Service = LoginLimitMiddleware<S>;

    fn create(&self, service: S) -> Self::Service {
        LoginLimitMiddleware {
            service,
            visitors: self.visitors,
        }
    }
}

pub struct LoginLimitMiddleware<S> {
    service: S,
    visitors: &'static DashMap<IpAddr, VisitorData>,
}

impl<S, Err> Service<WebRequest<Err>> for LoginLimitMiddleware<S>
where
    S: Service<WebRequest<Err>, Response = WebResponse, Error = Error>,
{
    type Error = Error;
    type Response = WebResponse;

    ntex::forward_poll_ready!(service);
    ntex::forward_poll_shutdown!(service);

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
                // This should be okay in prod because only cloudflare can put the header
                let ip_str = unsafe { std::str::from_utf8_unchecked(ip.as_ref()) };
                IpAddr::from_str(ip_str).map_err(|_| CommonError::InternalServerError)
            }?;

            let mut entry = self
                .visitors
                .entry(ip)
                .or_insert((0, now + RATE_LIMIT_DURATION));

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
