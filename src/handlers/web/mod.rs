use crate::dtos::{FallbackTemplate, IndexTemplate, LoginTemplate, RegisterTemplate};
use askama::Template;
use axum::response::Html;

pub async fn html_index() -> Html<String> {
    let template = IndexTemplate {};
    Html(template.render().unwrap())
}

pub async fn login() -> Html<String> {
    let template = LoginTemplate {};
    Html(template.render().unwrap())
}

pub async fn register() -> Html<String> {
    let template = RegisterTemplate {};
    Html(template.render().unwrap())
}

pub async fn fallback() -> Html<String> {
    let template = FallbackTemplate {};
    Html(template.render().unwrap())
}
