use axum::extract::State;
use axum::Json;
use axum::response::Response;
use bcrypt::{DEFAULT_COST, hash};
use chrono::{Duration, Utc};
use log::{error, info};
use rand::Rng;
use sqlx::PgPool;
use crate::AppState;
use crate::errors::AppError;
use crate::models::{ForgotPasswordRequest, ResetPasswordRequest, TokenData, UserData};

pub(crate) async fn request_password_reset(
    State(app_state): State<AppState>,
    Json(payload): Json<ForgotPasswordRequest>,
) -> Result<Response, AppError> {
    let user = get_user_by_email(&app_state.db_pool, &payload.email).await?;
    let reset_token = generate_password_reset_token();

    store_password_reset_token(&app_state.db_pool, user.id, &reset_token).await?;
    send_password_reset_email(&user.email, &reset_token);

    Ok(Response::new("Password reset email sent successfully".into()))
}


async fn get_user_by_email(db_pool: &PgPool, email: &str) -> Result<UserData, AppError> {
    info!("Fetching user data for email: {}", email);

    sqlx::query_as::<_, UserData>(
        "SELECT id, username, email, profile_picture, created_at, updated_at FROM users WHERE email = $1",
    )
        .bind(email)
        .fetch_one(db_pool)
        .await
        .map_err(|e| {
            error!("Failed to fetch user data: {}", e);
            AppError::new(format!("Failed to fetch user data: {}", e))
        })
}

async fn store_password_reset_token(db_pool: &PgPool, user_id: i32, reset_token: &str) -> Result<(), AppError> {
    let expiration_time = Utc::now() + Duration::minutes(30);

    sqlx::query(
        "INSERT INTO user_tokens (user_id, token, token_type, expires_at, used) VALUES ($1, $2, 'password_reset', $3, false)",
    )
        .bind(user_id)
        .bind(reset_token)
        .bind(expiration_time)
        .execute(db_pool)
        .await
        .map_err(|_| AppError::new("Failed to store password reset token".to_string()))?;

    Ok(())
}

fn send_password_reset_email(email: &str, reset_token: &str) {
    println!("Email: {}\nReset token: {}", email, reset_token);
}

fn generate_password_reset_token() -> String {
    let token_length = 30;
    let mut rng = rand::thread_rng();

    let token: String = std::iter::repeat(())
        .map(|()| rng.sample(rand::distributions::Alphanumeric))
        .map(char::from)
        .take(token_length)
        .collect();

    token
}


pub(crate) async fn reset_password(
    State(app_state): State<AppState>,
    Json(payload): Json<ResetPasswordRequest>,
) -> Result<Response, AppError> {
    let user_id = verify_password_reset_token(&app_state.db_pool, &payload.token).await?;

    let hashed_new_password = hash(&payload.new_password, DEFAULT_COST)
        .map_err(|e| AppError::new(format!("Failed to hash new password: {}", e)))?;

    sqlx::query(
        "UPDATE users SET hashed_password = $1 WHERE id = $2"
    )
        .bind(hashed_new_password)
        .bind(user_id)
        .execute(&app_state.db_pool)
        .await
        .map_err(|e| AppError::new(format!("Failed to update password: {}", e)))?;

    sqlx::query(
        "UPDATE user_tokens SET used = TRUE WHERE user_id = $1 AND token = $2",
    )
        .bind(user_id)
        .bind(&payload.token)
        .execute(&app_state.db_pool)
        .await
        .map_err(|e| AppError::new(format!("Failed to invalidate password reset token: {}", e)))?;

    Ok(Response::new("Password has been reset successfully".into()))
}


async fn verify_password_reset_token(db_pool: &PgPool, token: &str) -> Result<i32, AppError> {
    let token_data = sqlx::query_as::<_, TokenData>(
        "SELECT user_id, expires_at, used FROM user_tokens WHERE token = $1",
    )
        .bind(token)
        .fetch_optional(db_pool)
        .await?
        .ok_or_else(|| AppError::new("Invalid or expired token".to_string()))?;

    if token_data.used || Utc::now() > token_data.expires_at {
        return Err(AppError::new("Invalid or expired token".to_string()));
    }

    Ok(token_data.user_id)
}