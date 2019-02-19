// Copyright (C) 2019 Red Hat, Inc.
//
// Permission is hereby granted, free of charge, to any
// person obtaining a copy of this software and associated
// documentation files (the "Software"), to deal in the
// Software without restriction, including without
// limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software
// is furnished to do so, subject to the following
// conditions:
//
// The above copyright notice and this permission notice
// shall be included in all copies or substantial portions
// of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
// ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
// TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
// PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
// SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
// IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.
//
// Author: Gris Ge <fge@redhat.com>

use super::error::*;
use nm::DeviceExt;
use nm::{Client, DeviceState, DeviceType};
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum NmStateInterfaceDeviceType {
    Unknown,
    Ethernet,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum NmStateInterfaceState {
    Up,
    Down,
}

#[derive(Debug, Serialize)]
pub struct NmStateInterface {
    pub name: String,
    #[serde(rename = "type")]
    pub iface_type: NmStateInterfaceDeviceType,
    pub state: NmStateInterfaceState,
    pub mtu: u32,
}

impl NmStateInterface {
    pub fn list(cli: &Client) -> Result<Vec<NmStateInterface>> {
        let mut ret = Vec::new();
        for dev in cli.get_devices() {
            ret.push(NmStateInterface::new(&dev)?);
        }
        Ok(ret)
    }

    fn new(dev: &nm::Device) -> Result<NmStateInterface> {
        if let Some(n) = dev.get_iface() {
            Ok(NmStateInterface {
                name: n.to_string(),
                iface_type: match dev.get_device_type() {
                    DeviceType::Ethernet => {
                        NmStateInterfaceDeviceType::Ethernet
                    }
                    _ => NmStateInterfaceDeviceType::Unknown,
                },
                state: match dev.get_state() {
                    DeviceState::Activated => NmStateInterfaceState::Up,
                    _ => NmStateInterfaceState::Down,
                },
                mtu: dev.get_mtu(),
            })
        } else {
            Err(NmStateError::Bug("Device has no interface".to_string()))
        }
    }
}
