use serde::{Deserialize, Serialize};

use crate::{InterfaceIpv4, InterfaceIpv6, InterfaceState, InterfaceType};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct BaseInterface {
    pub name: String,
    #[serde(skip)]
    pub prop_list: Vec<&'static str>,
    #[serde(rename = "type")]
    pub iface_type: InterfaceType,
    #[serde(default = "default_state")]
    pub state: InterfaceState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mac_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv4: Option<InterfaceIpv4>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6: Option<InterfaceIpv6>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub controller: Option<String>,
    #[serde(skip)]
    pub controller_type: Option<InterfaceType>,
}

impl BaseInterface {
    pub(crate) fn update(&mut self, other: &BaseInterface) {
        if other.prop_list.contains(&"name") {
            self.name = other.name.clone();
        }
        if other.prop_list.contains(&"iface_type")
            && other.iface_type != InterfaceType::Unknown
        {
            self.iface_type = other.iface_type.clone();
        }
        if other.prop_list.contains(&"iface_type")
            && other.state != InterfaceState::Unknown
        {
            self.state = other.state.clone();
        }

        if other.prop_list.contains(&"ipv4") {
            if let Some(ref other_ipv4) = other.ipv4 {
                if let Some(ref mut self_ipv4) = self.ipv4 {
                    self_ipv4.update(other_ipv4);
                } else {
                    self.ipv4 = other.ipv4.clone();
                }
            }
        }

        if other.prop_list.contains(&"ipv6") {
            if let Some(ref other_ipv6) = other.ipv6 {
                if let Some(ref mut self_ipv6) = self.ipv6 {
                    self_ipv6.update(other_ipv6);
                } else {
                    self.ipv6 = other.ipv6.clone();
                }
            }
        }
    }

    pub(crate) fn pre_verify_cleanup(&mut self) {
        if let Some(ref mut ipv4) = self.ipv4 {
            ipv4.pre_verify_cleanup()
        }

        if let Some(ref mut ipv6) = self.ipv6 {
            ipv6.pre_verify_cleanup()
        }
    }

    pub fn can_have_ip(&self) -> bool {
        self.controller == None
    }
}

fn default_state() -> InterfaceState {
    InterfaceState::Up
}
