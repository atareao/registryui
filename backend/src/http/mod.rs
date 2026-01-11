use crate::models::ApiResponse;

pub mod health;
pub mod auth;
pub mod registry;

pub async fn fallback_404() -> impl axum::response::IntoResponse {
    ApiResponse::<serde_json::Value>::success( "Not found",None
    )
}

