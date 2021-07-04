use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    nispor::nispor_retrieve,
    nm::{
        nm_apply, nm_checkpoint_create, nm_checkpoint_destroy,
        nm_checkpoint_rollback, nm_retrieve,
    },
    Interface, Interfaces, NmstateError,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct NetworkState {
    pub interfaces: Option<Interfaces>,
}

impl NetworkState {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn append_interface_data(&mut self, iface: Interface) {
        if let Some(ref mut ifaces) = self.interfaces {
            ifaces.push(iface);
        } else {
            let mut interfaces = Interfaces::new();
            interfaces.push(iface);
            self.interfaces = Some(interfaces);
        }
    }

    pub fn interfaces(&self) -> Option<&[Interface]> {
        match &self.interfaces {
            Some(ifaces) => Some(ifaces.as_slice()),
            None => None,
        }
    }

    pub fn retrieve() -> Result<Self, NmstateError> {
        let mut state = nispor_retrieve()?;
        let nm_state = nm_retrieve()?;
        // TODO: Priority handling
        state.update_state(&nm_state)?;
        Ok(state)
    }

    pub fn apply(&self) -> Result<(), NmstateError> {
        // TODO: Verify
        with_nm_checkpoint(
            || nm_apply(self)
        )
    }

    fn update_state(&mut self, other: &Self) -> Result<(), NmstateError> {
        if let Some(ref mut self_ifaces) = self.interfaces {
            if let Some(other_ifaces) = &other.interfaces {
                self_ifaces.update(other_ifaces)?;
            }
        } else {
            self.interfaces = other.interfaces.clone();
        }
        Ok(())
    }

    pub fn gen_conf(
        &self,
    ) -> Result<HashMap<String, Vec<String>>, NmstateError> {
        todo!()
    }
}

fn with_nm_checkpoint<T>(func: T) -> Result<(), NmstateError>
where
    T: FnOnce() -> Result<(), NmstateError>,
{
    let checkpoint = nm_checkpoint_create()?;
    match func() {
        Ok(()) => nm_checkpoint_destroy(&checkpoint),
        Err(e) => {
            if let Err(e) = nm_checkpoint_rollback(&checkpoint) {
                eprintln!("nm_checkpoint_rollback() failed: {}", e);
            }
            Err(e)
        }
    }
}
