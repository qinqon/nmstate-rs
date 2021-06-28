use nispor::NisporError;

use crate::{
    nispor::linux_bridge::np_bridge_to_nmstate, BaseInterface, ErrorKind,
    Interface, InterfaceState, InterfaceType, NetworkState, NmstateError,
};

pub(crate) fn nispor_retrieve() -> Result<NetworkState, NmstateError> {
    let mut net_state = NetworkState::new();
    let mut np_state = nispor::NetState::retrieve()
        .or_else(|ref np_error| Err(np_error_to_nmstate(np_error)))?;
    for (_, np_iface) in np_state.ifaces.drain() {
        let base_iface = BaseInterface {
            name: np_iface.name.to_string(),
            state: np_iface_state_to_nmstate(&np_iface.state),
            iface_type: np_iface_type_to_nmstate(&np_iface.iface_type),
            ..Default::default()
        };
        let iface = match &base_iface.iface_type {
            InterfaceType::LinuxBridge => Interface::LinuxBridge(
                np_bridge_to_nmstate(np_iface, base_iface),
            ),
            _ => Interface::Unknown(base_iface),
        };
        net_state.append_interface_data(iface);
    }
    Ok(net_state)
}

fn np_iface_type_to_nmstate(
    np_iface_type: &nispor::IfaceType,
) -> InterfaceType {
    match np_iface_type {
        nispor::IfaceType::Bond => InterfaceType::Bond,
        nispor::IfaceType::Bridge => InterfaceType::LinuxBridge,
        nispor::IfaceType::Dummy => InterfaceType::Dummy,
        nispor::IfaceType::Ethernet => InterfaceType::Ethernet,
        nispor::IfaceType::Loopback => InterfaceType::Loopback,
        nispor::IfaceType::MacVlan => InterfaceType::MacVlan,
        nispor::IfaceType::MacVtap => InterfaceType::MacVtap,
        nispor::IfaceType::OpenvSwitch => InterfaceType::OvsInterface,
        nispor::IfaceType::Tun => InterfaceType::Tun,
        nispor::IfaceType::Veth => InterfaceType::Veth,
        nispor::IfaceType::Vlan => InterfaceType::Vlan,
        nispor::IfaceType::Vrf => InterfaceType::Vrf,
        nispor::IfaceType::Vxlan => InterfaceType::Vxlan,
        _ => InterfaceType::Other(format!("{:?}", np_iface_type)),
    }
}

fn np_error_to_nmstate(np_error: &NisporError) -> NmstateError {
    NmstateError::new(
        ErrorKind::Bug,
        format!("{}: {}", np_error.kind, np_error.msg),
    )
}

fn np_iface_state_to_nmstate(
    np_iface_state: &nispor::IfaceState,
) -> InterfaceState {
    match np_iface_state {
        nispor::IfaceState::Up => InterfaceState::Up,
        nispor::IfaceState::Down => InterfaceState::Down,
        _ => InterfaceState::Unknown,
    }
}
