use axum::{
    async_trait,
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub struct AppErrorResponse {
    message: Option<String>,
    status_code: StatusCode,
}

impl AppErrorResponse {
    pub fn send(code: StatusCode, message: Option<String>) -> Response {
        AppErrorResponse {
            message,
            status_code: code,
        }
        .into_response()
    }
}

#[async_trait]
impl IntoResponse for AppErrorResponse {
    fn into_response(self) -> Response {
        (
            self.status_code,
            self.message.unwrap_or_else(|| "Unknown Error".to_string()),
        )
            .into_response()
    }
}
