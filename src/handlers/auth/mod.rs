use crate::{dtos::RegisterUserDto, error::AppResult};
use axum::{
    extract::Form,
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub(crate) async fn register(Form(form): Form<RegisterUserDto>) -> AppResult<Response> {
    Ok(StatusCode::OK.into_response())
}
