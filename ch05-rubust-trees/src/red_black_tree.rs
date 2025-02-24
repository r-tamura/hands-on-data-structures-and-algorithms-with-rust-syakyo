use log::debug;
use std::{cell::RefCell, rc::Rc};

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

struct Node<T>
where
    T: std::fmt::Debug + std::fmt::Display + Clone + Eq + Ord,
{
    pub color: Color,
    pub v: T,
    pub parent: Option<Rc<RefCell<Node<T>>>>,
    left: Option<Rc<RefCell<Node<T>>>>,
    right: Option<Rc<RefCell<Node<T>>>>,
}

impl<T: std::fmt::Debug + std::fmt::Display + Clone + Eq + Ord> Node<T> {
    pub fn new(value: T) -> Node<T> {
        Node {
            color: Color::Red,
            v: value,
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

impl<T: std::fmt::Debug + std::fmt::Display + Clone + Eq + Ord> PartialEq for Node<T> {
    fn eq(&self, other: &Self) -> bool {
        self.v == other.v
    }
}

impl<T: std::fmt::Debug + std::fmt::Display + Clone + Eq + Ord> std::fmt::Debug for Node<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, r#"{{"color": {:?}, "v": "{:?}"}}"#, self.color, self.v)
    }
}

#[derive(PartialEq)]
pub struct DeviceRegistry<T>
where
    T: std::fmt::Debug + std::fmt::Display + Clone + Eq + Ord,
{
    root: Option<Rc<RefCell<Node<T>>>>,
    pub length: u64,
}

type Tree<T> = Rc<RefCell<Node<T>>>;
type MaybeTree<T> = Option<Tree<T>>;

impl<T: std::fmt::Debug + std::fmt::Display + Clone + Eq + Ord> DeviceRegistry<T> {
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
    pub fn insert(&mut self, value: T) {
        let new_node = self.insert_internal(value);
        debug!("--- start balancing {:?}", new_node.borrow().v);
        self.root = self.balance(new_node.clone());
        debug!("--- end balancing {:?}", new_node.borrow().v);
    }

    fn pair(
        parent: Option<Rc<RefCell<Node<T>>>>,
        child: Option<Rc<RefCell<Node<T>>>>,
        direction: RedBlackOp,
    ) {
        match (parent, child) {
            (Some(parent), Some(child)) => {
                match direction {
                    RedBlackOp::LeftNode => {
                        parent.borrow_mut().left = Some(child.clone());
                        debug!("{:?}.left <- {:?}", parent.borrow().v, child.borrow().v,);
                    }
                    RedBlackOp::RightNode => {
                        debug!("{:?}.right <- {:?}", parent.borrow().v, child.borrow().v,);
                        parent.borrow_mut().right = Some(child.clone());
                    }
                };
                child.borrow_mut().parent = Some(parent.clone());
                debug!(
                    "parent: {:?} child: {:?}",
                    parent.borrow().v,
                    child.borrow().v
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
    fn decide_direction(&self, a: &T, b: &T) -> RedBlackOp {
        if a <= b {
            RedBlackOp::RightNode
        } else {
            RedBlackOp::LeftNode
        }
    }

    fn insert_internal(&mut self, value: T) -> Rc<RefCell<Node<T>>> {
        self.length += 1;
        let maybe_root = self.root.take();
        let (maybe_root, new_node) = self.insert_rec(maybe_root.clone(), value);
        debug!("new_root: {:?}, new_node: {:?}", &maybe_root, &new_node);
        self.root = maybe_root;
        new_node.clone()
    }

    fn insert_rec(
        &mut self,
        mut maybe_current_node: Option<Rc<RefCell<Node<T>>>>,
        value: T,
    ) -> (MaybeTree<T>, Rc<RefCell<Node<T>>>) {
        match maybe_current_node.take() {
            None => {
                // 葉に到達したので、新しいノードを追加
                debug!("inserting new node {:?}", value);
                let new_node = Rc::new(RefCell::new(Node::new(value)));
                (Some(new_node.clone()), new_node)
            }
            Some(current_node) => {
                let new: Rc<RefCell<Node<T>>>;
                let current_value = current_node.borrow().v.clone();
                debug!("--- current: {:?} new: {:?}", current_value, value);

                match self.decide_direction(&current_value, &value) {
                    RedBlackOp::LeftNode => {
                        debug!("go to left: {:?} > new: {:?}", current_value, value);
                        let left = current_node.borrow().left.clone();
                        let (maybe_new_tree, new_node) = self.insert_rec(left, value);
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
                            current_value, value
                        );
                        let right = current_node.borrow().right.clone();
                        let (maybe_new_tree, new_node) = self.insert_rec(right, value);
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
                    current_value,
                    new.borrow().v,
                );

                (Some(current_node), new)
            }
        }
    }

    fn balance_single_node(
        &mut self,
        current: Rc<RefCell<Node<T>>>,
        parent: Rc<RefCell<Node<T>>>,
        maybe_uncle: Option<Rc<RefCell<Node<T>>>>,
        uncle_direction: RedBlackOp,
        grand_parent: Rc<RefCell<Node<T>>>,
    ) -> (Tree<T>, Tree<T>) {
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

    fn balance(&mut self, inserted: Rc<RefCell<Node<T>>>) -> Option<Rc<RefCell<Node<T>>>> {
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
        root.inspect(|node| {
            debug!("root ({:?}) color changed to black", node.borrow().v);
            node.borrow_mut().set_color(Color::Black);
        })
    }

    fn rotate(&self, node: Rc<RefCell<Node<T>>>, direction: Rotation) {
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
        node: Rc<RefCell<Node<T>>>,
        child: Option<Rc<RefCell<Node<T>>>>,
        grandchild: Option<Rc<RefCell<Node<T>>>>,
        rotation: Rotation,
    ) -> Rc<RefCell<Node<T>>> {
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

    fn parent_or_panic(&self, node: &Rc<RefCell<Node<T>>>) -> Rc<RefCell<Node<T>>> {
        node.borrow().parent.as_ref().unwrap().clone()
    }

    fn _grand_parent(&self, node: Rc<RefCell<Node<T>>>) -> Option<Rc<RefCell<Node<T>>>> {
        node.borrow().parent.as_ref()?.borrow().parent.clone()
    }

    /// uncleノードを取得
    /// which:
    fn uncle(&self, node: Rc<RefCell<Node<T>>>) -> Option<(MaybeTree<T>, RedBlackOp)> {
        let parent = (node.borrow().parent).clone()?;
        let grand_parent = (parent.borrow().parent).clone()?;
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

    pub fn find(&self, value: T) -> Option<T> {
        let root = self.root.as_ref()?.clone();
        Self::find_rec(&root, value)
    }

    fn find_rec(current: &Rc<RefCell<Node<T>>>, value: T) -> Option<T> {
        match current.borrow().v.cmp(&value) {
            std::cmp::Ordering::Less => current
                .borrow()
                .right
                .as_ref()
                .and_then(|r| Self::find_rec(r, value)),
            std::cmp::Ordering::Greater => current
                .borrow()
                .left
                .as_ref()
                .and_then(|l| Self::find_rec(l, value)),
            std::cmp::Ordering::Equal => Some(current.borrow().v.clone()),
        }
    }

    pub fn walk(&self, mut callback: impl FnMut(&T, usize)) {
        self.root.as_ref().inspect(|&root| {
            Self::walk_rec(root.clone(), &mut callback, 0);
        });
    }

    fn walk_rec(node: Rc<RefCell<Node<T>>>, callback: &mut impl FnMut(&T, usize), level: usize) {
        let left = node.borrow().left.clone();
        let right = node.borrow().right.clone();
        debug!("current: {:?} level: {}", node.borrow().v, level);
        callback(&node.clone().borrow().v, level);
        right.inspect(|r| Self::walk_rec(r.clone(), callback, level + 1));
        left.inspect(|l| Self::walk_rec(l.clone(), callback, level + 1));
    }
}

impl<T: std::fmt::Debug + std::fmt::Display + Clone + Eq + Ord> std::fmt::Display
    for DeviceRegistry<T>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.walk(&mut |value: &T, level| {
            let indent = "  ".repeat(level);
            writeln!(f, "{}- {}", indent, value).unwrap();
        });
        Ok(())
    }
}

impl<T: std::fmt::Debug + std::fmt::Display + Clone + Eq + Ord> Default for DeviceRegistry<T> {
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

    use crate::iot::IoTDevice;

    use super::{DeviceRegistry, Node};

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    fn value(id: u64) -> IoTDevice {
        IoTDevice::new(id, "", "")
    }

    fn node(value: IoTDevice) -> Rc<RefCell<Node<IoTDevice>>> {
        Rc::new(RefCell::new(Node::new(value)))
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
        registry.insert_internal(value(5));
        registry.insert_internal(value(6));
        registry.insert_internal(value(4));

        assert_eq!(registry.length, 3);
    }

    #[test]
    fn tree_should_be_balanced() {
        let mut registry = DeviceRegistry::default();
        registry.insert(value(1));
        registry.insert(value(2));
        registry.insert(value(3));

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
        registry.insert(value(2));
        registry.insert(value(1));
        registry.insert(value(4));
        registry.insert(value(3));
        registry.insert(value(7));
        registry.insert(value(6));
        registry.insert(value(5));

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
        let p = value(6);
        registry.insert_internal(p);
        let n = value(4);
        registry.insert_internal(n);
        let r = value(5);
        registry.insert_internal(r);
        let l = value(2);
        registry.insert_internal(l);
        let gl = value(1);
        registry.insert_internal(gl);
        let gr = value(3);
        registry.insert_internal(gr);

        assert_eq!(registry.length, 6);
        let should_p = &registry.root.as_ref().unwrap().borrow();
        assert!(should_p.is_root());
        let should_n = &should_p.left.as_ref().unwrap().borrow();
        assert_eq!(should_n.v, value(4));
        let should_r = &should_n.right.as_ref().unwrap().borrow();
        assert_eq!(should_r.v, value(5));
        let should_l = &should_n.left.as_ref().unwrap().borrow();
        assert_eq!(should_l.v, value(2));
        let should_gl = &should_l.left.as_ref().unwrap().borrow();
        assert_eq!(should_gl.v, value(1));
        let should_gr = &should_l.right.as_ref().unwrap().borrow();
        assert_eq!(should_gr.v, value(3));
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
        let p = value(6);
        registry.insert_internal(p.clone());
        let n = value(4);
        registry.insert_internal(n.clone());
        let r = value(5);
        registry.insert_internal(r.clone());
        let l = value(2);
        registry.insert_internal(l.clone());
        let gl = value(1);
        registry.insert_internal(gl.clone());
        let gr = value(3);
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
        let p = value(6);
        registry.insert_internal(p.clone());
        let n = value(10);
        registry.insert_internal(n.clone());
        let r = value(11);
        registry.insert_internal(r.clone());
        let l = value(8);
        registry.insert_internal(l.clone());
        let gl = value(7);
        registry.insert_internal(gl.clone());
        let gr = value(9);
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
        let n = value(6);
        registry.insert_internal(n.clone());
        let l = value(4);
        registry.insert_internal(l.clone());
        let r = value(10);
        registry.insert_internal(r.clone());
        let gl = value(2);
        registry.insert_internal(gl.clone());
        let gr = value(5);
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
        let p = value(6);
        registry.insert_internal(p.clone());
        let n = value(10);
        registry.insert_internal(n.clone());
        let l = value(8);
        registry.insert_internal(l.clone());
        let r = value(12);
        registry.insert_internal(r.clone());
        let gl = value(11);
        registry.insert_internal(gl.clone());
        let gr = value(13);
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

    #[test]
    fn when_only_root_node_and_search_value_exists_then_returns_the_matched_element() {
        let mut registry = DeviceRegistry::default();
        registry.insert(value(5));
        let result = registry.find(value(5));
        assert_eq!(result, Some(value(5)));
    }

    #[test]
    fn when_some_nodes_and_search_value_exists_then_returns_the_matched_element() {
        let mut registry = DeviceRegistry::default();
        registry.insert(value(5));
        registry.insert(value(6));
        registry.insert(value(4));
        let result = registry.find(value(6));
        assert_eq!(result, Some(value(6)));
    }

    #[test]
    fn when_threre_are_some_nodes_and_search_value_does_not_exist_then_returns_none() {
        let mut registry = DeviceRegistry::default();
        registry.insert(value(5));
        registry.insert(value(6));
        registry.insert(value(4));
        let result = registry.find(value(7));
        assert_eq!(result, None);
    }
}
