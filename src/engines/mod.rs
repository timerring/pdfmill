mod chromium;
mod libreoffice;
mod image;

pub use chromium::ChromiumEngine;
pub use libreoffice::LibreOfficeEngine;
pub use image::ImageEngine;

use crate::error::Result;
use async_trait::async_trait;
use std::path::Path;

/// Conversion options passed to engines
#[derive(Debug, Clone, Default)]
pub struct ConvertOptions {
    /// Page width (e.g., "8.5in", "210mm")
    pub page_width: Option<String>,
    /// Page height (e.g., "11in", "297mm")
    pub page_height: Option<String>,
    /// Top margin
    pub margin_top: Option<String>,
    /// Bottom margin
    pub margin_bottom: Option<String>,
    /// Left margin
    pub margin_left: Option<String>,
    /// Right margin
    pub margin_right: Option<String>,
    /// Landscape orientation
    pub landscape: bool,
    /// Print background
    pub print_background: bool,
    /// PDF/A format (e.g., "PDF/A-1b")
    pub pdf_format: Option<String>,
}

/// Result of a conversion operation
pub struct ConvertResult {
    pub data: Vec<u8>,
    pub filename: String,
    pub content_type: String,
}

/// Engine capability - what file types an engine can handle
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EngineType {
    Chromium,
    LibreOffice,
    Image,
}

/// Trait that all conversion engines must implement
#[async_trait]
pub trait ConvertEngine: Send + Sync {
    /// Get the engine type
    fn engine_type(&self) -> EngineType;

    /// Check if this engine can handle the given file extension
    fn supports_extension(&self, ext: &str) -> bool;

    /// Get list of supported extensions
    fn supported_extensions(&self) -> Vec<&'static str>;

    /// Check if the engine is available (dependencies installed)
    async fn is_available(&self) -> bool;

    /// Convert the input file to PDF
    async fn convert(
        &self,
        input_path: &Path,
        options: &ConvertOptions,
    ) -> Result<ConvertResult>;
}
