use crate::errors::AppError;
use crate::models::Claims;

use jsonwebtoken::{decode, DecodingKey, encode, EncodingKey, Header, Validation};

pub fn verify_jwt(token: &str, secret_key: &[u8]) -> Result<Claims, AppError> {
    let mut validation = Validation::default();
    validation.validate_exp = true;

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret_key),
        &validation,
    )
        .map(|data| data.claims)
        .map_err(|_| AppError::new("Token is invalid or expired".to_string()))
}


pub fn generate_jwt(claims: Claims, secret: &[u8]) -> Result<String, AppError> {
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret))
        .map_err(|_| AppError::new("Token creation failed... Try again later".to_string()))
}

pub fn generate_claims(user_id: String, duration_as_minutes: i64) -> Claims {
    Claims {
        sub: user_id,
        exp: (chrono::Utc::now() + chrono::Duration::seconds(duration_as_minutes)).timestamp() as usize,
    }
}