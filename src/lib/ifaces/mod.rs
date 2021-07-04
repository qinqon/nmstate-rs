mod base;
mod ethernet;
mod linux_bridge;

pub use base::*;
pub use ethernet::*;
pub use linux_bridge::*;

///////////////////////////////////////////////////////////////////////

use serde::{Deserialize, Deserializer, Serialize};

use crate::{ErrorKind, Interface, InterfaceType, NmstateError};

#[derive(Clone, Debug, PartialEq, Serialize, Default)]
pub struct Interfaces {
    #[serde(flatten)]
    ifaces: Vec<Interface>,
}

impl<'de> Deserialize<'de> for Interfaces {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut ifaces =
            <Vec<Interface> as Deserialize>::deserialize(deserializer)?;
        for iface in &mut ifaces {
            iface.tidy_up()
        }
        Ok(Interfaces { ifaces })
    }
}

impl Interfaces {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn as_slice(&self) -> &[Interface] {
        self.ifaces.as_slice()
    }

    pub fn to_vec(&self) -> Vec<Interface> {
        self.ifaces.clone()
    }

    pub fn push(&mut self, iface: Interface) {
        self.ifaces.push(iface);
    }

    pub fn update(&mut self, other: &Self) -> Result<(), NmstateError> {
        let mut new_ifaces: Vec<&Interface> = Vec::new();
        for other_iface in &other.ifaces {
            match self
                .get_iface_mut(other_iface.name(), other_iface.iface_type())?
            {
                Some(self_iface) => {
                    self_iface.update(other_iface);
                }
                None => {
                    new_ifaces.push(other_iface);
                }
            }
        }
        Ok(())
    }

    // When iface has valid interface type, we do extact match to search
    // the ifaces.
    // When iface is holding unknown interface, if there is only one interface
    // in ifaces, we take it, otherwise, return Error.
    // When no matching found, we return None
    // TODO: Handle OVS same name issue
    fn get_iface_mut(
        &mut self,
        iface_name: &str,
        iface_type: InterfaceType,
    ) -> Result<Option<&mut Interface>, NmstateError> {
        let mut found_ifaces: Vec<&mut Interface> = Vec::new();
        for self_iface in self.ifaces.as_mut_slice() {
            if self_iface.name() == iface_name
                && (iface_type == InterfaceType::Unknown
                    || iface_type == self_iface.iface_type())
            {
                found_ifaces.push(self_iface);
            }
        }

        if found_ifaces.len() > 1 {
            Err(NmstateError::new(
                ErrorKind::InvalidArgument,
                format!(
                    "Cannot match unknown type interfae {} against \
                    multiple interfaces holding the same name",
                    iface_name
                ),
            ))
        } else if found_ifaces.len() == 1 {
            Ok(Some(found_ifaces.remove(0)))
        } else {
            Ok(None)
        }
    }
}
