//! End-to-end regression tests for the leak patterns documented in
//! `specs/ctx.md`.
//!
//! Each fixture is a real-world page that previously leaked HTML/markup
//! into the markdown output. The tests run the full extraction pipeline
//! (`DensityTree` + `extract_content_as_markdown`) and assert that:
//!
//! 1. No leak signatures appear in the output (anti-leak).
//! 2. The expected article title/lead is present (content sanity), so we
//!    also catch over-aggressive filtering.
//!
//! Fixtures live in `html/pages.zip` (see `specs/ctx.md` for the source URLs).

#![cfg(feature = "markdown")]

use std::fs;
use std::io::Read;

use dom_content_extraction::{DensityTree, extract_content_as_markdown};
use scraper::Html;

/// Substrings that must never appear in extracted markdown output.
const FORBIDDEN: &[&str] = &[
    "data:image/",
    "<span",
    "<img",
    "<svg",
    "<script",
    "<style",
    "<iframe",
    "&lt;img",
    "&lt;span",
    "data-mce-type",
    "acf-media-credit",
    "acf-credit",
];

fn read_page_from_zip(zip_path: &str, entry: &str) -> String {
    let zipfile = fs::File::open(zip_path).expect("html/pages.zip must exist");
    let mut archive = zip::ZipArchive::new(zipfile).expect("zip must be valid");
    let mut buf = String::new();
    let mut file = archive
        .by_name(entry)
        .unwrap_or_else(|e| panic!("zip entry {entry} must exist: {e}"));
    file.read_to_string(&mut buf)
        .unwrap_or_else(|e| panic!("reading {entry} failed: {e}"));
    buf
}

fn assert_no_leaks(md: &str, fixture: &str) {
    for pat in FORBIDDEN {
        assert!(
            !md.contains(pat),
            "{fixture}: leak detected, output contains `{pat}`:\n{md}",
        );
    }
}

fn run_fixture(entry: &str, expected_substring: &str) {
    let html = read_page_from_zip("html/pages.zip", entry);
    let document = Html::parse_document(&html);
    let mut dtree = DensityTree::from_document(&document).expect("dtree builds");
    dtree.calculate_density_sum().expect("density sum computes");

    let md =
        extract_content_as_markdown(&dtree, &document).expect("markdown extracts");

    assert!(
        !md.trim().is_empty(),
        "{entry}: extracted markdown is empty"
    );
    assert_no_leaks(&md, entry);
    assert!(
        md.contains(expected_substring),
        "{entry}: expected `{expected_substring}` in output, got:\n{md}",
    );
}

#[test]
fn theblock_no_leak() {
    run_fixture(
        "pages/theblock.co-post-402903-grayscale-hyperliquid-etf.html",
        "Grayscale",
    );
}

#[test]
fn bitcoinmagazine_no_leak() {
    run_fixture(
        "pages/bitcoinmagazine.com-news-cosmos-health-cosm-buys-600k-bitcoin.html",
        "Cosmos Health",
    );
}

#[test]
fn cryptoslate_no_leak() {
    run_fixture(
        "pages/cryptoslate.com-solana-public-attack-on-starknet.html",
        "Starknet",
    );
}

#[test]
fn decrypt_no_leak() {
    run_fixture(
        "pages/decrypt.co-369195-openai-foundation-pledges-250-million.html",
        "OpenAI",
    );
}
