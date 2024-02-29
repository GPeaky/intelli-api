use ntex::{
    service::{Middleware, Service, ServiceCtx},
    web::{Error, WebRequest, WebResponse},
};
use tracing::info;

pub struct LoginLimit;

impl<S> Middleware<S> for LoginLimit {
    type Service = LoginLimitMiddleware<S>;

    fn create(&self, service: S) -> Self::Service {
        LoginLimitMiddleware { service }
    }
}

pub struct LoginLimitMiddleware<S> {
    service: S,
}

impl<S, Err> Service<WebRequest<Err>> for LoginLimitMiddleware<S>
where
    S: Service<WebRequest<Err>, Response = WebResponse, Error = Error>,
{
    type Response = WebResponse;
    type Error = Error;

    ntex::forward_poll_ready!(service);
    ntex::forward_poll_shutdown!(service);

    async fn call(
        &self,
        req: WebRequest<Err>,
        ctx: ServiceCtx<'_, Self>,
    ) -> Result<Self::Response, Self::Error> {
        let headers = req.headers();

        let ip = headers.get("CF-Connecting-IP");
        info!("headers: {:?}", ip);

        let res = ctx.call(&self.service, req).await?;
        Ok(res)
    }
}
