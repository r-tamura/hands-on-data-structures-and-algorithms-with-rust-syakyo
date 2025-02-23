#[derive(Clone, Debug, Eq)]
pub struct IoTDevice {
    pub numeriacl_id: u64,
    pub path: String,
    pub address: String,
}

impl IoTDevice {
    pub fn new(id: u64, address: impl Into<String>, path: impl Into<String>) -> IoTDevice {
        IoTDevice {
            numeriacl_id: id,
            address: address.into(),
            path: path.into(),
        }
    }
}

impl PartialEq for IoTDevice {
    fn eq(&self, other: &Self) -> bool {
        self.numeriacl_id == other.numeriacl_id
    }
}

impl PartialOrd for IoTDevice {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.numeriacl_id.cmp(&other.numeriacl_id))
    }
}

impl Ord for IoTDevice {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl std::fmt::Display for IoTDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.numeriacl_id)
    }
}

/// メッセージ通知
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageNotification {
    pub message_count: u64,
    pub device: IoTDevice,
}

impl MessageNotification {
    pub fn new(id: u64, device: IoTDevice) -> MessageNotification {
        MessageNotification {
            message_count: id,
            device,
        }
    }
}

impl PartialOrd for MessageNotification {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.message_count.cmp(&other.message_count))
    }
}

impl Ord for MessageNotification {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Default)]
pub struct DeviceRegistry {
    trie: crate::trie::TrieTree<IoTDevice>,
}

impl DeviceRegistry {
    pub fn add(&mut self, device: IoTDevice) {
        self.trie.add(device.path.clone(), device);
    }

    pub fn find(&self, path: &str) -> Option<&IoTDevice> {
        self.trie.find(path)
    }

    pub fn remove(&mut self, path: &str) {
        self.trie.remove(path);
    }

    pub fn length(&self) -> usize {
        self.trie.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    mod device_registry {
        use super::*;
        #[test]
        fn add_single_char_name_device() {
            // Arrange
            init();
            let mut registry = DeviceRegistry::default();

            // Act
            registry.add(IoTDevice::new(1, "", "a"));

            // Assert
            assert_eq!(registry.length(), 1);
            assert_eq!(registry.find("a").unwrap().numeriacl_id, 1);
        }

        #[test]
        fn should_add_multiple_char_name_device() {
            // Arrange
            init();
            let mut registry = DeviceRegistry::default();

            // Act
            registry.add(IoTDevice::new(1, "", "abc"));

            // Assert
            assert_eq!(registry.length(), 1);
            assert_eq!(registry.find("abc").unwrap().numeriacl_id, 1);
        }

        #[test]
        fn when_same_key_passed_should_update_with_new_device() {
            // Arrange
            init();
            let mut registry = DeviceRegistry::default();
            registry.add(IoTDevice::new(1, "", "abc"));

            // Act
            registry.add(IoTDevice::new(2, "", "abc"));

            // Assert
            assert_eq!(registry.length(), 1);
            assert_eq!(registry.find("abc").unwrap().numeriacl_id, 2);
        }

        #[test]
        fn when_same_prefixed_key_paased_should_add_new_device() {
            // Arrange
            init();
            let mut registry = DeviceRegistry::default();
            registry.add(IoTDevice::new(1, "", "abc"));

            // Act
            registry.add(IoTDevice::new(2, "", "ab"));

            // Assert
            assert_eq!(registry.length(), 2);
            assert_eq!(registry.find("abc").unwrap().numeriacl_id, 1);
            assert_eq!(registry.find("ab").unwrap().numeriacl_id, 2);
        }

        #[test]
        fn when_same_prefixed_but_different_key_passed_should_add_new_device() {
            // Arrange
            init();
            let mut registry = DeviceRegistry::default();
            registry.add(IoTDevice::new(1, "", "abc"));

            // Act
            registry.add(IoTDevice::new(2, "", "abx"));

            // Assert
            assert_eq!(registry.length(), 2);
            assert_eq!(registry.find("abc").unwrap().numeriacl_id, 1);
            assert_eq!(registry.find("abx").unwrap().numeriacl_id, 2);
        }
    }
}
