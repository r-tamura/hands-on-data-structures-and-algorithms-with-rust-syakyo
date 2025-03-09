use crate::iot::IoTDevice;

type Tree = Box<Node>;
type Key = u64;

type ValueChildPair = (Option<IoTDevice>, Option<Tree>);

/// B-treeノードが保持できる最大の子ノード数
/// キーを3つ以上持つノードは分割されます
const DEFAULT_ORDER: usize = 3;

#[derive(Clone, Debug, PartialEq)]
pub enum NodeType {
    Leaf,
    Regular,
}

#[derive(Clone, PartialEq)]
pub enum Direction {
    /// もっとも親に近い子要素
    Left,
    Right(usize),
}
#[derive(Debug, PartialEq)]
/// B木の各ノードを表現する構造体
/// - B木のノードはキーと値のペアを保持する(values)
/// - キーと値のペアの間には、子ノードへのポインタがある(children)
/// - もっとも左のペアの左側には、左の子ノードへのポインタがある(left_child)
///
/// ```text
/// 構造:
///
/// left_child: T0
///          ┌─────┐
///          │Tree │
///          └─────┘        values:    [value1]        [value2]        [value3]
///             │                        │               │               │
///             │                        v               v               v
///             │                    ┌────────┐     ┌────────┐     ┌────────┐
///             │                    │ Value1 │     │ Value2 │     │ Value3 │
///             │                    └────────┘     └────────┘     └────────┘
///             │                        │               │               │
///             │                        v               v               v
///             │    children:            T1     ->      T2     ->      T3      ->     T4
///             │                     ┌─────┐        ┌─────┐        ┌─────┐        ┌─────┐
///             │                     │Tree │        │Tree │        │Tree │        │Tree │
///             │                     └─────┘        └─────┘        └─────┘        └─────┘
///             v                        │               │               │               │
///           <min                       v               v               v               v
///                                    <50           50-70           70-90            >90
///
/// ノード例 (key=数値):
///
/// left_child: T0
///             │
///             v    values:   [ 50     70     90 ]  <- 値の配列（ソート済み）
///           <min                │      │      │
///                               v      v      v
///                  children:    T1     T2     T3    T4  <- 子ノードの配列
///                            /    \     \     \     \
///                           /      \     \     \     \
///                        <50    50-70  70-90   >90    <- 各部分木の値の範囲
///
/// 分割の例（次数3のB木）:
///
/// 挿入前:
/// left_child: T0
///             │
///             v    values:   [ 30     40 ]
///           <min              │      │
///                  children:   T1     T2    T3
///
/// 60を挿入（オーバーフロー）:
/// left_child: T0
///             │
///             v    values:   [ 30     40    60 ]   <- 最大2要素なのでオーバーフロー
///           <min               │      │      │
///                              v      v      v
///                  children:   T1     T2     T3    T4
///
/// 分割後:
///                           values:        [ 40 ]
///                                            │
///                           children:       /  \
///                                         /     \
///                            values:    [30]   [60]
///                left_child: T0    \   /  \   /  \
///                            │      v T1  T2 T3  T4
///                            v    <min
/// ```
pub struct Node {
    values: Vec<Option<IoTDevice>>,
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
            values: vec![],
            children: vec![],
            left_child: None,
            node_type,
        })
    }

    fn from_nodes(
        node_type: NodeType,
        left: Option<Box<Node>>,
        values: Vec<Option<IoTDevice>>,
        children: Vec<Option<Tree>>,
    ) -> Tree {
        Box::new(Node {
            values,
            children,
            left_child: left,
            node_type,
        })
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn find_closest_index(&self, key: Key) -> Direction {
        let mut index = Direction::Left;
        for (i, pair) in self.values.iter().enumerate() {
            if let Some(value) = pair {
                if value.numeriacl_id <= key {
                    index = Direction::Right(i);
                } else {
                    break;
                }
            }
        }
        index
    }

    pub fn add_key(&mut self, key: Key, value: ValueChildPair) -> bool {
        let index = match self.find_closest_index(key) {
            Direction::Left => 0,
            Direction::Right(i) => i + 1,
        };
        let (dev, tree) = value;

        self.values.insert(index, dev);
        self.children.insert(index, tree);
        true
    }

    fn set_left_child(&mut self, tree: Tree) {
        self.left_child = Some(tree);
    }

    // keyに一番近い子要素を削除する
    pub fn remove_key(&mut self, key: Key) -> Option<(Key, ValueChildPair)> {
        match self.find_closest_index(key) {
            Direction::Left => {
                let tree = self.left_child.take();
                Some((key, (None, tree)))
            }
            Direction::Right(i) => {
                let value = self.values.remove(i);
                let tree = self.children.remove(i);
                Some((key, (value, tree)))
            }
        }
    }

    /// 完全一致するキーのデバイスを取得する
    pub fn find_value(&self, key: Key) -> Option<&IoTDevice> {
        self.values
            .iter()
            .find_map(|value| value.as_ref().filter(|device| device.numeriacl_id == key))
    }

    /// キーに一番近い子要素を取得する
    pub fn find_child(&self, key: Key) -> Option<&Tree> {
        match self.find_closest_index(key) {
            Direction::Left => self.left_child.as_ref(),
            Direction::Right(i) => self.children.get(i).and_then(|child| child.as_ref()),
        }
    }

    /// キーに一番近い要素の可変な参照を取得します
    pub fn find_child_mut(&mut self, key: Key) -> Option<&mut Option<Tree>> {
        match self.find_closest_index(key) {
            Direction::Left => Some(&mut self.left_child),
            Direction::Right(i) => self.children.get_mut(i),
        }
    }

    /// trueの場合、ノードが保持できる要素数を超えており、分割が必要です
    fn is_overflow(&self) -> bool {
        self.len() >= DEFAULT_ORDER
    }

    /// index以降の値と子ノードを自身のノードから削除して、返します
    fn take_after(&mut self, index: usize) -> (IoTDevice, Tree) {
        let mid_value = self.values.remove(index);
        let mid_node = self.children.remove(index);
        let mut new_values = vec![];
        let mut new_children = vec![];
        for _ in index..self.len() {
            let value = self.values.remove(index);
            let child = self.children.remove(index);
            new_values.push(value);
            new_children.push(child);
        }

        let new_node = Node::from_nodes(self.node_type.clone(), mid_node, new_values, new_children);

        (mid_value.unwrap(), new_node)
    }

    /// ノードがオーバーフローした際にノードを分割します
    /// 新しいノードを作成し、中央の値より右側の値を新しいノードに移動します
    /// 中央の値とその子ノードを返します
    pub(self) fn split(&mut self) -> (IoTDevice, Tree) {
        if !self.is_overflow() {
            panic!("Node is not overflowed");
        }
        let mid = self.len() / 2;
        let (orphan_value, new_n) = self.take_after(mid);
        (orphan_value, new_n)
    }
}

pub struct BTree {
    root: Option<Tree>,
    order: usize,
    pub length: u64,
}

impl BTree {
    /// B木に値を追加します
    pub fn add(&mut self, key: Key, value: IoTDevice) {
        let root = self.root.take().unwrap_or(Node::new_leaf());
        let (new_root, _) = self.add_rec(root, key, value, true);
        self.root = Some(new_root);
    }

    fn add_rec(
        &mut self,
        target: Tree,
        key: Key,
        value: IoTDevice,
        is_root: bool,
    ) -> (Tree, Option<ValueChildPair>) {
        let mut target = target;
        match target.node_type {
            NodeType::Leaf => {
                if target.add_key(key, (Some(value), None)) {
                    self.length += 1;
                }
            }
            NodeType::Regular => {
                let (key, (dev, tree)) = target.remove_key(key).unwrap();
            }
        };
        (target, None)
    }

    /// B木から値を削除します
    pub fn remove(&mut self, _key: Key) {
        todo!();
    }

    /// B木から値を取得します
    pub fn find(&self, key: Key) -> Option<&IoTDevice> {
        let root = self.root.as_ref()?;
        let mut current = root;
        loop {
            match current.find_value(key) {
                Some(value) => return Some(value),
                None => {
                    let child = current.find_child(key)?;
                    current = child;
                }
            }
        }
    }

    /// B木を走査しますして、各要素に対して関数を適用します
    pub fn traverse(&self, _callback: impl Fn(&IoTDevice)) {
        todo!();
    }
}

impl Default for BTree {
    fn default() -> Self {
        BTree {
            root: None,
            order: MAX_KEYS,
            length: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod node {
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

        #[test]
        fn test_add_key_left() {
            // Arrange
            let mut leaf = Node::new_leaf();
            let key = 10;
            let value = (Some(IoTDevice::new(10, "device", "")), None);
            assert!(leaf.add_key(key, value));

            // Act
            let new_key = 20;
            let new_value = (Some(IoTDevice::new(20, "new_device", "")), None);
            assert!(leaf.add_key(new_key, new_value));

            // Assert
            assert_eq!(leaf.len(), 2);
            assert_eq!(
                leaf.values,
                vec![
                    Some(IoTDevice::new(10, "device", "")),
                    Some(IoTDevice::new(20, "new_device", ""))
                ]
            );
            assert_eq!(leaf.children, vec![None, None]);
        }

        #[test]
        fn should_add_key_right() {
            // Arrange
            let mut leaf = Node::new_leaf();
            let key = 10;
            let value = (Some(IoTDevice::new(10, "device", "")), None);
            assert!(leaf.add_key(key, value));

            // Act
            let new_key = 5;
            let new_value = (Some(IoTDevice::new(5, "new_device", "")), None);
            assert!(leaf.add_key(new_key, new_value));

            // Assert
            assert_eq!(leaf.len(), 2);
            assert_eq!(
                leaf.values,
                vec![
                    Some(IoTDevice::new(5, "new_device", "")),
                    Some(IoTDevice::new(10, "device", ""))
                ]
            );
            assert_eq!(leaf.children, vec![None, None]);
        }

        #[test]
        fn should_remove_key_left() {
            // Arrange
            let mut leaf = Node::new_leaf();
            let key = 10;
            let value = (Some(IoTDevice::new(10, "device", "")), None);
            assert!(leaf.add_key(key, value));

            // Act
            let removed = leaf.remove_key(10);

            // Assert
            assert_eq!(leaf.len(), 0);
            assert_eq!(
                removed,
                Some((10, (Some(IoTDevice::new(10, "device", "")), None)))
            );
        }

        #[test]
        fn should_remove_key_right() {
            // Arrange
            let mut leaf = Node::new_leaf();
            leaf.add_key(10, (Some(IoTDevice::new(10, "device", "")), None));
            leaf.add_key(20, (Some(IoTDevice::new(20, "new_device", "")), None));

            // Act
            let removed = leaf.remove_key(20);

            // Assert
            assert_eq!(leaf.len(), 1);
            assert_eq!(
                removed,
                Some((20, (Some(IoTDevice::new(20, "new_device", "")), None)))
            );
        }

        #[test]
        #[should_panic]
        fn should_panic_when_node_is_overflowed_and_split() {
            // Arrange
            let mut leaf = Node::new_leaf();
            leaf.add_key(10, (Some(IoTDevice::new(10, "device", "")), None));
            leaf.add_key(20, (Some(IoTDevice::new(20, "new_device", "")), None));
            leaf.add_key(30, (Some(IoTDevice::new(30, "new_device", "")), None));
            leaf.add_key(40, (Some(IoTDevice::new(40, "new_device", "")), None));

            // Act
            let (orphan, new_node) = leaf.split();

            // Assert
            assert_eq!(orphan, IoTDevice::new(20, "new_device", ""));
            assert_eq!(new_node.len(), 1);
            assert_eq!(
                new_node.values,
                vec![Some(IoTDevice::new(30, "new_device", ""))]
            );
        }

        #[test]
        fn should_be_split_when_overflowed() {
            // Arrange
            let mut leaf = Node::new_leaf();
            leaf.add_key(10, (Some(IoTDevice::new(10, "device", "")), None));
            leaf.add_key(20, (Some(IoTDevice::new(20, "new_device", "")), None));
            leaf.add_key(30, (Some(IoTDevice::new(30, "new_device", "")), None));
            leaf.add_key(40, (Some(IoTDevice::new(40, "new_device", "")), None));

            // Act
            let (orphan, new_node) = leaf.split();

            // Assert
            assert_eq!(orphan, IoTDevice::new(30, "new_device", ""));
            assert_eq!(new_node.len(), 1);
            assert_eq!(
                new_node.values,
                vec![Some(IoTDevice::new(40, "new_device", ""))]
            );
        }

        #[test]
        fn should_find_closest_mutable_child() {
            // Arrange
            let mut node = Node::new_leaf();
            node.add_key(10, (Some(IoTDevice::new(10, "device", "")), None));
            node.add_key(20, (Some(IoTDevice::new(20, "new_device", "")), None));

            // Act
            let child = node.find_child_mut(15);

            // Assert
            assert_eq!(child, Some(&mut None));
        }
    }

    mod btree {
        use super::*;

        #[test]
        fn should_add_value_when_btree_is_empty() {
            // Arrange
            let mut btree = BTree::default();
            let key = 10;
            let device1 = IoTDevice::new(10, "device", "");

            // Act
            btree.add(key, device1.clone());

            // Assert
            assert_eq!(btree.length, 1);
            assert_eq!(btree.find(key), Some(&device1));
        }

        #[test]
        fn should_add_value_when_btree_has_a_value() {
            // Arrange
            let mut btree = BTree::default();
            let device1 = IoTDevice::new(10, "device", "");
            let device2 = IoTDevice::new(20, "new_device", "");
            btree.add(10, device1.clone());

            // Act
            btree.add(20, device2.clone());

            // Assert
            assert_eq!(btree.length, 2);
            assert_eq!(btree.find(10), Some(&device1));
            assert_eq!(btree.find(20), Some(&device2));
        }
    }
}
