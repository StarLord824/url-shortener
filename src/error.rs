use actix_web::{HttpResponse, ResponseError};
use serde_json::json;
use std::fmt;

#[derive(Debug)]
pub enum ApiError {
    Validation(String),
    Conflict(String),
    NotFound,
    Gone,
    Internal(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Validation(msg) => write!(f, "Validation error: {}", msg),
            Self::Conflict(msg) => write!(f, "Conflict: {}", msg),
            Self::NotFound => write!(f, "Not Found"),
            Self::Gone => write!(f, "Expired or Deleted"),
            Self::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        match self {
            Self::Validation(msg) => HttpResponse::BadRequest().json(json!({ "error": msg })),
            Self::Conflict(msg) => HttpResponse::Conflict().json(json!({ "error": msg })),
            Self::NotFound => HttpResponse::NotFound().json(json!({ "error": "Not Found" })),
            Self::Gone => HttpResponse::Gone().json(json!({ "error": "Link expired" })),
            Self::Internal(msg) => HttpResponse::InternalServerError().json(json!({ "error": msg })),
        }
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => Self::NotFound,
            _ => Self::Internal(err.to_string()),
        }
    }
}

impl From<redis::RedisError> for ApiError {
    fn from(err: redis::RedisError) -> Self {
        Self::Internal(err.to_string())
    }
}

impl From<image::ImageError> for ApiError {
    fn from(err: image::ImageError) -> Self {
        Self::Internal(err.to_string())
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        Self::Internal(err.to_string())
    }
}

impl From<std::env::VarError> for ApiError {
    fn from(err: std::env::VarError) -> Self {
        Self::Internal(err.to_string())
    }
}

impl From<std::io::Error> for ApiError {
    fn from(err: std::io::Error) -> Self {
        Self::Internal(err.to_string())
    }
}

impl From<validator::ValidationErrors> for ApiError {
    fn from(err: validator::ValidationErrors) -> Self {
        Self::Validation(err.to_string())
    }
}

impl From<time::error::ComponentRange> for ApiError {
    fn from(err: time::error::ComponentRange) -> Self {
        Self::Internal(err.to_string())
    }
}

impl From<qrcode::types::QrError> for ApiError {
    fn from(err: qrcode::types::QrError) -> Self {
        Self::Internal(err.to_string())
    }
}