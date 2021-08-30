use std::collections::HashMap;

use nm_dbus::{
    NmApi, NmConnection, NmSettingBridge, NmSettingConnection, NmSettingIp,
    NmSettingIpMethod,
};

use crate::{
    nm::checkpoint::nm_checkpoint_timeout_extend,
    nm::error::nm_error_to_nmstate, ErrorKind, Interface, InterfaceIpv4,
    InterfaceIpv6, InterfaceType, LinuxBridgeConfig, LinuxBridgeOptions,
    LinuxBridgeStpOptions, NetworkState, NmstateError,
};

// We only adjust timeout for every 20 profile addtions.
const TIMEOUT_ADJUST_PROFILE_ADDTION_GROUP_SIZE: usize = 20;
const TIMEOUT_SECONDS_FOR_PROFILE_ADDTION: u32 = 60;
const TIMEOUT_SECONDS_FOR_PROFILE_ACTIVATION: u32 = 60;

pub(crate) fn nm_apply(
    add_net_state: &NetworkState,
    chg_net_state: &NetworkState,
    del_net_state: &NetworkState,
    _cur_net_state: &NetworkState,
    checkpoint: &str,
) -> Result<(), NmstateError> {
    let nm_api = NmApi::new()
        .or_else(|ref nm_error| Err(nm_error_to_nmstate(nm_error)))?;

    apply_single_state(&nm_api, del_net_state, checkpoint)?;
    apply_single_state(&nm_api, add_net_state, checkpoint)?;
    apply_single_state(&nm_api, chg_net_state, checkpoint)?;
    Ok(())
}

fn iface_type_to_nm(
    iface_type: &InterfaceType,
) -> Result<String, NmstateError> {
    match iface_type {
        InterfaceType::LinuxBridge => Ok("bridge".into()),
        InterfaceType::Ethernet => Ok("802-3-ethernet".into()),
        InterfaceType::Veth => Ok("802-3-ethernet".into()),
        _ => Err(NmstateError::new(
            ErrorKind::Bug,
            format!("BUG: NM does not support iface type: {:?}", iface_type),
        )),
    }
}

fn iface_to_nm_connection(
    iface: &Interface,
    exist_nm_conns: &[NmConnection],
    nm_ac_uuids: &[&str],
) -> Result<(String, NmConnection), NmstateError> {
    println!("HAHA {:?}", iface);
    let base_iface = iface.base_iface();
    let exist_nm_conn = get_exist_profile(
        exist_nm_conns,
        &base_iface.name,
        &base_iface.iface_type,
        nm_ac_uuids,
    );

    let uuid = if let Some(exist_nm_conn) = exist_nm_conn {
        if let Some(exist_uuid) = exist_nm_conn.uuid() {
            exist_uuid.to_string()
        } else {
            NmApi::uuid_gen()
        }
    } else {
        NmApi::uuid_gen()
    };
    let mut nm_conn_set = NmSettingConnection {
        id: Some(base_iface.name.clone()),
        uuid: Some(uuid.clone()),
        iface_type: Some(iface_type_to_nm(&base_iface.iface_type)?),
        iface_name: Some(base_iface.name.clone()),
        autoconnect_ports: Some(true),
        ..Default::default()
    };
    if let Some(ctrl_name) = &base_iface.controller {
        if let Some(ctrl_type) = &base_iface.controller_type {
            nm_conn_set.controller = Some(ctrl_name.to_string());
            nm_conn_set.controller_type = Some(iface_type_to_nm(ctrl_type)?);
        }
    }
    let mut nm_conn = NmConnection {
        connection: Some(nm_conn_set),
        ..Default::default()
    };
    if base_iface.can_have_ip() {
        if let Some(iface_ip) = &base_iface.ipv4 {
            nm_conn.ipv4 = Some(iface_ipv4_to_nm(&iface_ip)?);
        } else {
            nm_conn.ipv4 = Some(iface_ipv4_to_nm(&InterfaceIpv4 {
                enabled: false,
                ..Default::default()
            })?);
        }
        if let Some(iface_ip) = &base_iface.ipv6 {
            nm_conn.ipv6 = Some(iface_ipv6_to_nm(&iface_ip)?);
        } else {
            nm_conn.ipv6 = Some(iface_ipv6_to_nm(&InterfaceIpv6 {
                enabled: false,
                ..Default::default()
            })?);
        }
    }
    if let Interface::LinuxBridge(br_iface) = iface {
        if let Some(br_conf) = &br_iface.bridge {
            nm_conn.bridge = Some(linux_bridge_conf_to_nm(br_conf)?);
        }
    }
    Ok((uuid, nm_conn))
}

fn iface_ipv4_to_nm(
    iface_ip: &InterfaceIpv4,
) -> Result<NmSettingIp, NmstateError> {
    let mut addresses: Vec<String> = Vec::new();
    let method = if iface_ip.enabled {
        if iface_ip.dhcp {
            NmSettingIpMethod::Auto
        } else if iface_ip.addresses.len() > 0 {
            for ip_addr in &iface_ip.addresses {
                addresses
                    .push(format!("{}/{}", ip_addr.ip, ip_addr.prefix_length));
            }
            NmSettingIpMethod::Manual
        } else {
            NmSettingIpMethod::Disabled
        }
    } else {
        NmSettingIpMethod::Disabled
    };
    Ok(NmSettingIp {
        method: Some(method),
        addresses,
        ..Default::default()
    })
}

fn iface_ipv6_to_nm(
    iface_ip: &InterfaceIpv6,
) -> Result<NmSettingIp, NmstateError> {
    let mut addresses: Vec<String> = Vec::new();
    let method = if iface_ip.enabled {
        match (iface_ip.dhcp, iface_ip.autoconf) {
            (true, true) => NmSettingIpMethod::Auto,
            (true, false) => NmSettingIpMethod::Dhcp,
            (false, true) => {
                return Err(NmstateError::new(
                    ErrorKind::NotImplementedError,
                    "Autoconf without DHCP is not supported yet".to_string(),
                ))
            }
            (false, false) => {
                if iface_ip.addresses.len() > 0 {
                    for ip_addr in &iface_ip.addresses {
                        addresses.push(format!(
                            "{}/{}",
                            ip_addr.ip, ip_addr.prefix_length
                        ));
                    }
                    NmSettingIpMethod::Manual
                } else {
                    NmSettingIpMethod::Disabled
                }
            }
        }
    } else {
        NmSettingIpMethod::Disabled
    };
    Ok(NmSettingIp {
        method: Some(method),
        addresses,
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

fn nm_connection_matches(
    nm_conn: &NmConnection,
    iface_name: &str,
    iface_type: &InterfaceType,
) -> bool {
    // TODO Need to handle veth/ethernet
    let nm_iface_type = match iface_type_to_nm(iface_type) {
        Ok(i) => i,
        Err(e) => {
            eprintln!(
                "BUG: nm_connection_matches {:?}, {}, {:?}: {}",
                nm_conn, iface_name, iface_type, e
            );
            return false;
        }
    };
    nm_conn.iface_name() == Some(iface_name)
        && nm_conn.iface_type() == Some(&nm_iface_type)
}

fn delete_exist_profiles(
    nm_api: &NmApi,
    exist_nm_conns: &[NmConnection],
    iface_name: &str,
    iface_type: &InterfaceType,
    excluded_uuid: &str,
) -> Result<(), NmstateError> {
    for exist_nm_conn in exist_nm_conns {
        if let Some(uuid) = exist_nm_conn.uuid() {
            if uuid != excluded_uuid
                && nm_connection_matches(exist_nm_conn, iface_name, iface_type)
            {
                nm_api.connection_delete(uuid).or_else(|ref nm_error| {
                    Err(nm_error_to_nmstate(nm_error))
                })?;
            }
        }
    }
    Ok(())
}

// Found existing profile, prefer actived
fn get_exist_profile<'a>(
    exist_nm_conns: &'a [NmConnection],
    iface_name: &str,
    iface_type: &InterfaceType,
    nm_ac_uuids: &[&str],
) -> Option<&'a NmConnection> {
    let mut found_nm_conns: Vec<&NmConnection> = Vec::new();
    for exist_nm_conn in exist_nm_conns {
        if nm_connection_matches(exist_nm_conn, iface_name, iface_type) {
            if let Some(uuid) = exist_nm_conn.uuid() {
                if nm_ac_uuids.contains(&uuid) {
                    return Some(exist_nm_conn);
                }
            }
            found_nm_conns.push(exist_nm_conn);
        }
    }
    found_nm_conns.pop()
}

fn apply_single_state(
    nm_api: &NmApi,
    net_state: &NetworkState,
    checkpoint: &str,
) -> Result<(), NmstateError> {
    let mut nm_conn_uuids: Vec<String> = Vec::new();
    let mut ports: HashMap<String, (String, InterfaceType)> = HashMap::new();

    let exist_nm_conns = nm_api
        .nm_connections_get()
        .or_else(|ref nm_error| Err(nm_error_to_nmstate(nm_error)))?;
    let nm_acs = nm_api
        .nm_active_connections_get()
        .or_else(|ref nm_error| Err(nm_error_to_nmstate(nm_error)))?;
    let nm_ac_uuids: Vec<&str> =
        nm_acs.iter().map(|nm_ac| &nm_ac.uuid as &str).collect();

    let ifaces = net_state.interfaces.to_vec();
    for iface in &ifaces {
        if let Some(iface_ports) = iface.ports() {
            for port_name in iface_ports {
                ports.insert(
                    port_name.to_string(),
                    (iface.name().to_string(), iface.iface_type().clone()),
                );
            }
        }
    }

    let mut index: usize = 0;
    for iface in &ifaces {
        // Only extend the timeout every
        // TIMEOUT_ADJUST_PROFILE_ADDTION_GROUP_SIZE profile addition.
        if index % TIMEOUT_ADJUST_PROFILE_ADDTION_GROUP_SIZE
            == TIMEOUT_ADJUST_PROFILE_ADDTION_GROUP_SIZE - 1
        {
            nm_checkpoint_timeout_extend(
                checkpoint,
                TIMEOUT_SECONDS_FOR_PROFILE_ADDTION,
            )?;
        }
        index += 1;
        if iface.iface_type() != InterfaceType::Unknown {
            let (uuid, nm_conn) =
                iface_to_nm_connection(iface, &exist_nm_conns, &nm_ac_uuids)?;
            nm_api
                .connection_add(&nm_conn)
                .or_else(|ref nm_error| Err(nm_error_to_nmstate(nm_error)))?;
            delete_exist_profiles(
                &nm_api,
                &exist_nm_conns,
                iface.name(),
                &iface.iface_type(),
                &uuid,
            )?;
            nm_conn_uuids.push(uuid);
        }
    }
    for nm_conn_uuid in &nm_conn_uuids {
        nm_checkpoint_timeout_extend(
            checkpoint,
            TIMEOUT_SECONDS_FOR_PROFILE_ACTIVATION,
        )?;
        nm_api
            .connection_activate(nm_conn_uuid)
            .or_else(|ref nm_error| Err(nm_error_to_nmstate(nm_error)))?;
    }
    Ok(())
}
