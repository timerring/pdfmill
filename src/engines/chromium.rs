use super::{ConvertEngine, ConvertOptions, ConvertResult, EngineType};
use crate::error::{AppError, Result};
use async_trait::async_trait;
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::page::PrintToPdfParams;
use futures::StreamExt;
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::Mutex;
use tracing::info;

const SUPPORTED_EXTENSIONS: &[&str] = &["html", "htm", "xhtml", "md", "markdown"];

pub struct ChromiumEngine {
    /// Persistent browser instance for fast PDF generation via CDP
    browser: Arc<Mutex<Option<Browser>>>,
}

impl ChromiumEngine {
    pub fn new() -> Self {
        Self {
            browser: Arc::new(Mutex::new(None)),
        }
    }

    /// Initialize the persistent browser instance
    pub async fn init(&self) -> std::result::Result<(), String> {
        let chrome_path = get_chrome_path();

        let config = BrowserConfig::builder()
            .chrome_executable(chrome_path)
            .no_sandbox()
            .arg("--disable-gpu")
            .arg("--disable-dev-shm-usage")
            .arg("--disable-extensions")
            .arg("--disable-background-networking")
            .arg("--disable-sync")
            .arg("--disable-translate")
            .arg("--disable-default-apps")
            .arg("--headless")
            .build()
            .map_err(|e| format!("Failed to build browser config: {}", e))?;

        let (browser, mut handler) = Browser::launch(config)
            .await
            .map_err(|e| format!("Failed to launch browser: {}", e))?;

        // Spawn the browser event handler in background
        tokio::spawn(async move {
            while let Some(_event) = handler.next().await {}
        });

        let mut guard = self.browser.lock().await;
        *guard = Some(browser);

        info!("Persistent Chromium browser launched via CDP");
        Ok(())
    }

    async fn convert_html_to_pdf_cdp(
        &self,
        input_path: &Path,
        options: &ConvertOptions,
    ) -> Result<Vec<u8>> {
        let guard = self.browser.lock().await;
        let browser = guard.as_ref().ok_or_else(|| {
            AppError::EngineNotAvailable("Chromium browser not initialized".to_string())
        })?;

        let page = browser.new_page("about:blank").await.map_err(|e| {
            AppError::ConversionFailed(format!("Failed to create new tab: {}", e))
        })?;

        // Navigate to the local file (goto waits for load to complete)
        let input_url = format!("file://{}", input_path.canonicalize()?.display());
        page.goto(&input_url)
            .await
            .map_err(|e| AppError::ConversionFailed(format!("Failed to navigate: {}", e)))?;

        // Build PrintToPDF params
        let mut params = PrintToPdfParams::default();
        params.landscape = Some(options.landscape);
        params.print_background = Some(options.print_background);

        if let Some(ref width) = options.page_width {
            if let Some(inches) = parse_to_inches(width) {
                params.paper_width = Some(inches);
            }
        }
        if let Some(ref height) = options.page_height {
            if let Some(inches) = parse_to_inches(height) {
                params.paper_height = Some(inches);
            }
        }

        // Generate PDF via CDP
        let pdf_data = page.pdf(params).await.map_err(|e| {
            AppError::ConversionFailed(format!("PDF generation failed: {}", e))
        })?;

        // Page is automatically cleaned up when dropped
        Ok(pdf_data)
    }

    async fn convert_markdown_to_html(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        let content = tokio::fs::read_to_string(input_path).await?;

        // Simple markdown to HTML conversion
        // In production, use a proper markdown parser like pulldown-cmark
        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 40px; line-height: 1.6; }}
        pre {{ background: #f4f4f4; padding: 16px; overflow-x: auto; }}
        code {{ background: #f4f4f4; padding: 2px 6px; }}
    </style>
</head>
<body>
{}
</body>
</html>"#,
            markdown_to_html_simple(&content)
        );

        tokio::fs::write(output_path, html).await?;
        Ok(())
    }
}

impl Default for ChromiumEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ConvertEngine for ChromiumEngine {
    fn engine_type(&self) -> EngineType {
        EngineType::Chromium
    }

    fn supports_extension(&self, ext: &str) -> bool {
        SUPPORTED_EXTENSIONS.contains(&ext.to_lowercase().as_str())
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        SUPPORTED_EXTENSIONS.to_vec()
    }

    async fn is_available(&self) -> bool {
        let chrome_path = get_chrome_path();
        Command::new(chrome_path)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .map(|s| s.success())
            .unwrap_or(false)
    }

    async fn convert(&self, input_path: &Path, options: &ConvertOptions) -> Result<ConvertResult> {
        let ext = input_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        // If markdown, convert to HTML first
        let (html_path, _temp_dir) = if ext == "md" || ext == "markdown" {
            let temp_dir = tempfile::tempdir()?;
            let html_path = temp_dir.path().join("input.html");
            self.convert_markdown_to_html(input_path, &html_path)
                .await?;
            (html_path, Some(temp_dir))
        } else {
            (input_path.to_path_buf(), None)
        };

        info!("Converting {} to PDF using Chromium (CDP)", html_path.display());
        let data = self.convert_html_to_pdf_cdp(&html_path, options).await?;

        let original_name = input_path.file_stem().and_then(|s| s.to_str()).unwrap_or("output");

        Ok(ConvertResult {
            data,
            filename: format!("{}.pdf", original_name),
            content_type: "application/pdf".to_string(),
        })
    }
}

fn get_chrome_path() -> String {
    // Check environment variable first
    if let Ok(path) = std::env::var("CHROME_PATH") {
        return path;
    }

    // Fall back to OS-specific defaults
    if cfg!(target_os = "macos") {
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome".to_string()
    } else if cfg!(target_os = "windows") {
        r"C:\Program Files\Google\Chrome\Application\chrome.exe".to_string()
    } else {
        // Try common Linux paths
        for path in &["/usr/bin/chromium", "/usr/bin/chromium-browser", "/usr/bin/google-chrome"] {
            if std::path::Path::new(path).exists() {
                return path.to_string();
            }
        }
        "chromium".to_string()
    }
}

/// Parse dimension string (e.g., "8.5in", "210mm") to inches
fn parse_to_inches(s: &str) -> Option<f64> {
    let s = s.trim();
    if let Some(val) = s.strip_suffix("in") {
        val.trim().parse::<f64>().ok()
    } else if let Some(val) = s.strip_suffix("mm") {
        val.trim().parse::<f64>().ok().map(|v| v / 25.4)
    } else if let Some(val) = s.strip_suffix("cm") {
        val.trim().parse::<f64>().ok().map(|v| v / 2.54)
    } else {
        s.parse::<f64>().ok()
    }
}

/// Simple markdown to HTML converter
/// In production, use pulldown-cmark or similar
fn markdown_to_html_simple(md: &str) -> String {
    let mut html = String::new();
    let mut in_code_block = false;

    for line in md.lines() {
        if line.starts_with("```") {
            if in_code_block {
                html.push_str("</code></pre>\n");
                in_code_block = false;
            } else {
                html.push_str("<pre><code>");
                in_code_block = true;
            }
            continue;
        }

        if in_code_block {
            html.push_str(&html_escape(line));
            html.push('\n');
            continue;
        }

        if line.starts_with("# ") {
            html.push_str(&format!("<h1>{}</h1>\n", &line[2..]));
        } else if line.starts_with("## ") {
            html.push_str(&format!("<h2>{}</h2>\n", &line[3..]));
        } else if line.starts_with("### ") {
            html.push_str(&format!("<h3>{}</h3>\n", &line[4..]));
        } else if line.starts_with("- ") || line.starts_with("* ") {
            html.push_str(&format!("<li>{}</li>\n", &line[2..]));
        } else if line.is_empty() {
            html.push_str("<br>\n");
        } else {
            html.push_str(&format!("<p>{}</p>\n", line));
        }
    }

    html
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
