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

use std::process::{Command, Stdio};

use nm_dbus::{
    ErrorKind, NmApi, NmConnection, NmError, NmSettingBridge,
    NmSettingConnection, NmSettingIp, NmSettingIpMethod,
};

#[test]
fn test_full() {
    let nm = NmApi::new().unwrap();
    println!("NM version: {}", nm.version().unwrap());

    let cp = nm.checkpoint_create().unwrap();
    println!("checkpoint_create: {}", cp);
    let e = nm.checkpoint_create().unwrap_err();
    assert!(e.kind == ErrorKind::CheckpointConflict);

    println!("second checkpoint_create() got conflict error as expected");

    nm.checkpoint_destroy(&cp).unwrap();
    println!("checkpoint_destroy: done");

    let cp = nm.checkpoint_create().unwrap();
    println!("checkpoint_create: {}", cp);

    nm.checkpoint_rollback(&cp).unwrap();
    println!("checkpoint_rollback: done");

    let br_conn_uuid = &NmApi::uuid_gen();

    nm.connection_add(&NmConnection {
        connection: Some(NmSettingConnection {
            id: Some("br0".into()),
            uuid: Some(br_conn_uuid.into()),
            iface_type: Some("bridge".into()),
            iface_name: Some("br0".into()),
            autoconnect_ports: Some(true),
            ..Default::default()
        }),
        bridge: Some(NmSettingBridge {
            stp: Some(true),
            ..Default::default()
        }),
        ipv4: Some(NmSettingIp {
            method: Some(NmSettingIpMethod::Manual),
            addresses: vec!["192.0.2.251/24".to_string()],
            ..Default::default()
        }),
        ipv6: Some(NmSettingIp {
            method: Some(NmSettingIpMethod::Manual),
            addresses: vec!["2001:db8:1::1/64".to_string()],
            ..Default::default()
        }),
        ..Default::default()
    })
    .unwrap();

    let port_uuid = &NmApi::uuid_gen();
    nm.connection_add(&NmConnection {
        connection: Some(NmSettingConnection {
            id: Some("dummy0".into()),
            uuid: Some(port_uuid.into()),
            iface_type: Some("dummy".into()),
            iface_name: Some("dummy0".into()),
            controller: Some(br_conn_uuid.to_string()),
            controller_type: Some("bridge".into()),
            ..Default::default()
        }),
        bridge: Some(NmSettingBridge {
            stp: Some(false),
            ..Default::default()
        }),
        ..Default::default()
    })
    .unwrap();
    println!(
        "Bridge port connection created: {:?}",
        nm.nm_connection_get(port_uuid)
    );

    nm.connection_activate(br_conn_uuid).unwrap();
    println!("connection {} activated", br_conn_uuid);

    std::thread::sleep(std::time::Duration::from_millis(5000));

    // Update connection with STP false
    nm.connection_add(&NmConnection {
        connection: Some(NmSettingConnection {
            id: Some("br0".into()),
            uuid: Some(br_conn_uuid.into()),
            iface_type: Some("bridge".into()),
            iface_name: Some("br0".into()),
            ..Default::default()
        }),
        bridge: Some(NmSettingBridge {
            stp: Some(false),
            ..Default::default()
        }),
        ipv4: Some(NmSettingIp {
            method: Some(NmSettingIpMethod::Manual),
            addresses: vec!["192.0.2.251/24".to_string()],
            ..Default::default()
        }),
        ipv6: Some(NmSettingIp {
            method: Some(NmSettingIpMethod::Manual),
            addresses: vec!["2001:db8:1::1/64".to_string()],
            ..Default::default()
        }),
        ..Default::default()
    })
    .unwrap();
    println!(
        "Bridge connection created: {:?}",
        nm.nm_connection_get(br_conn_uuid)
    );
    if let Err(NmError {
        kind: ErrorKind::IncompatibleReapply,
        ..
    }) = nm.connection_reapply(br_conn_uuid)
    {
        println!("The reapply fail on bridge STP changes as expected");
        nm.connection_activate(br_conn_uuid).unwrap();
    } else {
        panic!("The reapply should fail in bridge STP changes, but did not");
    }

    std::thread::sleep(std::time::Duration::from_millis(5000));
    println!(
        "{}",
        String::from_utf8(
            Command::new("npc")
                .arg("br0")
                .stdout(Stdio::inherit())
                .stdin(Stdio::inherit())
                .stderr(Stdio::inherit())
                .output()
                .unwrap()
                .stdout
        )
        .unwrap()
    );

    println!("NM {:?}", nm.nm_applied_connections_get());

    std::thread::sleep(std::time::Duration::from_millis(100));
    nm.connection_deactivate(br_conn_uuid).unwrap();
    println!("connection {} deactivated", br_conn_uuid);

    nm.connection_delete(br_conn_uuid).unwrap();
    println!("bridge connection {} deleted", br_conn_uuid);

    nm.connection_delete(port_uuid).unwrap();
    println!("port connection {} deleted", port_uuid);
}
