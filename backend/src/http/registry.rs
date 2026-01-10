use axum::{
    Router,
    extract::{
        Query,
        State
    },
    response::IntoResponse,
    routing
};

use crate::AppState;

use std::sync::Arc;
use tracing::debug;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", routing::get(get_repositories))
        .route("/{respository}", routing::get(get_tags))
}

async fn get_repositories(State(app_state): State<Arc<AppState>>) -> impl IntoResponse {
    debug!("Fetching repositories from registry");
    app_state.registry_client.clone().get_catalog().await
}

struct Params {
    repository: String,
}

async fn get_tags(
    State(app_state): State<Arc<AppState>>,
    params: Query<Params>,
) -> impl IntoResponse {
    let repository = params.repository.to_string();
    debug!("Fetching tags for repository: {}", &repository);
    app_state
        .registry_client
        .clone()
        .get_tags(repository)
        .await
}
