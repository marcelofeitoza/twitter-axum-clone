pub mod auth;
pub mod account;
pub mod password;
pub mod retrieval;

use crate::AppState;
use axum::{middleware, Router, routing::{get, post}};
use auth::{auth, sign_in};
use account::sign_up;
use password::{request_password_reset, reset_password};
use retrieval::{fetch_user, fetch_all};

pub fn user_routes(state: AppState) -> Router<AppState> {
    let middleware_routes = Router::new()
        .route("/", get(fetch_all))
        .route("/:id", get(fetch_user))
        .layer(middleware::from_fn_with_state(state, auth));

    Router::new()
        .route("/signin", post(sign_in))
        .route("/signup", post(sign_up))
        .route("/forgot-password", post(request_password_reset))
        .route("/reset-password", post(reset_password))
        .nest("/", middleware_routes)
}
