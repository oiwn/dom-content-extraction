use ego_tree;
use scraper;

mod density_tree;
mod utils;

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

fn prepend_dash_n_times(n: usize) -> String {
    let dashes = std::iter::repeat("-").take(n).collect::<String>();
    format!("{}", dashes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils;

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
        let content =
            utils::read_file("html/sas-bankruptcy-protection.html").unwrap();
        let document = utils::build_dom(content.as_str());
        assert_eq!(document.errors.len() == 0, true);
    }

    #[test]
    fn test_build_density_tree() {
        // let content = utils::read_file("html/sas-bankruptcy-protection.html").unwrap();
        let content = utils::read_file("html/test_1.html").unwrap();
        let document = utils::build_dom(content.as_str());

        // let body_selector = Selector::parse("body").unwrap();
        // let body = &document.select(&body_selector).next().unwrap().to_owned();

        // let node_id = body.id();
        // let node = body.tree().get(node_id).unwrap();

        // let mut density_tree = density_tree::DensityTree::new(body.id());

        // build_density_tree(node, &mut density_tree.tree.root_mut(), 1);
        let dtree = density_tree::DensityTree::from_document(&document);

        // calculate_density_tree(&mut density_tree);
        // dtree.pretty_print();
        // let results = top_results(dtree);
        let sorted_nodes = dtree.sorted_nodes();
        println!("R: {:?}", sorted_nodes);
        let node_id = sorted_nodes.last().unwrap().node_id;
        let node = get_node_by_id(node_id, &document);
        // let node = body.tree().get(node_id).unwrap();
        println!("Result node: {:?}", node.value());
        println!("Result text: {}", get_node_text(node_id, &document))
    }
}
