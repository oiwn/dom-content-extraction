//! Regression tests for `DensityTree::extract_article` on real-world pages.
//!
//! Locks in the fix for the sidebar/ticker pollution bug described in
//! `specs/ctx.md`: the plain-text article extractor anchors at the densest
//! subtree and walks up to its container, so the "Latest Crypto News" ticker
//! that `extract_content` sweeps in must be excluded here.

use std::fs;
use std::io::Read;

use dom_content_extraction::DensityTree;
use scraper::Html;

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

#[test]
fn theblock_article_excludes_ticker() {
    let html = read_page_from_zip(
        "html/pages.zip",
        "pages/theblock.co-post-402971-scott-bessent-reiterates-no-cbdc.html",
    );
    let document = Html::parse_document(&html);
    let mut dtree = DensityTree::from_document(&document).expect("dtree builds");
    dtree.calculate_density_sum().expect("density sum computes");

    let article = dtree
        .extract_article(&document)
        .expect("extract_article succeeds");

    assert!(!article.trim().is_empty(), "extracted article is empty");

    // Article body must be present (content sanity, guards against
    // over-aggressive filtering).
    assert!(
        article.contains("Scott Bessent"),
        "article body missing 'Scott Bessent':\n{article}"
    );
    assert!(
        article.contains("CBDC"),
        "article body missing 'CBDC':\n{article}"
    );

    // Ticker/sidebar content must be excluded. The "Latest Crypto News" bar
    // rotates; this snapshot headlines Securitize/eToro/JPMorgan — entities
    // unrelated to the Bessent/CBDC article.
    assert!(
        !article.contains("Latest Crypto News"),
        "ticker leaked into extract_article:\n{article}"
    );
    assert!(
        !article.contains("Securitize becomes first to debut shares on NYSE"),
        "ticker headline leaked into extract_article:\n{article}"
    );
}
