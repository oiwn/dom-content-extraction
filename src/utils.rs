use crate::DomExtractionError;
use ego_tree::NodeId;
use scraper::Html;

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
