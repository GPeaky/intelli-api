use ntex::{
    web::{Error, WebRequest, WebResponse},
    Middleware, Service, ServiceCtx,
};
use tracing::Span;

pub struct Tracing;

impl<S> Middleware<S> for Tracing {
    type Service = TracingMiddleware<S>;

    fn create(&self, service: S) -> Self::Service {
        TracingMiddleware { service }
    }
}

pub struct TracingMiddleware<S> {
    service: S,
}

impl<S, Err> Service<WebRequest<Err>> for TracingMiddleware<S>
where
    S: Service<WebRequest<Err>, Response = WebResponse, Error = Error>,
{
    type Response = WebResponse;
    type Error = Error;

    ntex::forward_ready!(service);

    #[tracing::instrument(skip(self, req, ctx), fields(
        http.method = %req.method(),
        http.uri = %req.uri(),
        http.version = ?req.version(),
        http.user_agent = req.headers().get("user-agent").map(|h| h.to_str().unwrap_or("")).unwrap_or(""),
    ))]
    async fn call(
        &self,
        req: WebRequest<Err>,
        ctx: ServiceCtx<'_, Self>,
    ) -> Result<Self::Response, Self::Error> {
        let res = ctx.call(&self.service, req).await?;

        Span::current().record(
            "http.status_code",
            tracing::field::display(res.status().as_u16()),
        );

        Ok(res)
    }
}
