use ego_tree::NodeId;
use scraper::{Html, Selector};
use std::sync::LazyLock;

/// Selector for <body> tag
pub static BODY_SELECTOR: LazyLock<Selector> = LazyLock::new(|| {
    Selector::parse("body").expect("Can't be (parsing body selector)")
});

#[derive(Debug, Clone, Default)]
pub struct NodeMetrics {
    pub char_count: u32,
    pub tag_count: u32,
    pub link_char_count: u32,
    pub link_tag_count: u32,
}

pub trait TreeBuilder {
    fn build_metrics(&self, node_id: NodeId) -> NodeMetrics;
    fn get_children(&self, node_id: NodeId) -> Vec<NodeId>;
    fn get_parent(&self, node_id: NodeId) -> Option<NodeId>;
}

pub struct HtmlTreeBuilder<'a> {
    document: &'a Html,
}

impl<'a> HtmlTreeBuilder<'a> {
    pub fn new(document: &'a Html) -> Self {
        Self { document }
    }
}

impl TreeBuilder for HtmlTreeBuilder<'_> {
    fn build_metrics(&self, node_id: NodeId) -> NodeMetrics {
        let node = self.document.tree.get(node_id).unwrap();

        let mut metrics = NodeMetrics {
            char_count: 0,
            tag_count: 0,
            link_char_count: 0,
            link_tag_count: 0,
        };

        match node.value() {
            scraper::Node::Text(text) => {
                metrics.char_count = text.trim().len() as u32;
            }
            scraper::Node::Element(elem) => {
                metrics.tag_count = 1;
                if elem.name() == "a"
                    || elem.name() == "button"
                    || elem.name() == "select"
                {
                    metrics.link_tag_count = 1;
                }
            }
            _ => {}
        }

        metrics
    }

    fn get_children(&self, node_id: NodeId) -> Vec<NodeId> {
        self.document
            .tree
            .get(node_id)
            .map(|node| {
                node.children()
                    .filter(|child| match child.value() {
                        scraper::Node::Element(elem) => {
                            !matches!(elem.name(), "script" | "noscript" | "style")
                        }
                        scraper::Node::Text(text) => !text.trim().is_empty(),
                        scraper::Node::Comment(_) | scraper::Node::Document => {
                            false
                        }
                        _ => true,
                    })
                    .map(|child| child.id())
                    .collect()
            })
            .unwrap_or_default()
    }

    fn get_parent(&self, node_id: NodeId) -> Option<NodeId> {
        self.document
            .tree
            .get(node_id)
            .and_then(|node| node.parent())
            .map(|parent| parent.id())
    }
}

impl NodeMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    // Add metrics from another (child) node
    pub fn combine(&mut self, other: &NodeMetrics) {
        self.char_count += other.char_count;
        self.tag_count += other.tag_count;
        self.link_char_count += other.link_char_count;
        self.link_tag_count += other.link_tag_count;
    }

    // Calculate simple density (chars/tags ratio)
    pub fn calculate_simple_density(&self) -> f32 {
        if self.tag_count == 0 {
            0.0
        } else {
            self.char_count as f32 / self.tag_count as f32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scraper::Html;

    const TEST_HTML: &str = r#"
        <html>
        <body>
            <div class="content">
                Some text here
                <a href="\#">A link</a>
                <p>More content</p>
                <button>Click me</button>
                <script>console.log('skip');</script>
                <style>.skip{}</style>
            </div>
            <div class="sidebar">
                <select>
                    <option>Option 1</option>
                </select>
            </div>
        </body>
        </html>
    "#;

    #[test]
    fn test_body_selector_initialization() {
        // This will force the LazyLock to initialize
        let _ = &*BODY_SELECTOR;
    }

    #[test]
    fn test_node_metrics() {
        let document = Html::parse_document(TEST_HTML);
        let builder = HtmlTreeBuilder::new(&document);

        // Get content div
        let content_div = document
            .select(&scraper::Selector::parse("div.content").unwrap())
            .next()
            .unwrap();

        let metrics = builder.build_metrics(content_div.id());
        // notice that we do not count children nodes metrics
        assert_eq!(metrics.char_count, 0); // div itself has no direct text
        assert_eq!(metrics.tag_count, 1); // div counts as one tag
        assert_eq!(metrics.link_tag_count, 0); // div is not a link
    }

    #[test]
    fn test_get_children_filters() {
        let document = Html::parse_document(TEST_HTML);
        let builder = HtmlTreeBuilder::new(&document);

        let body = document
            .select(&scraper::Selector::parse("body").unwrap())
            .next()
            .unwrap();

        let children = builder.get_children(body.id());

        // Should get both divs but skip script and style
        assert_eq!(children.len(), 2);
    }

    #[test]
    fn test_link_metrics() {
        let document = Html::parse_document(TEST_HTML);
        let builder = HtmlTreeBuilder::new(&document);

        // Test link element
        let link = document
            .select(&scraper::Selector::parse("a").unwrap())
            .next()
            .unwrap();

        let metrics = builder.build_metrics(link.id());
        assert_eq!(metrics.link_tag_count, 1);

        // Test button
        let button = document
            .select(&scraper::Selector::parse("button").unwrap())
            .next()
            .unwrap();

        let metrics = builder.build_metrics(button.id());
        assert_eq!(metrics.link_tag_count, 1);

        // Test select
        let select = document
            .select(&scraper::Selector::parse("select").unwrap())
            .next()
            .unwrap();

        let metrics = builder.build_metrics(select.id());
        assert_eq!(metrics.link_tag_count, 1);
    }

    #[test]
    fn test_text_metrics() {
        let document = Html::parse_document(TEST_HTML);
        let builder = HtmlTreeBuilder::new(&document);

        // Find text node
        let text_node = document
            .select(&scraper::Selector::parse(".content").unwrap())
            .next()
            .unwrap()
            .first_child()
            .unwrap();

        let metrics = builder.build_metrics(text_node.id());
        assert_eq!(metrics.char_count, 14); // "Some text here"
        assert_eq!(metrics.tag_count, 0);
        assert_eq!(metrics.link_tag_count, 0);
    }

    #[test]
    fn test_get_parent() {
        let document = Html::parse_document(TEST_HTML);
        let builder = HtmlTreeBuilder::new(&document);

        let content_div = document
            .select(&scraper::Selector::parse("div.content").unwrap())
            .next()
            .unwrap();

        let parent = builder.get_parent(content_div.id());
        assert!(parent.is_some());

        // Body should be parent
        let body = document
            .select(&scraper::Selector::parse("body").unwrap())
            .next()
            .unwrap();

        assert_eq!(parent.unwrap(), body.id());
    }
}
