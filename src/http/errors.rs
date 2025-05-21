use actix_web::error::PayloadError;
use sqlx::Error as SqlxError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Database error: {0}")]
    Database(#[from] SqlxError),

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("User not found")]
    UserNotFound,

    #[error("Balance too low for transaction")]
    BalanceLow,

    #[error("JWT error: {0}")]
    Jwt(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Payload error: {0}")]
    Payload(#[from] PayloadError),

    #[error("Internal server error")]
    InternalServerError,
}

pub type Result<T> = std::result::Result<T, ApiError>;

use actix_web::{HttpResponse, ResponseError};

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ApiError::InvalidCredentials | ApiError::Unauthorized => {
                HttpResponse::Unauthorized().body(self.to_string())
            }
            ApiError::UserNotFound => HttpResponse::NotFound().body(self.to_string()),
            ApiError::BalanceLow => HttpResponse::BadRequest().body(self.to_string()),
            ApiError::Validation(_) => HttpResponse::BadRequest().body(self.to_string()),
            ApiError::Payload(_) => HttpResponse::BadRequest().body(self.to_string()),
            ApiError::Database(_) | ApiError::InternalServerError | ApiError::Jwt(_) => {
                HttpResponse::InternalServerError().body(self.to_string())
            }
        }
    }
}
