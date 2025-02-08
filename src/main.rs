use anyhow::{Context, Result};
use clap::Parser;
use dom_content_extraction::get_content;
use reqwest::blocking::Client;
use scraper::Html;
use std::{fs, path::PathBuf};
use tempfile::Builder;
use url::Url;

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
}

fn parse_url(s: &str) -> Result<Url, String> {
    Url::parse(s).map_err(|e| format!("Invalid URL: {}", e))
}

fn fetch_url(url: &Url) -> Result<String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .context("Failed to create HTTP client")?;

    Ok(client
        .get(url.as_str())
        .send()
        .and_then(|r| r.error_for_status())
        .and_then(|r| r.text())?)
}

fn process_html(html: &str) -> Result<String> {
    let document = Html::parse_document(html);
    get_content(&document).context("Failed to extract content")
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

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Get HTML content from either URL or file
    let html_content = if let Some(url) = cli.url {
        // Create temp dir for downloaded content
        let temp_dir = Builder::new()
            .prefix("dce-")
            .tempdir()
            .context("Failed to create temp directory")?;

        // Fetch and store content
        let content = fetch_url(&url)?;
        let temp_file = temp_dir.path().join("content.html");
        fs::write(&temp_file, &content).context("Failed to write temp file")?;

        content
    } else if let Some(file) = cli.file {
        fs::read_to_string(file).context("Failed to read input file")?
    } else {
        anyhow::bail!("Either --url or --file must be specified");
    };

    // Process HTML and extract content
    let extracted_content = process_html(&html_content)?;

    // Write output
    write_output(&extracted_content, cli.output)?;

    Ok(())
}
