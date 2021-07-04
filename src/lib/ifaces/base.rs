use serde::{Deserialize, Serialize};

use crate::{Interface, InterfaceIp, InterfaceState, InterfaceType};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct BaseInterface {
    pub name: String,
    #[serde(skip_serializing)] // Done by enum tag
    pub iface_type: InterfaceType,
    pub state: InterfaceState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv4: Option<InterfaceIp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6: Option<InterfaceIp>,
}

impl BaseInterface {
    pub(crate) fn update(&mut self, other: &Interface) {
        match other {
            Interface::Unknown(base_iface) => self.update_full(base_iface),
            _ => {
                println!(
                    "BUG: BaseInterface::update() got unexpect other {:?}",
                    other
                )
            }
        }
    }

    pub(crate) fn update_full(&mut self, other: &BaseInterface) {
        self.name = other.name.clone();
        if other.iface_type != InterfaceType::Unknown {
            self.iface_type = other.iface_type.clone();
        }
        if other.state != InterfaceState::Unknown {
            self.state = other.state.clone();
        }
        if let Some(ref other_ipv4) = other.ipv4 {
            if let Some(ref mut self_ipv4) = self.ipv4 {
                self_ipv4.update(other_ipv4);
            } else {
                self.ipv4 = other.ipv4.clone();
            }
        }

        if let Some(ref other_ipv6) = other.ipv6 {
            if let Some(ref mut self_ipv6) = self.ipv6 {
                self_ipv6.update(other_ipv6);
            } else {
                self.ipv6 = other.ipv6.clone();
            }
        }
    }
}
