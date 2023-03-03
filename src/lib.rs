use ego_tree::NodeId;
use scraper::{
    element_ref::ElementRef,
    node::{Element, Node, Text},
    Html, Selector,
};
use std::{fs, path, rc::Rc};
use thiserror::Error;

mod tree;

#[derive(Error, Debug)]
pub enum DomContentError {
    #[error("Error reading file")]
    UnableToReadFile(#[from] std::io::Error),
}

pub fn read_file(file_path: impl AsRef<path::Path>) -> Result<String, DomContentError> {
    let content: String =
        fs::read_to_string(file_path).map_err(DomContentError::UnableToReadFile)?;
    Ok(content)
}

pub fn build_dom(html: &str) -> Html {
    let document: Html = Html::parse_document(html);
    document
}

pub fn build_density_tree(node: ego_tree::NodeRef<Node>, &mut side_tree: tree::DCTree) {
    if !node.has_children() {
        return;
    }
    for child in node.children() {
        if child.value().is_element() {
            println!("Recursive nodes: {:?}", child.value());
            let side_part = side_tree.append()
            build_density_tree(child, side_tree)
        }
    }
}

pub fn compute_density_old<'a>(document: &'a Html) -> u32 {
    let body_selector = Selector::parse("body").unwrap();
    let body = &document.select(&body_selector).next().unwrap().to_owned();

    fn node_text_len(n: ego_tree::NodeRef<Node>) -> u32 {
        let mut text_len: u32 = 0;
        for x in n.descendants() {
            if x.value().is_text() {
                text_len += x.value().as_text().unwrap().text.len32();
            }
        }
        text_len
    }

    fn href_text_len(n: ego_tree::NodeRef<Node>) -> u32 {
        let mut text_len: u32 = 0;
        for x in n.descendants() {
            if x.value().is_element() {
                if x.value().as_element().unwrap().name() == "a" {
                    text_len += node_text_len(x);
                }
            }
        }
        text_len
    }

    let mut nodes: Vec<tree::DCNode> = vec![];

    for node in body.descendants() {
        if node.value().is_element() {
            match node.value().as_element().unwrap().name() {
                "script" => continue,
                "noscript" => continue,
                "style" => continue,
                _ => {}
            }
            // println!("[{:?}] Node: {:?}", node.id(), node.value());

            let mut total_valuable_nodes: u32 = 0;
            let mut total_text_len: u32 = 0;
            let mut total_href_tags: u32 = 0;
            for desc in node.descendants() {
                match desc.value() {
                    Node::Element(element) => match element.name() {
                        "script" => {}
                        "noscript" => {}
                        "style" => {}
                        "comment" => {}
                        "a" => {
                            total_valuable_nodes += 1;
                            total_href_tags += 1;
                        }
                        _ => {
                            total_valuable_nodes += 1;
                        }
                    },
                    Node::Text(text) => {
                        total_text_len += text.trim().len() as u32;
                    }
                    _ => {}
                }
            }

            let descendant_nodes_count = node.descendants().count();
            let text_len = node_text_len(node);
            let density = total_text_len as f32 / total_valuable_nodes as f32;

            if density > 0.0 {
                println!("Node name: {:?}", node.value());
                println!("Node nodes: {:?}", node.descendants());
                println!("Node nodes: {}", descendant_nodes_count);
                println!("Node text len: {:?}", text_len);
                println!("Node valuable nodes: {}", total_valuable_nodes);
                println!("Node href tags: {}", total_href_tags);
                println!("Node total text len: {}", total_text_len);
                println!("Density: {}", density);
                println!("=========================");
                nodes.push(tree::DCNode {
                    node_id: node.id(),
                    char_count: text_len,
                    tag_count: descendant_nodes_count as u32,
                    link_char_count: 0,
                    link_tag_count: 0,
                    density,
                });
            }
        }
    }

    // fn calculate_node(node: DCNode, bd: &ElementRef, nodes: &mut Vec<DCNode>) {
    //     for subnode in bd.tree().get(node.node_id).unwrap().children() {
    //         println!("[{:?}]Subnode: {:?}", subnode.id(), subnode.value());
    //         match subnode.value() {
    //             Node::Text(text) => {}
    //             Node::Element(element) => {
    //                 // element.id
    //                 println!("Element: {:?}", element);
    //                 calculate_node(DCNode::new(subnode.id()), bd, nodes);
    //             }
    //             _ => {}
    //         }
    //         println!("Subnode: {:?}", subnode.value());
    //         // match subnode.value() {};
    //     }
    // }

    // calculate_node(dc_node, body, &mut nodes);

    42
}

#[cfg(test)]
mod tests {
    use super::*;

    // static default_page_content: Result<String, DomContentError> =
    //     { read_file("html/sas-bankruptcy-protection.html") };

    // fn get_test_content() -> Result<String, DomContentError> {
    //     let content = read_file("html/sas-bankruptcy-protection.html");
    //     content
    // }

    #[test]
    fn test_load_file() {
        let content = read_file("html/sas-bankruptcy-protection.html");
        assert_eq!(content.is_ok(), true);
        assert_eq!(content.unwrap().len() > 0, true);
    }

    #[test]
    fn test_build_dom() {
        let content = read_file("html/sas-bankruptcy-protection.html").unwrap();
        let document = build_dom(content.as_str());
        assert_eq!(document.errors.len() == 0, true);
    }

    #[test]
    fn test_build_density_tree() {
        let content = read_file("html/test_1.html").unwrap();
        let document = build_dom(content.as_str());

        let body_selector = Selector::parse("body").unwrap();
        let body = &document.select(&body_selector).next().unwrap().to_owned();

        let mut side_tree = tree::DCTree::new(body.id());
        let tree = build_density_tree(body.tree().get(body.id()).unwrap(), side_tree);
        println!("Tree: {:?}", tree);
    }

    // #[test]
    #[allow(dead_code)]
    fn test_extract_body() {
        let content = read_file("html/test_1.html").unwrap();
        // let content = read_file("html/sas-bankruptcy-protection.html").unwrap();
        let document = build_dom(content.as_str());

        let result = compute_density_old(&document);

        assert_eq!(result, 41);
    }
}
