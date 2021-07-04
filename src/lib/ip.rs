use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct InterfaceIp {
    pub enabled: bool,
}

impl InterfaceIp {
    pub(crate) fn update(&mut self, other: &Self) {
        self.enabled = other.enabled;
    }
}
