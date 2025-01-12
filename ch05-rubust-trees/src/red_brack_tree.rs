use std::{cell::RefCell, io::Write, mem, rc::Rc};

use crate::IoTDevice;

#[derive(Clone, Debug, PartialEq)]
enum Color {
    Red,
    Black,
}

#[derive(PartialEq)]
enum RBOperation {
    LeftNode,
    RightNode,
}

#[derive(PartialEq)]
enum Rotation {
    Left,
    Right,
}

#[derive(Debug)]
struct Node {
    pub color: Color,
    pub value: IoTDevice,
    pub parent: Option<Rc<RefCell<Node>>>,
    left: Option<Rc<RefCell<Node>>>,
    right: Option<Rc<RefCell<Node>>>,
}

impl Node {
    pub fn new(device: IoTDevice) -> Node {
        Node {
            color: Color::Red,
            value: device,
            parent: None,
            left: None,
            right: None,
        }
    }

    pub fn expect_left(&self) -> Rc<RefCell<Node>> {
        self.left.as_ref().unwrap().clone()
    }

    pub fn expect_right(&self) -> Rc<RefCell<Node>> {
        self.right.as_ref().unwrap().clone()
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

#[derive(PartialEq)]
pub struct DeviceRegistry {
    root: Option<Rc<RefCell<Node>>>,
    pub length: u64,
}

impl DeviceRegistry {
    /// ノードの挿入
    /// - 挿入フェーズ
    ///    - 追加するノードの色は赤
    /// - 修正フェース
    ///   - (自ノードがrootの場合)、ルールに合うように色を塗り替える
    ///   - (1) parentノードが赤の場合
    ///     - parent/grandparentノードを黒に変更
    ///     - uncleノードを黒に変更
    ///     - grandparentノードから再帰的にルールにしたがっているかチェック
    ///   - (2) uncleノードが黒 && 自ノードがparentの左
    ///     - grandparentノードを右回転
    ///     - 色変え
    ///   - (3) uncleノードが黒 && 自ノードがparentの右
    ///     - parent左回転
    ///     - (2)を適用
    pub fn insert(&mut self, device: IoTDevice) {
        self.length += 1;
        let root = mem::replace(&mut self.root, None);
        let (maybe_node, new_node) = self.insert_internal(root.clone(), device);
        // self.root = self.fix_tree(new_tree.1);
        self.root = maybe_node;
    }

    fn link_pair(parent: &Rc<RefCell<Node>>, child: &Rc<RefCell<Node>>, direction: RBOperation) {
        match direction {
            RBOperation::LeftNode => {
                parent.borrow_mut().left = Some(child.clone());
            }
            RBOperation::RightNode => {
                parent.borrow_mut().right = Some(child.clone());
            }
        };
        child.borrow_mut().parent = Some(parent.clone());
        println!(
            "linked: {:?} <=> {:?}",
            &parent.borrow().value,
            &child.borrow().value
        );
    }

    fn decide_direction(&self, a: &IoTDevice, b: &IoTDevice) -> RBOperation {
        if a <= b {
            RBOperation::RightNode
        } else {
            RBOperation::LeftNode
        }
    }

    fn insert_internal(
        &mut self,
        maybe_root: Option<Rc<RefCell<Node>>>,
        device: IoTDevice,
    ) -> (Option<Rc<RefCell<Node>>>, Rc<RefCell<Node>>) {
        let (maybe_node, new_node) = self.insert_rec(maybe_root.clone(), device);
        // println!("{:?} {:?}", &maybe_node, &new_node);
        (maybe_node, new_node.clone())
    }

    fn insert_rec(
        &mut self,
        mut maybe_current_node: Option<Rc<RefCell<Node>>>,
        device: IoTDevice,
    ) -> (Option<Rc<RefCell<Node>>>, Rc<RefCell<Node>>) {
        match maybe_current_node.take() {
            None => {
                // 葉に到達したので、新しいノードを追加
                let new_node = Rc::new(RefCell::new(Node::new(device)));
                (Some(new_node.clone()), new_node)
            }
            Some(current_node) => {
                let new: Rc<RefCell<Node>>;
                let current_device = current_node.borrow().value.clone();

                match self.decide_direction(&current_device, &device) {
                    RBOperation::LeftNode => {
                        let left = current_node.borrow().left.clone();
                        let (maybe_new_tree, new_node) = self.insert_rec(left, device);
                        new = new_node.clone();

                        Self::link_pair(&current_node, &new_node, RBOperation::LeftNode);
                    }
                    RBOperation::RightNode => {
                        let right = current_node.borrow().right.clone();
                        let (maybe_new_tree, new_node) = self.insert_rec(right, device);
                        new = new_node.clone();

                        Self::link_pair(&current_node, &new_node, RBOperation::RightNode);
                    }
                }

                (Some(current_node), new)
            }
        }
    }

    fn fix_tree(&mut self, inserted: Rc<RefCell<Node>>) -> Option<Rc<RefCell<Node>>> {
        let mut not_root = inserted.borrow().parent.is_some();

        let root = if not_root {
            let mut parent_is_red = self.parent_color(&inserted) == Color::Red;
            let mut inserted_node = inserted.clone();
            while parent_is_red && not_root {
                let grand_parent = inserted_node.borrow().parent.as_ref().unwrap().clone();
                let Some((maybe_uncle, which)) = self.uncle(inserted_node.clone()) else {
                    break;
                };
                match which {
                    RBOperation::LeftNode => {
                        // uncle is on the left
                        let mut parent = self.expect_parent(&inserted_node);

                        match maybe_uncle {
                            Some(ref uncle) if uncle.borrow().color == Color::Red => {
                                parent.borrow_mut().color = Color::Black;
                                uncle.borrow_mut().color = Color::Black;
                                grand_parent.borrow_mut().color = Color::Red;
                                inserted_node = grand_parent;
                            }
                            Some(_) | None => {
                                if self.decide_direction(
                                    &parent.borrow().value,
                                    &inserted_node.borrow().value,
                                ) == RBOperation::LeftNode
                                {
                                    let tmp = self.expect_parent(&inserted_node);
                                    inserted_node = tmp;
                                    self.rotate(inserted_node.clone(), Rotation::Right);
                                    parent = self.expect_parent(&inserted_node);
                                }

                                parent.borrow_mut().color = Color::Black;
                                parent
                                    .borrow()
                                    .parent
                                    .as_ref()
                                    .unwrap()
                                    .clone()
                                    .borrow_mut()
                                    .color = Color::Red;
                                self.rotate(self.expect_parent(&parent), Rotation::Left);
                            }
                        }
                    }
                    RBOperation::RightNode => {
                        // uncle is on the right
                        let mut parent = self.expect_parent(&inserted_node);

                        match maybe_uncle {
                            Some(ref uncle) if uncle.borrow().color == Color::Red => {
                                parent.borrow_mut().color = Color::Black;
                                uncle.borrow_mut().color = Color::Black;
                                grand_parent.borrow_mut().color = Color::Red;
                                inserted_node = grand_parent;
                            }
                            Some(_) | None => {
                                if self.decide_direction(
                                    &parent.borrow().value,
                                    &inserted_node.borrow().value,
                                ) == RBOperation::LeftNode
                                {
                                    let tmp = self.expect_parent(&inserted_node);
                                    inserted_node = tmp;
                                    self.rotate(inserted_node.clone(), Rotation::Left);
                                    parent = self.expect_parent(&inserted_node);
                                }

                                parent.borrow_mut().color = Color::Black;
                                parent
                                    .borrow()
                                    .parent
                                    .as_ref()
                                    .unwrap()
                                    .clone()
                                    .borrow_mut()
                                    .color = Color::Red;
                                self.rotate(self.expect_parent(&parent), Rotation::Right);
                            }
                        }
                    }
                }

                not_root = inserted_node.borrow().parent.is_some();
                if not_root {
                    parent_is_red = self.parent_color(&inserted_node) == Color::Red;
                }
            }
            while inserted_node.borrow().parent.is_some() {
                inserted_node = self.expect_parent(&inserted_node);
            }
            Some(inserted_node)
        } else {
            Some(inserted)
        };
        root.map(|node| {
            node.borrow_mut().color = Color::Black;
            node
        })
    }

    fn rotate(&self, node: Rc<RefCell<Node>>, direction: Rotation) {
        match direction {
            Rotation::Left => self.rotate_left(node),
            Rotation::Right => self.rotate_right(node),
        }
    }

    fn rotate_right(&self, n: Rc<RefCell<Node>>) {
        let maybe_l = n.borrow().left.clone();
        // (1) 自ノードの左子ノード = 自ノードの元々の左子ノードの右子ノード
        n.borrow_mut().left = maybe_l
            .as_ref()
            .and_then(|child| child.borrow().right.clone());

        if maybe_l.is_some() {
            let left_child = n.borrow().expect_left();
            // (2) 左子ノードの親ノード = 自ノードの親ノード
            left_child.borrow_mut().parent = n.borrow().parent.clone();
            if left_child.borrow().right.is_some() {
                // (3) 左子ノードの右ノードの親ノード = 自ノード
                left_child.borrow().expect_right().borrow_mut().parent = Some(n.clone());
            }
        }

        match n.borrow().parent.as_ref() {
            // (4) 親ノードの子ノード = 左子ノード
            Some(p) => {
                let insert_direction = self.decide_direction(&p.borrow().value, &n.borrow().value);
                match insert_direction {
                    RBOperation::LeftNode => {
                        p.borrow_mut().right = maybe_l.clone();
                    }
                    RBOperation::RightNode => {
                        p.borrow_mut().left = maybe_l.clone();
                    }
                }
            }
            // (例外) 左子ノードの親ノード = None (左子ノードがrootになる場合)
            None => {
                maybe_l.as_ref().unwrap().borrow_mut().parent = None;
            }
        }
        // (5) 左子ノードの右子ノード = 自ノード
        // (6) 自ノードの親ノード = 左子ノード
        maybe_l.as_ref().unwrap().borrow_mut().right = Some(n.clone());
        n.borrow_mut().parent = maybe_l;
    }

    fn rotate_left(&self, node: Rc<RefCell<Node>>) {
        todo!();
    }

    fn parent_color(&self, node: &Rc<RefCell<Node>>) -> Color {
        node.borrow()
            .parent
            .as_ref()
            .expect("should have parent node")
            .borrow()
            .color
            .clone()
    }

    fn expect_parent(&self, node: &Rc<RefCell<Node>>) -> Rc<RefCell<Node>> {
        node.borrow().parent.as_ref().unwrap().clone()
    }

    fn grand_parent(&self, node: Rc<RefCell<Node>>) -> Option<Rc<RefCell<Node>>> {
        node.borrow().parent.as_ref()?.borrow().parent.clone()
    }

    fn uncle(&self, node: Rc<RefCell<Node>>) -> Option<(Option<Rc<RefCell<Node>>>, RBOperation)> {
        let parent = (&node.borrow().parent).clone()?;
        let grand_parent = (&parent.borrow().parent).clone()?;
        let uncle_and_which =
            match self.decide_direction(&grand_parent.borrow().value, &parent.borrow().value) {
                RBOperation::LeftNode => {
                    let uncle = grand_parent.borrow().right.clone();
                    Some((uncle, RBOperation::RightNode))
                }
                RBOperation::RightNode => {
                    let uncle = grand_parent.borrow().left.clone();
                    Some((uncle, RBOperation::LeftNode))
                }
            };
        uncle_and_which
    }

    pub fn find(&self, value: u64) -> Option<IoTDevice> {
        todo!();
    }

    pub fn find_rec(&self) {
        todo!();
    }

    pub fn walk(&self, mut callback: impl FnMut(&IoTDevice, usize)) {
        self.root.as_ref().map(|root| {
            self.walk_rec(root.clone(), callback, 0);
        });
    }

    fn walk_rec(
        &self,
        node: Rc<RefCell<Node>>,
        mut callback: impl FnMut(&IoTDevice, usize),
        level: usize,
    ) {
        let left = node.borrow().left.clone();
        let right = node.borrow().right.clone();
        left.map(|l| callback(&l.borrow().value, level + 1));
        callback(&node.clone().borrow().value, level);
        right.map(|r| callback(&r.borrow().value, level + 1));
    }
}

impl std::fmt::Display for DeviceRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.walk(|device, level| {
            let indent = "  ".repeat(level);
            writeln!(f, "{}{:?}", indent, device.numeriacl_id).unwrap();
        });
        Ok(())
    }
}

impl Default for DeviceRegistry {
    fn default() -> Self {
        DeviceRegistry {
            root: None,
            length: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use crate::IoTDevice;

    use super::{DeviceRegistry, Node};

    fn device(id: u64) -> IoTDevice {
        IoTDevice::new(id, "", "")
    }

    fn node(device: IoTDevice) -> Rc<RefCell<Node>> {
        Rc::new(RefCell::new(Node::new(device)))
    }

    #[test]
    fn test_frist_node() {
        let mut registry = DeviceRegistry::default();
        registry.insert(IoTDevice::new(5, "", ""));

        assert_eq!(registry.length, 1);
        assert_eq!(registry.root, Some(node(IoTDevice::new(5, "", ""))));
    }

    #[test]
    fn when_second_node_value_is_larger_then_root_node_should_have_right_child() {
        let mut registry = DeviceRegistry::default();
        registry.insert(device(5));
        registry.insert(device(6));
        registry.insert(device(4));

        assert_eq!(registry.length, 3);

        assert_eq!(format!("{}", registry), "  4\n5\n  6\n");
    }

    #[test]
    fn tree_should_be_balanced() {
        let mut registry = DeviceRegistry::default();
        registry.insert(device(1));
        registry.insert(device(2));
        registry.insert(device(3));

        assert_eq!(registry.length, 3);
        assert_eq!(format!("{}", registry), "  1\n2\n  3\n");
    }
}
