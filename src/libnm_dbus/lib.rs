// Copyright 2021 Red Hat, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

mod dbus;
mod dbus_proxy;
mod error;

pub use crate::error::{ErrorKind, NmError};
use dbus::NmDbus;

pub struct NmApi<'a> {
    dbus: dbus::NmDbus<'a>,
}

impl<'a> NmApi<'a> {
    pub fn new() -> Result<Self, NmError> {
        Ok(Self {
            dbus: NmDbus::new()?,
        })
    }

    pub fn version(&self) -> Result<String, NmError> {
        self.dbus.version()
    }

    pub fn checkpoint_create(&self) -> Result<String, NmError> {
        self.dbus.checkpoint_create()
    }

    pub fn checkpoint_destroy(&self, checkpoint: &str) -> Result<(), NmError> {
        self.dbus.checkpoint_destroy(checkpoint)
    }

    pub fn activate(&self, uuid: &str) -> Result<(), NmError> {
        let nm_conn = self.dbus.get_connection_by_uuid(uuid)?;
        self.dbus.activate(&nm_conn)
    }

    pub fn deactivate(&self, uuid: &str) -> Result<(), NmError> {
        let nm_ac = get_active_connection_by_uuid(&self.dbus, uuid)?;

        if !nm_ac.is_empty() {
            self.dbus.deactivate(&nm_ac)
        } else {
            Ok(())
        }
    }

    pub fn reapply(_uuid: &str) {
        todo!()
    }

    pub fn add_connection(&self, content: &str) -> Result<String, NmError> {
        todo!()
    }

    pub fn reload_connections(&self) -> Result<(), NmError> {
        todo!()
    }
}

fn get_active_connection_by_uuid(
    dbus: &NmDbus,
    uuid: &str,
) -> Result<String, NmError> {
    let nm_acs = dbus.active_connections()?;

    for nm_ac in nm_acs {
        if dbus.get_nm_ac_uuid(&nm_ac)? == uuid {
            return Ok(nm_ac);
        }
    }
    Ok("".into())
}
