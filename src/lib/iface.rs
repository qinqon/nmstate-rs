use serde::{Deserialize, Serialize};

use crate::{
    state::get_json_value_difference, BaseInterface, ErrorKind,
    EthernetInterface, LinuxBridgeInterface, NmstateError,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    Ethernet(EthernetInterface),
    Unknown(BaseInterface),
}

impl Interface {
    pub fn name(&self) -> &str {
        match self {
            Self::LinuxBridge(iface) => iface.base.name.as_str(),
            Self::Ethernet(iface) => iface.base.name.as_str(),
            Self::Unknown(iface) => iface.name.as_str(),
        }
    }

    pub fn iface_type(&self) -> InterfaceType {
        match self {
            Self::LinuxBridge(iface) => iface.base.iface_type.clone(),
            Self::Ethernet(iface) => iface.base.iface_type.clone(),
            Self::Unknown(iface) => iface.iface_type.clone(),
        }
    }

    pub fn base_iface(&self) -> &BaseInterface {
        match self {
            Self::LinuxBridge(iface) => &iface.base,
            Self::Ethernet(iface) => &iface.base,
            Self::Unknown(iface) => &iface,
        }
    }

    pub fn update(&mut self, other: &Interface) {
        match self {
            Self::LinuxBridge(iface) => {
                if let Self::LinuxBridge(other_iface) = other {
                    iface.update(other_iface);
                } else {
                    eprintln!(
                        "BUG: Don't know how to update linux bridge iface \
                        with {:?}",
                        other
                    );
                }
            }
            Self::Ethernet(iface) => {
                if let Self::Ethernet(other_iface) = other {
                    iface.update(other_iface);
                } else {
                    eprintln!(
                        "BUG: Don't know how to update ethernet iface \
                        with {:?}",
                        other
                    );
                }
            }
            Self::Unknown(iface) => {
                if let Self::Unknown(other_iface) = other {
                    iface.update(other_iface);
                } else {
                    eprintln!(
                        "BUG: Don't know how to update unknown iface \
                        with {:?}",
                        other
                    );
                }
            }
        }
    }

    pub(crate) fn tidy_up(&mut self) {
        match self {
            Self::LinuxBridge(iface) => {
                iface.base.iface_type = InterfaceType::LinuxBridge
            }
            Self::Ethernet(iface) => {
                iface.base.iface_type = InterfaceType::Ethernet
            }
            _ => (),
        }
    }

    pub(crate) fn ports(&self) -> Vec<String> {
        if let Self::LinuxBridge(iface) = self {
            iface.ports()
        } else {
            Vec::new()
        }
    }

    pub(crate) fn clone_for_verify(&self) -> Self {
        let mut iface_clone = self.clone();

        match iface_clone {
            Self::LinuxBridge(ref mut iface) => {
                iface.pre_verify_cleanup();
            },
            Self::Ethernet(ref mut iface) => {
                iface.pre_verify_cleanup();
            },
            Self::Unknown(ref mut iface) => {
                iface.pre_verify_cleanup();
            }
        }

        iface_clone
    }

    pub(crate) fn verify(&self, current: &Self) -> Result<(), NmstateError> {
        let self_value = serde_json::to_value(&self.clone_for_verify())?;
        let current_value = serde_json::to_value(&current.clone_for_verify())?;
        if let Some(diff_value) = get_json_value_difference(
            format!("{}.interface", self.name()),
            &self_value,
            &current_value,
        ) {
            Err(NmstateError::new(
                ErrorKind::VerificationError,
                format!("Verification failure: {}", diff_value.to_string()),
            ))
        } else {
            Ok(())
        }
    }
}

impl Default for Interface {
    fn default() -> Self {
        Interface::Unknown(BaseInterface::default())
    }
}
