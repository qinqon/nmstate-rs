use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IfaceType {
    Ethernet,
    Unknown,
    Other(String),
}

impl Default for IfaceType {
    fn default() -> Self {
        Self::Unknown
    }
}

impl From<&str> for IfaceType {
    fn from(s: &str) -> Self {
        match s {
            "ethernet" => IfaceType::Ethernet,
            _ => IfaceType::Other(s.to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetIface {
    pub name: String,
    #[serde(rename = "type")]
    #[serde(default)]
    pub iface_type: IfaceType,
}

impl Default for NetIface {
    fn default() -> Self {
        NetIface {
            name: "".into(),
            iface_type: IfaceType::Unknown,
        }
    }
}
