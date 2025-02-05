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
//! - [`DomExtractionError::NoBodyElement`]: When the HTML document lacks a body tag
//! - [`DomExtractionError::NodeAccessError`]: When a node cannot be accessed in the tree
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
//! Currently, no optional features are provided. All functionality is included in the default build.
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
pub mod utils;
pub use cetd::{DensityNode, DensityTree};
pub use utils::{get_node_links, get_node_text};

/// Re-export scraper crate
pub mod scraper {
    pub use scraper::*;
}

#[derive(Debug, thiserror::Error)]
pub enum DomExtractionError {
    #[error("Failed to access tree node: {0:?}")]
    NodeAccessError(NodeId),
}
