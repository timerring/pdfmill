mod engines;
mod error;
mod handlers;
mod router;

use axum::{
    routing::{get, post},
    Router,
};
use handlers::{convert_handler, health_handler, info_handler, AppState};
use router::SmartRouter;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "pdfmill=info,tower_http=info".into()),
        )
        .init();

    // Create the smart router (detects available engines)
    let smart_router = SmartRouter::new().await;

    let state = Arc::new(AppState {
        router: smart_router,
    });

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build the application router
    let app = Router::new()
        .route("/convert", post(convert_handler))
        .route("/health", get(health_handler))
        .route("/info", get(info_handler))
        .route("/", get(info_handler))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state);

    // Start the server
    let addr = std::env::var("PDFMILL_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string());
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    info!("🚀 PDFMill server running on http://{}", addr);
    info!("📖 API documentation: http://{}/info", addr);
    info!("🔄 Convert files: POST http://{}/convert", addr);

    axum::serve(listener, app).await.unwrap();
}
