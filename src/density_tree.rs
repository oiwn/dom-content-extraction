#![allow(dead_code)]
use ego_tree::{NodeId, NodeRef, Tree};
use scraper::{Html, Selector};

/// Prevend division by zero
#[inline]
fn normalize_denominator(value: u32) -> f32 {
    match value {
        0 => 1.0,
        _ => value as f32,
    }
}

pub struct DensityTree {
    pub tree: Tree<DensityNode>,
}

#[derive(Debug, Clone)]
pub struct DensityNode {
    pub node_id: NodeId,

    pub char_count: u32,
    pub tag_count: u32,
    pub link_char_count: u32,
    pub link_tag_count: u32,
    pub density: f32,
}

impl<'a> DensityTree {
    pub fn new(node_id: NodeId) -> Self {
        Self {
            tree: Tree::new(DensityNode::new(node_id)),
        }
    }

    pub fn from_document(document: &Html) -> Self {
        // TODO: wrap it on once_cell
        let body_selector = Selector::parse("body").unwrap();
        // TODO: process possible errors (when page is completely broken)
        let body = &document.select(&body_selector).next().unwrap().to_owned();
        // NOTE: there is usable value in document, such as error field
        let body_node_id = body.id();
        let body_node = body.tree().get(body_node_id).unwrap();

        let mut density_tree = Self::new(body_node_id);
        Self::build_density_tree(body_node, &mut density_tree.tree.root_mut(), 1);
        density_tree.calculate_density_tree();
        density_tree
    }

    pub fn sorted_nodes(&'a self) -> Vec<&'a DensityNode> {
        let mut nodes = self
            .tree
            .values()
            .filter(|n| n.density.gt(&0.0))
            .collect::<Vec<&DensityNode>>();
        nodes.sort_by(|a, b| {
            a.density
                .partial_cmp(&b.density)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        nodes
    }

    pub fn composite_text_density(
        char_count: u32,
        tag_count: u32,
        link_char_count: u32,
        link_tag_count: u32,
        body_tag_node: DensityNode,
    ) -> f32 {
        if char_count == 0 {
            // can guess whole expression will be zero
            return 0.0;
        }
        let ci = char_count as f32;
        let ti = normalize_denominator(tag_count);
        let nlci = normalize_denominator(char_count - link_char_count);
        let lci = normalize_denominator(link_char_count);
        let cb = normalize_denominator(body_tag_node.char_count);
        let lcb = body_tag_node.link_char_count as f32;
        let lti = normalize_denominator(link_tag_count);

        // checks
        assert_eq!(nlci > 0.0, true);

        let density = ci / ti;

        let ln_1 = (ci / nlci) * lci;
        let ln_2 = (lcb / cb) * ci;
        let e = std::f32::consts::E;

        assert_eq!(ln_1 >= 0.0, true);
        assert_eq!(ln_2 >= 0.0, true);

        let log_base = (ln_1 + ln_2 + e).ln();

        let value = (ci / lcb) * (ti / lti);
        let result = value.log(log_base) * density;

        result
    }

    pub fn calculate_density_tree(&mut self) {
        let body_tag_node = self.tree.root().value().clone();
        for node in self.tree.values_mut() {
            node.density = Self::composite_text_density(
                node.char_count,
                node.tag_count,
                node.link_char_count,
                node.link_tag_count,
                body_tag_node.clone(),
            );
        }
    }

    pub fn build_density_tree(
        node: ego_tree::NodeRef<scraper::node::Node>,
        density_node: &mut ego_tree::NodeMut<DensityNode>,
        depth: usize,
    ) {
        for child in node.children() {
            // some nodes makes no sense
            match child.value() {
                scraper::Node::Element(elem) => {
                    if elem.name() == "script"
                        || elem.name() == "noscript"
                        || elem.name() == "style"
                    {
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

            let child_density_node = DensityNode::new(child.id());
            let mut te = density_node.append(child_density_node);
            Self::build_density_tree(child, &mut te, depth + 1);
        }

        match node.value() {
            scraper::Node::Text(text) => {
                let char_count = text.trim().len() as u32;
                density_node.value().char_count += char_count;
            }
            scraper::Node::Element(elem) => {
                let tag_count = 1;
                density_node.value().tag_count += tag_count;
                if elem.name() == "a" {
                    let link_tag_count = 1;
                    density_node.value().link_tag_count += link_tag_count;
                }
            }
            _ => {}
        }

        let char_count = density_node.value().char_count;
        let tag_count = density_node.value().tag_count;
        let link_tag_count = density_node.value().link_tag_count;
        let mut link_char_count = density_node.value().link_char_count;

        if tag_count > 0 {
            density_node.value().density = density_node.value().char_count as f32
                / density_node.value().tag_count as f32;
        }

        if node.parent().unwrap().value().as_element().unwrap().name() == "a" {
            link_char_count += char_count;
        }

        if let Some(mut parent) = density_node.parent() {
            parent.value().char_count += char_count;
            parent.value().tag_count += tag_count;
            parent.value().link_tag_count += link_tag_count;
            parent.value().link_char_count += link_char_count;
        }
    }
}

impl std::fmt::Debug for DensityTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn pretty_print(
            f: &mut std::fmt::Formatter<'_>,
            node: NodeRef<DensityNode>,
            depth: usize,
        ) {
            for child in node.children() {
                let dashes =
                    std::iter::repeat(" ").take(2 * depth).collect::<String>();
                let _ = writeln!(f, "{}{:?}", dashes, child.value());
                pretty_print(f, child, depth + 1);
            }
        }

        writeln!(f, "DensityTree {{")?;
        pretty_print(f, self.tree.root(), 1);
        writeln!(f, "}}")
    }
}

impl DensityNode {
    pub fn new(node_id: NodeId) -> Self {
        Self {
            node_id,
            char_count: 0,
            tag_count: 0,
            link_char_count: 0,
            link_tag_count: 0,
            density: 0.0,
        }
    }
}
