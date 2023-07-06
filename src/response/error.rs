use axum::{
    async_trait,
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub struct AppErrorResponse {
    message: Option<String>,
    code: StatusCode,
}

impl AppErrorResponse {
    pub fn send(code: StatusCode, message: Option<String>) -> Response {
        AppErrorResponse { code, message }.into_response()
    }
}

#[async_trait]
impl IntoResponse for AppErrorResponse {
    fn into_response(self) -> Response {
        (
            self.code,
            self.message
                .unwrap_or_else(|| "Internal Server Error".to_string()),
        )
            .into_response()
    }
}
