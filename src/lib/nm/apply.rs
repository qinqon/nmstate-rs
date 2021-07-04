use std::collections::HashMap;

use nm_dbus::{
    NmApi, NmConnection, NmSettingBridge, NmSettingConnection, NmSettingIp,
    NmSettingIpMethod,
};

use crate::{
    nm::error::nm_error_to_nmstate, ErrorKind, Interface, InterfaceIp,
    InterfaceType, LinuxBridgeConfig, LinuxBridgeOptions,
    LinuxBridgeStpOptions, NetworkState, NmstateError,
};

pub(crate) fn nm_apply(net_state: &NetworkState) -> Result<(), NmstateError> {
    let nm_api = NmApi::new()
        .or_else(|ref nm_error| Err(nm_error_to_nmstate(nm_error)))?;

    let mut nm_conn_uuids: Vec<String> = Vec::new();
    let mut ports: HashMap<String, (String, InterfaceType)> = HashMap::new();

    if let Some(ifaces) = &net_state.interfaces {
        for iface in ifaces.as_slice() {
            for port_name in iface.ports() {
                ports.insert(
                    port_name,
                    (iface.name().to_string(), iface.iface_type().clone()),
                );
            }
        }
    }

    if let Some(ifaces) = &net_state.interfaces {
        for iface in ifaces.as_slice() {
            if iface.iface_type() != InterfaceType::Unknown {
                let (uuid, nm_conn) = iface_to_nm_connection(iface, &ports)?;
                nm_api.connection_add(&nm_conn).or_else(|ref nm_error| {
                    Err(nm_error_to_nmstate(nm_error))
                })?;
                nm_conn_uuids.push(uuid);
            }
        }
    }
    for nm_conn_uuid in &nm_conn_uuids {
        nm_api
            .connection_activate(nm_conn_uuid)
            .or_else(|ref nm_error| Err(nm_error_to_nmstate(nm_error)))?;
    }

    Ok(())
}

fn iface_type_to_nm(
    iface_type: &InterfaceType,
) -> Result<String, NmstateError> {
    match iface_type {
        InterfaceType::LinuxBridge => Ok("bridge".into()),
        InterfaceType::Ethernet => Ok("802-3-ethernet".into()),
        _ => Err(NmstateError::new(
            ErrorKind::Bug,
            format!("BUG: NM does not support iface type: {:?}", iface_type),
        )),
    }
}

fn iface_to_nm_connection(
    iface: &Interface,
    ports: &HashMap<String, (String, InterfaceType)>,
) -> Result<(String, NmConnection), NmstateError> {
    let base_iface = iface.base_iface();
    let uuid = NmApi::uuid_gen();
    let mut nm_conn_set = NmSettingConnection {
        id: Some(base_iface.name.clone()),
        uuid: Some(uuid.clone()),
        iface_type: Some(iface_type_to_nm(&base_iface.iface_type)?),
        iface_name: Some(base_iface.name.clone()),
        autoconnect_ports: Some(true),
        ..Default::default()
    };
    if let Some((controler, controler_type)) = ports.get(&base_iface.name) {
        nm_conn_set.controller = Some(controler.to_string());
        nm_conn_set.controller_type = Some(iface_type_to_nm(controler_type)?);
    }
    let mut nm_conn = NmConnection {
        connection: Some(nm_conn_set),
        ..Default::default()
    };
    if let Some(iface_ip) = &base_iface.ipv4 {
        nm_conn.ipv4 = Some(iface_ip_to_nm(&iface_ip)?);
    }
    if let Some(iface_ip) = &base_iface.ipv6 {
        nm_conn.ipv6 = Some(iface_ip_to_nm(&iface_ip)?);
    }
    if let Interface::LinuxBridge(br_iface) = iface {
        if let Some(br_conf) = &br_iface.bridge {
            nm_conn.bridge = Some(linux_bridge_conf_to_nm(br_conf)?);
        }
    }
    Ok((uuid, nm_conn))
}

fn iface_ip_to_nm(iface_ip: &InterfaceIp) -> Result<NmSettingIp, NmstateError> {
    Ok(NmSettingIp {
        method: Some(if iface_ip.enabled {
            NmSettingIpMethod::Auto
        } else {
            NmSettingIpMethod::Disabled
        }),
        ..Default::default()
    })
}

fn linux_bridge_conf_to_nm(
    br_conf: &LinuxBridgeConfig,
) -> Result<NmSettingBridge, NmstateError> {
    if let Some(LinuxBridgeOptions {
        stp:
            Some(LinuxBridgeStpOptions {
                enabled: stp_enabled,
                ..
            }),
        ..
    }) = br_conf.options
    {
        return Ok(NmSettingBridge {
            stp: stp_enabled,
            ..Default::default()
        });
    }
    Ok(NmSettingBridge::default())
}
