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
//

use std::collections::HashMap;
use std::convert::TryFrom;

use crate::{
    connection::bridge::{NmSettingBridge, NmSettingBridgePort},
    connection::ip::NmSettingIp,
    dbus_value::{
        value_hash_get_bool, value_hash_get_i32, value_hash_get_string,
    },
    error::NmError,
};

const NM_AUTOCONENCT_PORT_DEFAULT: i32 = -1;
const NM_AUTOCONENCT_PORT_YES: i32 = 1;
const NM_AUTOCONENCT_PORT_NO: i32 = 0;

pub(crate) type NmConnectionDbusOwnedValue = std::collections::HashMap<
    String,
    std::collections::HashMap<String, zvariant::OwnedValue>,
>;

pub(crate) type NmConnectionDbusValue<'a> =
    HashMap<&'a str, HashMap<&'a str, zvariant::Value<'a>>>;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct NmConnection {
    pub connection: Option<NmSettingConnection>,
    pub bridge: Option<NmSettingBridge>,
    pub bridge_port: Option<NmSettingBridgePort>,
    pub ipv4: Option<NmSettingIp>,
    pub ipv6: Option<NmSettingIp>,
}

impl TryFrom<NmConnectionDbusOwnedValue> for NmConnection {
    type Error = NmError;
    fn try_from(
        value: NmConnectionDbusOwnedValue,
    ) -> Result<Self, Self::Error> {
        //eprintln!("connection keys: {:?}", value.keys());
        let mut nm_con: Self = Default::default();
        if let Some(con_value) = value.get("connection") {
            nm_con.connection = Some(NmSettingConnection::try_from(con_value)?);
        }
        if let Some(ipv4_set) = value.get("ipv4") {
            nm_con.ipv4 = Some(NmSettingIp::try_from(ipv4_set)?);
        }
        if let Some(ipv6_set) = value.get("ipv6") {
            nm_con.ipv6 = Some(NmSettingIp::try_from(ipv6_set)?);
        }
        if let Some(br_value) = value.get("bridge") {
            nm_con.bridge = Some(NmSettingBridge::try_from(br_value)?);
        }
        if let Some(br_port_value) = value.get("bridge-port") {
            nm_con.bridge_port =
                Some(NmSettingBridgePort::try_from(br_port_value)?);
        }
        Ok(nm_con)
    }
}

impl NmConnection {
    pub fn iface_name(&self) -> Option<&str> {
        if let Some(NmSettingConnection {
            iface_name: Some(iface_name),
            ..
        }) = &self.connection
        {
            Some(iface_name.as_str())
        } else {
            None
        }
    }

    pub fn iface_type(&self) -> Option<&str> {
        if let Some(NmSettingConnection {
            iface_type: Some(iface_type),
            ..
        }) = &self.connection
        {
            Some(iface_type.as_str())
        } else {
            None
        }
    }

    pub(crate) fn to_value(&self) -> Result<NmConnectionDbusValue, NmError> {
        let mut ret = HashMap::new();
        if let Some(con_set) = &self.connection {
            ret.insert("connection", con_set.to_value()?);
        }
        if let Some(br_set) = &self.bridge {
            ret.insert("bridge", br_set.to_value()?);
        }
        if let Some(br_port_set) = &self.bridge_port {
            ret.insert("bridge-port", br_port_set.to_value()?);
        }
        if let Some(ipv4_set) = &self.ipv4 {
            ret.insert("ipv4", ipv4_set.to_value()?);
        }
        if let Some(ipv6_set) = &self.ipv6 {
            ret.insert("ipv6", ipv6_set.to_value()?);
        }
        Ok(ret)
    }

    pub fn uuid(&self) -> Option<&str> {
        if let Some(nm_conn_set) = &self.connection {
            if let Some(ref uuid) = nm_conn_set.uuid {
                return Some(uuid);
            }
        }
        None
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct NmSettingConnection {
    pub id: Option<String>,
    pub uuid: Option<String>,
    pub iface_type: Option<String>,
    pub iface_name: Option<String>,
    pub controller: Option<String>,
    pub controller_type: Option<String>,
    pub autoconnect: Option<bool>,
    pub autoconnect_ports: Option<bool>,
}

impl TryFrom<&HashMap<String, zvariant::OwnedValue>> for NmSettingConnection {
    type Error = NmError;
    fn try_from(
        value: &HashMap<String, zvariant::OwnedValue>,
    ) -> Result<Self, Self::Error> {
        let autoconnect_ports =
            match value_hash_get_i32(value, "autoconnect-slaves")? {
                Some(NM_AUTOCONENCT_PORT_YES) => Some(true),
                Some(NM_AUTOCONENCT_PORT_NO) => Some(false),
                _ => None,
            };
        Ok(Self {
            id: value_hash_get_string(value, "id")?,
            uuid: value_hash_get_string(value, "uuid")?,
            iface_type: value_hash_get_string(value, "type")?,
            iface_name: value_hash_get_string(value, "interface-name")?,
            controller: value_hash_get_string(value, "master")?,
            controller_type: value_hash_get_string(value, "slave-type")?,
            autoconnect: match value_hash_get_bool(value, "autoconnect")? {
                Some(v) => Some(v),
                // For autoconnect, None means true
                None => Some(true),
            },
            autoconnect_ports,
        })
    }
}

impl NmSettingConnection {
    pub(crate) fn to_value(
        &self,
    ) -> Result<HashMap<&str, zvariant::Value>, NmError> {
        let mut ret = HashMap::new();
        if let Some(v) = &self.id {
            ret.insert("id", zvariant::Value::new(v.as_str()));
        }
        if let Some(v) = &self.uuid {
            ret.insert("uuid", zvariant::Value::new(v.as_str()));
        }
        if let Some(v) = &self.iface_type {
            ret.insert("type", zvariant::Value::new(v.as_str()));
        }
        if let Some(v) = &self.iface_name {
            ret.insert("interface-name", zvariant::Value::new(v.as_str()));
        }
        if let Some(v) = &self.controller {
            ret.insert("master", zvariant::Value::new(v.as_str()));
        }
        if let Some(v) = &self.controller_type {
            ret.insert("slave-type", zvariant::Value::new(v.as_str()));
        }
        ret.insert(
            "autoconnect",
            if let Some(false) = &self.autoconnect {
                zvariant::Value::new(false)
            } else {
                zvariant::Value::new(true)
            },
        );
        ret.insert(
            "autoconnect-slaves",
            match &self.autoconnect_ports {
                Some(true) => zvariant::Value::new(NM_AUTOCONENCT_PORT_YES),
                Some(false) => zvariant::Value::new(NM_AUTOCONENCT_PORT_NO),
                None => zvariant::Value::new(NM_AUTOCONENCT_PORT_DEFAULT),
            },
        );
        Ok(ret)
    }
}
