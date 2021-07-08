use serde::{Deserialize, Serialize};

use crate::{BaseInterface, InterfaceType};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinuxBridgeInterface {
    #[serde(flatten)]
    pub base: BaseInterface,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bridge: Option<LinuxBridgeConfig>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct LinuxBridgeConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<LinuxBridgeOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<Vec<LinuxBridgePortConfig>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct LinuxBridgePortConfig {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stp_hairpin_mode: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stp_path_cost: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stp_priority: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct LinuxBridgeOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stp: Option<LinuxBridgeStpOptions>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct LinuxBridgeStpOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

impl Default for LinuxBridgeInterface {
    fn default() -> Self {
        Self {
            base: BaseInterface {
                iface_type: InterfaceType::LinuxBridge,
                ..Default::default()
            },
            bridge: None,
        }
    }
}

impl LinuxBridgeInterface {
    pub(crate) fn update(&mut self, other_iface: &LinuxBridgeInterface) {
        // TODO: this should be done by Trait
        self.base.update(&other_iface.base);
    }

    pub(crate) fn ports(&self) -> Vec<String> {
        let mut ret = Vec::new();
        if let Some(LinuxBridgeConfig {
            port: Some(port_configs),
            ..
        }) = &self.bridge
        {
            for port_config in port_configs {
                ret.push(port_config.name.clone())
            }
        }
        ret
    }
}
