use log::debug;
use std::collections::BTreeMap;

enum InsertResult<V> {
    Added,
    Updated(V),
}

enum TrieNode<V> {
    /// 中間ノード。文字列の途中の文字を表し、値は持たない
    Internal {
        next: BTreeMap<char, Box<TrieNode<V>>>,
    },
    /// 終端ノード。文字列の最後の文字を表し、値を持つ
    Leaf {
        value: V,
        next: BTreeMap<char, Box<TrieNode<V>>>,
    },
}

impl<V> TrieNode<V> {
    fn new_internal() -> Self {
        Self::Internal {
            next: BTreeMap::new(),
        }
    }

    fn next(&self) -> &BTreeMap<char, Box<TrieNode<V>>> {
        match self {
            Self::Internal { next, .. } => next,
            Self::Leaf { next, .. } => next,
        }
    }

    fn next_mut(&mut self) -> &mut BTreeMap<char, Box<TrieNode<V>>> {
        match self {
            Self::Internal { next, .. } => next,
            Self::Leaf { next, .. } => next,
        }
    }

    fn make_leaf(&mut self, value: V) -> InsertResult<V> {
        match self {
            Self::Internal { next } => {
                let next = std::mem::take(next);
                *self = Self::Leaf { value, next };
                InsertResult::Added
            }
            Self::Leaf {
                value: old_value, ..
            } => {
                let old = std::mem::replace(old_value, value);
                InsertResult::Updated(old)
            }
        }
    }

    fn value(&self) -> Option<&V> {
        match self {
            Self::Internal { .. } => None,
            Self::Leaf { value, .. } => Some(value),
        }
    }
}

pub struct TrieTree<V> {
    length: usize,
    root: BTreeMap<char, Box<TrieNode<V>>>,
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

        let chars: Vec<char> = key.chars().collect();
        let mut current = self
            .root
            .entry(chars[0])
            .or_insert_with(|| Box::new(TrieNode::new_internal()));

        for &c in chars[1..].iter() {
            let next = current
                .next_mut()
                .entry(c)
                .or_insert_with(|| Box::new(TrieNode::new_internal()));
            current = next;
        }

        let result = current.make_leaf(v);
        match result {
            InsertResult::Added => {
                self.length += 1;
                debug!("added: {key}");
            }
            InsertResult::Updated(_) => debug!("updated: {key}"),
        }
    }

    pub fn find(&self, s: &str) -> Option<&V> {
        debug!("[trie::find] s: {}", s);
        let chars: Vec<char> = s.chars().collect();

        if chars.is_empty() {
            return None;
        }

        let mut current = self.root.get(&chars[0])?;

        for &c in chars[1..].iter() {
            current = current.next().get(&c)?;
        }

        current.value()
    }

    pub fn remove(&mut self, _key: &str) -> Option<V> {
        todo!()
    }
}
