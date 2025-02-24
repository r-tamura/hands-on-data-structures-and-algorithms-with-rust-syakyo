use log::debug;
use std::collections::BTreeMap;

enum InsertResult<V> {
    Added,
    Updated(V),
}

enum TrieNode<V> {
    /// 中間ノード。文字列の途中の文字を表し、値は持たない
    ///
    /// 例: "rust"と"rust-lang"という文字列を格納する場合
    /// 'r', 'u', 's'の各文字はInternalノード
    ///
    /// ```text
    /// [I] = Internal node (値なし)
    /// [E] = Entry node (値あり、他の文字への参照も持ちうる)
    ///
    ///      r[I]
    ///      |
    ///      u[I]
    ///      |
    ///      s[I]
    ///      |
    ///      t[E]  <- "rust"の終端。同時に"rust-lang"の途中の文字
    ///      |
    ///      -[I]
    ///      |
    ///      l[I]
    ///      |
    ///      a[I]
    ///      |
    ///      n[I]
    ///      |
    ///      g[E]  <- "rust-lang"の終端
    /// ```
    Internal {
        next: BTreeMap<char, Box<TrieNode<V>>>,
    },
    /// エントリーノード。文字列の最後の文字を表し、値を持つ。
    /// 他の文字列の途中の文字である可能性があるため、nextも持つ
    ///
    /// 例: "rust"と"rust-lang"という文字列を格納する場合
    /// - t[E] はEntryノード（"rust"のエントリー）であり、同時に"rust-lang"の途中の文字
    ///   なので、'-'への参照も持つ
    /// - g[E] はEntryノード（"rust-lang"のエントリー）でnextは空
    Entry {
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
            Self::Internal { next } => next,
            Self::Entry { next, .. } => next,
        }
    }

    fn next_mut(&mut self) -> &mut BTreeMap<char, Box<TrieNode<V>>> {
        match self {
            Self::Internal { next } => next,
            Self::Entry { next, .. } => next,
        }
    }

    fn make_entry(&mut self, value: V) -> InsertResult<V> {
        match self {
            Self::Internal { next } => {
                let next = std::mem::take(next);
                *self = Self::Entry { value, next };
                InsertResult::Added
            }
            Self::Entry {
                value: old_value, ..
            } => {
                let old = std::mem::replace(old_value, value);
                InsertResult::Updated(old)
            }
        }
    }

    /// Entryノードの値を取得します
    /// 値が取得されたEntryノードはInternalノードに変換されます
    fn take_value(&mut self) -> Option<V> {
        match self {
            Self::Internal { .. } => None,
            Self::Entry { next, .. } => {
                let next = std::mem::take(next);
                let temp = std::mem::replace(self, Self::Internal { next });
                if let Self::Entry { value, .. } = temp {
                    Some(value)
                } else {
                    unreachable!()
                }
            }
        }
    }

    fn value(&self) -> Option<&V> {
        match self {
            Self::Internal { .. } => None,
            Self::Entry { value, .. } => Some(value),
        }
    }

    fn is_internal(&self) -> bool {
        matches!(self, Self::Internal { .. })
    }

    fn is_unused(&self) -> bool {
        self.is_internal() && self.next().is_empty()
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

        // 2文字目以降があれば処理
        for &c in chars[1..].iter() {
            let next = current
                .next_mut()
                .entry(c)
                .or_insert_with(|| Box::new(TrieNode::new_internal()));
            current = next;
        }

        // currentは常に最後の文字のノードを指している
        let result = current.make_entry(v);
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

    /// キーに対応する値を削除します
    ///
    /// # 例
    /// ```
    /// # use ch05_rubust_trees::trie::TrieTree;
    /// let mut trie = TrieTree::default();
    /// trie.add("rust".to_string(), 1);
    /// trie.add("rust-lang".to_string(), 2);
    ///
    /// assert_eq!(trie.remove("rust"), Some(1));  // "rust"を削除。"rust-lang"は保持
    /// assert_eq!(trie.find("rust"), None);       // "rust"は見つからない
    /// assert_eq!(trie.find("rust-lang"), Some(&2)); // "rust-lang"はまだ存在
    /// ```
    pub fn remove(&mut self, key: &str) -> Option<V> {
        debug!("[trie::remove] key: {}", key);
        if key.is_empty() {
            return None;
        }

        let chars: Vec<char> = key.chars().collect();
        let mut path: Vec<(usize, char)> = Vec::new();

        // 最初の文字のノード取得
        let first = chars[0];
        let mut current = self.root.get_mut(&first)?;
        path.push((0, first));

        // ノードまで移動しつつパスを記録
        for (i, &c) in chars[1..].iter().enumerate() {
            current = current.next_mut().get_mut(&c)?;
            path.push((i + 1, c));
        }

        // 最後のノードはEntryではなくなるため、Internalに変換
        let value = current.take_value()?;
        self.length -= 1;

        // nextが空でなければ、他の文字列で使用中なのでノードを削除しない
        if !current.next().is_empty() {
            return Some(value);
        }

        // パスを逆順に走査し、未使用のノードを削除
        let mut can_remove_parent = true;

        for (i, c) in path.into_iter().rev() {
            if !can_remove_parent {
                break;
            }
            // 削除が失敗（None）の場合は、それ以上の削除を停止
            let (_, removed) = self.remove_node(&chars, i, c);
            can_remove_parent = removed;
        }

        Some(value)
    }

    fn get_node_at_mut(&mut self, chars: &[char], index: usize) -> Option<&mut Box<TrieNode<V>>> {
        if index == 0 {
            self.root.get_mut(&chars[0])
        } else {
            let mut current = self.root.get_mut(&chars[0])?;
            for &c in chars[1..index].iter() {
                current = current.next_mut().get_mut(&c)?;
            }
            Some(current)
        }
    }

    fn remove_node(&mut self, chars: &[char], index: usize, c: char) -> (Option<V>, bool) {
        if index == 0 {
            if let Some(node) = self.root.get_mut(&c) {
                if node.is_unused() {
                    let value = node.take_value();
                    self.root.remove(&c);
                    return (value, true);
                }
            }
        } else if let Some(parent) = self.get_node_at_mut(chars, index) {
            if let Some(node) = parent.next_mut().get_mut(&c) {
                if node.is_unused() {
                    let value = node.take_value();
                    parent.next_mut().remove(&c);
                    return (value, true);
                }
            }
        }
        (None, false)
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

    #[test]
    fn remove_should_remove_key() {
        // Arrange
        init();
        let mut trie = TrieTree::default();
        trie.add("abc".to_string(), TestValue::new(1));

        // Act
        let removed = trie.remove("abc");

        // Assert
        assert_eq!(trie.len(), 0);
        assert_eq!(removed.unwrap().id, 1);
        assert_eq!(trie.find("abc"), None);
    }

    #[test]
    fn remove_should_keep_other_keys_with_same_prefix() {
        // Arrange
        init();
        let mut trie = TrieTree::default();
        trie.add("rust".to_string(), TestValue::new(1));
        trie.add("rust-lang".to_string(), TestValue::new(2));

        // Act: "rust"を削除
        let removed = trie.remove("rust");

        // Assert: "rust"は削除され、"rust-lang"は保持される
        assert_eq!(trie.len(), 1);
        assert_eq!(removed.unwrap().id, 1);
        assert_eq!(trie.find("rust"), None);
        assert_eq!(trie.find("rust-lang").unwrap().id, 2);
    }

    #[test]
    fn remove_should_cleanup_unused_nodes() {
        // Arrange
        init();
        let mut trie = TrieTree::default();
        trie.add("rust".to_string(), TestValue::new(1));

        // Act: "rust"を削除
        let removed = trie.remove("rust");

        // Assert
        assert_eq!(trie.len(), 0);
        assert_eq!(removed.unwrap().id, 1);
        // 'r', 'u', 's', 't' のノードが全て削除されていることを確認
        assert!(trie.root.is_empty());
    }

    #[test]
    fn remove_should_keep_nodes_used_by_other_keys() {
        // Arrange
        init();
        let mut trie = TrieTree::default();
        trie.add("rust".to_string(), TestValue::new(1));
        trie.add("rust-lang".to_string(), TestValue::new(2));
        trie.add("ruby".to_string(), TestValue::new(3));

        // Act: "rust-lang"を削除
        let removed = trie.remove("rust-lang");

        // Assert: "rust"と"ruby"は保持される
        assert_eq!(trie.len(), 2);
        assert_eq!(removed.unwrap().id, 2);
        assert_eq!(trie.find("rust").unwrap().id, 1);
        assert_eq!(trie.find("ruby").unwrap().id, 3);
        assert_eq!(trie.find("rust-lang"), None);
    }

    #[test]
    fn remove_should_return_none_for_missing_key() {
        // Arrange
        init();
        let mut trie = TrieTree::default();
        trie.add("rust".to_string(), TestValue::new(1));

        // Act
        let removed = trie.remove("not_exists");

        // Assert
        assert_eq!(trie.len(), 1);
        assert_eq!(removed, None);
    }

    #[test]
    fn remove_should_return_none_when_tree_is_empty() {
        // Arrange
        init();
        let mut trie = TrieTree::<TestValue>::default();

        // Act
        let removed = trie.remove("not_exists");

        // Assert
        assert_eq!(trie.len(), 0);
        assert_eq!(removed, None);
    }

    #[test]
    fn remove_should_remove_key_for_single_char_key() {
        // Arrange
        init();
        let mut trie = TrieTree::default();
        trie.add("a".to_string(), TestValue::new(1));

        // Act
        let removed = trie.remove("a");

        // Assert
        assert_eq!(trie.len(), 0);
        assert_eq!(removed.unwrap().id, 1);
        assert_eq!(trie.find("a"), None);
    }

    #[test]
    fn remove_node_should_return_none_for_unused_nodes() {
        // Arrange
        init();
        let mut trie = TrieTree::default();
        trie.add("rust".to_string(), TestValue::new(1));
        trie.add("rust-lang".to_string(), TestValue::new(2));
        // 一度"rust"を削除してInternalノードにする
        trie.remove("rust");
        // Act
        let actual = trie.remove_node(&"rust-lang".chars().collect::<Vec<char>>(), 4, 'l');

        // Act & Assert: 未使用になったノードを削除
        assert_eq!(actual, (None, false));
    }

    #[test]
    fn remove_should_keep_intermediate_values() {
        // Arrange
        init();
        let mut trie = TrieTree::default();
        // "r"と"rust"をこの順で追加（"r"がBranchノードになる）
        trie.add("r".to_string(), TestValue::new(1));
        trie.add("rust".to_string(), TestValue::new(2));

        // Act: "rust"を削除
        let removed = trie.remove("rust");

        // Assert: "rust"は削除され、"r"は保持される
        assert_eq!(removed.unwrap().id, 2);
        assert_eq!(trie.len(), 1);
        assert_eq!(trie.find("r").unwrap().id, 1);
        assert_eq!(trie.find("rust"), None);
    }
}
