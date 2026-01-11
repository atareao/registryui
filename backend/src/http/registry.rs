use serde::Deserialize;
use super::ApiResponse;
use axum::{
    Router,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing,
};

use crate::AppState;

use std::sync::Arc;
use tracing::debug;

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/", routing::get(get_repositories))
}

#[derive(Deserialize)]
struct Params {
    repository: Option<String>,
}

#[axum::debug_handler]
async fn get_repositories(
    State(app_state): State<Arc<AppState>>,
    Query(params): Query<Params>,
) -> impl IntoResponse {
    match &params.repository {
        Some(repo) => {
            if repo.is_empty() {
                debug!("Repository parameter is empty, fetching all repositories");
                return ApiResponse::error(
                    StatusCode::BAD_REQUEST,
                    "El parámetro 'repository' no puede estar vacío",
                )
                .into_response();
            }
            debug!("Fetching tags for repository: {}", repo);
            app_state
                .registry_client
                .clone()
                .get_tags(repo)
                .await
                .into_response()
        }
        _ => {
            debug!("Fetching all repositories");
            app_state.registry_client.clone()
                .get_catalog()
                .await
                .into_response()
        }
    }
}
