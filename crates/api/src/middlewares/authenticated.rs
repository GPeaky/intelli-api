use ntex::{
    service::{Middleware, Service, ServiceCtx},
    web::{Error, WebRequest, WebResponse},
};

use error::{CommonError, TokenError, UserError};
use token::{Token, TokenIntent};

use crate::states::AppState;

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

    ntex::forward_ready!(service);

    async fn call(
        &self,
        req: WebRequest<Err>,
        ctx: ServiceCtx<'_, Self>,
    ) -> Result<Self::Response, Self::Error> {
        let Some(header) = req.headers().get("Authorization") else {
            Err(TokenError::MissingToken)?
        };

        let token = {
            let token_str = header.to_str().map_err(|_| TokenError::InvalidToken)?;

            Token::from_base64(token_str)?
        };

        let Some(state) = req.app_state::<AppState>() else {
            Err(CommonError::InternalServerError)?
        };

        let id = state.token_mgr.validate(&token, TokenIntent::Auth)?;
        let user = state.user_repo.find(id).await?.ok_or(UserError::NotFound)?;
        req.extensions_mut().insert(user);

        let res = ctx.call(&self.service, req).await?;
        Ok(res)
    }
}
