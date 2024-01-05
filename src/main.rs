mod errors;
mod routes;
mod models;
mod utils;

use routes::users;

use axum::{routing::get, Json, Router, http::StatusCode};
use sqlx::PgPool;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let db_connection_string =
        "postgres://twitter_clone:twitter_clone@localhost:5432/?sslmode=disable";
    let db_pool = PgPool::connect(db_connection_string).await?;

    let app_state = AppState { db_pool, jwt_secret: "my_secret".to_string() };

    let api_routes = Router::new().nest("/users", users::user_routes(app_state.clone()));

    let app = Router::new()
        .nest("/v1", api_routes)
        .route("/", get(|| async { "Hello, X!" }))
        .fallback(fallback)
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:5500").await?;
    println!("Listening at http://{}/ 🦀", listener.local_addr().unwrap());
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub jwt_secret: String,
}

async fn fallback() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "status": "Not Found" })),
    )
}

