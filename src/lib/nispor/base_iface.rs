use crate::{
    nispor::ip::{np_ipv4_to_nmstate, np_ipv6_to_nmstate},
    BaseInterface, InterfaceState, InterfaceType,
};

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
        // Do not differentiate veth over ethernet
        nispor::IfaceType::Veth => InterfaceType::Ethernet,
        nispor::IfaceType::Vlan => InterfaceType::Vlan,
        nispor::IfaceType::Vrf => InterfaceType::Vrf,
        nispor::IfaceType::Vxlan => InterfaceType::Vxlan,
        _ => InterfaceType::Other(format!("{:?}", np_iface_type)),
    }
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

pub(crate) fn np_iface_to_base_iface(
    np_iface: &nispor::Iface,
) -> BaseInterface {
    let base_iface = BaseInterface {
        name: np_iface.name.to_string(),
        state: np_iface_state_to_nmstate(&np_iface.state),
        iface_type: np_iface_type_to_nmstate(&np_iface.iface_type),
        ipv4: np_ipv4_to_nmstate(&np_iface.ipv4),
        ipv6: np_ipv6_to_nmstate(&np_iface.ipv6),
        ..Default::default()
    };
    base_iface
}
