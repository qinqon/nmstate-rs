use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetIface {
    pub name: String,
    #[serde(rename = "type")]
    #[serde(default)]
    pub iface_type: String,
}

impl Default for NetIface {
    fn default() -> Self {
        NetIface {
            name: "".into(),
            iface_type: "unknown".into(),
        }
    }
}
