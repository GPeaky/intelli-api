use std::sync::Arc;

use ntex::{
    service::{Middleware, Service, ServiceCtx},
    util::BoxFuture,
    web,
};

use crate::{error::{TokenError, UserError}, states::AppState, services::TokenServiceTrait, repositories::UserRepositoryTrait};

pub struct Authentication;

impl<S> Middleware<S> for Authentication {
    type Service = AuthenticationMiddleware<S>;

    fn create(&self, service: S) -> Self::Service {
        AuthenticationMiddleware { service }
    }
}

pub struct AuthenticationMiddleware<S> {
    service: S,
}

impl<S, Err> Service<web::WebRequest<Err>> for AuthenticationMiddleware<S>
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
        if let Some(state) = req.app_state::<AppState>() {
            let header = req
                .headers()
                .get("Authorization")
                .ok_or(TokenError::MissingToken)?;

            let mut header = header.to_str().map_err(|_| TokenError::InvalidToken)?;
            if !header.starts_with("Bearer ") {
                return Box::pin(async move { Err(TokenError::InvalidToken.into()) });
            }

            header = &header[7..];

            let token = state.token_service.validate(header)?;

            if let Some(user) = state.user_repository.find(&token.claims.sub).await? else {
                return Err(UserError::NotFound)?
            }

            if !user.active {
                return Err(UserError::Inactive)?
            }

            req.extensions_mut().insert(Arc::new(user))

            ctx.call(&self.service, req);
        } else {
            panic!("No app state")
        }
    }
}
