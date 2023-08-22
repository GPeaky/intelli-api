use crate::dtos::IndexTemplate;
use askama::Template;
use axum::response::Html;

pub async fn html_index() -> Html<String> {
    let template = IndexTemplate {};
    Html(template.render().unwrap())
}
