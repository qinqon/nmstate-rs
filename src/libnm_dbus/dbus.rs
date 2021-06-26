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

use std::collections::HashMap;
use std::convert::TryFrom;

use crate::{
    connection::{
        NmConnection, NmConnectionDbusOwnedValue, NmConnectionDbusValue,
    },
    dbus_proxy::{NetworkManagerProxy, NetworkManagerSettingProxy},
    error::{ErrorKind, NmError},
};

const NM_CHECKPOINT_CREATE_FLAG_DELETE_NEW_CONNECTIONS: u32 = 0x02;
const NM_CHECKPOINT_CREATE_FLAG_DISCONNECT_NEW_DEVICES: u32 = 0x04;

const CHECKPOINT_TMO: u32 = 30;

const OBJ_PATH_NULL_STR: &str = "/";

const NM_DBUS_INTERFACE_ROOT: &str = "org.freedesktop.NetworkManager";
const NM_DBUS_INTERFACE_AC: &str =
    "org.freedesktop.NetworkManager.Connection.Active";
const NM_DBUS_INTERFACE_SETTING: &str =
    "org.freedesktop.NetworkManager.Settings.Connection";

const NM_SETTINGS_CREATE2_FLAGS_TO_DISK: u32 = 1;
const NM_SETTINGS_CREATE2_FLAGS_BLOCK_AUTOCONNECT: u32 = 32;

const NM_SETTINGS_UPDATE2_FLAGS_TO_DISK: u32 = 1;
const NM_SETTINGS_UPDATE2_FLAGS_BLOCK_AUTOCONNECT: u32 = 32;

pub(crate) struct NmDbus<'a> {
    connection: zbus::Connection,
    proxy: NetworkManagerProxy<'a>,
    setting_proxy: NetworkManagerSettingProxy<'a>,
}

impl<'a> NmDbus<'a> {
    pub(crate) fn new() -> Result<Self, NmError> {
        let connection = zbus::Connection::new_system()?;
        let proxy = NetworkManagerProxy::new(&connection)?;
        let setting_proxy = NetworkManagerSettingProxy::new(&connection)?;

        Ok(Self {
            connection,
            proxy,
            setting_proxy,
        })
    }

    pub(crate) fn version(&self) -> Result<String, NmError> {
        Ok(self.proxy.version()?)
    }

    pub(crate) fn checkpoint_create(&self) -> Result<String, NmError> {
        match self.proxy.checkpoint_create(
            &[],
            CHECKPOINT_TMO,
            NM_CHECKPOINT_CREATE_FLAG_DELETE_NEW_CONNECTIONS
                | NM_CHECKPOINT_CREATE_FLAG_DISCONNECT_NEW_DEVICES,
        ) {
            Ok(cp) => Ok(obj_path_to_string(cp)),
            Err(e) => {
                Err(if let zbus::Error::MethodError(ref error_type, ..) = e {
                    if error_type
                        == "org.freedesktop.NetworkManager.InvalidArguments"
                    {
                        NmError::new(
                            ErrorKind::CheckpointConflict,
                            "Another checkpoint exists, \
                            please wait its timeout or destroy it"
                                .to_string(),
                        )
                    } else {
                        e.into()
                    }
                } else {
                    e.into()
                })
            }
        }
    }

    pub(crate) fn checkpoint_destroy(
        &self,
        checkpoint: &str,
    ) -> Result<(), NmError> {
        Ok(self
            .proxy
            .checkpoint_destroy(&str_to_obj_path(checkpoint)?)?)
    }

    pub(crate) fn checkpoint_rollback(
        &self,
        checkpoint: &str,
    ) -> Result<(), NmError> {
        self.proxy
            .checkpoint_rollback(&str_to_obj_path(checkpoint)?)?;
        Ok(())
    }

    pub(crate) fn get_connection_by_uuid(
        &self,
        uuid: &str,
    ) -> Result<String, NmError> {
        match self.setting_proxy.get_connection_by_uuid(uuid) {
            Ok(c) => Ok(obj_path_to_string(c)),
            Err(e) => {
                if let zbus::Error::MethodError(ref error_type, ..) = e {
                    if error_type
                        == &format!(
                            "{}.Settings.InvalidConnection",
                            NM_DBUS_INTERFACE_ROOT,
                        )
                    {
                        Err(NmError::new(
                            ErrorKind::NotFound,
                            format!("Connection with UUID {} not found", uuid),
                        ))
                    } else {
                        Err(e.into())
                    }
                } else {
                    Err(e.into())
                }
            }
        }
    }

    pub(crate) fn activate(&self, nm_conn: &str) -> Result<(), NmError> {
        self.proxy.activate_connection(
            &str_to_obj_path(nm_conn)?,
            &str_to_obj_path(OBJ_PATH_NULL_STR)?,
            &str_to_obj_path(OBJ_PATH_NULL_STR)?,
        )?;
        Ok(())
    }

    pub(crate) fn active_connections(&self) -> Result<Vec<String>, NmError> {
        let mut ret = Vec::new();
        for nm_ac in self.proxy.active_connections()? {
            ret.push(obj_path_to_string(nm_ac))
        }
        Ok(ret)
    }

    pub(crate) fn deactivate(&self, nm_ac: &str) -> Result<(), NmError> {
        Ok(self.proxy.deactivate_connection(&str_to_obj_path(nm_ac)?)?)
    }

    pub(crate) fn add_connection(
        &self,
        nm_conn: &NmConnection,
    ) -> Result<(), NmError> {
        let value = nm_conn.to_value()?;
        self.setting_proxy.add_connection2(
            value,
            NM_SETTINGS_CREATE2_FLAGS_TO_DISK
                + NM_SETTINGS_CREATE2_FLAGS_BLOCK_AUTOCONNECT,
            HashMap::new(),
        )?;
        Ok(())
    }

    pub(crate) fn get_nm_ac_uuid(
        &self,
        nm_ac: &str,
    ) -> Result<String, NmError> {
        let proxy = zbus::Proxy::new(
            &self.connection,
            NM_DBUS_INTERFACE_ROOT,
            nm_ac,
            NM_DBUS_INTERFACE_AC,
        )?;
        match proxy.get_property::<String>("Uuid") {
            Ok(uuid) => Ok(uuid),
            Err(e) => Err(NmError::new(
                ErrorKind::Bug,
                format!(
                    "Failed to retrieve UUID of active connection {}: {}",
                    nm_ac, e
                ),
            )),
        }
    }

    pub(crate) fn get_nm_connection(
        &self,
        con_obj_path: &str,
    ) -> Result<NmConnectionDbusOwnedValue, NmError> {
        let proxy = zbus::Proxy::new(
            &self.connection,
            NM_DBUS_INTERFACE_ROOT,
            con_obj_path,
            NM_DBUS_INTERFACE_SETTING,
        )?;
        Ok(proxy.call::<(), NmConnectionDbusOwnedValue>("GetSettings", &())?)
    }

    pub(crate) fn delete_connection(
        &self,
        con_obj_path: &str,
    ) -> Result<(), NmError> {
        let proxy = zbus::Proxy::new(
            &self.connection,
            NM_DBUS_INTERFACE_ROOT,
            con_obj_path,
            NM_DBUS_INTERFACE_SETTING,
        )?;
        Ok(proxy.call::<(), ()>("Delete", &())?)
    }

    pub(crate) fn update_connection(
        &self,
        con_obj_path: &str,
        nm_conn: &NmConnection,
    ) -> Result<(), NmError> {
        let value = nm_conn.to_value()?;
        let proxy = zbus::Proxy::new(
            &self.connection,
            NM_DBUS_INTERFACE_ROOT,
            con_obj_path,
            NM_DBUS_INTERFACE_SETTING,
        )?;
        proxy.call::<(
                NmConnectionDbusValue,
                u32,
                HashMap<&str, zvariant::Value>,
            ), HashMap<String, zvariant::OwnedValue>>(
                "Update2",
                &(
                    value,
                    NM_SETTINGS_UPDATE2_FLAGS_BLOCK_AUTOCONNECT
                        + NM_SETTINGS_UPDATE2_FLAGS_TO_DISK,
                    HashMap::new()
                ),
            )?;
        Ok(())
    }
}

fn str_to_obj_path(obj_path: &str) -> Result<zvariant::ObjectPath, NmError> {
    match zvariant::ObjectPath::try_from(obj_path) {
        Ok(o) => Ok(o),
        Err(e) => Err(NmError::new(
            ErrorKind::InvalidArgument,
            format!("Invalid object path: {}", e),
        )),
    }
}

fn obj_path_to_string(obj_path: zvariant::OwnedObjectPath) -> String {
    obj_path.into_inner().as_str().to_string()
}
