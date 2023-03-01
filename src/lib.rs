use ego_tree::NodeId;
use scraper::{element_ref::ElementRef, node::Node, Html, Selector};
use std::{fs, path, rc::Rc};
use thiserror::Error;

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

struct DCNode {
    node_id: NodeId,
    char_count: u32,
    tag_count: u32,
}

pub fn compute_density<'a>(document: &'a Html) -> Vec<DCNode> {
    let body_selector = Selector::parse("body").unwrap();
    let body = &document.select(&body_selector).next().unwrap().to_owned();
    println!("Body tag: {:?}", body.value());

    // println!("Body all: {:?}", body.children)
    println!("Body childs: {:?}", body.children().count());
    for ch in &mut body.children() {
        println!("Child: {:?}", ch.id());
    }
    42
}

// pub fn compute_density(tree: ego_tree::Tree<Node>) -> u32 {
//     for node in tree.root().children() {
//         if node.value().is_element() {
//             println!("Node: {:?}", node.value().as_element().unwrap().name());
//             if node.value().as_element().unwrap().name() == "html" {
//                 println!("Yes!");
//             }
//         }
//     }
//     42
// }

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
    fn test_extract_body() {
        let content = read_file("html/test_1.html").unwrap();
        let document = build_dom(content.as_str());

        let result = extract_body(&document);

        assert_eq!(result, 41);
    }

    // #[test]
    // fn test_compute_density() {
    //     let content = read_file("html/sas-bankruptcy-protection.html").unwrap();
    //     let document = build_dom(content.as_str());
    //     let density = compute_density(document.tree);
    //     assert_eq!(density, 41);
    // }
}
