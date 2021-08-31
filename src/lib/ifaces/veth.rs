use serde::{Deserialize, Serialize};

use crate::{BaseInterface, InterfaceType};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VethInterface {
    #[serde(flatten)]
    pub base: BaseInterface,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub veth: Option<VethConfig>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct VethConfig {
    pub peer: String,
}

impl Default for VethInterface {
    fn default() -> Self {
        Self {
            base: BaseInterface {
                iface_type: InterfaceType::Veth,
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

impl VethInterface {
    pub(crate) fn update(&mut self, other_iface: &VethInterface) {
        // TODO: this should be done by Trait
        self.base.update(&other_iface.base);
        if let Some(_) = other_iface.veth {
            self.veth = other_iface.veth.clone();
        }
    }

    pub(crate) fn pre_verify_cleanup(&mut self) {
        self.base.pre_verify_cleanup();
        self.base.iface_type = InterfaceType::Ethernet;
    }
}
