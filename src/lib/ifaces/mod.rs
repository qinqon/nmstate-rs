mod base;
mod ethernet;
mod linux_bridge;

pub use base::*;
pub use ethernet::*;
pub use linux_bridge::*;

///////////////////////////////////////////////////////////////////////

use std::collections::HashMap;

use serde::{
    ser::SerializeSeq, Deserialize, Deserializer, Serialize, Serializer,
};

use crate::{ErrorKind, Interface, InterfaceType, NmstateError};

#[derive(Clone, Debug, Default)]
pub struct Interfaces {
    data: HashMap<(String, InterfaceType), Interface>,
}

impl<'de> Deserialize<'de> for Interfaces {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut ret = Self::new();
        let mut ifaces =
            <Vec<Interface> as Deserialize>::deserialize(deserializer)?;
        for iface in &mut ifaces {
            iface.tidy_up();
        }
        for iface in ifaces {
            ret.push(iface)
        }
        Ok(ret)
    }
}

impl Serialize for Interfaces {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let ifaces = self.to_vec();
        let mut seq = serializer.serialize_seq(Some(ifaces.len()))?;
        for iface in ifaces {
            seq.serialize_element(iface)?;
        }
        seq.end()
    }
}

impl Interfaces {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn to_vec(&self) -> Vec<&Interface> {
        let mut ifaces = Vec::new();
        for iface in self.data.values() {
            ifaces.push(iface);
        }
        ifaces
    }

    fn to_vec_mut(&mut self) -> Vec<&mut Interface> {
        let mut ifaces = Vec::new();
        for iface in self.data.values_mut() {
            ifaces.push(iface);
        }
        ifaces
    }

    pub fn push(&mut self, iface: Interface) {
        self.data
            .insert((iface.name().to_string(), iface.iface_type()), iface);
    }

    pub fn update(&mut self, other: &Self) -> Result<(), NmstateError> {
        let mut new_ifaces: Vec<&Interface> = Vec::new();
        let other_ifaces = other.to_vec();
        for other_iface in &other_ifaces {
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
        if iface_type != InterfaceType::Unknown {
            Ok(self.data.get_mut(&(iface_name.to_string(), iface_type)))
        } else {
            // Ensure only one found
            let mut found_ifaces: Vec<&mut Interface> = Vec::new();
            for iface in self.to_vec_mut() {
                if iface.name() == iface_name {
                    found_ifaces.push(iface);
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

    fn get_iface(
        &self,
        iface_name: &str,
        iface_type: InterfaceType,
    ) -> Result<Option<&Interface>, NmstateError> {
        if iface_type != InterfaceType::Unknown {
            Ok(self.data.get(&(iface_name.to_string(), iface_type)))
        } else {
            Err(NmstateError::new(
                ErrorKind::Bug,
                format!(
                    "The Interfaces.get_iface() got unknown interface \
                    type for {}",
                    iface_name
                ),
            ))
        }
    }

    pub(crate) fn verify(
        &self,
        current_ifaces: &Self,
    ) -> Result<(), NmstateError> {
        for iface in self.to_vec() {
            if let Some(cur_iface) =
                current_ifaces.get_iface(iface.name(), iface.iface_type())?
            {
                iface.verify(cur_iface)?;
            } else {
                return Err(NmstateError::new(
                    ErrorKind::VerificationError,
                    format!(
                        "Failed to find desired interface {} {:?}",
                        iface.name(),
                        iface.iface_type()
                    ),
                ));
            }
        }
        Ok(())
    }
}
