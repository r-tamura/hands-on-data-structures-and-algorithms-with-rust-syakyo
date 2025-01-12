pub mod red_brack_tree;

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
