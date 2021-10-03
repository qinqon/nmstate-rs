use log::warn;
use nm_dbus::{NmApi, NmConnection, NmSettingIp, NmSettingIpMethod};

use crate::{
    nm::error::nm_error_to_nmstate, BaseInterface, EthernetInterface,
    Interface, InterfaceIpv4, InterfaceIpv6, InterfaceState, InterfaceType,
    LinuxBridgeInterface, NetworkState, NmstateError, UnknownInterface,
};

const NM_SETTING_BRIDGE_SETTING_NAME: &str = "bridge";
const NM_SETTING_WIRED_SETTING_NAME: &str = "802-3-ethernet";

pub(crate) fn nm_retrieve() -> Result<NetworkState, NmstateError> {
    let mut net_state = NetworkState::new();
    let nm_api =
        NmApi::new().map_err(|ref nm_error| nm_error_to_nmstate(nm_error))?;
    let nm_conns = nm_api
        .nm_applied_connections_get()
        .map_err(|ref nm_error| nm_error_to_nmstate(nm_error))?;

    for nm_conn in nm_conns {
        if let Some(base_iface) = nm_conn_to_base_iface(&nm_conn) {
            let iface = match &base_iface.iface_type {
                InterfaceType::LinuxBridge => {
                    Interface::LinuxBridge(LinuxBridgeInterface {
                        base: base_iface,
                        ..Default::default()
                    })
                }
                InterfaceType::Ethernet => {
                    Interface::Ethernet(EthernetInterface {
                        base: base_iface,
                        ..Default::default()
                    })
                }
                _ => Interface::Unknown(UnknownInterface::new(base_iface)),
            };
            net_state.append_interface_data(iface);
        }
    }
    Ok(net_state)
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
            let ipv4 = nm_conn.ipv4.as_ref().map(|nm_ipv4_setting| {
                nm_ip_setting_to_nmstate4(nm_ipv4_setting)
            });
            let ipv6 = nm_conn.ipv6.as_ref().map(|nm_ipv6_setting| {
                nm_ip_setting_to_nmstate6(nm_ipv6_setting)
            });

            return Some(BaseInterface {
                name: iface_name.to_string(),
                prop_list: vec!["name", "state", "iface_type", "ipv4", "ipv6"],
                state: InterfaceState::Up,
                iface_type: nm_iface_type_to_nmstate(iface_type),
                ipv4,
                ipv6,
                ..Default::default()
            });
        }
    }
    None
}

fn nm_ip_setting_to_nmstate4(nm_ip_setting: &NmSettingIp) -> InterfaceIpv4 {
    if let Some(nm_ip_method) = &nm_ip_setting.method {
        let (enabled, dhcp) = match nm_ip_method {
            NmSettingIpMethod::Disabled => (false, false),
            NmSettingIpMethod::LinkLocal
            | NmSettingIpMethod::Manual
            | NmSettingIpMethod::Shared => (true, false),
            NmSettingIpMethod::Auto => (true, true),
            _ => {
                warn!("Unexpected NM IP method {:?}", nm_ip_method);
                (true, false)
            }
        };
        InterfaceIpv4 {
            enabled,
            dhcp,
            prop_list: vec!["enabled", "dhcp"],
            ..Default::default()
        }
    } else {
        InterfaceIpv4::default()
    }
}

fn nm_ip_setting_to_nmstate6(nm_ip_setting: &NmSettingIp) -> InterfaceIpv6 {
    if let Some(nm_ip_method) = &nm_ip_setting.method {
        let (enabled, dhcp, autoconf) = match nm_ip_method {
            NmSettingIpMethod::Disabled => (false, false, false),
            NmSettingIpMethod::LinkLocal
            | NmSettingIpMethod::Manual
            | NmSettingIpMethod::Shared => (true, false, true),
            NmSettingIpMethod::Auto => (true, true, true),
            NmSettingIpMethod::Dhcp => (true, true, false),
            NmSettingIpMethod::Ignore => (true, false, true),
        };
        InterfaceIpv6 {
            enabled,
            dhcp,
            autoconf,
            prop_list: vec!["enabled", "dhcp", "autoconf"],
            ..Default::default()
        }
    } else {
        InterfaceIpv6::default()
    }
}
