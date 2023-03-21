use ego_tree;
use scraper;

mod density_tree;
mod utils;

fn prepend_dash_n_times(n: usize) -> String {
    let dashes = std::iter::repeat("-").take(n).collect::<String>();
    format!("{}", dashes)
}

pub fn composite_text_density(
    char_count: u32,
    tag_count: u32,
    link_char_count: u32,
    link_tag_count: u32,
    body_tag_node: density_tree::DensityNode,
) -> f32 {
    let ci = char_count as f32;
    let ti = {
        if tag_count == 0 {
            1
        } else {
            tag_count
        }
    } as f32;
    let nlci = {
        let tmp = char_count - link_char_count;
        if tmp <= 0 {
            1
        } else {
            tmp
        }
    } as f32;
    let lci = {
        if link_char_count == 0 {
            1
        } else {
            link_char_count
        }
    } as f32;
    let cb = {
        if body_tag_node.char_count == 0 {
            1
        } else {
            body_tag_node.char_count
        }
    } as f32;
    let lcb = body_tag_node.link_char_count as f32;
    let lti = {
        if link_tag_count == 0 {
            1
        } else {
            link_tag_count
        }
    } as f32;

    let density = ci / ti;

    let ln_1 = (ci / nlci) * lci;
    let ln_2 = (lcb / cb) * ci;
    let e = std::f32::consts::E;

    let log_base = (ln_1 + ln_2 + e).ln();
    let value = (ci / lci) * (ti / lti);
    println!("log_base {}", log_base);
    let result = value.log(log_base) * density;

    result
}

pub fn calculate_density_tree(
    density_tree: &mut density_tree::DensityTree,
    // body_node: ego_tree::NodeRef<density_tree::DensityNode>,
) {
    let body_tag_node = density_tree.tree.root().value().clone();
    for node in density_tree.tree.values_mut() {
        node.density = composite_text_density(
            node.char_count,
            node.tag_count,
            node.link_char_count,
            node.link_tag_count,
            body_tag_node.clone(),
        );
        println!("node: {:?}", node);
    }
    println!("body_node: {:?}", body_tag_node);
}

pub fn build_density_tree(
    node: ego_tree::NodeRef<scraper::node::Node>,
    density_node: &mut ego_tree::NodeMut<density_tree::DensityNode>,
    depth: usize,
) {
    for child in node.children() {
        // skip some nodes
        match child.value() {
            scraper::Node::Element(elem) => {
                if elem.name() == "script" || elem.name() == "noscript" {
                    continue;
                }
            }
            scraper::Node::Comment(_) => {
                continue;
            }
            scraper::Node::Document => {
                continue;
            }
            _ => {}
        };
        let _dashes = prepend_dash_n_times(depth);
        let child_density_node = density_tree::DensityNode::new(child.id());
        let mut te = density_node.append(child_density_node);
        build_density_tree(child, &mut te, depth + 1);
    }

    println!("[{:?}] Node: {:?}", node.id(), node.value());

    let mut char_count = 0;
    let mut tag_count = 0;
    let mut link_tag_count = 0;
    let mut link_char_count = 0;

    match node.value() {
        scraper::Node::Text(text) => {
            char_count = text.trim().len() as u32;
            density_node.value().char_count += char_count;
        }
        scraper::Node::Element(elem) => {
            tag_count = 1;
            density_node.value().tag_count += tag_count;
            if elem.name() == "a" {
                link_tag_count = 1;
                density_node.value().link_tag_count += link_tag_count;
            }
        }
        _ => {}
    }

    char_count = density_node.value().char_count;
    tag_count = density_node.value().tag_count;
    link_tag_count = density_node.value().link_tag_count;
    link_char_count = density_node.value().link_char_count;

    if tag_count > 0 {
        density_node.value().density =
            density_node.value().char_count as f32 / density_node.value().tag_count as f32;
    }

    println!("Density node: {:?}", density_node.value());

    // println!(
    //     "char_count: {} tag_count: {} link_tag_count: {} link_char_count: {}",
    //     char_count, tag_count, link_tag_count, link_char_count
    // );
    if node.parent().unwrap().value().as_element().unwrap().name() == "a" {
        link_char_count += char_count;
    }

    if let Some(mut parent) = density_node.parent() {
        parent.value().char_count += char_count;
        parent.value().tag_count += tag_count;
        parent.value().link_tag_count += link_tag_count;
        parent.value().link_char_count += link_char_count;

        println!("Have parent! {:?}", parent.value());
    }

    let dashes = std::iter::repeat("-").take(depth).collect::<String>();
    println!("{}", dashes);
}

pub fn calculate_nodes(
    node: ego_tree::NodeRef<scraper::node::Node>,
    density_node: &mut ego_tree::NodeMut<density_tree::DensityNode>,
    depth: usize,
) {
    for child in density_node.tree().values_mut().rev() {
        let current_node = node.tree().get(child.node_id).unwrap();

        if let Some(parent_node) = current_node.parent() {
            println!("Parent: {:?}", parent_node.value());
        }

        child.char_count = current_node.value().as_text().unwrap().len() as u32;
        println!("Child: {:?}", child);
    }

    println!("Node: {:?}, Depth: {}", node.value(), depth);
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
        let content = utils::read_file("html/sas-bankruptcy-protection.html").unwrap();
        // let content = utils::read_file("html/test_1.html").unwrap();
        let document = utils::build_dom(content.as_str());

        let body_selector = Selector::parse("body").unwrap();
        let body = &document.select(&body_selector).next().unwrap().to_owned();

        let node_id = body.id();
        let node = body.tree().get(node_id).unwrap();

        let mut density_tree = density_tree::DensityTree::new(body.id());

        build_density_tree(node, &mut density_tree.tree.root_mut(), 1);

        calculate_density_tree(&mut density_tree);
        // density_tree.pretty_print();
        // calculate_nodes(node, &mut density_tree.tree.root_mut(), 1)
    }
}
