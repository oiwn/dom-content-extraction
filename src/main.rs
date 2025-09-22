use anyhow::{Context, Result};
use clap::Parser;
use dom_content_extraction::get_content;
use scraper::Html;
use std::{fs, path::PathBuf};
use tempfile::Builder;
use url::Url;
use wreq::Client;
use wreq_util::Emulation;

use tracing::{debug, info};

/// Detect encoding and convert bytes to UTF-8 string using chardetng
fn detect_and_convert_to_utf8(
    bytes: &[u8],
) -> Result<String, std::string::FromUtf8Error> {
    // First try UTF-8 directly (most common case)
    if let Ok(text) = String::from_utf8(bytes.to_vec()) {
        return Ok(text);
    }

    // Use chardetng for automatic encoding detection
    let mut detector = chardetng::EncodingDetector::new();
    detector.feed(bytes, true);
    let encoding = detector.guess(None, true);

    // Decode using the detected encoding
    let (cow, _, had_errors) = encoding.decode(bytes);

    if had_errors && encoding != encoding_rs::UTF_8 {
        // Fallback to UTF-8 if the detected encoding fails
        let (cow, _, _) = encoding_rs::UTF_8.decode(bytes);
        Ok(cow.into_owned())
    } else {
        Ok(cow.into_owned())
    }
}

fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        )
        .init();
}

#[derive(Parser)]
#[command(version, about = "Extract main content from HTML documents")]
struct Cli {
    /// URL to fetch HTML content from
    #[arg(short, long, conflicts_with = "file", value_parser = parse_url)]
    url: Option<Url>,

    /// Local HTML file to process
    #[arg(short, long, conflicts_with = "url")]
    file: Option<PathBuf>,

    /// Output file (stdout if not specified)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Output format (text or markdown)
    #[arg(long, default_value = "text", value_parser = ["text", "markdown"])]
    format: String,
}

fn parse_url(s: &str) -> Result<Url, String> {
    Url::parse(s).map_err(|e| format!("Invalid URL: {}", e))
}

async fn fetch_url(url: &Url) -> Result<String> {
    info!("Fetching URL: {}", url);

    let client = Client::builder()
        .emulation(Emulation::Chrome120)
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .context("Failed to create HTTP client")?;

    debug!("Sending HTTP request...");
    let response = client
        .get(url.as_str())
        .send()
        .await
        .context("Failed to send HTTP request")?;

    info!("Response status: {}", response.status());

    let response = response.error_for_status().context("HTTP request failed")?;

    debug!("Reading response bytes...");
    let bytes = response
        .bytes()
        .await
        .context("Failed to read response bytes")?;
    info!("Response bytes length: {} bytes", bytes.len());

    // Convert to UTF-8 using encoding detection
    let text = detect_and_convert_to_utf8(&bytes)
        .context("Failed to convert response to UTF-8")?;
    info!("Converted text length: {} bytes", text.len());

    Ok(text)
}

fn process_html(html: &str, format: &str) -> Result<String> {
    let document = Html::parse_document(html);

    match format {
        "text" => get_content(&document).context("Failed to extract content"),
        "markdown" => {
            #[cfg(not(feature = "markdown"))]
            {
                anyhow::bail!(
                    "Markdown output requires the 'markdown' feature to be enabled"
                );
            }

            #[cfg(feature = "markdown")]
            {
                use dom_content_extraction::{
                    DensityTree, extract_content_as_markdown,
                };
                let mut dtree = DensityTree::from_document(&document)
                    .context("Failed to create density tree")?;
                dtree
                    .calculate_density_sum()
                    .context("Failed to calculate density sums")?;
                extract_content_as_markdown(&dtree, &document)
                    .map_err(|e| anyhow::anyhow!(e))
                    .context("Failed to extract content as markdown")
            }
        }
        _ => anyhow::bail!("Invalid format: {}. Use 'text' or 'markdown'", format),
    }
}

fn write_output(content: &str, output_path: Option<PathBuf>) -> Result<()> {
    match output_path {
        Some(path) => {
            fs::write(path, content).context("Failed to write output file")
        }
        None => {
            println!("{}", content);
            Ok(())
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logging();

    let cli = Cli::parse();

    // Get HTML content from either URL or file
    let html_content = if let Some(url) = cli.url {
        info!("Processing URL input");
        // Create temp dir for downloaded content
        let temp_dir = Builder::new()
            .prefix("dce-")
            .tempdir()
            .context("Failed to create temp directory")?;

        // Fetch and store content
        let content = fetch_url(&url).await?;
        debug!("Creating temporary file for content");
        let temp_file = temp_dir.path().join("content.html");
        fs::write(&temp_file, &content).context("Failed to write temp file")?;

        content
    } else if let Some(file) = cli.file {
        info!("Processing file input: {:?}", file);
        let bytes = fs::read(file).context("Failed to read input file")?;
        detect_and_convert_to_utf8(&bytes)
            .context("Failed to convert file to UTF-8")?
    } else {
        anyhow::bail!("Either --url or --file must be specified");
    };

    info!("HTML content loaded, length: {} bytes", html_content.len());

    // Process HTML and extract content
    let extracted_content = process_html(&html_content, &cli.format)?;

    // Write output
    write_output(&extracted_content, cli.output)?;

    Ok(())
}
