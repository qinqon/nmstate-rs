use serde::{Deserialize, Serialize};

use crate::{
    state::get_json_value_difference, BaseInterface, ErrorKind,
    EthernetInterface, LinuxBridgeInterface, NmstateError, VethInterface,
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

impl std::fmt::Display for InterfaceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                InterfaceType::Bond => "bond",
                InterfaceType::LinuxBridge => "linuxbridge",
                InterfaceType::Dummy => "dummy",
                InterfaceType::Ethernet => "ethernet",
                InterfaceType::Loopback => "loopback",
                InterfaceType::MacVlan => "macvlan",
                InterfaceType::MacVtap => "macvtap",
                InterfaceType::OvsInterface => "ovsinterface",
                InterfaceType::Tun => "tun",
                InterfaceType::Veth => "veth",
                InterfaceType::Vlan => "vlan",
                InterfaceType::Vrf => "vrf",
                InterfaceType::Vxlan => "vxlan",
                InterfaceType::Unknown => "unknown",
                InterfaceType::Other(ref s) => s,
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InterfaceState {
    Up,
    Down,
    Absent,
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
            "absent" => Self::Absent,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct UnknownInterface {
    #[serde(flatten)]
    pub base: BaseInterface,
}

impl UnknownInterface {
    pub fn new(base: BaseInterface) -> Self {
        Self { base }
    }

    pub(crate) fn update(&mut self, other_iface: &UnknownInterface) {
        // TODO: this should be done by Trait
        self.base.update(&other_iface.base);
    }

    pub(crate) fn pre_verify_cleanup(&mut self) {
        self.base.pre_verify_cleanup();
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
    Veth(VethInterface),
    Unknown(UnknownInterface),
}

impl Interface {
    pub fn name(&self) -> &str {
        match self {
            Self::LinuxBridge(iface) => iface.base.name.as_str(),
            Self::Ethernet(iface) => iface.base.name.as_str(),
            Self::Veth(iface) => iface.base.name.as_str(),
            Self::Unknown(iface) => iface.base.name.as_str(),
        }
    }

    pub(crate) fn is_userspace(&self) -> bool {
        false
    }

    pub(crate) fn set_iface_type(&mut self, iface_type: InterfaceType) {
        self.base_iface_mut().iface_type = iface_type;
    }

    pub fn iface_type(&self) -> InterfaceType {
        match self {
            Self::LinuxBridge(iface) => iface.base.iface_type.clone(),
            Self::Ethernet(iface) => iface.base.iface_type.clone(),
            Self::Veth(iface) => iface.base.iface_type.clone(),
            Self::Unknown(iface) => iface.base.iface_type.clone(),
        }
    }

    pub fn is_up(&self) -> bool {
        self.base_iface().state == InterfaceState::Up
    }

    pub fn is_absent(&self) -> bool {
        self.base_iface().state == InterfaceState::Absent
    }

    pub fn base_iface(&self) -> &BaseInterface {
        match self {
            Self::LinuxBridge(iface) => &iface.base,
            Self::Ethernet(iface) => &iface.base,
            Self::Veth(iface) => &iface.base,
            Self::Unknown(iface) => &iface.base,
        }
    }

    pub(crate) fn base_iface_mut(&mut self) -> &mut BaseInterface {
        match self {
            Self::LinuxBridge(iface) => &mut iface.base,
            Self::Ethernet(iface) => &mut iface.base,
            Self::Veth(iface) => &mut iface.base,
            Self::Unknown(iface) => &mut iface.base,
        }
    }

    pub fn ports(&self) -> Option<Vec<&str>> {
        match self {
            Self::LinuxBridge(iface) => iface.ports(),
            _ => None,
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
            Self::Veth(iface) => {
                if let Self::Veth(other_iface) = other {
                    iface.update(other_iface);
                } else {
                    eprintln!(
                        "BUG: Don't know how to update veth iface \
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

    // TODO: This is not required if we create our own Deserializer.
    pub(crate) fn tidy_up(&mut self) {
        match self {
            Self::LinuxBridge(iface) => {
                iface.base.iface_type = InterfaceType::LinuxBridge
            }
            Self::Veth(iface) => iface.base.iface_type = InterfaceType::Veth,
            Self::Ethernet(iface) => {
                iface.base.iface_type = InterfaceType::Ethernet
            }
            _ => (),
        }
    }

    pub(crate) fn pre_verify_cleanup(&mut self) {
        match self {
            Self::LinuxBridge(ref mut iface) => {
                iface.pre_verify_cleanup();
            }
            Self::Ethernet(ref mut iface) => {
                iface.pre_verify_cleanup();
            }
            Self::Veth(ref mut iface) => {
                iface.pre_verify_cleanup();
            }
            Self::Unknown(ref mut iface) => {
                iface.pre_verify_cleanup();
            }
        }
    }

    pub(crate) fn verify(&self, current: &Self) -> Result<(), NmstateError> {
        let mut self_clone = self.clone();
        self_clone.pre_verify_cleanup();
        let self_value = serde_json::to_value(&self_clone)?;

        let mut current_clone = current.clone();
        current_clone.pre_verify_cleanup();
        let current_value = serde_json::to_value(&current_clone)?;

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
        Interface::Unknown(UnknownInterface::default())
    }
}
