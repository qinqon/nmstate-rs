use nm_dbus::{NmApi, NmConnection, NmError, NmSettingIp, NmSettingIpMethod};

use crate::{
    BaseInterface, ErrorKind, Interface, InterfaceIp, InterfaceState,
    InterfaceType, LinuxBridgeInterface, NetworkState, NmstateError,
};

const NM_SETTING_BRIDGE_SETTING_NAME: &str = "bridge";
const NM_SETTING_WIRED_SETTING_NAME: &str = "802-3-ethernet";

pub(crate) fn nm_retrieve() -> Result<NetworkState, NmstateError> {
    let mut net_state = NetworkState::new();
    let nm_api = NmApi::new()
        .or_else(|ref nm_error| Err(nm_error_to_nmstate(nm_error)))?;
    let nm_conns = nm_api
        .nm_applied_connections_get()
        .or_else(|ref nm_error| Err(nm_error_to_nmstate(nm_error)))?;
    for nm_conn in nm_conns {
        if let Some(base_iface) = nm_conn_to_base_iface(&nm_conn) {
            let iface = match &base_iface.iface_type {
                InterfaceType::LinuxBridge => {
                    Interface::LinuxBridge(LinuxBridgeInterface {
                        base: base_iface,
                        ..Default::default()
                    })
                }
                _ => Interface::Unknown(base_iface),
            };
            net_state.append_interface_data(iface);
        }
    }
    Ok(net_state)
}

fn nm_error_to_nmstate(nm_error: &NmError) -> NmstateError {
    NmstateError::new(
        ErrorKind::Bug,
        format!("{}: {}", nm_error.kind, nm_error.msg),
    )
}

fn nm_iface_type_to_nmstate(nm_iface_type: &str) -> InterfaceType {
    match nm_iface_type {
        NM_SETTING_WIRED_SETTING_NAME => InterfaceType::Ethernet,
        NM_SETTING_BRIDGE_SETTING_NAME => InterfaceType::LinuxBridge,
        _ => InterfaceType::Unknown,
    }
}

fn nm_conn_to_base_iface(nm_conn: &NmConnection) -> Option<BaseInterface> {
    if let Some(iface_name) = nm_conn.iface_name() {
        if let Some(iface_type) = nm_conn.iface_type() {
            let ipv4 = if let Some(ref nm_ipv4_setting) = nm_conn.ipv4 {
                Some(nm_ip_setting_to_nmstate(nm_ipv4_setting))
            } else {
                None
            };
            let ipv6 = if let Some(ref nm_ipv6_setting) = nm_conn.ipv6 {
                Some(nm_ip_setting_to_nmstate(nm_ipv6_setting))
            } else {
                None
            };

            return Some(BaseInterface {
                name: iface_name.to_string(),
                state: InterfaceState::Up,
                iface_type: nm_iface_type_to_nmstate(iface_type),
                ipv4: ipv4,
                ipv6: ipv6,
                ..Default::default()
            });
        }
    }
    return None;
}

fn nm_ip_setting_to_nmstate(nm_ip_setting: &NmSettingIp) -> InterfaceIp {
    InterfaceIp {
        enabled: if let Some(NmSettingIpMethod::Disabled) = nm_ip_setting.method
        {
            false
        } else {
            // By default NetworkManager is using auto method.
            true
        },
        ..Default::default()
    }
}
