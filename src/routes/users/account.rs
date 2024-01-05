use axum::extract::State;
use axum::Json;
use bcrypt::{DEFAULT_COST, hash};
use log::{info, warn};
use crate::AppState;
use crate::errors::AppError;
use crate::models::{SignUpPayload, SignUpResponse, UserData};
use crate::utils::{generate_claims, generate_jwt};

pub(crate) async fn sign_up(
    State(app_state): State<AppState>,
    Json(payload): Json<SignUpPayload>,
) -> Result<Json<SignUpResponse>, AppError> {
    payload.verify()?;

    let user = create_user(&app_state, &payload).await?;

    let claims = generate_claims(user.id.to_string(), 30);
    let token = generate_jwt(claims, app_state.jwt_secret.as_bytes())?;

    Ok(Json(SignUpResponse { token, user }))
}


async fn create_user(app_state: &AppState, payload: &SignUpPayload) -> Result<UserData, AppError> {
    info!("Creating user for username: {}", payload.username);

    let username = &payload.username;
    let email = &payload.email;
    let password = &payload.password;

    let db_pool = &app_state.db_pool;

    let user = sqlx::query_as::<_, UserData>(
        "SELECT id, username, email, profile_picture, created_at, updated_at FROM users WHERE username = $1"
    )
        .bind(username)
        .fetch_optional(db_pool)
        .await?;

    if let Some(user) = user {

        warn!("User already exists with id {}", user.id);

        return Err(AppError::new(format!("User already exists with id {}", user.id)));
    }

    let hashed_password = hash(password, DEFAULT_COST)
        .map_err(|e| AppError::new(format!("Failed to hash password: {}", e)))?;

    let user_image_response = reqwest::get(format!("https://github.com/{}.png", username)).await?;
    let user_image_url = if user_image_response.status() == reqwest::StatusCode::NOT_FOUND {
        String::from("https://placehold.co/512x512")
    } else {
        format!("https://github.com/{}.png", username)
    };

    let _ = sqlx::query(
        "INSERT INTO users (username, email, hashed_password, profile_picture) VALUES ($1, $2, $3, $4) RETURNING id",
    )
        .bind(&username)
        .bind(&email)
        .bind(&hashed_password)
        .bind(&user_image_url)
        .execute(db_pool)
        .await
        .map_err(|e| AppError::new(format!("Failed to create user: {}", e)));

    info!("User created with username: {}", username);

    sqlx::query_as::<_, UserData>(
        "SELECT id, username, email, profile_picture, created_at, updated_at FROM users WHERE username = $1 AND email = $2"
    )
        .bind(username)
        .bind(email)
        .fetch_one(db_pool)
        .await
        .map_err(|e| AppError::new(format!("Failed to find the user after insertion: {}", e)))
}