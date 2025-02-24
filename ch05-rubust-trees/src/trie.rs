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

    pub fn remove(&mut self, key: &str) -> Option<V> {
        debug!("[trie::remove] key: {}", key);
        None // TODO: implement remove functionality
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[derive(Debug, PartialEq)]
    struct TestValue {
        id: u64,
    }

    impl TestValue {
        fn new(id: u64) -> Self {
            Self { id }
        }
    }

    #[test]
    fn add_should_insert_single_char_key() {
        // Arrange
        init();
        let mut trie = TrieTree::default();

        // Act
        trie.add("a".to_string(), TestValue::new(1));

        // Assert
        assert_eq!(trie.len(), 1);
        assert_eq!(trie.find("a").unwrap().id, 1);
    }

    #[test]
    fn add_should_insert_multiple_char_key() {
        // Arrange
        init();
        let mut trie = TrieTree::default();

        // Act
        trie.add("abc".to_string(), TestValue::new(1));

        // Assert
        assert_eq!(trie.len(), 1);
        assert_eq!(trie.find("abc").unwrap().id, 1);
    }

    #[test]
    fn add_should_update_existing_key() {
        // Arrange
        init();
        let mut trie = TrieTree::default();
        trie.add("abc".to_string(), TestValue::new(1));

        // Act
        trie.add("abc".to_string(), TestValue::new(2));

        // Assert
        assert_eq!(trie.len(), 1);
        assert_eq!(trie.find("abc").unwrap().id, 2);
    }

    #[test]
    fn add_should_insert_prefix_key() {
        // Arrange
        init();
        let mut trie = TrieTree::default();
        trie.add("abc".to_string(), TestValue::new(1));

        // Act
        trie.add("ab".to_string(), TestValue::new(2));

        // Assert
        assert_eq!(trie.len(), 2);
        assert_eq!(trie.find("abc").unwrap().id, 1);
        assert_eq!(trie.find("ab").unwrap().id, 2);
    }

    #[test]
    fn add_should_insert_different_key_with_same_prefix() {
        // Arrange
        init();
        let mut trie = TrieTree::default();
        trie.add("abc".to_string(), TestValue::new(1));

        // Act
        trie.add("abx".to_string(), TestValue::new(2));

        // Assert
        assert_eq!(trie.len(), 2);
        assert_eq!(trie.find("abc").unwrap().id, 1);
        assert_eq!(trie.find("abx").unwrap().id, 2);
    }

    #[test]
    fn add_should_update_single_char_key() {
        // Arrange
        init();
        let mut trie = TrieTree::default();
        trie.add("a".to_string(), TestValue::new(1));

        // Act
        trie.add("a".to_string(), TestValue::new(2));

        // Assert
        assert_eq!(trie.len(), 1);
        assert_eq!(trie.find("a").unwrap().id, 2);
    }

    #[test]
    #[should_panic(expected = "key must not be empty")]
    fn add_should_panic_on_empty_key() {
        // Arrange
        init();
        let mut trie = TrieTree::default();

        // Act
        trie.add("".to_string(), TestValue::new(1));
    }

    #[test]
    fn find_should_return_none_for_missing_key() {
        // Arrange
        init();
        let trie = TrieTree::<TestValue>::default();

        // Assert
        assert_eq!(trie.find("not_exists"), None);
        assert_eq!(trie.find(""), None);
    }

    #[test]
    fn add_should_handle_many_items() {
        // Arrange
        init();
        let mut trie = TrieTree::default();
        let n = 1000;

        // Act: Add items with incrementing ids
        for i in 0..n {
            let key = format!("key{}", i);
            trie.add(key, TestValue::new(i as u64));
        }

        // Assert
        assert_eq!(trie.len(), n);

        // verify random access
        assert_eq!(trie.find("key0").unwrap().id, 0);
        assert_eq!(trie.find("key42").unwrap().id, 42);
        assert_eq!(trie.find("key999").unwrap().id, 999);

        // verify all items are accessible
        for i in 0..n {
            let key = format!("key{}", i);
            assert_eq!(trie.find(&key).unwrap().id, i as u64);
        }
    }
}
