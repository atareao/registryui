use axum::http::StatusCode;
use crate::models::{ApiResponse, Data};

pub mod health;
pub mod auth;

pub async fn fallback_404() -> impl axum::response::IntoResponse {
    ApiResponse::new(
        StatusCode::NOT_FOUND,
        "Not found",
        Data::None
    )
}

