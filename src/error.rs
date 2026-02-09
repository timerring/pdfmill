use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Unsupported file format: {0}")]
    UnsupportedFormat(String),

    #[error("No file provided")]
    NoFileProvided,

    #[error("Conversion failed: {0}")]
    ConversionFailed(String),

    #[error("Engine not available: {0}")]
    EngineNotAvailable(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::UnsupportedFormat(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::NoFileProvided => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::InvalidRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::ConversionFailed(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::EngineNotAvailable(_) => {
                (StatusCode::SERVICE_UNAVAILABLE, self.to_string())
            }
            AppError::IoError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(json!({
            "error": message,
        }));

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
