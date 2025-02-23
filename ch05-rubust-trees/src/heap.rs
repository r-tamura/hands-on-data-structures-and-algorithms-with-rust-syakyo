use crate::iot::MessageNotification;

#[derive(Default)]
pub struct MessageChecker {
    heap: HeapTree<MessageNotification>,
}

impl MessageChecker {
    pub fn length(&self) -> usize {
        self.heap.length()
    }

    pub fn add(&mut self, notification: MessageNotification) {
        self.heap.add(notification);
    }

    pub fn pop(&mut self) -> Option<MessageNotification> {
        self.heap.pop()
    }
}

#[derive(Debug)]
struct HeapTree<T: Ord> {
    heap: Vec<T>,
}

impl<T: Ord> Default for HeapTree<T> {
    fn default() -> Self {
        HeapTree { heap: Vec::new() }
    }
}

impl<T: Ord> HeapTree<T> {
    fn parent(&self, index: usize) -> Option<usize> {
        if index == 0 {
            return None;
        }
        Some((index - 1) / 2)
    }

    fn is_higher_priority(&self, i1: usize, i2: usize) -> bool {
        self.heap[i1] >= self.heap[i2]
    }

    fn get_largest_child(&self, index: usize) -> usize {
        let left = index * 2;
        let right = index * 2 + 1;
        if self.is_higher_priority(left, right) {
            left
        } else {
            right
        }
    }

    fn bubble_up(&mut self, index: usize) {
        let mut current = index;
        while let Some(parent) = self.parent(current) {
            if self.is_higher_priority(current, parent) {
                self.heap.swap(current, parent);
                current = parent;
            } else {
                break;
            }
        }
    }

    pub fn length(&self) -> usize {
        self.heap.len()
    }

    pub fn add(&mut self, v: T) {
        // Vecへ追加
        self.heap.push(v);

        // ヒープ再構築
        // メッセージ数が多いデバイスを優先する
        self.bubble_up(self.length() - 1);
    }

    pub fn bubble_down(&mut self, index: usize) {
        let mut current = index;
        while (current * 2) + 1 < self.length() {
            let largest_child = self.get_largest_child(current);
            // 親ノードが子ノードよりも優先度が高い場合はバブルダウンを終了
            if self.is_higher_priority(current, largest_child) {
                break;
            } else {
                self.heap.swap(current, largest_child);
                current = largest_child;
            }
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.length() == 0 {
            None
        } else {
            // vecの最後の要素が先頭に移動する
            let result = self.heap.swap_remove(0);
            self.bubble_down(1);
            Some(result)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use env_logger;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_no_notification() {
        init();
        let mut checker = MessageChecker::default();
        assert_eq!(checker.length(), 0);
        let notification = checker.pop();
        assert_eq!(notification, None);
    }

    #[test]
    fn test_single_notification() {
        init();
        let mut checker = MessageChecker::default();
        let device = crate::iot::IoTDevice::new(1, "", "");
        checker.add(MessageNotification::new(1, device));
        assert_eq!(checker.length(), 1);
        let notification = checker.pop();
        assert_eq!(notification.unwrap().message_count, 1);
        assert_eq!(checker.length(), 0);
    }

    #[test]
    fn test_multiple_notifications() {
        init();
        let mut checker = MessageChecker::default();
        let device = crate::iot::IoTDevice::new(1, "", "");
        checker.add(MessageNotification::new(1, device.clone()));
        checker.add(MessageNotification::new(2, device.clone()));
        checker.add(MessageNotification::new(3, device.clone()));
        assert_eq!(checker.length(), 3);
        let notification = checker.pop();
        assert_eq!(
            notification.unwrap().message_count,
            3,
            "メッセージ数が1番多い通知"
        );
        assert_eq!(checker.length(), 2, "要素は2つになる");
        let notification = checker.pop();
        assert_eq!(
            notification.unwrap().message_count,
            2,
            "メッセージ数が2番目に多い通知"
        );
        assert_eq!(checker.length(), 1, "要素は1つになる");
        let notification = checker.pop();
        assert_eq!(
            notification.unwrap().message_count,
            1,
            "メッセージ数が3番目に多い通知"
        );
        assert_eq!(checker.length(), 0);
    }
}
