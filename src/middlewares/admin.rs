use crate::{
    entity::{Role, UserExtension},
    error::UserError,
};
use ntex::{util::BoxFuture, web, Middleware, Service, ServiceCtx};

pub struct Admin;
pub struct AdminMiddleware<S> {
    service: S,
}

impl<S> Middleware<S> for Admin {
    type Service = AdminMiddleware<S>;

    fn create(&self, service: S) -> Self::Service {
        AdminMiddleware { service }
    }
}

impl<S, Err> Service<web::WebRequest<Err>> for AdminMiddleware<S>
where
    S: Service<web::WebRequest<Err>, Response = web::WebResponse, Error = web::Error>,
    Err: web::ErrorRenderer,
{
    type Response = web::WebResponse;
    type Error = web::Error;
    type Future<'f> = BoxFuture<'f, Result<Self::Response, Self::Error>> where Self: 'f;

    ntex::forward_poll_ready!(service);

    fn call<'a>(
        &'a self,
        req: web::WebRequest<Err>,
        ctx: ServiceCtx<'a, Self>,
    ) -> Self::Future<'_> {
        let Some(user) = req.extensions().get::<UserExtension>().cloned() else {
            return Box::pin(async { Err(web::Error::from(UserError::Unauthorized))? });
        };

        if user.role != Role::Admin {
            return Box::pin(async { Err(web::Error::from(UserError::Unauthorized))? });
        }

        Box::pin(ctx.call(&self.service, req))
    }
}
