use std::sync::Arc;

use axum::{
    body,
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing, Json, Router,
};
use bcrypt::verify;
use tracing::{debug, error};

use axum_extra::extract::cookie::{Cookie, SameSite};
use jsonwebtoken::{encode, EncodingKey, Header};

use crate::models::{ApiResponse, AppState, TokenClaims, User};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", routing::post(login))
        .route("/logout", routing::get(logout))
}

type Result = std::result::Result<ApiResponse, ApiResponse>;

pub async fn login(State(app_state): State<Arc<AppState>>, Json(user_pass): Json<User>) -> Result {
    //) -> Result<Json<serde_json::Value>,(StatusCode, Json<serde_json::Value>)>{
    tracing::info!("init login");
    tracing::info!("User pass: {:?}", user_pass);
    let registered_user = app_state.user.clone();
    if !verify(&user_pass.hashed_password, &registered_user.hashed_password).unwrap() {
        let message = "Invalid name or password";
        error!("{}", message);
        return Err(ApiResponse::error(StatusCode::FORBIDDEN, message));
    }

    let now = chrono::Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + chrono::Duration::minutes(60)).timestamp() as usize;
    let claims: TokenClaims = TokenClaims {
        sub: registered_user.username.to_string(),
        exp,
        iat,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(app_state.secret.as_bytes()),
    )
    .map_err(|e| {
        let message = format!("Encoding JWT error: {}", e);
        error!("{}", message);
        ApiResponse::error(StatusCode::INTERNAL_SERVER_ERROR, &message)
    })
    .map(|token| {
        let value = serde_json::json!({"token": token});
        ApiResponse::success("Ok", Some(value))
    })
}

pub async fn logout() -> impl IntoResponse {
    debug!("Logout");
    let cookie = Cookie::build(("token", ""))
        .path("/")
        .max_age(cookie::time::Duration::ZERO)
        .same_site(SameSite::Lax)
        .http_only(true)
        .build();

    tracing::info!("The cookie: {}", cookie.to_string());

    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, "/")
        .header(header::SET_COOKIE, cookie.to_string())
        .body(body::Body::empty())
        .unwrap()
}
