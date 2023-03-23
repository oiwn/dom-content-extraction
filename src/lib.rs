use ego_tree;
use scraper;

mod density_tree;

pub fn get_node_by_id<'a>(
    node_id: ego_tree::NodeId,
    document: &'a scraper::Html,
) -> ego_tree::NodeRef<'a, scraper::node::Node> {
    document.tree.get(node_id).unwrap()
}

pub fn get_node_text(
    node_id: ego_tree::NodeId,
    document: &scraper::Html,
) -> String {
    let mut text: Vec<String> = vec![];
    let root_node = document.tree.get(node_id).unwrap();
    for node in root_node.descendants() {
        if let Some(txt) = node.value().as_text() {
            let clean_text = txt.trim();
            text.push(clean_text.to_string());
        }
    }
    text.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    use scraper::Html;
    use std::{fs, path};

    pub fn read_file(
        file_path: impl AsRef<path::Path>,
    ) -> Result<String, std::io::Error> {
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

        let result = density_tree::DensityTree::composite_text_density(
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
        let result_zero_char_count =
            density_tree::DensityTree::composite_text_density(
                0,
                tag_count,
                link_char_count,
                link_tag_count,
                body_tag_char_count,
                body_tag_link_char_count,
            );
        assert_eq!(result_zero_char_count, 0.0);

        let result_zero_tag_count =
            density_tree::DensityTree::composite_text_density(
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

        let dtree = density_tree::DensityTree::from_document(&document);
        assert_eq!(dtree.tree.values().count(), 52);
    }

    #[test]
    fn test_sorted_density_results() {
        let document = load_content("test_1.html");

        let dtree = density_tree::DensityTree::from_document(&document);
        let sorted_nodes = dtree.sorted_nodes();
        let node_id = sorted_nodes.last().unwrap().node_id;
        assert_eq!(format!("{:?}", node_id), "NodeId(22)");

        let node = get_node_by_id(node_id, &document);

        let node_attr = node.value().as_element().unwrap().attrs().last().unwrap();
        assert_eq!(node_attr.0, "class");
        assert_eq!(node_attr.1, "articleBody");
    }

    #[test]
    fn test_result_node_text() {
        let content = read_file("html/test_1.html").unwrap();
        let document = build_dom(content.as_str());

        let dtree = density_tree::DensityTree::from_document(&document);
        let sorted_nodes = dtree.sorted_nodes();
        let node_id = sorted_nodes.last().unwrap().node_id;
        assert_eq!(get_node_text(node_id, &document).len(), 200);
    }
}
