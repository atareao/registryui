use axum::{
    routing,
    Router,
    response::IntoResponse,
    http::StatusCode,
};
use crate::models::{Data, ApiResponse, AppState};
use std::sync::Arc;


pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", routing::get(check_health))
}

async fn check_health() -> impl IntoResponse {
    ApiResponse::new(StatusCode::OK, "Up and running", Data::None)
}


