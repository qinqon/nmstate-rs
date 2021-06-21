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

use std::ffi::CString;
use std::fs::File;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;

use crate::dbus::NmDbus;
use crate::error::{ErrorKind, NmError};

pub struct NmApi<'a> {
    dbus: NmDbus<'a>,
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

    pub fn checkpoint_rollback(&self, checkpoint: &str) -> Result<(), NmError> {
        self.dbus.checkpoint_rollback(checkpoint)
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

    // TODO:
    //  * Ask user to provide NmConnection struct instead of content and
    //    file_name
    //  * Support cusomized folder
    //  * Return UUID of connection created
    //  * Checkpoint rollback will not remove the newly created file.
    //    Need a workaround.
    pub fn add_connection(
        &self,
        file_name: &str,
        content: &str,
    ) -> Result<(), NmError> {
        let file_path = format!(
            "/etc/NetworkManager/system-connections/{}.nmconnection",
            file_name
        );
        let mut fd = File::create(&file_path)?;
        fd.write_all(content.as_bytes())?;
        let metadata = fd.metadata()?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o600);
        drop(fd);
        chown_to_root(&file_path)?;
        Ok(())
    }

    pub fn reload_connections(&self) -> Result<(), NmError> {
        self.dbus.reload_connections()
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

fn chown_to_root(file_path: &str) -> Result<(), NmError> {
    let file_path_cstring = match CString::new(file_path) {
        Ok(f) => f,
        Err(e) => {
            return Err(NmError::new(
                ErrorKind::Bug,
                format!(
                    "BUG: Failed to create CString from {}: {}",
                    file_path, e
                ),
            ));
        }
    };
    let rc = unsafe { libc::chown(file_path_cstring.as_ptr(), 0, 0) };
    if rc == 0 {
        Ok(())
    } else {
        Err(NmError::new(
            ErrorKind::Bug,
            format!("BUG: libc::chown failed for {}: errno {}", file_path, rc),
        ))
    }
}
