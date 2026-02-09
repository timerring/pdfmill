use crate::engines::ConvertOptions;
use crate::error::{AppError, Result};
use crate::router::SmartRouter;
use axum::{
    body::Bytes,
    extract::{Multipart, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::sync::Arc;
use tracing::{error, info};

pub struct AppState {
    pub router: SmartRouter,
}

/// Main conversion endpoint - automatically routes based on file extension
pub async fn convert_handler(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Response> {
    let mut file_data: Option<(String, Vec<u8>)> = None;
    let mut options = ConvertOptions::default();

    // Parse multipart form data
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        AppError::InvalidRequest(format!("Failed to parse multipart data: {}", e))
    })? {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "file" => {
                let filename = field
                    .file_name()
                    .ok_or_else(|| AppError::InvalidRequest("No filename provided".to_string()))?
                    .to_string();

                let data = field.bytes().await.map_err(|e| {
                    AppError::InvalidRequest(format!("Failed to read file data: {}", e))
                })?;

                info!("Received file: {} ({} bytes)", filename, data.len());
                file_data = Some((filename, data.to_vec()));
            }
            "landscape" => {
                if let Ok(value) = field.text().await {
                    options.landscape = value == "true" || value == "1";
                }
            }
            "printBackground" => {
                if let Ok(value) = field.text().await {
                    options.print_background = value == "true" || value == "1";
                }
            }
            "pageWidth" => {
                if let Ok(value) = field.text().await {
                    options.page_width = Some(value);
                }
            }
            "pageHeight" => {
                if let Ok(value) = field.text().await {
                    options.page_height = Some(value);
                }
            }
            "pdfFormat" => {
                if let Ok(value) = field.text().await {
                    options.pdf_format = Some(value);
                }
            }
            _ => {
                // Ignore unknown fields
            }
        }
    }

    let (filename, data) = file_data.ok_or(AppError::NoFileProvided)?;

    // Save to temp file
    let temp_dir = tempfile::tempdir()?;
    let input_path = temp_dir.path().join(&filename);
    tokio::fs::write(&input_path, &data).await?;

    // Find the appropriate engine based on file extension
    let engine = state.router.find_engine_for_file(&input_path)?;
    info!("Using {:?} engine for {}", engine.engine_type(), filename);

    // Perform the conversion
    let result = engine.convert(&input_path, &options).await?;

    // Return the PDF
    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, result.content_type),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{}\"", result.filename),
            ),
        ],
        result.data,
    )
        .into_response())
}

/// Health check endpoint
pub async fn health_handler() -> impl IntoResponse {
    Json(json!({
        "status": "healthy",
        "service": "pdfmill"
    }))
}

/// Information endpoint - lists supported formats
pub async fn info_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let extensions = state.router.supported_extensions();

    Json(json!({
        "service": "pdfmill",
        "version": env!("CARGO_PKG_VERSION"),
        "supported_formats": extensions,
        "endpoints": {
            "convert": {
                "path": "/convert",
                "method": "POST",
                "description": "Convert any supported file to PDF. The engine is automatically selected based on file extension.",
                "content_type": "multipart/form-data",
                "fields": {
                    "file": "The file to convert (required)",
                    "landscape": "Boolean - use landscape orientation (optional)",
                    "printBackground": "Boolean - print background graphics (optional, HTML only)",
                    "pageWidth": "Page width (optional, e.g., '8.5in', '210mm')",
                    "pageHeight": "Page height (optional, e.g., '11in', '297mm')",
                    "pdfFormat": "PDF format (optional, e.g., 'PDF/A-1b')"
                }
            },
            "health": {
                "path": "/health",
                "method": "GET",
                "description": "Health check endpoint"
            },
            "info": {
                "path": "/info",
                "method": "GET",
                "description": "Service information and supported formats"
            }
        }
    }))
}
