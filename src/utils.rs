use crate::DomExtractionError;
use ego_tree::NodeId;
use scraper::Html;
#[cfg(test)]
use std::{fs, io, path};

/// Helper function to extract a node with the given `NodeId`
/// from a `scraper::Html` document.
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
/// * Result with `String` containing the concatenated text from all
///   descendant nodes of the specified node, or `DomExtractionError`
// Replace the existing get_node_text function with this updated version:
pub fn get_node_text(
    node_id: NodeId,
    document: &Html,
) -> Result<String, DomExtractionError> {
    let mut text_fragments: Vec<String> = vec![];
    let root_node = get_node_by_id(node_id, document)?;
    collect_text_filtered(&root_node, &mut text_fragments);
    // Use the Unicode join function instead of simple join
    Ok(crate::unicode::join_text_fragments(text_fragments))
}

pub(crate) fn is_non_content_text(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return true;
    }

    let len = trimmed.chars().count();
    let lower = trimmed.to_ascii_lowercase();

    let looks_like_css_block = (lower.starts_with('.')
        || lower.starts_with('#')
        || lower.starts_with("@media")
        || lower.starts_with("@import"))
        && trimmed.contains('{')
        && trimmed.contains('}');
    if looks_like_css_block && (len >= 80 || punctuation_ratio(trimmed) > 0.18) {
        return true;
    }

    if len < 80 {
        return false;
    }

    let assignment_count = count_substrings(&lower, " = ")
        + count_substrings(&lower, "=\"")
        + count_substrings(&lower, "='")
        + count_substrings(&lower, "={")
        + count_substrings(&lower, "=[");
    let call_count = count_substrings(&lower, "function(")
        + count_substrings(&lower, "function (")
        + count_substrings(&lower, "=>")
        + count_substrings(&lower, ");")
        + count_substrings(&lower, "});");
    let js_api_count = [
        "window.",
        "document.",
        "queryselector",
        "getelement",
        "createelement",
        "addeventlistener",
        "settimeout(",
        "datalayer",
        "gtag(",
    ]
    .iter()
    .filter(|marker| lower.contains(**marker))
    .count();
    let ad_script_count = ["yacontextcb", "adfox", "xboost", "cartsettings"]
        .iter()
        .filter(|marker| lower.contains(**marker))
        .count();

    let punctuation_ratio = punctuation_ratio(trimmed);
    let has_long_encoded_token = has_long_encoded_token(trimmed);
    let has_code_delimiters =
        trimmed.contains('{') || trimmed.contains('}') || trimmed.contains(';');
    let has_js_assignment =
        lower.contains("window.") && assignment_count > 0 && has_code_delimiters;
    let looks_like_js_blob = punctuation_ratio > 0.12
        && ((js_api_count >= 2 && call_count > 0)
            || (js_api_count >= 1 && assignment_count >= 2)
            || (ad_script_count >= 1 && (call_count > 0 || assignment_count > 0)))
        || (ad_script_count >= 2 && has_code_delimiters);
    let looks_like_config_blob = has_long_encoded_token
        && ((punctuation_ratio > 0.16
            && (assignment_count > 0 || lower.contains("window.")))
            || has_js_assignment);

    looks_like_js_blob || looks_like_config_blob
}

pub(crate) fn should_skip_element(elem: &scraper::node::Element) -> bool {
    if matches!(
        elem.name(),
        "script" | "noscript" | "style" | "svg" | "template" | "canvas" | "iframe"
    ) {
        return true;
    }

    if elem.attr("hidden").is_some()
        || elem
            .attr("aria-hidden")
            .is_some_and(|value| value.eq_ignore_ascii_case("true"))
    {
        return true;
    }

    if let Some(style) = elem.attr("style") {
        let style = style.to_ascii_lowercase().replace(' ', "");
        if style.contains("display:none") || style.contains("visibility:hidden") {
            return true;
        }
    }

    // Tailwind/utility-class hiding: skip elements whose `class` attribute
    // contains a token like `hidden`, `invisible`, or `sr-only` (used by
    // Tailwind, Bootstrap, and common CSS frameworks for `display:none`,
    // `visibility:hidden`, and visually-hidden-but-still-accessible).
    // Token-based to avoid false positives on e.g. `class="menu-hidden-link`.
    if let Some(class) = elem.attr("class") {
        const HIDDEN_CLASS_TOKENS: &[&str] = &["hidden", "invisible", "sr-only"];
        if class
            .split_whitespace()
            .any(|tok| HIDDEN_CLASS_TOKENS.contains(&tok))
        {
            return true;
        }
    }

    let class = elem.attr("class").unwrap_or("");
    let id = elem.attr("id").unwrap_or("");
    let marker_source = format!("{class} {id}").to_ascii_lowercase();
    let non_content_markers = [
        "robots-nocontent",
        "sharedaddy",
        "sd-sharing",
        "jetpack-likes-widget",
        "jp-relatedposts",
        "ads__",
        "adfox",
        "yatag",
    ];

    if non_content_markers
        .iter()
        .any(|marker| marker_source.contains(marker))
    {
        return true;
    }

    elem.attr("data-content")
        .is_some_and(|value| value.eq_ignore_ascii_case("webr"))
}

fn count_substrings(haystack: &str, needle: &str) -> usize {
    haystack.matches(needle).count()
}

fn punctuation_ratio(text: &str) -> f32 {
    let mut total = 0;
    let mut punctuation = 0;
    for ch in text.chars() {
        total += 1;
        if matches!(
            ch,
            '{' | '}'
                | '['
                | ']'
                | '('
                | ')'
                | ';'
                | '='
                | '<'
                | '>'
                | ':'
                | '/'
                | '\\'
                | '"'
                | '\''
        ) {
            punctuation += 1;
        }
    }

    if total == 0 {
        0.0
    } else {
        punctuation as f32 / total as f32
    }
}

fn has_long_encoded_token(text: &str) -> bool {
    let mut run_len = 0;
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '+' | '/' | '_' | '-' | '=') {
            run_len += 1;
            if run_len >= 96 {
                return true;
            }
        } else {
            run_len = 0;
        }
    }
    false
}

/// Recursively collect text from nodes while filtering out script/style content
fn collect_text_filtered(
    node: &ego_tree::NodeRef<'_, scraper::node::Node>,
    text_fragments: &mut Vec<String>,
) {
    match node.value() {
        scraper::Node::Text(txt) => {
            let clean_text = txt.trim();
            if !is_non_content_text(clean_text) {
                text_fragments.push(clean_text.to_string());
            }
        }
        scraper::Node::Element(elem) => {
            // Skip script, noscript, and style elements entirely
            if !should_skip_element(elem) {
                // Process children only if this isn't a filtered element
                for child in node.children() {
                    collect_text_filtered(&child, text_fragments);
                }
            }
        }
        _ => {
            // For other node types, process children
            for child in node.children() {
                collect_text_filtered(&child, text_fragments);
            }
        }
    }
}

/// HTML void elements: self-closing form, never get an end tag.
///
/// Only used by the markdown path; gated behind the `markdown` feature.
#[cfg(feature = "markdown")]
const VOID_TAGS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta",
    "param", "source", "track", "wbr",
];

/// Serialize the inner HTML of `node`, omitting subtrees flagged by
/// [`should_skip_element`] as well as `<img>`/`<source>`/`<picture>` with
/// inline `data:` URIs and `<span>` elements carrying editor-bookmark
/// attributes (`data-mce-type`).
///
/// This is the markdown-path counterpart of [`collect_text_filtered`]:
/// instead of extracting text, it produces an HTML string with non-content
/// subtrees pruned, so a downstream HTML-to-markdown converter receives
/// only the article's real DOM (no script payloads, SVG blobs, or
/// editor artifacts).
#[cfg(feature = "markdown")]
pub(crate) fn filtered_inner_html(
    node: &ego_tree::NodeRef<'_, scraper::node::Node>,
) -> String {
    let mut out = String::new();
    for child in node.children() {
        serialize_node_filtered(&child, &mut out);
    }
    out
}

#[cfg(feature = "markdown")]
fn serialize_node_filtered(
    node: &ego_tree::NodeRef<'_, scraper::node::Node>,
    out: &mut String,
) {
    match node.value() {
        scraper::Node::Text(txt) => {
            escape_html_text(txt, out);
        }
        scraper::Node::Element(elem) => {
            if should_skip_element(elem)
                || is_editor_artifact_span(elem)
                || has_data_uri_media(elem)
            {
                return;
            }
            out.push('<');
            out.push_str(elem.name());
            for (name, value) in elem.attrs() {
                out.push(' ');
                out.push_str(name);
                out.push_str("=\"");
                escape_attr_value(value, out);
                out.push('"');
            }
            out.push('>');
            if VOID_TAGS.contains(&elem.name()) {
                return;
            }
            for child in node.children() {
                serialize_node_filtered(&child, out);
            }
            out.push_str("</");
            out.push_str(elem.name());
            out.push('>');
        }
        _ => {
            for child in node.children() {
                serialize_node_filtered(&child, out);
            }
        }
    }
}

/// Returns true if `elem` is a `<span data-mce-type="...">` editor bookmark,
/// which carries no user-visible content.
#[cfg(feature = "markdown")]
fn is_editor_artifact_span(elem: &scraper::node::Element) -> bool {
    elem.attr("data-mce-type").is_some()
}

/// Returns true if `elem` is an `<img>`/`<source>`/`<picture>` whose `src`
/// is a `data:` URI or whose `srcset` contains a `data:` entry.
#[cfg(feature = "markdown")]
fn has_data_uri_media(elem: &scraper::node::Element) -> bool {
    if !matches!(elem.name(), "img" | "source" | "picture") {
        return false;
    }
    if let Some(src) = elem.attr("src")
        && src.trim_start().to_ascii_lowercase().starts_with("data:")
    {
        return true;
    }
    if let Some(srcset) = elem.attr("srcset")
        && srcset.contains("data:")
    {
        return true;
    }
    false
}

#[cfg(feature = "markdown")]
fn escape_html_text(s: &str, out: &mut String) {
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(c),
        }
    }
}

#[cfg(feature = "markdown")]
fn escape_attr_value(s: &str, out: &mut String) {
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
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
        if let Some(elem) = node.value().as_element()
            && let Some(link) = elem.attr("href")
        {
            links.push(link.trim().to_string());
        };
    }
    Ok(links)
}

#[cfg(test)]
pub(crate) fn build_dom(html: &str) -> Html {
    let document: Html = Html::parse_document(html);
    document
}

#[cfg(test)]
pub(crate) fn read_file(
    file_path: impl AsRef<path::Path>,
) -> Result<String, io::Error> {
    let content: String = fs::read_to_string(file_path)?;
    Ok(content)
}

#[cfg(test)]
pub(crate) fn build_dom_from_file(test_file_name: &str) -> Html {
    let content = read_file(format!("html/{}", test_file_name)).unwrap();
    build_dom(content.as_str())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::tree::BODY_SELECTOR;

    const TEST_1_HTML: &str = include_str!("../html/test_1.html");
    const TEST_2_HTML: &str = include_str!("../html/test_2.html");

    #[test]
    fn test_body_selector() {
        // let content = read_file("html/test_1.html").unwrap();
        // let document = build_dom(content.as_str());
        let document = build_dom(TEST_1_HTML);

        // This will force initialization and use of BODY_SELECTOR
        let body_elements: Vec<_> = document.select(&BODY_SELECTOR).collect();
        assert_eq!(body_elements.len(), 1); // Should find exactly one body tag
    }

    #[test]
    fn test_load_file() {
        let content = read_file("html/test_1.html");
        assert!(content.is_ok());
        assert!(!content.unwrap().is_empty());
    }

    #[test]
    fn test_build_dom() {
        let document = build_dom(TEST_2_HTML);
        assert!(document.errors.len() == 1);
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
    fn test_get_node_text_filters_unwrapped_script_fragments() {
        let document = build_dom(
            r#"
            <html><body><article>
                <p>Main article text remains.</p>
                <span>
                    window.yaContextCb.push(function () {
                        Ya.adfoxCode.createAdaptive({
                            ownerId: 173858,
                            containerId: 'adfox_151179074300466320'
                        });
                    });
                </span>
            </article></body></html>
            "#,
        );

        let article = document
            .select(&scraper::Selector::parse("article").unwrap())
            .next()
            .unwrap();
        let text = get_node_text(article.id(), &document).unwrap();

        assert!(text.contains("Main article text remains."));
        assert!(!text.contains("window.yaContextCb"));
        assert!(!text.contains("Ya.adfoxCode"));
    }

    #[test]
    fn test_is_non_content_text_filters_machine_blobs() {
        assert!(is_non_content_text(
            ".xboost-cart-slide { background: rgba(0, 0, 0, 0.5); position: fixed; right: 0; top: 0; bottom: 0; left: 0; z-index: 2000; justify-content: flex-end; }"
        ));
        assert!(is_non_content_text(
            "window.yaContextCb.push(function () { Ya.adfoxCode.createAdaptive({ ownerId: 173858, containerId: 'adfox_151179074300466320', params: { p1: 'csljp', p2: 'hjrx' } }); });"
        ));
        assert!(is_non_content_text(
            "window.cartSettings = {\"data\":\"eyJnbG9iYWxfd2lkZ2V0X3RoZW1lX2NvbG9yIjoiI0U5NUM1QSIsImdsb2JhbF93aWRnZXRfc2Vjb25kYXJ5X2J1dHRvbl9iZ19ob3Zlcl9jb2xvciI6IiNmZmZmZmYiLCJzdGlja3lfY2FydF9pY29uX2NvbG9yIjoiI0ZBRjVGNSJ9\"};"
        ));
    }

    #[test]
    fn test_is_non_content_text_keeps_programming_prose() {
        assert!(!is_non_content_text(
            r#"{"@context":"https://schema.org","@type":"Organization"}"#
        ));
        assert!(!is_non_content_text(
            "This article mentions window dressing, documents, and functions in prose."
        ));
        assert!(!is_non_content_text(
            "Call document.querySelector() to select an element, then pass the result to a function."
        ));
        assert!(!is_non_content_text(
            "Function follows form in this design document."
        ));
    }

    #[test]
    fn test_get_node_text_skips_svg_and_hidden_content() {
        let document = build_dom(
            r#"
            <html><body><article>
                <p>Main article text remains.</p>
                <svg><title>Hidden icon title</title><text>SVG label</text></svg>
                <div hidden>Hidden text</div>
                <div style="display: none">Invisible text</div>
                <div class="sharedaddy">Share this: Facebook</div>
            </article></body></html>
            "#,
        );

        let article = document
            .select(&scraper::Selector::parse("article").unwrap())
            .next()
            .unwrap();
        let text = get_node_text(article.id(), &document).unwrap();

        assert!(text.contains("Main article text remains."));
        assert!(!text.contains("Hidden icon title"));
        assert!(!text.contains("SVG label"));
        assert!(!text.contains("Hidden text"));
        assert!(!text.contains("Invisible text"));
        assert!(!text.contains("Share this"));
    }

    #[test]
    fn test_get_node_text_skips_iframe_fallback_text() {
        let document = build_dom(
            r#"
            <html><body><article>
                <p>Visible text before.</p>
                <p>
                    <iframe loading="lazy" src="https://example.com/embed">
                        <span data-mce-type="bookmark" style="display:inline-block;width:0px;overflow:hidden;line-height:0" class="mce_SELRES_start">﻿</span>
                    </iframe>
                    Text after the iframe.
                </p>
                <p>A third paragraph.</p>
            </article></body></html>
            "#,
        );

        let article = document
            .select(&scraper::Selector::parse("article").unwrap())
            .next()
            .unwrap();
        let text = get_node_text(article.id(), &document).unwrap();

        assert!(text.contains("Visible text before"));
        assert!(text.contains("Text after the iframe"));
        assert!(text.contains("A third paragraph"));
        assert!(!text.contains("data-mce-type"));
        assert!(!text.contains("mce_SELRES"));
        assert!(!text.contains("<span"));
        assert!(!text.contains("</span"));
        assert!(!text.contains("display:inline-block"));
    }
}
