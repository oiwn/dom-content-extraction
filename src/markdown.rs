use crate::{DensityTree, DomExtractionError};
use ego_tree::NodeRef;
use scraper::{ElementRef, Html};

/// Extracts the main content from an HTML document as markdown using CETD analysis.
///
/// This function identifies the highest density content node using the CETD algorithm
/// and converts its HTML content to markdown format.
///
/// # Arguments
/// * `dtree` - A DensityTree that has been built and analyzed
/// * `document` - The original HTML document for node reference
///
/// # Returns
/// A Result containing the extracted markdown content or an error
#[cfg(feature = "markdown")]
pub fn extract_content_as_markdown(
    dtree: &DensityTree,
    document: &Html,
) -> Result<String, DomExtractionError> {
    // Get the node with maximum density sum
    let max_node = match dtree.get_max_density_sum_node() {
        Some(node) => node,
        None => return Ok(String::new()), // No content found
    };

    // Calculate the average density of ancestors to establish a threshold
    let ancestor_densities: Vec<f32> =
        max_node.ancestors().map(|n| n.value().density).collect();
    let threshold = if ancestor_densities.is_empty() {
        0.0
    } else {
        ancestor_densities.iter().sum::<f32>() / ancestor_densities.len() as f32
    };

    // Find high-density nodes (similar to text extraction logic)
    let mut high_density_nodes: Vec<NodeRef<'_, crate::cetd::DensityNode>> =
        Vec::new();
    for node in dtree.tree.nodes() {
        if node.value().density >= threshold
            && node.value().density_sum.unwrap_or(0.0) > 0.0
        {
            high_density_nodes.push(node);
        }
    }

    // If no high-density nodes found, fall back to original single-node approach
    if high_density_nodes.is_empty() {
        return extract_single_node_content(max_node.value().node_id, document);
    }

    // Collect scraper nodes for high-density content
    let mut candidate_nodes: Vec<ego_tree::NodeId> = Vec::new();

    for density_node in high_density_nodes {
        candidate_nodes.push(density_node.value().node_id);
    }

    // Try to find a content container, but fall back to single node if it fails
    match find_content_container(
        candidate_nodes,
        max_node.value().node_id,
        document,
    ) {
        Ok(markdown) => Ok(markdown),
        Err(_) => extract_single_node_content(max_node.value().node_id, document),
    }
}

/// Find a content container that includes multiple high-density nodes
fn find_content_container(
    _node_ids: Vec<ego_tree::NodeId>,
    max_node_id: ego_tree::NodeId,
    document: &Html,
) -> Result<String, DomExtractionError> {
    // For now, use a simple approach: walk up from the max density node
    // until we find a container that likely includes related content
    let mut current_node = document
        .tree
        .get(max_node_id)
        .ok_or(DomExtractionError::NodeAccessError(max_node_id))?;

    // Walk up a few levels to find a reasonable container
    for _ in 0..5 {
        // Increased to 5 levels to handle deeper structures
        if let Some(parent) = current_node.parent() {
            current_node = parent;

            // Check if this might be a good container (e.g., article, div, section)
            if let Some(element) = ElementRef::wrap(current_node) {
                let tag_name = element.value().name();
                // Include more container types
                if tag_name == "article"
                    || tag_name == "main"
                    || tag_name == "section"
                    || tag_name == "div"
                    || tag_name == "content"
                {
                    break;
                }
            }
        } else {
            break;
        }
    }

    // Find the nearest element that can be wrapped
    let mut element_node = current_node;
    while let Some(parent) = element_node.parent() {
        if ElementRef::wrap(element_node).is_some() {
            break;
        }
        element_node = parent;
    }

    // Extract HTML from the container
    let element_ref = ElementRef::wrap(element_node)
        .ok_or(DomExtractionError::NodeAccessError(max_node_id))?;

    let html_content = element_ref.inner_html();
    let converter = htmd::HtmlToMarkdown::builder()
        .skip_tags(vec!["script", "style"])
        .build();

    converter
        .convert(&html_content)
        .map_err(|_| DomExtractionError::NodeAccessError(max_node_id))
        .map(|md| md.trim().to_string())
}

/// Fallback: extract content from a single node (original behavior)
fn extract_single_node_content(
    node_id: ego_tree::NodeId,
    document: &Html,
) -> Result<String, DomExtractionError> {
    let scraper_node = document
        .tree
        .get(node_id)
        .ok_or(DomExtractionError::NodeAccessError(node_id))?;

    // Find the nearest parent element that can be wrapped as ElementRef
    let mut current_node = scraper_node;
    let element_ref = loop {
        if let Some(element) = ElementRef::wrap(current_node) {
            break element;
        }

        // Move to parent if current node is not an element
        if let Some(parent) = current_node.parent() {
            current_node = parent;
        } else {
            return Err(DomExtractionError::NodeAccessError(node_id));
        }
    };

    let html_content = element_ref.inner_html();
    let converter = htmd::HtmlToMarkdown::builder()
        .skip_tags(vec!["script", "style"])
        .build();

    converter
        .convert(&html_content)
        .map_err(|_| DomExtractionError::NodeAccessError(node_id))
        .map(|md| md.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DensityTree;
    use std::fs;

    #[test]
    #[cfg(feature = "markdown")]
    fn test_extract_content_as_markdown() {
        let html = r#"
            <html>
            <body>
                <div class="header">Navigation</div>
                <article>
                    <h1>Main Article</h1>
                    <p>This is the main content with lots of text that should have high density.</p>
                    <p>Another paragraph with substantial content for density analysis.</p>
                </article>
                <div class="sidebar">Sidebar content</div>
            </body>
            </html>
        "#;

        let document = Html::parse_document(html);
        let mut dtree = DensityTree::from_document(&document).unwrap();
        dtree.calculate_density_sum().unwrap();

        let markdown = extract_content_as_markdown(&dtree, &document).unwrap();

        // Should contain the main content
        assert!(!markdown.is_empty(), "Markdown should not be empty");
        assert!(markdown.contains("Main Article"));
        assert!(markdown.contains("main content"));
    }

    #[test]
    #[cfg(feature = "markdown")]
    fn test_extract_from_test1_html() {
        let html_content = fs::read_to_string("html/test_1.html")
            .expect("Unable to read test_1.html");
        let document = Html::parse_document(&html_content);
        let mut dtree = DensityTree::from_document(&document).unwrap();
        dtree.calculate_density_sum().unwrap();

        let markdown = extract_content_as_markdown(&dtree, &document).unwrap();

        // Debug: print what we actually got
        println!("test1 markdown: '{}'", markdown);

        // Should extract article body content (highest density)
        assert!(!markdown.is_empty(), "Markdown should not be empty");
        // Check for content that should be present in article body
        assert!(markdown.contains("Here is text"));
        assert!(markdown.contains("Paragraph text"));
        assert!(markdown.contains("huge paragraph"));
        // Should not contain footer navigation
        assert!(!markdown.contains("Menu"));
        assert!(!markdown.contains("link1"));
    }

    #[test]
    #[cfg(feature = "markdown")]
    fn test_extract_from_test2_html() {
        let html_content = fs::read_to_string("html/test_2.html")
            .expect("Unable to read test_2.html");
        let document = Html::parse_document(&html_content);
        let mut dtree = DensityTree::from_document(&document).unwrap();
        dtree.calculate_density_sum().unwrap();

        let markdown = extract_content_as_markdown(&dtree, &document).unwrap();

        // Debug: print what we actually got
        println!("test2 markdown: '{}'", markdown);

        // Should extract article body content (highest density)
        assert!(!markdown.is_empty(), "Markdown should not be empty");
        // Check for content that should be present in article body
        assert!(markdown.contains("Here is text"));
        assert!(markdown.contains("long paragraph"));
        // Links should be converted to markdown format
        assert!(markdown.contains("wikipedia"));
    }

    #[test]
    #[cfg(feature = "markdown")]
    fn test_extract_from_test4_html() {
        let html_content = fs::read_to_string("html/test_4.html")
            .expect("Unable to read test_4.html");
        let document = Html::parse_document(&html_content);
        let mut dtree = DensityTree::from_document(&document).unwrap();
        dtree.calculate_density_sum().unwrap();

        let markdown = extract_content_as_markdown(&dtree, &document).unwrap();

        // Debug: print what we actually got
        println!("test4 markdown: '{}'", markdown);

        // Should extract article content and filter out scripts/comments
        assert!(!markdown.is_empty(), "Markdown should not be empty");
        // Check for content that should be present
        assert!(markdown.contains("Lorem ipsum"));
        assert!(markdown.contains("long paragraph"));
        assert!(markdown.contains("wikipedia"));
        // Should not contain script content
        assert!(!markdown.contains("myFunction"));
        assert!(!markdown.contains("Some comments"));
    }

    #[test]
    #[cfg(feature = "markdown")]
    fn test_empty_content_returns_empty_markdown() {
        let html = r#"
            <html>
            <body>
                <script>console.log("empty")</script>
            </body>
            </html>
        "#;

        let document = Html::parse_document(html);
        let mut dtree = DensityTree::from_document(&document).unwrap();
        dtree.calculate_density_sum().unwrap();

        let markdown = extract_content_as_markdown(&dtree, &document).unwrap();

        // Debug: print what we actually got
        println!("empty content markdown: '{}'", markdown);

        // Empty content should return empty string
        assert!(
            markdown.is_empty(),
            "Expected empty markdown for content-less HTML, got: '{}'",
            markdown
        );
    }
}
