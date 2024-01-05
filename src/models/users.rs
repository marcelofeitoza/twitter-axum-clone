use crate::errors::AppError;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, FromRow, Debug)]
pub struct UserData {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub profile_picture: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct SignUpPayload {
    pub username: String,
    pub email: String,
    pub password: String,
}

impl SignUpPayload {
    pub fn verify(&self) -> Result<(), AppError> {
        if self.username.is_empty() || self.username.len() < 3 {
            Err(AppError::new("Verify username field: It might not be empty and contain at least 5 characters".to_string()))
        } else if self.email.is_empty() || !self.email.contains("@") {
            Err(AppError::new("Verify email filed".to_string()))
        } else if self.password.is_empty() {
            Err(AppError::new("Verify password field: It might not be empty and contain at least 8 characters".to_string()))
        } else {
            Ok(())
        }
    }
}

#[derive(Serialize)]
pub struct SignUpResponse {
    pub token: String,
    pub user: UserData,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

#[derive(Serialize)]
pub struct SignInResponse {
    pub token: String,
}

#[derive(Deserialize)]
pub struct SignInPayload {
    pub username: String,
    pub password: String,
}

impl SignInPayload {
    pub fn verify(&self) -> Result<(), AppError> {
        if self.username.is_empty() {
            Err(AppError::new("Username is missing".to_string()))
        } else if self.password.is_empty() {
            Err(AppError::new("Password is missing".to_string()))
        } else {
            Ok(())
        }
    }
}

#[derive(sqlx::FromRow)]
pub struct UserPassword {
    pub hashed_password: String,
}

#[derive(Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
}

#[derive(sqlx::FromRow)]
pub struct TokenData {
    pub user_id: i32,
    pub expires_at: DateTime<Utc>,
    pub used: bool,
}


#[derive(sqlx::FromRow)]
pub struct UserId {
    pub id: i32,
}
