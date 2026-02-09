use crate::engines::{ChromiumEngine, ConvertEngine, ImageEngine, LibreOfficeEngine};
use crate::error::{AppError, Result};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use crate::engines::EngineType;

/// Smart router that automatically selects the appropriate engine
/// based on file extension
pub struct SmartRouter {
    engines: Vec<Arc<dyn ConvertEngine>>,
    /// Cached availability results from startup
    availability: HashMap<EngineType, bool>,
}

impl SmartRouter {
    pub async fn new() -> Self {
        let chromium = Arc::new(ChromiumEngine::new());

        // Initialize persistent Chromium browser via CDP
        if let Err(e) = chromium.init().await {
            tracing::warn!("Failed to initialize Chromium CDP: {}", e);
        }

        let engines: Vec<Arc<dyn ConvertEngine>> = vec![
            chromium,
            Arc::new(LibreOfficeEngine::new()),
            Arc::new(ImageEngine::new()),
        ];

        // Cache engine availability at startup
        let mut availability = HashMap::new();
        for engine in &engines {
            let available = engine.is_available().await;
            let status = if available { "✓" } else { "✗" };
            tracing::info!(
                "{} {:?} engine - supports: {}",
                status,
                engine.engine_type(),
                engine.supported_extensions().join(", ")
            );
            availability.insert(engine.engine_type(), available);
        }

        Self { engines, availability }
    }

    /// Find the appropriate engine for a given file extension
    pub fn find_engine_for_extension(
        &self,
        ext: &str,
    ) -> Result<Arc<dyn ConvertEngine>> {
        let ext_lower = ext.to_lowercase();

        // Find all engines that support this extension
        let candidates: Vec<_> = self
            .engines
            .iter()
            .filter(|e| e.supports_extension(&ext_lower))
            .collect();

        if candidates.is_empty() {
            return Err(AppError::UnsupportedFormat(format!(
                "No engine supports .{} files",
                ext
            )));
        }

        // Use cached availability instead of checking every request
        let available: Vec<_> = candidates
            .iter()
            .filter(|e| *self.availability.get(&e.engine_type()).unwrap_or(&false))
            .collect();

        if available.is_empty() {
            let supported_by = candidates
                .iter()
                .map(|e| format!("{:?}", e.engine_type()))
                .collect::<Vec<_>>()
                .join(", ");

            return Err(AppError::EngineNotAvailable(format!(
                ".{} files are supported by {} but the required dependencies are not installed",
                ext, supported_by
            )));
        }

        // Return the first available engine
        Ok(Arc::clone(available[0]))
    }

    /// Find engine for a file path (extracts extension automatically)
    pub fn find_engine_for_file(&self, path: &Path) -> Result<Arc<dyn ConvertEngine>> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| AppError::InvalidRequest("File has no extension".to_string()))?;

        self.find_engine_for_extension(ext)
    }

    /// Get a list of all supported extensions
    pub fn supported_extensions(&self) -> Vec<String> {
        let mut extensions = Vec::new();
        for engine in &self.engines {
            extensions.extend(
                engine
                    .supported_extensions()
                    .iter()
                    .map(|s| s.to_string()),
            );
        }
        extensions.sort();
        extensions.dedup();
        extensions
    }

    /// Check if an extension is supported
    pub fn is_extension_supported(&self, ext: &str) -> bool {
        let ext_lower = ext.to_lowercase();
        self.engines
            .iter()
            .any(|e| e.supports_extension(&ext_lower))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_router_creation() {
        let router = SmartRouter::new().await;
        assert!(!router.engines.is_empty());
    }

    #[tokio::test]
    async fn test_supported_extensions() {
        let router = SmartRouter::new().await;
        let extensions = router.supported_extensions();
        
        // Should support common formats
        assert!(extensions.contains(&"html".to_string()));
        assert!(extensions.contains(&"docx".to_string()));
        assert!(extensions.contains(&"jpg".to_string()));
    }

    #[tokio::test]
    async fn test_is_extension_supported() {
        let router = SmartRouter::new().await;
        
        assert!(router.is_extension_supported("html"));
        assert!(router.is_extension_supported("HTML"));
        assert!(router.is_extension_supported("docx"));
        assert!(!router.is_extension_supported("xyz"));
    }
}
