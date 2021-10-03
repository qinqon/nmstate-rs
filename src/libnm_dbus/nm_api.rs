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
    active_connection::NmActiveConnection,
    connection::{NmConnection, NmSettingConnection},
    dbus::NmDbus,
    error::{ErrorKind, NmError},
};

pub struct NmApi<'a> {
    dbus: NmDbus<'a>,
}

const RETRY_INTERVAL_MILLISECOND: u64 = 500;
const RETRY_COUNT: usize = 60;

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
        // Race: Connection might just created
        with_retry(RETRY_INTERVAL_MILLISECOND, RETRY_COUNT, || {
            let nm_conn = self.dbus.get_connection_by_uuid(uuid)?;
            self.dbus.connection_activate(&nm_conn)
        })
    }

    pub fn connection_deactivate(&self, uuid: &str) -> Result<(), NmError> {
        let nm_ac = get_nm_ac_obj_path_by_uuid(&self.dbus, uuid)?;

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

    pub fn nm_connections_get(&self) -> Result<Vec<NmConnection>, NmError> {
        let mut nm_conns = Vec::new();
        for nm_conn_obj_path in self.dbus.nm_conn_obj_paths_get()? {
            nm_conns.push(NmConnection::try_from(
                self.dbus.nm_connection_get(&nm_conn_obj_path)?,
            )?);
        }
        Ok(nm_conns)
    }

    pub fn nm_applied_connections_get(
        &self,
    ) -> Result<Vec<NmConnection>, NmError> {
        let mut nm_conns = Vec::new();
        let nm_devs = self.dbus.nm_dev_obj_paths_get()?;
        for nm_dev in &nm_devs {
            nm_conns.push(
                match self.dbus.nm_dev_applied_connection_get(nm_dev) {
                    Ok(n) => NmConnection::try_from(n)?,
                    Err(_) => {
                        continue;
                    }
                },
            );
        }
        Ok(nm_conns)
    }

    pub fn connection_add(
        &self,
        nm_conn: &NmConnection,
    ) -> Result<(), NmError> {
        if let NmConnection {
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
        if let Ok(con_obj_path) = self.dbus.get_connection_by_uuid(uuid) {
            self.dbus.connection_delete(&con_obj_path)
        } else {
            Ok(())
        }
    }

    pub fn connection_reapply(&self, uuid: &str) -> Result<(), NmError> {
        let nm_conn = self.nm_connection_get(uuid)?;
        if let NmConnection {
            connection:
                Some(NmSettingConnection {
                    iface_name: Some(ref iface_name),
                    ..
                }),
            ..
        } = nm_conn
        {
            let nm_dev_obj_path = self.dbus.nm_dev_obj_path_get(iface_name)?;
            self.dbus.nm_dev_reapply(&nm_dev_obj_path, &nm_conn)
        } else {
            Err(NmError::new(
                ErrorKind::InvalidArgument,
                format!(
                    "Failed to extract interface name from connection {}",
                    uuid
                ),
            ))
        }
    }

    pub fn uuid_gen() -> String {
        // Use Linux random number generator (RNG) to generate UUID
        uuid::Uuid::new_v4().to_hyphenated().to_string()
    }

    pub fn nm_active_connections_get(
        &self,
    ) -> Result<Vec<NmActiveConnection>, NmError> {
        let mut nm_acs = Vec::new();
        let nm_ac_obj_paths = self.dbus.active_connections()?;
        for nm_ac_obj_path in nm_ac_obj_paths {
            nm_acs.push(NmActiveConnection {
                uuid: self.dbus.nm_ac_obj_path_uuid_get(&nm_ac_obj_path)?,
            });
        }
        Ok(nm_acs)
    }

    pub fn checkpoint_timeout_extend(
        &self,
        checkpoint: &str,
        added_time_sec: u32,
    ) -> Result<(), NmError> {
        self.dbus
            .checkpoint_timeout_extend(checkpoint, added_time_sec)
    }
}

fn get_nm_ac_obj_path_by_uuid(
    dbus: &NmDbus,
    uuid: &str,
) -> Result<String, NmError> {
    let nm_ac_obj_paths = dbus.active_connections()?;

    for nm_ac_obj_path in nm_ac_obj_paths {
        if dbus.nm_ac_obj_path_uuid_get(&nm_ac_obj_path)? == uuid {
            return Ok(nm_ac_obj_path);
        }
    }
    Ok("".into())
}

fn with_retry<T>(interval_ms: u64, count: usize, func: T) -> Result<(), NmError>
where
    T: FnOnce() -> Result<(), NmError> + Copy,
{
    let mut cur_count = 0usize;
    while cur_count < count {
        if let Err(e) = func() {
            if cur_count == count - 1 {
                return Err(e);
            } else {
                eprintln!("Retrying on NM dbus failure: {}", e);
                std::thread::sleep(std::time::Duration::from_millis(
                    interval_ms,
                ));
                cur_count += 1;
                continue;
            }
        } else {
            return Ok(());
        }
    }
    Ok(())
}
