//! # dom-content-extraction
//!
//! A library for extracting main content from HTML documents using text density analysis.
//! This crate implements the Content Extraction via Text Density (CETD) algorithm described
//! in the paper by Fei Sun, Dandan Song and Lejian Liao.
//!
//! ## Overview
//!
//! Web pages typically contain various elements beyond the main content, such as navigation
//! menus, advertisements, sidebars, and footers. This library helps identify and extract
//! the main content by analyzing text density patterns within the HTML document structure.
//!
//! The core concept is that content-rich sections of a webpage tend to have different text
//! density characteristics compared to navigational or peripheral elements. By building a
//! density tree and applying composite text density calculations, we can identify and
//! extract the main content regions.
//!
//! ## Main Components
//!
//! - [`DensityTree`]: The primary structure representing text density analysis of an HTML document
//! - [`DensityNode`]: Individual nodes in the density tree containing text metrics
//! - Helper functions for node text extraction and link analysis
//!
//! ## Basic Usage
//!
//! ```no_run
//! use dom_content_extraction::{DensityTree, scraper::Html};
//!
//! // Parse your HTML document
//! let html_content = "<html><body><article>Main content</article></body></html>";
//! let document = Html::parse_document(html_content);
//!
//! // Create and analyze density tree
//! let mut dtree = DensityTree::from_document(&document)?;
//!
//! // Calculate density sums for better content identification
//! dtree.calculate_density_sum()?;
//!
//! // Extract the main content
//! let content = dtree.extract_content(&document)?;
//! println!("{}", content);
//! # Ok::<(), dom_content_extraction::DomExtractionError>(())
//! ```
//!
//! ## Advanced Usage
//!
//! For more precise control, you can work directly with the density-sorted nodes:
//!
//! ```no_run
//! use dom_content_extraction::{DensityTree, get_node_text, scraper::Html};
//!
//! let document = Html::parse_document("<html>...</html>");
//! let dtree = DensityTree::from_document(&document)?;
//!
//! // Get nodes sorted by density
//! let sorted_nodes = dtree.sorted_nodes();
//!
//! // Process the densest nodes
//! for node in sorted_nodes.iter().rev().take(3) {
//!     println!("Node density: {}", node.density);
//!     let text = get_node_text(node.node_id, &document)?;
//!     println!("Node content: {}", text);
//! }
//! # Ok::<(), dom_content_extraction::DomExtractionError>(())
//! ```
//!
//! ## Algorithm Details
//!
//! The content extraction process involves several steps:
//!
//! 1. Building a density tree that mirrors the HTML document structure
//! 2. Calculating text density metrics for each node:
//!    - Character count
//!    - Tag count
//!    - Link character count
//!    - Link tag count
//! 3. Computing composite text density using a formula that considers:
//!    - Text to tag ratio
//!    - Link density
//!    - Content distribution
//! 4. Identifying high-density regions that likely contain main content
//!
//! ## Error Handling
//!
//! The library uses custom error types to handle various failure cases:
//!
//! - [`DomExtractionError::NodeAccessError`]: When a node cannot be accessed in the tree (e.g., missing body)
//!
//! ## Performance Considerations
//!
//! - The library performs a full traversal of the HTML document to build the density tree
//! - Memory usage scales with document size and complexity
//! - Text density calculations are performed once and cached
//! - Node sorting operations are O(n log n) where n is the number of content nodes
//!
//! ## Feature Flags
//!
//! - **`markdown`**: Enables HTML-to-markdown extraction via `htmd`. Disabled by default.
//!
//! ## Examples
//!
//! More examples can be found in the `examples/` directory of the source repository:
//!
//! - `check.rs`: Basic content extraction from test documents
//! - `ce_score.rs`: Evaluation tool for measuring extraction accuracy
//!
//! ## References
//!
//! 1. Sun, F., Song, D., & Liao, L. (2011). "DOM Based Content Extraction via Text Density"
//! 2. CleanEval dataset: <https://sigwac.org.uk/cleaneval/>
//!
//! [`DensityTree`]: struct.DensityTree.html
//! [`DensityNode`]: struct.DensityNode.html
//! [`DomExtractionError`]: enum.DomExtractionError.html
#![crate_name = "dom_content_extraction"]
use ego_tree::NodeId;

pub mod cetd;
pub mod tree;
pub mod unicode;
pub mod utils;

#[cfg(feature = "markdown")]
pub mod markdown;
pub use cetd::{DensityNode, DensityTree};
pub use utils::{get_node_links, get_node_text};

#[cfg(feature = "markdown")]
pub use markdown::extract_content_as_markdown;

// Re-export
pub use scraper;

#[derive(Debug, thiserror::Error)]
pub enum DomExtractionError {
    #[error("Failed to access tree node: {0:?}")]
    NodeAccessError(NodeId),
}

pub fn get_content(document: &scraper::Html) -> Result<String, DomExtractionError> {
    let mut dtree = DensityTree::from_document(document)?;
    dtree.calculate_density_sum()?;
    dtree.extract_content(document)
}

/// Extracts the main article content from an HTML document as plain text.
///
/// Convenience wrapper around [`DensityTree::extract_article`]: builds the
/// [`DensityTree`], computes density sums, and returns the article text in
/// one call. Mirrors [`get_content`] for the article-extraction path.
///
/// Unlike [`get_content`] (which can include high-density sidebar/ticker
/// content), this anchors at the densest subtree and walks up to its
/// enclosing container, excluding sibling noise.
pub fn get_article(document: &scraper::Html) -> Result<String, DomExtractionError> {
    let mut dtree = DensityTree::from_document(document)?;
    dtree.calculate_density_sum()?;
    dtree.extract_article(document)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // Same fixture used by cetd::tests::test_extract_content, where the
    // expected extracted content ("Here is article", "Even more huge") and
    // excluded noise ("Menu") are already known. Using it here exercises the
    // get_content wrapper end-to-end on a realistic document.
    const TEST_1_HTML: &str = include_str!("../html/test_1.html");

    #[test]
    fn get_content_returns_article_text() {
        let document = scraper::Html::parse_document(TEST_1_HTML);
        let content = get_content(&document).unwrap();
        assert!(
            content.contains("Here is article"),
            "get_content should return the article body:\n{content}"
        );
        assert!(content.contains("Even more huge"));
        assert!(
            !content.contains("Menu"),
            "get_content should exclude navigation:\n{content}"
        );
    }

    #[test]
    fn get_article_excludes_ticker() {
        // Mirrors cetd::tests::test_extract_article_excludes_ticker, but
        // exercises the get_article() wrapper end-to-end (it builds its own
        // DensityTree rather than receiving one). The article paragraph must be
        // substantial enough to win the max-density-sum anchor over the ticker.
        let html = r#"<html><body>
            <div class="ticker">
                <a href="/1">Breaking: Aave Labs secures UK license</a>
                <a href="/2">SpaceX perps plunge 45% on Hyperliquid</a>
                <a href="/3">Paxos secures SEC registration</a>
            </div>
            <article>
                <h1>Treasury Secretary reiterates no CBDC commitment</h1>
                <p>U.S. Treasury Secretary Scott Bessent reiterated that the current
                administration will not allow a central bank digital currency
                (CBDC). During a White House press briefing, Bessent said CBDCs are
                clearly off the table and reaffirmed the administration's focus on
                making the U.S. a hub for digital assets. Bessent also mentioned
                that the GENIUS stablecoin legislation passed with bipartisan
                support, and the Clarity Act is gaining similar legislative
                momentum.</p>
            </article>
        </body></html>"#;
        let document = scraper::Html::parse_document(html);
        let article = get_article(&document).unwrap();
        assert!(article.contains("Scott Bessent"));
        assert!(article.contains("CBDC"));
        assert!(
            !article.contains("Aave Labs"),
            "ticker leaked through get_article:\n{article}"
        );
        assert!(!article.contains("SpaceX"));
        assert!(!article.contains("Hyperliquid"));
    }

    #[test]
    fn get_article_on_contentless_document_returns_empty() {
        // No article body text → the extractor returns an empty string,
        // not an error. Guards the degenerate-input path through the wrapper.
        let html = r#"<html><body><script>var x = 1;</script></body></html>"#;
        let document = scraper::Html::parse_document(html);
        let article = get_article(&document).unwrap();
        assert!(
            article.trim().is_empty(),
            "expected empty output for contentless HTML, got:\n{article}"
        );
    }
}
