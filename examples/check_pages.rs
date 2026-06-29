//! Run extraction on every page in `html/pages.zip` and print the output.
//!
//! Usage:
//!   cargo run --example check_pages                     # text (default)
//!   cargo run --features markdown --example check_pages -- --markdown
//!
//! Used for manual inspection of how extraction behaves on real-world pages
//! (see `tests/e2e_leaks.rs` for automated regression coverage).

use anyhow::{Context, Result};
use dom_content_extraction::{get_content, scraper::Html};
use std::fs::File;
use std::io::Read;
use zip::ZipArchive;

#[cfg(feature = "markdown")]
use dom_content_extraction::{DensityTree, extract_content_as_markdown};

fn main() -> Result<()> {
    let markdown_mode = std::env::args().any(|a| a == "--markdown" || a == "-m");

    #[cfg(not(feature = "markdown"))]
    if markdown_mode {
        anyhow::bail!("--markdown requires building with --features markdown");
    }

    let zipfile =
        File::open("html/pages.zip").context("html/pages.zip not found")?;
    let mut archive = ZipArchive::new(zipfile).context("invalid zip archive")?;

    let names: Vec<String> = (0..archive.len())
        .map(|i| archive.by_index(i).unwrap().name().to_string())
        .filter(|n| n.starts_with("pages/") && n.ends_with(".html"))
        .collect();

    for name in names {
        let mut buf = String::new();
        archive
            .by_name(&name)
            .with_context(|| format!("reading {name}"))?
            .read_to_string(&mut buf)
            .with_context(|| format!("decoding {name}"))?;
        let kb = buf.len() / 1024;

        println!("\n===== {name} ({kb} KB) =====");

        let document = Html::parse_document(&buf);
        let out = extract(&document, markdown_mode)?;
        println!("{out}");
    }

    Ok(())
}

#[cfg(feature = "markdown")]
fn extract(document: &Html, markdown_mode: bool) -> Result<String> {
    if markdown_mode {
        let mut dtree = DensityTree::from_document(document)?;
        dtree.calculate_density_sum()?;
        Ok(extract_content_as_markdown(&dtree, document)?)
    } else {
        Ok(get_content(document)?)
    }
}

#[cfg(not(feature = "markdown"))]
fn extract(document: &Html, _markdown_mode: bool) -> Result<String> {
    Ok(get_content(document)?)
}
