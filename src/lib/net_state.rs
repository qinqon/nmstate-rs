use crate::error::NmstateError;
use crate::plugin::NmstatePlugin;
use crate::NetIface;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NetState {
    pub interfaces: Option<Vec<NetIface>>,
}

impl NetState {
    pub fn gen_conf(
        &self,
    ) -> Result<HashMap<String, Vec<String>>, NmstateError> {
        println!("gen_conf {:?}", &self);

        let mut confs = HashMap::new();
        let plugins = NmstatePlugin::load_plugins()?;
        for mut plugin in plugins {
            match plugin.gen_conf(&self) {
                Ok(conf) => {
                    confs.insert(plugin.name.clone(), conf);
                }
                Err(e) => eprintln!(
                    "Plugin {} failed to generate config: {}",
                    &plugin.name, e
                ),
            };
            plugin.stop()
        }
        Ok(confs)
    }
}
