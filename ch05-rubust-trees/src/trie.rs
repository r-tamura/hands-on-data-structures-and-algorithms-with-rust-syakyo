use std::collections::BTreeMap;

#[derive(Default)]
struct TrieNode<V> {
    key: char,
    next: BTreeMap<char, TrieNode<V>>,
    value: V,
}

impl<V> TrieNode<V> {
    fn new(key: char, value: V) -> Self {
        Self {
            key,
            next: BTreeMap::new(),
            value,
        }
    }
}
pub struct TrieTree<V> {
    length: usize,
    root: BTreeMap<char, TrieNode<V>>,
}

impl<V> Default for TrieTree<V> {
    fn default() -> Self {
        Self {
            length: usize::default(),
            root: BTreeMap::new(),
        }
    }
}

impl<V> TrieTree<V> {
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn add(&mut self, s: String, v: V) {
        self.length += 1;
        self.root
            .entry(s.chars().next().unwrap())
            .or_insert_with(|| TrieNode::new(s.chars().next().unwrap(), v));
    }

    /// 指定した文字列sに完全一致する値を取得します
    pub fn find(&self, s: &str) -> Option<&V> {
        let (first, _) = s.split_at(1);
        self.root
            .get(&first.chars().next().unwrap())
            .map(|node| &node.value)
    }

    pub fn remove(&mut self, s: &str) {
        todo!();
    }
}
