use ntex::{
    web::{Error, WebRequest, WebResponse},
    Middleware, Service, ServiceCtx,
};

use crate::{
    entity::{Role, UserExtension},
    error::UserError,
};

pub struct Admin;

impl<S> Middleware<S> for Admin {
    type Service = AdminMiddleware<S>;

    fn create(&self, service: S) -> Self::Service {
        AdminMiddleware { service }
    }
}

pub struct AdminMiddleware<S> {
    service: S,
}

impl<S, Err> Service<WebRequest<Err>> for AdminMiddleware<S>
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
        let Some(user) = req.extensions().get::<UserExtension>().cloned() else {
            return Err(UserError::Unauthorized)?;
        };

        if user.role != Role::Admin {
            return Err(UserError::Unauthorized)?;
        }

        let res = ctx.call(&self.service, req).await?;
        Ok(res)
    }
}
