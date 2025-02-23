use std::collections::BTreeMap;

use log::debug;

#[derive(Default)]
struct TrieNode<V> {
    key: char,
    next: BTreeMap<char, TrieNode<V>>,
    /// 登録済みのキーのパスとして存在していた場合にvalueはNoneとなる
    value: Option<V>,
}

impl<V> TrieNode<V> {
    fn new(key: char) -> Self {
        Self {
            key,
            next: BTreeMap::new(),
            value: None,
        }
    }

    fn new_with_value(key: char, value: V) -> Self {
        Self {
            key,
            next: BTreeMap::new(),
            value: Some(value),
        }
    }

    fn replace_value(&mut self, value: V) -> Option<V> {
        self.value.replace(value)
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

    pub fn add(&mut self, key: String, v: V) {
        assert!(!key.is_empty(), "key must not be empty");
        debug!("[trie::add] key: {}", key);
        let first = key[0..1].chars().next().unwrap();
        if key.len() == 1 {
            let old_value = self.root.insert(first, TrieNode::new_with_value(first, v));

            match old_value {
                Some(TrieNode { value: Some(_), .. }) => {
                    debug!("updated: {key}");
                }
                _ => {
                    self.length += 1;
                    debug!("added: {key}");
                }
            };
            return;
        }
        let rest = &key[1..key.len() - 1];
        let last = key.chars().last().unwrap();

        let mut current = self
            .root
            .entry(first)
            .or_insert_with(|| TrieNode::new(first));
        for c in rest.chars() {
            current = current.next.entry(c).or_insert_with(|| TrieNode::new(c));
        }

        let node = current
            .next
            .entry(last)
            .or_insert_with(|| TrieNode::new(last));

        let old_value = node.replace_value(v);
        match old_value {
            Some(_) => {
                debug!("updated: {key}");
            }
            _ => {
                self.length += 1;
                debug!("added: {key}");
            }
        }
    }

    /// 指定した文字列keyに完全一致する値を取得します
    pub fn find(&self, s: &str) -> Option<&V> {
        debug!("[trie::find] s: {}", s);
        let first = s[0..1].chars().next().unwrap();
        if s.len() == 1 {
            return self.root.get(&first).and_then(|node| node.value.as_ref());
        }
        let rest = &s[1..s.len() - 1];
        let last = s.chars().last().unwrap();

        let mut current = self.root.get(&first)?;
        debug!("first: {}", first);
        for c in rest.chars() {
            debug!("c: {}", c);
            current = current.next.get(&c)?;
        }
        debug!(
            "last: {} {:?}",
            last,
            current.next.keys().collect::<Vec<&char>>()
        );
        current.next.get(&last).and_then(|node| node.value.as_ref())
    }

    pub fn remove(&mut self, key: &str) {
        todo!();
    }
}
