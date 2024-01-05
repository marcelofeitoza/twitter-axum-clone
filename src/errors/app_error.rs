use anyhow::Error;
use axum::{http::StatusCode, response::{IntoResponse, Response}};
use std::fmt;

pub struct AppError(pub Error);

impl AppError {
    pub fn new(msg: String) -> AppError {
        AppError(Error::msg(msg))
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "❌ Something went wrong: {}", self.0)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            self.to_string(),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError where E: Into<Error> {
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
