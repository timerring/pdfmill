use super::{ConvertEngine, ConvertOptions, ConvertResult, EngineType};
use crate::error::{AppError, Result};
use async_trait::async_trait;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;
use tracing::info;

const SUPPORTED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "bmp", "tiff", "tif", "webp"];

pub struct ImageEngine {
    /// Path to ImageMagick convert executable
    convert_path: Option<String>,
}

impl ImageEngine {
    pub fn new() -> Self {
        Self {
            convert_path: None,
        }
    }

    pub fn with_convert_path(mut self, path: String) -> Self {
        self.convert_path = Some(path);
        self
    }

    fn get_convert_path(&self) -> String {
        // First check instance config
        if let Some(path) = &self.convert_path {
            return path.clone();
        }
        
        // Then check environment variable
        if let Ok(path) = std::env::var("CONVERT_PATH") {
            return path;
        }
        
        // Fall back to default
        "convert".to_string()
    }

    async fn convert_to_pdf(
        &self,
        input_path: &Path,
        output_path: &Path,
        options: &ConvertOptions,
    ) -> Result<()> {
        let convert_path = self.get_convert_path();

        let mut args = vec![input_path.to_str().unwrap().to_string()];

        // Add page size options if specified
        if let (Some(width), Some(height)) = (&options.page_width, &options.page_height) {
            args.push("-page".to_string());
            args.push(format!("{}x{}", width, height));
        }

        args.push(output_path.to_str().unwrap().to_string());

        let output = Command::new(convert_path)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| {
                AppError::EngineNotAvailable(format!("ImageMagick not found: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::ConversionFailed(format!(
                "ImageMagick conversion failed: {}",
                stderr
            )));
        }

        Ok(())
    }
}

impl Default for ImageEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ConvertEngine for ImageEngine {
    fn engine_type(&self) -> EngineType {
        EngineType::Image
    }

    fn supports_extension(&self, ext: &str) -> bool {
        SUPPORTED_EXTENSIONS.contains(&ext.to_lowercase().as_str())
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        SUPPORTED_EXTENSIONS.to_vec()
    }

    async fn is_available(&self) -> bool {
        let convert_path = self.get_convert_path();
        Command::new(convert_path)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .map(|s| s.success())
            .unwrap_or(false)
    }

    async fn convert(&self, input_path: &Path, options: &ConvertOptions) -> Result<ConvertResult> {
        let temp_dir = tempfile::tempdir()?;
        let output_path = temp_dir.path().join("output.pdf");

        info!(
            "Converting {} to PDF using ImageMagick",
            input_path.display()
        );
        self.convert_to_pdf(input_path, &output_path, options)
            .await?;

        let data = tokio::fs::read(&output_path).await?;
        let original_name = input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");

        Ok(ConvertResult {
            data,
            filename: format!("{}.pdf", original_name),
            content_type: "application/pdf".to_string(),
        })
    }
}
