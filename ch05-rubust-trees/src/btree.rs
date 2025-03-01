use crate::iot::IoTDevice;

type Tree = Box<Node>;

pub enum NodeType {
    Leaf,
    Regular,
}
pub struct Node {
    devices: Vec<Option<IoTDevice>>,
    children: Vec<Option<Tree>>,
    left_child: Option<Tree>,
    pub node_type: NodeType,
}

impl Node {
    pub fn new_leaf() -> Tree {
        Node::new(NodeType::Leaf)
    }

    pub fn new_regular() -> Tree {
        Node::new(NodeType::Regular)
    }

    fn new(node_type: NodeType) -> Tree {
        Box::new(Node {
            devices: vec![],
            children: vec![],
            left_child: None,
            node_type,
        })
    }

    pub fn len(&self) -> usize {
        self.devices.len()
    }

    pub fn is_empty(&self) -> bool {
        self.devices.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_leaf() {
        let leaf = Node::new_leaf();
        assert_eq!(leaf.len(), 0);
        assert!(leaf.is_empty());
    }

    #[test]
    fn test_new_regular() {
        let regular = Node::new_regular();
        assert_eq!(regular.len(), 0);
        assert!(regular.is_empty());
    }
}
