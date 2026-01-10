mod models;
mod http;
mod constants;

use axum::{
    Router,
    http::{
        header::{
            ACCEPT,
            AUTHORIZATION,
            CONTENT_TYPE
        },
        Method,
    },
};
use tower_http::{
    trace::TraceLayer,
    cors::{
        CorsLayer,
        Any,
    },
};
use std::sync::Arc;
use tracing_subscriber::{
    EnvFilter, layer::SubscriberExt,
    util::SubscriberInitExt
};
use tower_http::services::{ServeDir, ServeFile};
use tracing::{
    info,
    debug,
};
use std::{
    str::FromStr,
    env::var,
};
use models::{
    User,
    RegistryClient,
};
use http::{
    health,
    auth,
    fallback_404,
};
use dotenv::dotenv;
use models::{
    AppState,
    Error,
};

const STATIC_DIR: &str = "static";

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();
    let log_level = var("RUST_LOG").unwrap_or("debug".to_string());
    tracing_subscriber::registry()
        .with(EnvFilter::from_str(&log_level).unwrap())
        .with(tracing_subscriber::fmt::layer())
        .init();
    info!("Log level: {log_level}");

    let username = var("USERNAME").expect("USERNAME environment mandatory");
    let hashed_password = var("HASHED_PASSWORD").expect("HASHED_PASSWORD environment mandatory");
    let registry_url = var("REGISTRY_URL").expect("REGISTRY_URL environment mandatory");
    let basic_auth = var("BASIC_AUTH").expect("BASIC_AUTH environment mandatory");
    let port = var("PORT").unwrap_or("3000".to_string());
    info!("Port: {}", port);
    let secret = var("SECRET").unwrap_or("esto-es-un-secreto".to_string());
    debug!("Secret: {}", secret);


    let cors = CorsLayer::new()
        //.allow_origin(url.parse::<HeaderValue>().unwrap())
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::PATCH,
            Method::DELETE])
        //.allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let api_routes = Router::new()
        .nest("/health", health::router())
        .nest("/auth", auth::router())
        .fallback(fallback_404)
        .with_state(Arc::new(AppState {
            secret,
            static_dir: STATIC_DIR.to_string(),
            user: User {
                username,
                hashed_password,
            },
            registry_client: RegistryClient::new(
                registry_url,
                basic_auth),
    }));

    let app = Router::new()
        .nest("/api/v1", api_routes)
        .fallback_service(ServeDir::new(STATIC_DIR)
            .fallback(ServeFile::new("static/index.html")))
        .layer(TraceLayer::new_for_http())
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("🚀 Server started successfully 🚀");
    axum::serve(listener, app).await?;

    Ok(())
}

