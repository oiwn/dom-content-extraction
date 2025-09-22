use crate::{
    DomExtractionError, get_node_text,
    tree::{BODY_SELECTOR, NodeMetrics},
};
use ego_tree::{NodeId, NodeRef, Tree};
use scraper::Html;

/// Prevent division by zero and convert integers into f32
#[inline]
pub fn normalize_denominator(value: u32) -> f32 {
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
    pub metrics: NodeMetrics,
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
        Self::build_density_tree(body_node, &mut density_tree.tree.root_mut());
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

    pub fn composite_text_density(
        metrics: &NodeMetrics,
        body_metrics: &NodeMetrics,
    ) -> f32 {
        // Return 0.0 if there's no content
        if metrics.char_count == 0 {
            return 0.0;
        }

        // labeled same as in paper's formula
        let ci = metrics.char_count as f32;
        let ti = normalize_denominator(metrics.tag_count);
        let nlci = normalize_denominator(
            metrics.char_count.saturating_sub(metrics.link_char_count),
        );
        let lci = metrics.link_char_count as f32;
        let cb = normalize_denominator(body_metrics.char_count);
        let lcb = body_metrics.link_char_count as f32;
        let lti = normalize_denominator(metrics.link_tag_count);

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

    /// Calculates composite text density index.
    pub fn composite_text_densityi_old(
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
        let body_node = self.tree.root().value().clone();
        for node in self.tree.values_mut() {
            node.density =
                Self::composite_text_density(&node.metrics, &body_node.metrics);
        }
    }

    /// Recursively builds a density tree, separate from the `scraper::Html` tree.
    /// Uses the same `NodeId` values, making it possible to retrieve document nodes
    /// from `scraper::Html`.
    pub fn build_density_tree(
        node: ego_tree::NodeRef<scraper::node::Node>,
        density_node: &mut ego_tree::NodeMut<DensityNode>,
    ) {
        // Process children first
        for child in node.children() {
            // Skip irrelevant nodes
            match child.value() {
                scraper::Node::Element(elem) => {
                    if elem.name() == "script"
                        || elem.name() == "noscript"
                        || elem.name() == "style"
                    {
                        continue;
                    };
                }
                scraper::Node::Comment(_) | scraper::Node::Document => {
                    continue;
                }
                _ => {}
            };

            let child_density_node = DensityNode::new(child.id());
            let mut te = density_node.append(child_density_node);
            Self::build_density_tree(child, &mut te);
        }

        // Process current node
        match node.value() {
            scraper::Node::Text(text) => {
                // let char_count = text.trim().len() as u32;
                // density_node.value().metrics.char_count += char_count;
                // NOTE: adding unicode support
                let char_count = crate::unicode::count_graphemes(text.trim());
                density_node.value().metrics.char_count += char_count;
            }
            scraper::Node::Element(elem) => {
                density_node.value().metrics.tag_count += 1;
                if elem.name() == "a"
                    || elem.name() == "button"
                    || elem.name() == "select"
                {
                    density_node.value().metrics.link_tag_count += 1;
                };
            }
            _ => {}
        }

        // Handle link char count for text within links
        if let Some(parent) = node.parent()
            && let Some(element) = parent.value().as_element()
            && element.name() == "a"
        {
            density_node.value().metrics.link_char_count +=
                density_node.value().metrics.char_count;
        }

        // Update parent metrics by combining current node's metrics
        let current_metrics = density_node.value().metrics.clone();
        if let Some(mut parent) = density_node.parent() {
            parent.value().metrics.combine(&current_metrics);
        }
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
    pub fn get_max_density_sum_node(&self) -> Option<NodeRef<'_, DensityNode>> {
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
            let threshold = if ancestor_densities.is_empty() {
                // If no ancestors, use the max node's density as threshold
                max_node.value().density
            } else {
                ancestor_densities.iter().sum::<f32>()
                    / ancestor_densities.len() as f32
            };

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
            // Ok(content.trim().to_string())
            Ok(crate::unicode::normalize_text(&content))
        } else {
            Ok(String::new())
        }
    }
}

impl std::fmt::Debug for DensityTree {
    /// Format tree with indentation
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
            metrics: NodeMetrics::new(), // Initialize with new NodeMetrics
            density: 0.0,
            density_sum: None,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::utils::{
        build_dom, build_dom_from_file, get_node_by_id, get_node_links, read_file,
    };

    #[test]
    fn test_normalize_denominator() {
        assert_eq!(normalize_denominator(32), 32.0);
        assert_eq!(normalize_denominator(0), 1.0);
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
        let document = build_dom_from_file("test_1.html");

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
        assert_eq!(
            crate::unicode::count_graphemes(
                &get_node_text(node_id, &document).unwrap()
            ),
            186
        );
        // assert_eq!(get_node_text(node_id, &document).unwrap().len(), 200);
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

        println!("Node: {:?}", sorted_nodes.last().unwrap());
        println!(
            "Node html: {:?}",
            get_node_by_id(node_id, &document).unwrap().value()
        );

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
        assert!(
            dtree
                .tree
                .values()
                .any(|node| node.density_sum.unwrap() == max_density_sum)
        );
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

    #[test]
    fn test_document_node_handling() {
        // Create a minimal HTML document that forces Document node traversal
        let html = r#"<!DOCTYPE html><html><body><div>Test</div></body></html>"#;
        let document = Html::parse_document(html);

        // Get the root node which should be a Document node
        let root_node = document.tree.root();
        assert!(matches!(root_node.value(), scraper::Node::Document));

        // Create a DensityTree starting from root to ensure Document node is encountered
        let mut density_tree = DensityTree::new(root_node.id());
        DensityTree::build_density_tree(
            root_node,
            &mut density_tree.tree.root_mut(),
        );

        // If we reach here without panicking and the tree is built,
        // it means the Document node was properly skipped
        assert!(density_tree.tree.root().children().count() > 0);

        // Verify the content is still processed despite skipping Document node
        let text_nodes: Vec<_> = density_tree
            .tree
            .nodes()
            .filter(|n| n.value().metrics.char_count > 0)
            .collect();
        assert!(!text_nodes.is_empty());
    }

    #[test]
    fn test_composite_text_density() {
        let node_metrics = NodeMetrics {
            char_count: 100,
            tag_count: 10,
            link_char_count: 20,
            link_tag_count: 4,
        };
        let body_metrics = NodeMetrics {
            char_count: 1000,
            tag_count: 300,
            link_char_count: 200,
            link_tag_count: 100,
        };

        let result =
            DensityTree::composite_text_density(&node_metrics, &body_metrics);

        assert!(result.is_finite());
        assert!(result >= 0.0);

        // Edge case - no chars in node
        let node_metrics = NodeMetrics {
            char_count: 0,
            tag_count: 10,
            link_char_count: 20,
            link_tag_count: 4,
        };
        let result_zero_char_count =
            DensityTree::composite_text_density(&node_metrics, &body_metrics);
        assert_eq!(result_zero_char_count, 0.0);

        let node_metrics = NodeMetrics {
            char_count: 100,
            tag_count: 1, // situation with 0 is impossible (propagate to parent and skip)
            link_char_count: 0,
            link_tag_count: 0,
        };
        let result_zero_tag_count =
            DensityTree::composite_text_density(&node_metrics, &body_metrics);
        assert!(result_zero_tag_count.is_finite());
        // println!("Zero Tag: {}", result_zero_tag_count);
        // TODO: figure out how to actually check
        assert!(result_zero_tag_count < 0.0);
    }
}
