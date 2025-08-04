use actix_web::{HttpResponse, ResponseError};
use serde_json::json;
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    DatabaseError(sqlx::Error),
    ValidationError(String),
    NotFound(String),
    Unauthorized(String),
    InternalError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::DatabaseError(err) => write!(f, "Database error: {}", err),
            AppError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            AppError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::DatabaseError(_) => {
                HttpResponse::InternalServerError().json(json!({
                    "error": "Database error occurred",
                    "code": "DATABASE_ERROR"
                }))
            }
            AppError::ValidationError(msg) => {
                HttpResponse::BadRequest().json(json!({
                    "error": msg,
                    "code": "VALIDATION_ERROR"
                }))
            }
            AppError::NotFound(msg) => {
                HttpResponse::NotFound().json(json!({
                    "error": msg,
                    "code": "NOT_FOUND"
                }))
            }
            AppError::Unauthorized(msg) => {
                HttpResponse::Unauthorized().json(json!({
                    "error": msg,
                    "code": "UNAUTHORIZED"
                }))
            }
            AppError::InternalError(msg) => {
                HttpResponse::InternalServerError().json(json!({
                    "error": msg,
                    "code": "INTERNAL_ERROR"
                }))
            }
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::DatabaseError(err)
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::InternalError(err.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
