use axum::{
    routing,
    Router,
    response::IntoResponse,
};
use crate::models::{ApiResponse, AppState};
use std::sync::Arc;


pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", routing::get(check_health))
}

async fn check_health() -> impl IntoResponse {
    ApiResponse::<serde_json::Value>::success("Up and running",None)
}


