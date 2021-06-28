use serde::{Deserialize, Serialize};

use crate::ifaces::{BaseInterface, LinuxBridgeInterface};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InterfaceType {
    Bond,
    LinuxBridge,
    Dummy,
    Ethernet,
    Loopback,
    MacVlan,
    MacVtap,
    OvsInterface,
    Tun,
    Veth,
    Vlan,
    Vrf,
    Vxlan,
    Unknown,
    Other(String),
}

impl Default for InterfaceType {
    fn default() -> Self {
        Self::Unknown
    }
}

impl From<&str> for InterfaceType {
    fn from(s: &str) -> Self {
        match s {
            "ethernet" => InterfaceType::Ethernet,
            _ => InterfaceType::Other(s.to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InterfaceState {
    Up,
    Down,
    Unknown,
}

impl Default for InterfaceState {
    fn default() -> Self {
        Self::Unknown
    }
}

impl From<&str> for InterfaceState {
    fn from(s: &str) -> Self {
        match s {
            "up" => Self::Up,
            "down" => Self::Down,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
// TODO: above line is just for temp use, use customize
// serilization/deserilization.
#[serde(rename_all = "kebab-case")]
pub enum Interface {
    LinuxBridge(LinuxBridgeInterface),
    Other(BaseInterface),
    Unknown(BaseInterface),
}

impl Interface {
    pub fn name(&self) -> &str {
        match self {
            Self::LinuxBridge(iface) => iface.base.name.as_str(),
            Self::Other(iface) => iface.name.as_str(),
            Self::Unknown(iface) => iface.name.as_str(),
        }
    }
}

impl Default for Interface {
    fn default() -> Self {
        Interface::Unknown(BaseInterface::default())
    }
}
