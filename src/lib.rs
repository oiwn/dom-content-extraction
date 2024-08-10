#![crate_name = "dom_content_extraction"]
use crate::scraper::{Html, Selector};
use ego_tree::{NodeId, NodeRef, Tree};
use once_cell::sync::Lazy;

/// Re-export scraper crate
pub mod scraper {
    pub use scraper::*;
}

/// Selector for <body> tag
static BODY_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("body").unwrap());

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
    pub node_id: NodeId,

    pub char_count: u32,
    pub tag_count: u32,
    pub link_char_count: u32,
    pub link_tag_count: u32,
    pub density: f32,
}

impl<'a> DensityTree {
    /// Create new `DensityTree` structure with a single root node.
    pub fn new(node_id: NodeId) -> Self {
        Self {
            tree: Tree::new(DensityNode::new(node_id)),
        }
    }

    /// Creates and calculates a `DensityTree` from a `scraper::Html` DOM tree.
    pub fn from_document(document: &Html) -> Self {
        // NOTE: process possible errors (when page is completely broken)
        let body = &document.select(&BODY_SELECTOR).next().unwrap().to_owned();
        // NOTE: there is usable value in document, such as error field
        let body_node_id = body.id();
        let body_node = body.tree().get(body_node_id).unwrap();

        let mut density_tree = Self::new(body_node_id);
        Self::build_density_tree(body_node, &mut density_tree.tree.root_mut(), 1);
        density_tree.calculate_density_tree();
        density_tree
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
        let nlci = normalize_denominator(char_count - link_char_count);
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
#[inline]
pub fn get_node_by_id(
    node_id: NodeId,
    document: &Html,
) -> ego_tree::NodeRef<'_, scraper::node::Node> {
    document.tree.get(node_id).unwrap()
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
/// * A `String` containing the concatenated text from all descendant nodes of the specified node.
pub fn get_node_text(node_id: NodeId, document: &Html) -> String {
    let mut text: Vec<String> = vec![];
    let root_node = get_node_by_id(node_id, document);
    for node in root_node.descendants() {
        if let Some(txt) = node.value().as_text() {
            let clean_text = txt.trim();
            if !clean_text.is_empty() {
                text.push(clean_text.to_string());
            };
        };
    }
    text.join(" ")
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
/// * A `Vec<String>` containing the extracted links from the specified node and its descendants.
pub fn get_node_links(node_id: NodeId, document: &Html) -> Vec<String> {
    let mut links: Vec<String> = vec![];
    let root_node = get_node_by_id(node_id, document);
    for node in root_node.descendants() {
        if let Some(elem) = node.value().as_element() {
            if let Some(link) = elem.attr("href") {
                links.push(link.trim().to_string());
            };
        };
    }
    links
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, io, path};

    pub fn read_file(
        file_path: impl AsRef<path::Path>,
    ) -> Result<String, io::Error> {
        let content: String = fs::read_to_string(file_path)?;
        Ok(content)
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
        assert_eq!(content.is_ok(), true);
        assert_eq!(content.unwrap().len() > 0, true);
    }

    #[test]
    fn test_build_dom() {
        let document = load_content("test_2.html");
        assert_eq!(document.errors.len() == 1, true);
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

        let dtree = DensityTree::from_document(&document);
        assert_eq!(dtree.tree.values().count(), 52);
    }

    #[test]
    fn test_sorted_density_results() {
        let document = load_content("test_1.html");

        let dtree = DensityTree::from_document(&document);
        let sorted_nodes = dtree.sorted_nodes();
        let node_id = sorted_nodes.last().unwrap().node_id;
        assert_eq!(format!("{:?}", node_id), "NodeId(22)");

        let node = get_node_by_id(node_id, &document);

        let node_attr = node.value().as_element().unwrap().attrs().last().unwrap();
        assert_eq!(node_attr.0, "class");
        assert_eq!(node_attr.1, "articleBody");
    }

    #[test]
    fn test_get_node_text() {
        let content = read_file("html/test_1.html").unwrap();
        let document = build_dom(content.as_str());

        let dtree = DensityTree::from_document(&document);
        let sorted_nodes = dtree.sorted_nodes();
        let node_id = sorted_nodes.last().unwrap().node_id;
        assert_eq!(get_node_text(node_id, &document).len(), 200);
    }

    #[test]
    fn test_get_node_links() {
        let content = read_file("html/test_1.html").unwrap();
        let document = build_dom(content.as_str());

        let dtree = DensityTree::from_document(&document);
        let sorted_nodes = dtree.sorted_nodes();
        let node_id = sorted_nodes.last().unwrap().node_id;
        assert_eq!(get_node_links(node_id, &document).len(), 2);
    }

    #[test]
    fn test_print_dtree() {
        let content = read_file("html/test_2.html").unwrap();
        let document = build_dom(content.as_str());

        let dtree = DensityTree::from_document(&document);

        assert_eq!(format!("{:?}", dtree).lines().count(), 18);
    }

    #[test]
    fn test_leftovers() {
        let content = read_file("html/test_4.html").unwrap();
        let document = build_dom(content.as_str());

        let dtree = DensityTree::from_document(&document);
        let sorted_nodes = dtree.sorted_nodes();
        let node_id = sorted_nodes.last().unwrap().node_id;

        assert_eq!(format!("{:?}", node_id), "NodeId(12)");
    }
}
