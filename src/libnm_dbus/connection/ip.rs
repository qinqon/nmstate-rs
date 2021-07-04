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
    dbus_value::value_to_string,
    error::{ErrorKind, NmError},
};

#[derive(Debug, Clone, PartialEq)]
pub enum NmSettingIpMethod {
    Auto,
    Disabled,
    LinkLocal,
    Manual,
    Shared,
    Dhcp,   // IPv6 only,
    Ignore, // Ipv6 only,
}

impl Default for NmSettingIpMethod {
    fn default() -> Self {
        Self::Auto
    }
}

impl std::fmt::Display for NmSettingIpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Auto => "auto",
                Self::Disabled => "disabled",
                Self::LinkLocal => "link-local",
                Self::Manual => "manual",
                Self::Shared => "shared",
                Self::Dhcp => "dhcp",
                Self::Ignore => "ignore",
            }
        )
    }
}

impl TryFrom<&str> for NmSettingIpMethod {
    type Error = NmError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "auto" => Ok(Self::Auto),
            "disabled" => Ok(Self::Disabled),
            "link_local" => Ok(Self::LinkLocal),
            "manual" => Ok(Self::Manual),
            "shared" => Ok(Self::Shared),
            "dhcp" => Ok(Self::Dhcp),
            "ignore" => Ok(Self::Ignore),
            _ => Err(NmError::new(
                ErrorKind::InvalidArgument,
                format!("Invalid IP method {}", value),
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct NmSettingIp {
    pub method: Option<NmSettingIpMethod>,
}

impl TryFrom<&HashMap<String, zvariant::OwnedValue>> for NmSettingIp {
    type Error = NmError;
    fn try_from(
        value: &HashMap<String, zvariant::OwnedValue>,
    ) -> Result<Self, Self::Error> {
        let method = if let Some(method_str) = value_to_string(value, "method")?
        {
            Some(NmSettingIpMethod::try_from(method_str.as_str())?)
        } else {
            return Err(NmError::new(
                ErrorKind::InvalidArgument,
                "No IP method found".to_string(),
            ));
        };
        Ok(Self { method })
    }
}

impl NmSettingIp {
    pub(crate) fn to_value(
        &self,
    ) -> Result<HashMap<&str, zvariant::Value>, NmError> {
        let mut ret = HashMap::new();
        if let Some(v) = &self.method {
            ret.insert("method", zvariant::Value::new(format!("{}", v)));
        }
        Ok(ret)
    }
}
