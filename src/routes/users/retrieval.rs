use axum::extract::{Path, State};
use axum::Json;
use log::{error, info};
use sqlx::PgPool;
use crate::AppState;
use crate::errors::AppError;
use crate::models::UserData;

pub(crate) async fn fetch_user(
    Path(user_id): Path<i32>,
    State(app_state): State<AppState>,
) -> Result<Json<UserData>, AppError> {
    let user = get_user_by_id(user_id, &app_state.db_pool).await?;

    Ok(Json(user))
}

pub(crate) async fn fetch_all(
    State(app_state): State<AppState>,
) -> Result<Json<Vec<UserData>>, AppError> {
    let users = get_all_users(&app_state.db_pool).await?;

    Ok(Json(users))
}

async fn get_all_users(db_pool: &PgPool) -> Result<Vec<UserData>, AppError> {
    sqlx::query_as::<_, UserData>(
        "SELECT id, username, email, profile_picture, updated_at, created_at FROM users",
    )
        .fetch_all(db_pool)
        .await
        .map_err(|e| AppError::new(format!("Failed to fetch users: {}", e)))
}

async fn get_user_by_id(user_id: i32, db_pool: &PgPool) -> Result<UserData, AppError> {
    info!("Fetching user data for user_id: {}", user_id);

    sqlx::query_as::<_, UserData>(
        "SELECT id, username, email, profile_picture, created_at, updated_at FROM users WHERE id = $1",
    )
        .bind(user_id)
        .fetch_one(db_pool)
        .await
        .map_err(|e| {
            error!("Failed to fetch user data: {}", e);
            AppError::new(format!("Failed to fetch user data: {}", e))
        })
}