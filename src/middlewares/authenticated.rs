use crate::{
    error::{CommonError, TokenError, UserError},
    repositories::UserRepositoryTrait,
    services::TokenServiceTrait,
    states::AppState,
};
use ntex::{
    service::{Middleware, Service, ServiceCtx},
    util::BoxFuture,
    web,
};
use std::sync::Arc;

const BEARER_PREFIX: &str = "Bearer ";

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
        let state = req.app_state::<AppState>().cloned();
        let header = req.headers().get("Authorization").cloned();

        let fut = async move {
            let state = state.ok_or(CommonError::InternalServerError)?;
            let header = {
                let header = header.ok_or(TokenError::MissingToken)?;
                let header_str = header.to_str().map_err(|_| TokenError::InvalidToken)?;

                if !header_str.starts_with(BEARER_PREFIX) {
                    return Err(TokenError::InvalidToken.into());
                }

                header_str[BEARER_PREFIX.len()..].to_string()
            };

            let id = state.token_service.validate(&header)?.claims.sub;
            let user = state
                .user_repository
                .find(&id)
                .await?
                .ok_or(UserError::NotFound)?;

            if !user.active {
                return Err(web::Error::from(UserError::NotVerified));
            }

            req.extensions_mut().insert(Arc::new(user));
            ctx.call(&self.service, req).await
        };

        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}
