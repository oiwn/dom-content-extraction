use ego_tree::{NodeId, Tree};

#[derive(Debug)]
pub struct DCTree {
    tree: Tree<DCNode>,
}

impl DCTree {
    pub fn new(node_id: NodeId) -> Self {
        DCTree {
            tree: Tree::new(DCNode::new(node_id)),
        }
    }

    pub fn append(&mut self) {}
}

#[derive(Debug, Clone)]
pub struct DCNode {
    pub node_id: NodeId,

    pub char_count: u32,
    pub tag_count: u32,
    pub link_char_count: u32,
    pub link_tag_count: u32,
    pub density: f32,
}

impl DCNode {
    fn new(node_id: NodeId) -> Self {
        DCNode {
            node_id,
            char_count: 0,
            tag_count: 0,
            link_char_count: 0,
            link_tag_count: 0,
            density: 0.0,
        }
    }
}
