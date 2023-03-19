use ego_tree::{NodeId, NodeRef, Tree};

#[derive(Debug)]
pub struct DensityTree {
    pub tree: Tree<DensityNode>,
}

impl DensityTree {
    pub fn new(node_id: NodeId) -> Self {
        Self {
            tree: Tree::new(DensityNode::new(node_id)),
        }
    }

    fn recursive(&self, node: NodeRef<DensityNode>, depth: usize) {
        for child in node.children() {
            let dashes = std::iter::repeat("-").take(depth).collect::<String>();
            println!("{} child: {:#?}", dashes, child.value());
            self.recursive(child, depth + 1);
        }
    }

    pub fn pretty_print(&mut self) {
        self.recursive(self.tree.root().into(), 1);
    }
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
