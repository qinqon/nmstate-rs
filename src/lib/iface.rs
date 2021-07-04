use serde::{Deserialize, Serialize};

use crate::{
    ifaces::{BaseInterface, LinuxBridgeInterface},
    ErrorKind, NmstateError,
};

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
// TODO: above line is just for temp use, should use `#[serde(untagged)]`
// and customerize deserilization.
#[serde(rename_all = "kebab-case")]
pub enum Interface {
    LinuxBridge(LinuxBridgeInterface),
    Unknown(BaseInterface),
}

impl Interface {
    pub fn name(&self) -> &str {
        match self {
            Self::LinuxBridge(iface) => iface.base.name.as_str(),
            Self::Unknown(iface) => iface.name.as_str(),
        }
    }
    pub fn iface_type(&self) -> InterfaceType {
        match self {
            Self::LinuxBridge(iface) => iface.base.iface_type.clone(),
            Self::Unknown(iface) => iface.iface_type.clone(),
        }
    }

    pub fn update(&mut self, other: &Interface) {
        match self {
            Self::LinuxBridge(iface) => iface.update(other),
            Self::Unknown(iface) => iface.update(other),
        }
    }
}

impl Default for Interface {
    fn default() -> Self {
        Interface::Unknown(BaseInterface::default())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Interfaces {
    #[serde(flatten)]
    ifaces: Vec<Interface>,
}

impl Interfaces {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn as_slice(&self) -> &[Interface] {
        self.ifaces.as_slice()
    }

    pub fn to_vec(&self) -> Vec<Interface> {
        self.ifaces.clone()
    }

    pub fn push(&mut self, iface: Interface) {
        self.ifaces.push(iface);
    }

    pub fn update(&mut self, other: &Self) -> Result<(), NmstateError> {
        let mut new_ifaces: Vec<&Interface> = Vec::new();
        for other_iface in &other.ifaces {
            match self
                .get_iface_mut(other_iface.name(), other_iface.iface_type())?
            {
                Some(self_iface) => {
                    self_iface.update(other_iface);
                }
                None => {
                    new_ifaces.push(other_iface);
                }
            }
        }
        Ok(())
    }

    // When iface has valid interface type, we do extact match to search
    // the ifaces.
    // When iface is holding unknown interface, if there is only one interface
    // in ifaces, we take it, otherwise, return Error.
    // When no matching found, we return None
    fn get_iface_mut(
        &mut self,
        iface_name: &str,
        iface_type: InterfaceType,
    ) -> Result<Option<&mut Interface>, NmstateError> {
        let mut found_ifaces: Vec<&mut Interface> = Vec::new();
        for self_iface in self.ifaces.as_mut_slice() {
            if self_iface.name() == iface_name
                && (iface_type == InterfaceType::Unknown
                    || iface_type == self_iface.iface_type())
            {
                found_ifaces.push(self_iface);
            }
        }

        if found_ifaces.len() > 1 {
            Err(NmstateError::new(
                ErrorKind::InvalidArgument,
                format!(
                    "Cannot match unknown type interfae {} against \
                    multiple interfaces holding the same name",
                    iface_name
                ),
            ))
        } else if found_ifaces.len() == 1 {
            Ok(Some(found_ifaces.remove(0)))
        } else {
            Ok(None)
        }
    }
}
