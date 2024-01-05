use axum::extract::{Request, State};
use axum::Json;
use axum::middleware::Next;
use axum::response::Response;
use log::{error, info};
use sqlx::PgPool;
use crate::AppState;
use crate::errors::AppError;
use crate::models::{SignInPayload, SignInResponse, UserId, UserPassword};
use crate::utils::{generate_claims, generate_jwt, verify_jwt};

pub async fn auth(State(app_state): State<AppState>, mut request: Request, next: Next) -> Result<Response, AppError> {
    let jwt_token = request.headers()
        .get("Authorization")
        .and_then(|header| header.to_str().ok())
        .ok_or(AppError::new("The Authorization header is invalid".to_string()))?
        .trim_start_matches("Bearer ");

    let user_claims = verify_jwt(jwt_token, app_state.jwt_secret.as_bytes())?;
    request.extensions_mut().insert(user_claims);

    let response = next.run(request).await;
    Ok(response)
}

pub(crate) async fn sign_in(
    State(app_state): State<AppState>,
    Json(payload): Json<SignInPayload>,
) -> Result<Json<SignInResponse>, AppError> {
    info!("Signing in user: {}", payload.username);

    payload.verify()?;

    let username = &payload.username;
    let password = &payload.password;

    let user_password = get_user_password(username, &app_state.db_pool).await?;

    let is_password_valid = bcrypt::verify(password, &user_password.hashed_password)
        .map_err(|_| AppError::new("Failed to verify password".to_string()))?;

    if is_password_valid {
        info!("Password verified for user: {}", username);

        let user_id = sqlx::query_as::<_, UserId>(
            "SELECT id FROM users WHERE username = $1"
        )
            .bind(username)
            .fetch_one(&app_state.db_pool)
            .await
            .map_err(|e| AppError::new(format!("Could not find user in database: {}", e)))?;

        let claims = generate_claims(user_id.id.to_string(), 30);
        let token = generate_jwt(claims, app_state.jwt_secret.as_bytes())?;

        return Ok(Json(SignInResponse { token }));
    }

    error!("Invalid username or password for user: {}", username);
    Err(AppError::new("Invalid username or password".to_string()))
}

async fn get_user_password(username: &str, db_pool: &PgPool) -> Result<UserPassword, AppError> {
    sqlx::query_as::<_, UserPassword>(
        "SELECT hashed_password FROM users WHERE username = $1"
    )
        .bind(username)
        .fetch_one(db_pool)
        .await
        .map_err(|e| AppError::new(format!("User not found in database: {}", e)))
}
