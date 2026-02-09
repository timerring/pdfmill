use super::{ConvertEngine, ConvertOptions, ConvertResult, EngineType};
use crate::error::{AppError, Result};
use async_trait::async_trait;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;
use tracing::info;

const SUPPORTED_EXTENSIONS: &[&str] = &[
    "doc", "docx", "xls", "xlsx", "ppt", "pptx", "odt", "ods", "odp", "rtf",
];

pub struct LibreOfficeEngine {
    /// Path to LibreOffice/soffice executable
    soffice_path: Option<String>,
}

impl LibreOfficeEngine {
    pub fn new() -> Self {
        Self {
            soffice_path: None,
        }
    }

    pub fn with_soffice_path(mut self, path: String) -> Self {
        self.soffice_path = Some(path);
        self
    }

    fn get_soffice_path(&self) -> String {
        // First check instance config
        if let Some(path) = &self.soffice_path {
            return path.clone();
        }
        
        // Then check environment variable
        if let Ok(path) = std::env::var("SOFFICE_PATH") {
            return path;
        }
        
        // Fall back to OS-specific defaults
        if cfg!(target_os = "macos") {
            "/Applications/LibreOffice.app/Contents/MacOS/soffice".to_string()
        } else if cfg!(target_os = "windows") {
            r"C:\Program Files\LibreOffice\program\soffice.exe".to_string()
        } else {
            // Try common Linux paths
            for path in &["/usr/bin/soffice", "/usr/bin/libreoffice"] {
                if std::path::Path::new(path).exists() {
                    return path.to_string();
                }
            }
            "soffice".to_string()
        }
    }

    async fn convert_to_pdf(
        &self,
        input_path: &Path,
        output_dir: &Path,
        _options: &ConvertOptions,
    ) -> Result<()> {
        let soffice_path = self.get_soffice_path();

        let args = vec![
            "--headless",
            "--convert-to",
            "pdf",
            "--outdir",
            output_dir.to_str().unwrap(),
            input_path.to_str().unwrap(),
        ];

        let output = Command::new(soffice_path)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| {
                AppError::EngineNotAvailable(format!("LibreOffice not found: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::ConversionFailed(format!(
                "LibreOffice conversion failed: {}",
                stderr
            )));
        }

        Ok(())
    }
}

impl Default for LibreOfficeEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ConvertEngine for LibreOfficeEngine {
    fn engine_type(&self) -> EngineType {
        EngineType::LibreOffice
    }

    fn supports_extension(&self, ext: &str) -> bool {
        SUPPORTED_EXTENSIONS.contains(&ext.to_lowercase().as_str())
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        SUPPORTED_EXTENSIONS.to_vec()
    }

    async fn is_available(&self) -> bool {
        let soffice_path = self.get_soffice_path();
        Command::new(soffice_path)
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

        info!(
            "Converting {} to PDF using LibreOffice",
            input_path.display()
        );
        self.convert_to_pdf(input_path, temp_dir.path(), options)
            .await?;

        // LibreOffice creates a PDF with the same base name
        let input_stem = input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        let output_path = temp_dir.path().join(format!("{}.pdf", input_stem));

        let data = tokio::fs::read(&output_path).await?;

        Ok(ConvertResult {
            data,
            filename: format!("{}.pdf", input_stem),
            content_type: "application/pdf".to_string(),
        })
    }
}
