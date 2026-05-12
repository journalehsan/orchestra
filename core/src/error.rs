use actix_web::{HttpResponse, ResponseError};
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Record not found")]
    NotFound,

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden")]
    Forbidden,

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Internal server error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let (status, message) = match self {
            AppError::NotFound => (
                actix_web::http::StatusCode::NOT_FOUND,
                self.to_string(),
            ),
            AppError::Validation(msg) => (
                actix_web::http::StatusCode::UNPROCESSABLE_ENTITY,
                msg.clone(),
            ),
            AppError::Unauthorized => (
                actix_web::http::StatusCode::UNAUTHORIZED,
                self.to_string(),
            ),
            AppError::Forbidden => (
                actix_web::http::StatusCode::FORBIDDEN,
                self.to_string(),
            ),
            AppError::Database(e) => {
                tracing::error!("Database error: {:?}", e);
                (
                    actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "Database error".to_string(),
                )
            }
            AppError::Internal(e) => {
                tracing::error!("Internal error: {:?}", e);
                (
                    actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
        };

        HttpResponse::build(status).json(serde_json::json!({
            "error": message
        }))
    }
}
