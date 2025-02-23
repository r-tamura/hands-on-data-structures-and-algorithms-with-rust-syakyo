use std::collections::BTreeMap;

#[derive(Default)]
struct TrieNode<V> {
    key: char,
    next: BTreeMap<char, TrieNode<V>>,
    value: V,
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
        todo!();
    }

    /// 指定した文字列sに完全一致する値を取得します
    pub fn find(&self, s: &str) -> Option<&V> {
        todo!();
    }

    pub fn remove(&mut self, s: &str) {
        todo!();
    }
}
