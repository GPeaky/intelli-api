use std::sync::Arc;

use ntex::{
    service::{Middleware, Service, ServiceCtx},
    web::{Error, WebRequest, WebResponse},
};

use crate::{
    error::{CommonError, TokenError, UserError},
    repositories::UserRepositoryTrait,
    services::TokenServiceTrait,
    states::AppState,
};

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

impl<S, Err> Service<WebRequest<Err>> for AuthenticationMiddleware<S>
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
        let Some(state) = req.app_state::<AppState>() else {
            Err(CommonError::InternalServerError)?
        };

        let Some(header) = req.headers().get("Authorization") else {
            Err(TokenError::MissingToken)?
        };

        let header = {
            let header_str = header.to_str().map_err(|_| TokenError::InvalidToken)?;

            if !header_str.starts_with(BEARER_PREFIX) {
                return Err(TokenError::InvalidToken)?;
            }

            &header_str[BEARER_PREFIX.len()..]
        };

        let id = state.token_svc.validate(header)?.claims.sub;
        let user = state.user_repo.find(id).await?.ok_or(UserError::NotFound)?;

        req.extensions_mut().insert(Arc::new(user));
        let res = ctx.call(&self.service, req).await?;
        Ok(res)
    }
}
