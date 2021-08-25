use crate::{
    nispor::{
        base_iface::np_iface_to_base_iface, error::np_error_to_nmstate,
        ethernet::np_ethernet_to_nmstate, linux_bridge::np_bridge_to_nmstate,
    },
    Interface, InterfaceType, NetworkState, NmstateError, UnknownInterface,
};

pub(crate) fn nispor_retrieve() -> Result<NetworkState, NmstateError> {
    let mut net_state = NetworkState::new();
    net_state.prop_list.push("interfaces");
    let mut np_state = nispor::NetState::retrieve()
        .or_else(|ref np_error| Err(np_error_to_nmstate(np_error)))?;
    for (_, np_iface) in np_state.ifaces.drain() {
        let base_iface = np_iface_to_base_iface(&np_iface);
        let iface = match &base_iface.iface_type {
            InterfaceType::LinuxBridge => Interface::LinuxBridge(
                np_bridge_to_nmstate(np_iface, base_iface),
            ),
            InterfaceType::Ethernet => Interface::Ethernet(
                np_ethernet_to_nmstate(np_iface, base_iface),
            ),
            _ => Interface::Unknown(UnknownInterface::new(base_iface)),
        };
        net_state.append_interface_data(iface);
    }
    Ok(net_state)
}
