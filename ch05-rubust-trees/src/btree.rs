use crate::iot::IoTDevice;

type Tree = Box<Node>;
type Key = u64;

type ValueChildPair = (Option<IoTDevice>, Option<Tree>);
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
    pub fn find_device_by_key(&self, key: Key) -> Option<&IoTDevice> {
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
    fn node_should_add_key_right() {
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
    fn node_should_remove_key_left() {
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
    fn node_should_remove_key_right() {
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
}
