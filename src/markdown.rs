use crate::{DensityTree, DomExtractionError};
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

    // Get the NodeId from the density node
    let node_id = max_node.value().node_id;

    // Get the scraper node from the document
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

    // Extract the HTML content
    let html_content = element_ref.inner_html();

    // Convert HTML to markdown using htmd
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

        // Debug: print what we actually got
        println!("Generated markdown: '{}'", markdown);

        // Should contain the main content
        assert!(!markdown.is_empty(), "Markdown should not be empty");
        // Relaxed assertions for debugging
        assert!(markdown.contains("Main Article"));
        assert!(markdown.contains("main content"));
    }
}
