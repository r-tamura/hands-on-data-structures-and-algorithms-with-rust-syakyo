use log::debug;
use std::{cell::RefCell, mem, rc::Rc};

use crate::IoTDevice;

#[derive(Clone, Debug, PartialEq)]
enum Color {
    Red,
    Black,
}

#[derive(PartialEq)]
enum RedBlackOp {
    LeftNode,
    RightNode,
}

#[derive(PartialEq)]
enum Rotation {
    Left,
    Right,
}

struct Node {
    pub color: Color,
    pub v: IoTDevice,
    pub parent: Option<Rc<RefCell<Node>>>,
    left: Option<Rc<RefCell<Node>>>,
    right: Option<Rc<RefCell<Node>>>,
}

impl Node {
    pub fn new(device: IoTDevice) -> Node {
        Node {
            color: Color::Red,
            v: device,
            parent: None,
            left: None,
            right: None,
        }
    }

    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }

    pub fn switch_color(&mut self, color: Color) {
        assert!(
            self.color != color,
            "color should be different error: current color {:?}, but got {:?}",
            self.color,
            color
        );
        self.color = color;
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.v == other.v
    }
}

impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, r#"{{"color": {:?}, "v": "{:?}"}}"#, self.color, self.v)
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
        let new_node = self.insert_internal(device);
        debug!("--- start balancing {:?}", new_node.borrow().v);
        self.root = self.balance(new_node.clone());
        debug!("--- end balancing {:?}", new_node.borrow().v);
    }

    fn pair(
        parent: Option<Rc<RefCell<Node>>>,
        child: Option<Rc<RefCell<Node>>>,
        direction: RedBlackOp,
    ) {
        match (parent, child) {
            (Some(parent), Some(child)) => {
                match direction {
                    RedBlackOp::LeftNode => {
                        parent.borrow_mut().left = Some(child.clone());
                        debug!(
                            "{:?}.left <- {:?}",
                            parent.borrow().v.numeriacl_id,
                            child.borrow().v.numeriacl_id,
                        );
                    }
                    RedBlackOp::RightNode => {
                        debug!(
                            "{:?}.right <- {:?}",
                            parent.borrow().v.numeriacl_id,
                            child.borrow().v.numeriacl_id,
                        );
                        parent.borrow_mut().right = Some(child.clone());
                    }
                };
                child.borrow_mut().parent = Some(parent.clone());
                debug!(
                    "parent: {:?} child: {:?}",
                    parent.borrow().v.numeriacl_id,
                    child.borrow().v.numeriacl_id
                );
            }
            (Some(parent), None) => {
                match direction {
                    RedBlackOp::LeftNode => {
                        parent.borrow_mut().left = None;
                    }
                    RedBlackOp::RightNode => {
                        parent.borrow_mut().right = None;
                    }
                };
            }
            (None, Some(child)) => {
                child.borrow_mut().parent = None;
            }
            _ => {}
        }
    }

    /// aの子供として挿入する場合、bが左/右どちらになるかを判定します
    /// RedBlackOp::LeftNode: bはaの左側の子供になります
    /// RedBlackOp::RightNode: bはaの右側の子供になります
    fn decide_direction(&self, a: &IoTDevice, b: &IoTDevice) -> RedBlackOp {
        if a <= b {
            RedBlackOp::RightNode
        } else {
            RedBlackOp::LeftNode
        }
    }

    fn insert_internal(&mut self, device: IoTDevice) -> Rc<RefCell<Node>> {
        self.length += 1;
        let maybe_root = mem::replace(&mut self.root, None);
        let (maybe_root, new_node) = self.insert_rec(maybe_root.clone(), device);
        debug!("new_root: {:?}, new_node: {:?}", &maybe_root, &new_node);
        self.root = maybe_root;
        new_node.clone()
    }

    fn insert_rec(
        &mut self,
        mut maybe_current_node: Option<Rc<RefCell<Node>>>,
        device: IoTDevice,
    ) -> (Option<Rc<RefCell<Node>>>, Rc<RefCell<Node>>) {
        match maybe_current_node.take() {
            None => {
                // 葉に到達したので、新しいノードを追加
                debug!("inserting new node {:?}", device);
                let new_node = Rc::new(RefCell::new(Node::new(device)));
                (Some(new_node.clone()), new_node)
            }
            Some(current_node) => {
                let new: Rc<RefCell<Node>>;
                let current_device = current_node.borrow().v.clone();
                debug!("--- current: {:?} new: {:?}", current_device, device);

                match self.decide_direction(&current_device, &device) {
                    RedBlackOp::LeftNode => {
                        debug!(
                            "go to left: {:?} > new: {:?}",
                            current_device.numeriacl_id, device.numeriacl_id
                        );
                        let left = current_node.borrow().left.clone();
                        let (maybe_new_tree, new_node) = self.insert_rec(left, device);
                        new = new_node.clone();

                        Self::pair(
                            Some(current_node.clone()),
                            maybe_new_tree,
                            RedBlackOp::LeftNode,
                        );
                    }
                    RedBlackOp::RightNode => {
                        debug!(
                            "go to right: current: {:?} <= new: {:?}",
                            current_device.numeriacl_id, device.numeriacl_id
                        );
                        let right = current_node.borrow().right.clone();
                        let (maybe_new_tree, new_node) = self.insert_rec(right, device);
                        new = new_node.clone();

                        Self::pair(
                            Some(current_node.clone()),
                            maybe_new_tree,
                            RedBlackOp::RightNode,
                        );
                    }
                }
                debug!(
                    "--- return current: {:?} new: {:?}",
                    current_device,
                    new.borrow().v,
                );

                (Some(current_node), new)
            }
        }
    }

    fn balance_single_node(
        &mut self,
        current: Rc<RefCell<Node>>,
        parent: Rc<RefCell<Node>>,
        maybe_uncle: Option<Rc<RefCell<Node>>>,
        uncle_direction: RedBlackOp,
        grand_parent: Rc<RefCell<Node>>,
    ) -> (Rc<RefCell<Node>>, Rc<RefCell<Node>>) {
        let (next_parent, next_current) = match maybe_uncle {
            Some(ref uncle) if uncle.borrow().color == Color::Red => {
                debug!("uncle is red");
                parent.borrow_mut().switch_color(Color::Black);
                uncle.borrow_mut().switch_color(Color::Black);
                grand_parent.borrow_mut().switch_color(Color::Red);
                (parent, grand_parent)
            }
            Some(_) | None => {
                debug!("uncle is black or None");

                let (next_parent, next_current) = if self
                    .decide_direction(&parent.borrow().v, &current.borrow().v)
                    == uncle_direction
                {
                    let tmp = self.parent_or_panic(&current);
                    let direction = match uncle_direction {
                        RedBlackOp::LeftNode => Rotation::Right,
                        RedBlackOp::RightNode => Rotation::Left,
                    };
                    self.rotate(tmp.clone(), direction);
                    (self.parent_or_panic(&tmp), tmp)
                } else {
                    (parent, current)
                };

                next_parent.borrow_mut().color = Color::Black;
                next_parent
                    .borrow()
                    .parent
                    .as_ref()
                    .unwrap()
                    .clone()
                    .borrow_mut()
                    .color = Color::Red;
                let direction = match uncle_direction {
                    RedBlackOp::LeftNode => Rotation::Left,
                    RedBlackOp::RightNode => Rotation::Right,
                };
                self.rotate(self.parent_or_panic(&next_parent), direction);
                (next_parent, next_current)
            }
        };
        (next_parent, next_current)
    }

    fn balance(&mut self, inserted: Rc<RefCell<Node>>) -> Option<Rc<RefCell<Node>>> {
        let mut current_is_not_root = !inserted.borrow().is_root();

        let root = if current_is_not_root {
            let mut parent_is_red = self.parent_or_panic(&inserted).borrow().color == Color::Red;
            let mut current = inserted.clone();
            debug!(
                "inserted node {:?} is not root, start balancing..",
                inserted.borrow().v
            );

            debug!("parent is {:?}", self.parent_or_panic(&inserted),);
            while parent_is_red && current_is_not_root {
                debug!("current: {:?}", current.borrow().v);
                let grand_parent = current.borrow().parent.as_ref().unwrap().clone();
                let Some((maybe_uncle, which)) = self.uncle(current.clone()) else {
                    debug!("current does not have grand parent");
                    break;
                };
                let parent = self.parent_or_panic(&current);
                match which {
                    //                 o  <- grand_parent
                    //                / \
                    //       uncle-> o   o <- parent
                    //                   |
                    //         current-> o
                    RedBlackOp::LeftNode => {
                        // uncle is on the left
                        debug!("uncle is left child");
                        let (_parent, next_current) = self.balance_single_node(
                            current.clone(),
                            parent.clone(),
                            maybe_uncle,
                            RedBlackOp::LeftNode,
                            grand_parent.clone(),
                        );
                        current = next_current;
                    }
                    //                 o  <- grand_parent
                    //                / \
                    //       parent-> o o <- uncle
                    //                |
                    //      current-> o
                    RedBlackOp::RightNode => {
                        // uncle is on the right
                        debug!("uncle is right child");
                        let (_parent, next_current) = self.balance_single_node(
                            current.clone(),
                            parent.clone(),
                            maybe_uncle,
                            RedBlackOp::RightNode,
                            grand_parent.clone(),
                        );
                        current = next_current;
                    }
                }

                current_is_not_root = !current.borrow().is_root();
                if current_is_not_root {
                    parent_is_red = self.parent_or_panic(&current).borrow().color == Color::Red;
                }
            }
            while !current.borrow().is_root() {
                current = self.parent_or_panic(&current);
            }
            Some(current)
        } else {
            debug!("new node {:?} is root", inserted.borrow().v);
            Some(inserted)
        };
        root.map(|node| {
            debug!("root ({:?}) color changed to black", node.borrow().v);
            node.borrow_mut().set_color(Color::Black);
            node
        })
    }

    fn rotate(&self, node: Rc<RefCell<Node>>, direction: Rotation) {
        match direction {
            Rotation::Left => {
                let r = node.borrow().right.clone();
                let gl = r.as_ref().and_then(|child| child.borrow().left.clone());
                self.rotate_internal(node, r, gl, Rotation::Left);
            }
            Rotation::Right => {
                let l = node.borrow().left.clone();
                let gr = l.as_ref().and_then(|child| child.borrow().right.clone());
                self.rotate_internal(node, l, gr, Rotation::Right);
            }
        }
    }

    fn rotate_internal(
        &self,
        node: Rc<RefCell<Node>>,
        child: Option<Rc<RefCell<Node>>>,
        grandchild: Option<Rc<RefCell<Node>>>,
        rotation: Rotation,
    ) -> Rc<RefCell<Node>> {
        let p = node.borrow().parent.clone();
        assert!(
            child.as_ref().is_some(),
            "if node does not have a child, it can not rotate"
        );
        // (5)/(6) 左子ノードの右子ノード <=> 自ノード
        let child_direction = match rotation {
            Rotation::Left => RedBlackOp::LeftNode,
            Rotation::Right => RedBlackOp::RightNode,
        };
        Self::pair(child.clone(), Some(node.clone()), child_direction);
        // (1)/(3) 自ノードの左子ノード <=> 自ノードの元々の左子ノードの右子ノード
        let grandchild_direction = match rotation {
            Rotation::Left => RedBlackOp::RightNode,
            Rotation::Right => RedBlackOp::LeftNode,
        };
        Self::pair(Some(node.clone()), grandchild.clone(), grandchild_direction);

        // (2)/(4) 左子ノードの親ノード <=> 自ノードの親ノード
        match p {
            // (4) 親ノードの子ノード = 左子ノード
            Some(p) => {
                let insert_direction =
                    self.decide_direction(&p.clone().borrow().v, &node.borrow().v);
                Self::pair(Some(p.clone()), child.clone(), insert_direction);
                p.clone()
            }
            // (例外) 左子ノードの親ノード = None (左子ノードがrootになる場合)
            None => {
                child.as_ref().unwrap().borrow_mut().parent = None;
                child.clone().unwrap()
            }
        }
    }

    fn parent_or_panic(&self, node: &Rc<RefCell<Node>>) -> Rc<RefCell<Node>> {
        node.borrow().parent.as_ref().unwrap().clone()
    }

    fn _grand_parent(&self, node: Rc<RefCell<Node>>) -> Option<Rc<RefCell<Node>>> {
        node.borrow().parent.as_ref()?.borrow().parent.clone()
    }

    /// uncleノードを取得
    /// which:
    fn uncle(&self, node: Rc<RefCell<Node>>) -> Option<(Option<Rc<RefCell<Node>>>, RedBlackOp)> {
        let parent = (&node.borrow().parent).clone()?;
        let grand_parent = (&parent.borrow().parent).clone()?;
        // 親ノードが祖父ノードのある方向にある場合、uncleノードは親ノードの反対側になる
        let uncle_and_which =
            match self.decide_direction(&grand_parent.borrow().v, &parent.borrow().v) {
                RedBlackOp::LeftNode => {
                    let uncle = grand_parent.borrow().right.clone();
                    Some((uncle, RedBlackOp::RightNode))
                }
                RedBlackOp::RightNode => {
                    let uncle = grand_parent.borrow().left.clone();
                    Some((uncle, RedBlackOp::LeftNode))
                }
            };
        uncle_and_which
    }

    pub fn find(&self, _value: u64) -> Option<IoTDevice> {
        todo!();
    }

    pub fn find_rec(&self) {
        todo!();
    }

    pub fn walk(&self, mut callback: impl FnMut(&IoTDevice, usize)) {
        self.root.as_ref().map(|root| {
            self.walk_rec(root.clone(), &mut callback, 0);
        });
    }

    fn walk_rec(
        &self,
        node: Rc<RefCell<Node>>,
        callback: &mut impl FnMut(&IoTDevice, usize),
        level: usize,
    ) {
        let left = node.borrow().left.clone();
        let right = node.borrow().right.clone();
        debug!("current: {:?} level: {}", node.borrow().v, level);
        callback(&node.clone().borrow().v, level);
        right.map(|r| self.walk_rec(r.clone(), callback, level + 1));
        left.map(|l| self.walk_rec(l.clone(), callback, level + 1));
    }
}

impl std::fmt::Display for DeviceRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.walk(&mut |device: &IoTDevice, level| {
            let indent = "  ".repeat(level);
            writeln!(f, "{}- {:?}", indent, device.numeriacl_id).unwrap();
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

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

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
        registry.insert_internal(device(5));
        registry.insert_internal(device(6));
        registry.insert_internal(device(4));

        assert_eq!(registry.length, 3);
    }

    #[test]
    fn tree_should_be_balanced() {
        let mut registry = DeviceRegistry::default();
        registry.insert(device(1));
        registry.insert(device(2));
        registry.insert(device(3));

        assert_eq!(registry.length, 3);
        assert_eq!(
            format!("{}", registry),
            r#"- 2
  - 3
  - 1
"#
        );
    }

    #[test]
    fn when_complex_tree_should_be_balanced() {
        let mut registry = DeviceRegistry::default();
        registry.insert(device(2));
        registry.insert(device(1));
        registry.insert(device(4));
        registry.insert(device(3));
        registry.insert(device(7));
        registry.insert(device(6));
        registry.insert(device(5));

        assert_eq!(registry.length, 7);
        assert_eq!(
            format!("{}", registry),
            "- 4\n  - 6\n    - 7\n    - 5\n  - 2\n    - 3\n    - 1\n"
        );
    }

    #[test]
    fn test_insert_internal() {
        init();
        let mut registry = DeviceRegistry::default();
        let p = device(6);
        registry.insert_internal(p);
        let n = device(4);
        registry.insert_internal(n);
        let r = device(5);
        registry.insert_internal(r);
        let l = device(2);
        registry.insert_internal(l);
        let gl = device(1);
        registry.insert_internal(gl);
        let gr = device(3);
        registry.insert_internal(gr);

        assert_eq!(registry.length, 6);
        let should_p = &registry.root.as_ref().unwrap().borrow();
        assert!(should_p.is_root());
        let should_n = &should_p.left.as_ref().unwrap().borrow();
        assert_eq!(should_n.v, device(4));
        let should_r = &should_n.right.as_ref().unwrap().borrow();
        assert_eq!(should_r.v, device(5));
        let should_l = &should_n.left.as_ref().unwrap().borrow();
        assert_eq!(should_l.v, device(2));
        let should_gl = &should_l.left.as_ref().unwrap().borrow();
        assert_eq!(should_gl.v, device(1));
        let should_gr = &should_l.right.as_ref().unwrap().borrow();
        assert_eq!(should_gr.v, device(3));
    }

    #[test]
    fn test_rotate_when_node_is_left_child_then_rotate_right_should_make_left_child_of_parent_left_child_of_node(
    ) {
        // 6
        //  l 4
        //    l 2
        //      l 1
        //      r 3
        //    r 5
        // after rotate based on 4
        // 6
        //  l 2
        //    l 1
        //    r 4
        //      l 3
        //      r 5
        init();
        let mut registry = DeviceRegistry::default();
        let p = device(6);
        registry.insert_internal(p.clone());
        let n = device(4);
        registry.insert_internal(n.clone());
        let r = device(5);
        registry.insert_internal(r.clone());
        let l = device(2);
        registry.insert_internal(l.clone());
        let gl = device(1);
        registry.insert_internal(gl.clone());
        let gr = device(3);
        registry.insert_internal(gr.clone());

        // Act
        let node = registry
            .root
            .as_ref()
            .unwrap()
            .borrow()
            .left
            .as_ref()
            .unwrap()
            .clone();

        registry.rotate(node.clone(), super::Rotation::Right);

        // Assert
        assert_eq!(registry.length, 6);
        let new_p = registry.root.as_ref().unwrap().clone();
        assert!(new_p.borrow().is_root());
        let new_pl = new_p.borrow().left.as_ref().unwrap().clone();
        assert_eq!(new_pl.borrow().v, l);
        let new_ll = new_pl.borrow().left.as_ref().unwrap().clone();
        assert_eq!(new_ll.borrow().v, gl);
        let new_lr = new_pl.borrow().right.as_ref().unwrap().clone();
        assert_eq!(new_lr.borrow().v, n);
        let new_nl = new_lr.borrow().left.as_ref().unwrap().clone();
        assert_eq!(new_nl.borrow().v, gr);
        let new_nr = new_lr.borrow().right.as_ref().unwrap().clone();
        assert_eq!(new_nr.borrow().v, r);
    }

    #[test]
    fn test_rotate_when_node_is_left_child_then_rotate_right_should_make_right_child_of_parent_left_child_of_node(
    ) {
        // 6
        //  r 10
        //    l 8
        //      l 7
        //      r 9
        //    r 11
        // after rotate based on 10
        // 6
        //  r 8
        //    l 7
        //    r 10
        //      l 9
        //      r 11

        init();
        let mut registry = DeviceRegistry::default();
        let p = device(6);
        registry.insert_internal(p.clone());
        let n = device(10);
        registry.insert_internal(n.clone());
        let r = device(11);
        registry.insert_internal(r.clone());
        let l = device(8);
        registry.insert_internal(l.clone());
        let gl = device(7);
        registry.insert_internal(gl.clone());
        let gr = device(9);
        registry.insert_internal(gr.clone());

        // Act
        let node = registry
            .root
            .as_ref()
            .unwrap()
            .borrow()
            .right
            .as_ref()
            .unwrap()
            .clone();

        registry.rotate(node.clone(), super::Rotation::Right);

        // Assert
        assert_eq!(registry.length, 6);
        let new_p = registry.root.as_ref().unwrap().clone();
        assert!(new_p.borrow().is_root());
        let new_pl = new_p.borrow().right.as_ref().unwrap().clone();
        assert_eq!(new_pl.borrow().v, l);
        let new_ll = new_pl.borrow().left.as_ref().unwrap().clone();
        assert_eq!(new_ll.borrow().v, gl);
        let new_lr = new_pl.borrow().right.as_ref().unwrap().clone();
        assert_eq!(new_lr.borrow().v, n);
        let new_nl = new_lr.borrow().left.as_ref().unwrap().clone();
        assert_eq!(new_nl.borrow().v, gr);
        let new_nr = new_lr.borrow().right.as_ref().unwrap().clone();
        assert_eq!(new_nr.borrow().v, r);
    }

    #[test]
    fn test_rotate_when_node_is_root_rotate_right_should_make_left_child_root() {
        // 6
        //  l 4
        //    l 2
        //    r 5
        //  r 10
        // after rotate based on 6
        // 4
        //  l 2
        //  r 6
        //    l 5
        //    r 10

        init();
        let mut registry = DeviceRegistry::default();
        let n = device(6);
        registry.insert_internal(n.clone());
        let l = device(4);
        registry.insert_internal(l.clone());
        let r = device(10);
        registry.insert_internal(r.clone());
        let gl = device(2);
        registry.insert_internal(gl.clone());
        let gr = device(5);
        registry.insert_internal(gr.clone());

        // Act
        let node = registry.root.as_ref().unwrap().clone();

        registry.rotate(node.clone(), super::Rotation::Right);

        // Assert
        let new_p = registry
            .root
            .as_ref()
            .unwrap()
            .borrow()
            .parent
            .as_ref()
            .unwrap()
            .clone();
        assert!(new_p.borrow().is_root());
        assert_eq!(new_p.borrow().v, l);
        let new_l = new_p
            .borrow()
            .right
            .as_ref()
            .unwrap()
            .borrow()
            .left
            .as_ref()
            .unwrap()
            .clone();
        assert_eq!(new_l.borrow().v, gr);
    }

    #[test]
    fn test_rotate_when_node_is_left_child_then_rotate_left_should_make_right_child_of_parent_right_child_of_node(
    ) {
        // 6
        //  r 10
        //    l 8
        //    r 12
        //      l 11
        //      r 13
        // after rotate based on 10
        // 6
        //  r 12
        //    l 10
        //      l 8
        //      r 11
        //    r 13
        init();
        let mut registry = DeviceRegistry::default();
        let p = device(6);
        registry.insert_internal(p.clone());
        let n = device(10);
        registry.insert_internal(n.clone());
        let l = device(8);
        registry.insert_internal(l.clone());
        let r = device(12);
        registry.insert_internal(r.clone());
        let gl = device(11);
        registry.insert_internal(gl.clone());
        let gr = device(13);
        registry.insert_internal(gr.clone());

        // Act
        let node = registry
            .root
            .as_ref()
            .unwrap()
            .borrow()
            .right
            .as_ref()
            .unwrap()
            .clone();

        registry.rotate(node.clone(), super::Rotation::Left);

        // Assert
        assert_eq!(registry.length, 6);
        let new_p = registry.root.as_ref().unwrap().clone();
        assert!(new_p.borrow().is_root());
        assert_eq!(new_p.borrow().v, p);
        let new_pr = new_p.borrow().right.as_ref().unwrap().clone();
        assert_eq!(new_pr.borrow().v, r);
        let new_rl = new_pr.borrow().left.as_ref().unwrap().clone();
        assert_eq!(new_rl.borrow().v, n);
        let new_rr = new_pr.borrow().right.as_ref().unwrap().clone();
        assert_eq!(new_rr.borrow().v, gr);
        let new_nl = new_rl.borrow().left.as_ref().unwrap().clone();
        assert_eq!(new_nl.borrow().v, l);
        let new_nr = new_rl.borrow().right.as_ref().unwrap().clone();
        assert_eq!(new_nr.borrow().v, gl);
    }
}
