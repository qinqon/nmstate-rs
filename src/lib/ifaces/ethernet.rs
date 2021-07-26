use serde::{Deserialize, Serialize};

use crate::{BaseInterface, InterfaceType};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EthernetInterface {
    #[serde(flatten)]
    pub base: BaseInterface,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ethernet: Option<EthernetConfig>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct EthernetConfig {}

impl Default for EthernetInterface {
    fn default() -> Self {
        Self {
            base: BaseInterface {
                iface_type: InterfaceType::Ethernet,
                ..Default::default()
            },
            ethernet: None,
        }
    }
}

impl EthernetInterface {
    pub(crate) fn update(&mut self, other_iface: &EthernetInterface) {
        // TODO: this should be done by Trait
        self.base.update(&other_iface.base);
    }

    pub(crate) fn pre_verify_cleanup(&mut self) {
        self.base.pre_verify_cleanup();
    }
}
