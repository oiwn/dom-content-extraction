use ego_tree;
// use scraper::{element_ref::ElementRef, node, Html, Selector};
use scraper::node;

mod tree;
mod utils;

fn prepend_dash_n_times(n: usize) -> String {
    let dashes = std::iter::repeat("-").take(n).collect::<String>();
    format!("{}", dashes)
}

pub fn build_density_tree(
    node: ego_tree::NodeRef<node::Node>,
    density_node: &mut ego_tree::NodeMut<tree::DensityNode>,
    parent_id: ego_tree::NodeId,
    depth: usize,
) {
    // println!("DensityNode: {:?}", density_node);
    for child in node.children() {
        let _dashes = prepend_dash_n_times(depth);
        // println!("{} Child: {:#?}", dashes, child.value());
        let child_density_node = tree::DensityNode {
            node_id: child.id(),
            char_count: 0,
            tag_count: 0,
            link_char_count: 0,
            link_tag_count: 0,
            density: 0.0,
        };

        let mut te = density_node.append(child_density_node);
        build_density_tree(child, &mut te, child.id(), depth + 1);
    }

    let mut current_node = density_node.tree().get_mut(node.id()).unwrap();
    current_node.value().tag_count += 1;

    /*
    if let Some(element) = node.value().as_element() {
        if element.name() == "a" {
            current_node.link_tag_count += 1;
            current_node.link_char_count += current_node.char_count;
        }

        current_node.tag_count += 1;
    } else if let Some(text) = node.value().as_text() {
        current_node.char_count += text.text.len() as u32;
    }

    if current_node.tag_count > 0 {
        current_node.density = current_node.char_count as f32 / current_node.tag_count as f32;
    }
    */
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils;
    use scraper::Selector;

    // static default_page_content: Result<String, DomContentError> =
    //     { read_file("html/sas-bankruptcy-protection.html") };

    // fn get_test_content() -> Result<String, DomContentError> {
    //     let content = read_file("html/sas-bankruptcy-protection.html");
    //     content
    // }

    #[test]
    fn test_load_file() {
        let content = utils::read_file("html/sas-bankruptcy-protection.html");
        assert_eq!(content.is_ok(), true);
        assert_eq!(content.unwrap().len() > 0, true);
    }

    #[test]
    fn test_build_dom() {
        let content = utils::read_file("html/sas-bankruptcy-protection.html").unwrap();
        let document = utils::build_dom(content.as_str());
        assert_eq!(document.errors.len() == 0, true);
    }

    #[test]
    fn test_build_density_tree() {
        let content = utils::read_file("html/test_1.html").unwrap();
        let document = utils::build_dom(content.as_str());

        let body_selector = Selector::parse("body").unwrap();
        let body = &document.select(&body_selector).next().unwrap().to_owned();

        let node_id = body.id();
        let node = body.tree().get(node_id).unwrap();

        let mut density_tree = tree::DensityTree::new(body.id());

        build_density_tree(node, &mut density_tree.tree.root_mut(), node_id, 1);
        // println!("Tree: {:#?}", density_tree);
        density_tree.pretty_print();
        // println!("DensityTree: {:?}", density_tree);
    }
}
