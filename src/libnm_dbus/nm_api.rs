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

use std::convert::TryFrom;

use crate::{
    connection::{NmConnection, NmSettingConnection},
    dbus::NmDbus,
    error::NmError,
};

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

    pub fn connection_activate(&self, uuid: &str) -> Result<(), NmError> {
        let nm_conn = self.dbus.get_connection_by_uuid(uuid)?;
        self.dbus.connection_activate(&nm_conn)
    }

    pub fn connection_deactivate(&self, uuid: &str) -> Result<(), NmError> {
        let nm_ac = get_active_connection_by_uuid(&self.dbus, uuid)?;

        if !nm_ac.is_empty() {
            self.dbus.connection_deactivate(&nm_ac)
        } else {
            Ok(())
        }
    }

    pub fn nm_connection_get(
        &self,
        uuid: &str,
    ) -> Result<NmConnection, NmError> {
        let con_obj_path = self.dbus.get_connection_by_uuid(uuid)?;
        NmConnection::try_from(self.dbus.nm_connection_get(&con_obj_path)?)
    }

    pub fn connection_add(
        &self,
        nm_conn: &NmConnection,
    ) -> Result<(), NmError> {
        if let &NmConnection {
            connection:
                Some(NmSettingConnection {
                    uuid: Some(ref uuid),
                    ..
                }),
            ..
        } = nm_conn
        {
            if let Ok(con_obj_path) = self.dbus.get_connection_by_uuid(uuid) {
                return self.dbus.connection_update(&con_obj_path, nm_conn);
            }
        };
        self.dbus.connection_add(nm_conn)?;
        Ok(())
    }

    pub fn connection_delete(&self, uuid: &str) -> Result<(), NmError> {
        let con_obj_path = self.dbus.get_connection_by_uuid(uuid)?;
        self.dbus.connection_delete(&con_obj_path)
    }

    pub fn uuid_gen() -> String {
        // Use Linux random number generator (RNG) to generate UUID
        uuid::Uuid::new_v4().to_hyphenated().to_string()
    }
}

fn get_active_connection_by_uuid(
    dbus: &NmDbus,
    uuid: &str,
) -> Result<String, NmError> {
    let nm_acs = dbus.active_connections()?;

    for nm_ac in nm_acs {
        if dbus.nm_ac_get_by_uuid(&nm_ac)? == uuid {
            return Ok(nm_ac);
        }
    }
    Ok("".into())
}
