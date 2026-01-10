use axum::{Router, extract::State, http::StatusCode, response::IntoResponse, routing};

use crate::AppState;

use std::sync::Arc;
use tracing::{debug, error};

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/", routing::get(get_repositories))
}

async fn get_repositories(State(app_state): State<Arc<AppState>>) -> impl IntoResponse {
    debug!("Fetching repositories from registry");
    app_state.registry_client.clone().get_catalog().await
}
