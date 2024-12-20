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
#![deny(clippy::unwrap_used)]
use crate::scraper::{Html, Selector};
use ego_tree::{NodeId, NodeRef, Tree};
use std::sync::LazyLock;

/// Re-export scraper crate
pub mod scraper {
    pub use scraper::*;
}

#[derive(Debug, thiserror::Error)]
pub enum DomExtractionError {
    #[error("Failed to access tree node: {0:?}")]
    NodeAccessError(NodeId),
}

/// Selector for <body> tag
#[allow(clippy::unwrap_used)]
static BODY_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("body").unwrap());

/// Prevent division by zero and convert integers into f32
#[inline]
fn normalize_denominator(value: u32) -> f32 {
    match value {
        0 => 1.0,
        _ => value as f32,
    }
}

/// A tree representation of the text density of an HTML document.
pub struct DensityTree {
    pub tree: Tree<DensityNode>,
}

/// A node in a `DensityTree` containing text density information.
#[derive(Debug, Clone)]
pub struct DensityNode {
    // node id in DOM provided by `scraper` crate
    pub node_id: NodeId,

    pub char_count: u32,
    pub tag_count: u32,
    pub link_char_count: u32,
    pub link_tag_count: u32,
    pub density: f32,

    pub density_sum: Option<f32>,
}

impl<'a> DensityTree {
    /// Create new `DensityTree` structure with a single root node.
    pub fn new(node_id: NodeId) -> Self {
        Self {
            tree: Tree::new(DensityNode::new(node_id)),
        }
    }

    /// Creates and calculates a `DensityTree` from a `scraper::Html` DOM tree.
    pub fn from_document(document: &Html) -> Result<Self, DomExtractionError> {
        // NOTE: process possible errors (when page is completely broken)
        let body = &document
            .select(&BODY_SELECTOR)
            .next()
            // NOTE:
            // Since scraper always adds a body tag, this unwrap is safe
            .expect("scraper always provides a body tag");

        // NOTE: there is usable value in document, such as error field
        let body_node_id = body.id();
        let body_node = body
            .tree()
            .get(body_node_id)
            .ok_or(DomExtractionError::NodeAccessError(body_node_id))?;

        let mut density_tree = Self::new(body_node_id);
        Self::build_density_tree(body_node, &mut density_tree.tree.root_mut(), 1);
        density_tree.calculate_density_tree();
        Ok(density_tree)
    }

    /// Returns a vector of nodes sorted by density in ascending order.
    /// Nodes with zero density are skipped.
    pub fn sorted_nodes(&'a self) -> Vec<&'a DensityNode> {
        let mut nodes = self
            .tree
            .values()
            .filter(|n| n.density.gt(&0.0))
            .collect::<Vec<&DensityNode>>();
        nodes.sort_by(|a, b| {
            a.density
                .partial_cmp(&b.density)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        nodes
    }

    /// Calculates composite text density index.
    pub fn composite_text_density(
        char_count: u32,
        tag_count: u32,
        link_char_count: u32,
        link_tag_count: u32,
        body_tag_char_count: u32,
        body_tag_link_char_count: u32,
    ) -> f32 {
        // can guess whole expression will be zero
        if char_count == 0 {
            return 0.0;
        };

        // labeled same as in paper's formula
        let ci = char_count as f32;
        let ti = normalize_denominator(tag_count);
        // let nlci = normalize_denominator(char_count - link_char_count);
        // ^^^^ can cause panic in certain cases
        // The panic is occurring because link_char_count is larger than
        // char_count, causing an integer underflow when trying to subtract. This can
        // happen if there are more characters in links than total characters, which is
        // possible in certain HTML structures.
        let nlci =
            normalize_denominator(char_count.saturating_sub(link_char_count));
        let lci = link_char_count as f32;
        let cb = normalize_denominator(body_tag_char_count);
        let lcb = body_tag_link_char_count as f32;
        let lti = normalize_denominator(link_tag_count);

        // checks
        debug_assert!(nlci > 0.0);

        let density = ci / ti;

        let ln_1 = (ci / nlci) * lci;
        let ln_2 = (lcb / cb) * ci;
        let e = std::f32::consts::E;

        debug_assert!(ln_1 >= 0.0);
        debug_assert!(ln_2 >= 0.0);

        let log_base = (ln_1 + ln_2 + e).ln();

        let value = (ci / lcb) * (ti / lti);
        value.log(log_base) * density
    }

    /// Computes the density for each node in the tree.
    pub fn calculate_density_tree(&mut self) {
        let body_tag_node = self.tree.root().value().clone();
        for node in self.tree.values_mut() {
            node.density = Self::composite_text_density(
                node.char_count,
                node.tag_count,
                node.link_char_count,
                node.link_tag_count,
                body_tag_node.char_count,
                body_tag_node.link_char_count,
            );
        }
    }

    /// Recursively builds a density tree, separate from the `scraper::Html` tree.
    /// Uses the same `NodeId` values, making it possible to retrieve document nodes
    /// from `scraper::Html`.
    pub fn build_density_tree(
        node: ego_tree::NodeRef<scraper::node::Node>,
        density_node: &mut ego_tree::NodeMut<DensityNode>,
        _depth: usize,
    ) {
        for child in node.children() {
            // some nodes makes no sense
            match child.value() {
                scraper::Node::Element(elem) => {
                    if elem.name() == "script"
                        || elem.name() == "noscript"
                        || elem.name() == "style"
                    {
                        continue;
                    };
                }
                scraper::Node::Comment(_) => {
                    continue;
                }
                scraper::Node::Document => {
                    continue;
                }
                _ => {}
            };

            let child_density_node = DensityNode::new(child.id());
            let mut te = density_node.append(child_density_node);
            Self::build_density_tree(child, &mut te, _depth + 1);
        }

        // Here dive into the deepest recurstion depth

        match node.value() {
            scraper::Node::Text(text) => {
                let char_count = text.trim().len() as u32;
                density_node.value().char_count += char_count;
            }
            scraper::Node::Element(elem) => {
                let tag_count = 1;
                density_node.value().tag_count += tag_count;
                // count buttons and selects as links as well
                if elem.name() == "a"
                    || elem.name() == "button"
                    || elem.name() == "select "
                {
                    let link_tag_count = 1;
                    density_node.value().link_tag_count += link_tag_count;
                };
            }
            _ => {}
        }

        let char_count = density_node.value().char_count;
        let tag_count = density_node.value().tag_count;
        let link_tag_count = density_node.value().link_tag_count;
        let mut link_char_count = density_node.value().link_char_count;

        if tag_count > 0 {
            density_node.value().density = density_node.value().char_count as f32
                / density_node.value().tag_count as f32;
        };

        if let Some(parent) = node.parent() {
            if let Some(element) = parent.value().as_element() {
                if element.name() == "a" {
                    link_char_count += char_count;
                }
            }
        }

        if let Some(mut parent) = density_node.parent() {
            parent.value().char_count += char_count;
            parent.value().tag_count += tag_count;
            parent.value().link_tag_count += link_tag_count;
            parent.value().link_char_count += link_char_count;
        };
    }

    /// Calculates the density sum for each node in the tree.
    ///
    /// This method iterates through all nodes in the tree and computes the sum of
    /// the densities of each node's children. The result is stored in the `density_sum`
    /// field of each node.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let mut dtree = DensityTree::from_document(&document).unwrap();
    /// dtree.calculate_density_sum();
    /// ```
    pub fn calculate_density_sum(&mut self) -> Result<(), DomExtractionError> {
        for node in self.tree.clone().nodes() {
            let sum = node.children().map(|child| child.value().density).sum();
            let mut mut_node = self
                .tree
                .get_mut(node.id())
                .ok_or(DomExtractionError::NodeAccessError(node.id()))?;
            mut_node.value().density_sum = Some(sum);
        }
        Ok(())
    }

    /// Finds the node with the maximum density sum in the tree.
    ///
    /// This method compares the `density_sum` of all nodes and returns the node
    /// with the highest value. If the tree is empty or all nodes have `None` as
    /// their `density_sum`, it returns `None`.
    ///
    /// # Returns
    ///
    /// An `Option<NodeRef<DensityNode>>` representing the node with the highest
    /// density sum, or `None` if no such node exists.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let dtree = DensityTree::from_document(&document).unwrap();
    /// if let Some(max_node) = dtree.get_max_density_sum_node() {
    ///     println!("Max density sum: {:?}", max_node.value().density_sum);
    /// }
    /// ```
    pub fn get_max_density_sum_node(&self) -> Option<NodeRef<DensityNode>> {
        self.tree.nodes().max_by(|a, b| {
            a.value()
                .density_sum
                .partial_cmp(&b.value().density_sum)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Extracts the main content from the HTML document.
    ///
    /// This method uses the density and density sum information to identify
    /// and extract the main content from the HTML document. It follows these steps:
    /// 1. Finds the node with the maximum density sum.
    /// 2. Calculates a threshold based on the average density of the node's ancestors.
    /// 3. Identifies the largest contiguous block of high-density content.
    /// 4. Extracts and concatenates the text from the identified content nodes.
    ///
    /// # Arguments
    ///
    /// * `document` - A reference to the HTML document.
    ///
    /// # Returns
    ///
    /// Result with `String` containing the extracted main content of the
    /// document or `DomExtractionError`
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let document = Html::parse_document(&html_string);
    /// let mut dtree = DensityTree::from_document(&document).unwrap();
    /// dtree.calculate_density_sum();
    /// let content = dtree.extract_content(&document).unwrap();
    /// println!("Extracted content: {}", content);
    /// ```
    pub fn extract_content(
        &self,
        document: &Html,
    ) -> Result<String, DomExtractionError> {
        if let Some(max_node) = self.get_max_density_sum_node() {
            // Calculate the average density of ancestors
            let ancestor_densities: Vec<f32> =
                max_node.ancestors().map(|n| n.value().density).collect();
            let threshold = ancestor_densities.iter().sum::<f32>()
                / ancestor_densities.len() as f32;

            // Find the largest contiguous block of high-density content
            let mut content_nodes: Vec<NodeRef<DensityNode>> = Vec::new();
            let mut current_block: Vec<NodeRef<DensityNode>> = Vec::new();
            for node in self.tree.nodes() {
                if node.value().density >= threshold
                    && node.value().density_sum.unwrap_or(0.0) > 0.0
                {
                    current_block.push(node);
                } else if !current_block.is_empty() {
                    if current_block.len() > content_nodes.len() {
                        content_nodes = current_block;
                    }
                    current_block = Vec::new();
                }
            }
            if current_block.len() > content_nodes.len() {
                content_nodes = current_block;
            }

            // Extract text from the content nodes, avoiding duplication
            let mut content = String::new();
            let mut seen_text = std::collections::HashSet::new();
            for node in content_nodes {
                let node_text = get_node_text(node.value().node_id, document)?;
                if !seen_text.contains(&node_text) {
                    content.push_str(&node_text);
                    content.push(' ');
                    seen_text.insert(node_text);
                }
            }
            Ok(content.trim().to_string())
        } else {
            Ok(String::new())
        }
    }
}

impl std::fmt::Debug for DensityTree {
    /// Format tree with identation
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn pretty_print(
            f: &mut std::fmt::Formatter<'_>,
            node: NodeRef<DensityNode>,
            depth: usize,
        ) {
            for child in node.children() {
                let dashes = " ".repeat(2 * depth);
                let _ = writeln!(f, "{}{:?}", dashes, child.value());
                pretty_print(f, child, depth + 1);
            }
        }

        writeln!(f, "DensityTree {{")?;
        pretty_print(f, self.tree.root(), 1);
        writeln!(f, "}}")
    }
}

impl DensityNode {
    /// Creates a new `DensityNode` with the given `NodeId` and zero values.
    pub fn new(node_id: NodeId) -> Self {
        Self {
            node_id,
            char_count: 0,
            tag_count: 0,
            link_char_count: 0,
            link_tag_count: 0,
            density: 0.0,
            density_sum: None,
        }
    }
}

/// Helper function to extract a node with the given `NodeId` from a `scraper::Html` document.
///
/// # Arguments
///
/// * `node_id` - The `NodeId` of the node to extract.
/// * `document` - A reference to the `scraper::Html` document.
///
/// # Returns
///
/// * An `ego_tree::NodeRef` representing the node with the specified `NodeId`.
///   or `DomExtractionError::NodeAccessError`
#[inline]
pub fn get_node_by_id(
    node_id: NodeId,
    document: &Html,
) -> Result<ego_tree::NodeRef<'_, scraper::node::Node>, DomExtractionError> {
    document
        .tree
        .get(node_id)
        .ok_or(DomExtractionError::NodeAccessError(node_id))
}

/// Helper function to extract all text from a `scraper::Html` document
/// by collecting text from all descendant nodes of the node with the given `NodeId`.
///
/// # Arguments
///
/// * `node_id` - The `NodeId` of the node whose descendant text should be extracted.
/// * `document` - A reference to the `scraper::Html` document.
///
/// # Returns
///
/// * Result with `String` containing the concatenated text from all descendant nodes
///   of the specified node, or `DomExtractionError`
pub fn get_node_text(
    node_id: NodeId,
    document: &Html,
) -> Result<String, DomExtractionError> {
    let mut text: Vec<String> = vec![];
    let root_node = get_node_by_id(node_id, document)?;
    for node in root_node.descendants() {
        if let Some(txt) = node.value().as_text() {
            let clean_text = txt.trim();
            if !clean_text.is_empty() {
                text.push(clean_text.to_string());
            };
        };
    }
    Ok(text.join(" "))
}

/// Helper function to extract all links (`href` attributes) from a `scraper::Html`
/// document by collecting links from the node with the given `NodeId` and
/// its descendants.
///
/// # Arguments
///
/// * `node_id` - The `NodeId` of the node whose descendant links should be extracted.
/// * `document` - A reference to the `scraper::Html` document.
///
/// # Returns
///
/// * Result with `Vec<String>` containing the extracted links from the
///   specified node and its descendants, or `DomExtractionError`
pub fn get_node_links(
    node_id: NodeId,
    document: &Html,
) -> Result<Vec<String>, DomExtractionError> {
    let mut links: Vec<String> = vec![];
    let root_node = get_node_by_id(node_id, document)?;
    for node in root_node.descendants() {
        if let Some(elem) = node.value().as_element() {
            if let Some(link) = elem.attr("href") {
                links.push(link.trim().to_string());
            };
        };
    }
    Ok(links)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::{fs, io, path};

    pub fn read_file(
        file_path: impl AsRef<path::Path>,
    ) -> Result<String, io::Error> {
        let content: String = fs::read_to_string(file_path)?;
        Ok(content)
    }

    #[test]
    fn test_body_selector_initialization() {
        // This will force the LazyLock to initialize
        let _ = &*BODY_SELECTOR;
    }

    #[test]
    fn test_document_always_has_body() {
        // Test with various malformed HTML
        let test_cases = [
            "",
            "<div>No body here</div>",
            "<<<>>>",
            "Plain text",
            "<html><div>No explicit body</div></html>",
        ];

        for html in test_cases {
            let document = build_dom(html);
            let body_elements: Vec<_> = document.select(&BODY_SELECTOR).collect();
            assert_eq!(
                body_elements.len(),
                1,
                "HTML parser should always provide a body tag"
            );
        }
    }

    #[test]
    fn test_body_selector() {
        let content = read_file("html/test_1.html").unwrap();
        let document = build_dom(content.as_str());

        // This will force initialization and use of BODY_SELECTOR
        let body_elements: Vec<_> = document.select(&BODY_SELECTOR).collect();
        assert_eq!(body_elements.len(), 1); // Should find exactly one body tag
    }

    pub fn build_dom(html: &str) -> Html {
        let document: Html = Html::parse_document(html);
        document
    }

    fn load_content(test_file_name: &str) -> Html {
        let content = read_file(format!("html/{}", test_file_name)).unwrap();
        build_dom(content.as_str())
    }

    #[test]
    fn test_normalize_denominator() {
        assert_eq!(normalize_denominator(32), 32.0);
        assert_eq!(normalize_denominator(0), 1.0);
    }

    #[test]
    fn test_load_file() {
        let content = read_file("html/test_1.html");
        assert!(content.is_ok());
        assert!(!content.unwrap().is_empty());
    }

    #[test]
    fn test_build_dom() {
        let document = load_content("test_2.html");
        assert!(document.errors.len() == 1);
    }

    #[test]
    fn test_composite_text_density() {
        let char_count = 100;
        let tag_count = 10;
        let link_char_count = 20;
        let link_tag_count = 4;
        let body_tag_char_count = 500;
        let body_tag_link_char_count = 100;

        let result = DensityTree::composite_text_density(
            char_count,
            tag_count,
            link_char_count,
            link_tag_count,
            body_tag_char_count,
            body_tag_link_char_count,
        );

        assert!(result.is_finite());
        assert!(result >= 0.0);

        // Test edge cases
        let result_zero_char_count = DensityTree::composite_text_density(
            0,
            tag_count,
            link_char_count,
            link_tag_count,
            body_tag_char_count,
            body_tag_link_char_count,
        );
        assert_eq!(result_zero_char_count, 0.0);

        let result_zero_tag_count = DensityTree::composite_text_density(
            0,
            tag_count,
            link_char_count,
            link_tag_count,
            body_tag_char_count,
            body_tag_link_char_count,
        );
        assert!(result_zero_tag_count.is_finite());
        assert!(result_zero_tag_count >= 0.0);
    }

    #[test]
    fn test_build_density_tree() {
        let content = read_file("html/test_1.html").unwrap();
        let document = build_dom(content.as_str());

        let dtree = DensityTree::from_document(&document).unwrap();
        assert_eq!(dtree.tree.values().count(), 55);
    }

    #[test]
    fn test_sorted_density_results() {
        let document = load_content("test_1.html");

        let dtree = DensityTree::from_document(&document).unwrap();
        let sorted_nodes = dtree.sorted_nodes();
        let node_id = sorted_nodes.last().unwrap().node_id;
        assert_eq!(format!("{:?}", node_id), "NodeId(22)");

        let node = get_node_by_id(node_id, &document).unwrap();

        let node_attr = node.value().as_element().unwrap().attrs().last().unwrap();
        assert_eq!(node_attr.0, "class");
        assert_eq!(node_attr.1, "articleBody");
    }

    #[test]
    fn test_get_node_text() {
        let content = read_file("html/test_1.html").unwrap();
        let document = build_dom(content.as_str());

        let dtree = DensityTree::from_document(&document).unwrap();
        let sorted_nodes = dtree.sorted_nodes();
        let node_id = sorted_nodes.last().unwrap().node_id;
        assert_eq!(get_node_text(node_id, &document).unwrap().len(), 200);
    }

    #[test]
    fn test_get_node_links() {
        let content = read_file("html/test_1.html").unwrap();
        let document = build_dom(content.as_str());

        let dtree = DensityTree::from_document(&document).unwrap();
        let sorted_nodes = dtree.sorted_nodes();
        let node_id = sorted_nodes.last().unwrap().node_id;
        assert_eq!(get_node_links(node_id, &document).unwrap().len(), 2);
    }

    #[test]
    fn test_print_dtree() {
        let content = read_file("html/test_2.html").unwrap();
        let document = build_dom(content.as_str());

        let dtree = DensityTree::from_document(&document).unwrap();

        assert_eq!(format!("{:?}", dtree).lines().count(), 18);
    }

    #[test]
    fn test_leftovers() {
        let content = read_file("html/test_4.html").unwrap();
        let document = build_dom(content.as_str());

        let dtree = DensityTree::from_document(&document).unwrap();
        let sorted_nodes = dtree.sorted_nodes();
        let node_id = sorted_nodes.last().unwrap().node_id;

        assert_eq!(format!("{:?}", node_id), "NodeId(12)");
    }

    #[test]
    fn test_calculate_density_sum() {
        let content = read_file("html/test_1.html").unwrap();
        let document = build_dom(content.as_str());
        let mut dtree = DensityTree::from_document(&document).unwrap();

        // Before calculation, all density_sum values should be None
        for node in dtree.tree.values() {
            assert_eq!(node.density_sum, None);
        }

        dtree.calculate_density_sum().unwrap();

        // After calculation, all density_sum values should be Some
        for node in dtree.tree.values() {
            assert!(node.density_sum.is_some());
        }

        // Verify that leaf nodes have a density_sum of 0
        for node in dtree.tree.nodes() {
            if node.children().count() == 0 {
                assert_eq!(node.value().density_sum, Some(0.0));
            }
        }

        // Verify that a parent's density_sum is equal to the sum of its children's densities
        for node in dtree.tree.nodes() {
            if node.children().count() > 0 {
                let expected_sum: f32 =
                    node.children().map(|child| child.value().density).sum();
                assert!(
                    (node.value().density_sum.unwrap() - expected_sum).abs()
                        < f32::EPSILON
                );
            }
        }

        // Find the maximum density_sum
        let max_density_sum = dtree
            .tree
            .values()
            .map(|node| node.density_sum.unwrap())
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap();

        // Verify that no node has a density_sum greater than the maximum
        for node in dtree.tree.values() {
            assert!(node.density_sum.unwrap() <= max_density_sum);
        }

        // Verify that at least one node has the maximum density_sum
        assert!(dtree
            .tree
            .values()
            .any(|node| node.density_sum.unwrap() == max_density_sum));
    }

    #[test]
    fn test_get_max_density_sum_node() {
        let content = read_file("html/test_1.html").unwrap();
        let document = build_dom(content.as_str());
        let mut dtree = DensityTree::from_document(&document).unwrap();

        dtree.calculate_density_sum().unwrap();
        let max_node = dtree.get_max_density_sum_node().unwrap();

        // Check if the max_node has the highest density_sum
        let max_density_sum = max_node.value().density_sum.unwrap();
        for node in dtree.tree.values() {
            assert!(node.density_sum.unwrap() <= max_density_sum);
        }
    }

    #[test]
    fn test_extract_content() {
        let content = read_file("html/test_1.html").unwrap();
        let document = build_dom(content.as_str());
        let mut dtree = DensityTree::from_document(&document).unwrap();

        dtree
            .calculate_density_sum()
            .expect("Error while calculating Density Sum");
        let extracted_content = dtree.extract_content(&document).unwrap();

        // Check if the extracted content is not empty
        assert!(!extracted_content.is_empty());

        // Check if the extracted content contains expected text
        assert!(extracted_content.contains("Here is text"));
        assert!(extracted_content.contains("Here is article"));
        assert!(extracted_content.contains("Even more huge"));

        assert!(!extracted_content.contains("Menu"));
    }
}
