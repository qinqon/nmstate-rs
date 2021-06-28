use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{error::NmstateError, iface::Interface, nispor::nispor_retrieve};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct NetworkState {
    pub interfaces: Option<Vec<Interface>>,
}

impl NetworkState {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn append_interface_data(&mut self, iface: Interface) {
        if let Some(ref mut ifaces) = self.interfaces {
            ifaces.push(iface);
        } else {
            self.interfaces = Some(vec![iface]);
        }
    }

    pub fn interfaces(&self) -> Option<&[Interface]> {
        match &self.interfaces {
            Some(ifaces) => Some(ifaces.as_slice()),
            None => None,
        }
    }

    pub fn retrieve() -> Result<Self, NmstateError> {
        let state = nispor_retrieve()?;
        //let nm_state = nm_retrieve()?;
        //state.merge(&nm_state);
        Ok(state)
    }

    pub fn apply(&self) -> Result<(), NmstateError> {
        todo!()
    }

    // Merged data from other which hold priority over self
    fn merge(&mut self, other: &Self) {
        println!("{:?}, {:?}", self, other);
    }

    pub fn gen_conf(
        &self,
    ) -> Result<HashMap<String, Vec<String>>, NmstateError> {
        todo!()
    }
}
