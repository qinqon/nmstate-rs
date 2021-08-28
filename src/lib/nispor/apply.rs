use crate::{
    ErrorKind, Interface, InterfaceType, NetworkState, NmstateError, VethConfig,
};

pub(crate) fn nispor_apply(
    add_net_state: &NetworkState,
    chg_net_state: &NetworkState,
    del_net_state: &NetworkState,
) -> Result<(), NmstateError> {
    let np_net_conf = net_state_to_nispor(del_net_state)?;
    if let Err(e) = np_net_conf.apply() {
        return Err(NmstateError::new(
            ErrorKind::PluginFailure,
            format!("Unknown error from nipsor plugin: {}", e),
        ));
    }
    let np_net_conf = net_state_to_nispor(add_net_state)?;
    if let Err(e) = np_net_conf.apply() {
        return Err(NmstateError::new(
            ErrorKind::PluginFailure,
            format!("Unknown error from nipsor plugin: {}", e),
        ));
    }
    let np_net_conf = net_state_to_nispor(chg_net_state)?;
    if let Err(e) = np_net_conf.apply() {
        return Err(NmstateError::new(
            ErrorKind::PluginFailure,
            format!("Unknown error from nipsor plugin: {}", e),
        ));
    }
    Ok(())
}

fn net_state_to_nispor(
    net_state: &NetworkState,
) -> Result<nispor::NetConf, NmstateError> {
    let mut np_ifaces: Vec<nispor::IfaceConf> = Vec::new();

    for iface in net_state.interfaces.to_vec() {
        if !iface.is_up() {
            continue;
        }
        let np_iface_type = nmstate_iface_type_to_np(&iface.iface_type());
        if np_iface_type == nispor::IfaceType::Unknown {
            eprintln!(
                "ERROR: Unknown interface type {} for interface {}",
                iface.iface_type(),
                iface.name()
            );
            continue;
        }
        np_ifaces.push(nmstate_iface_to_np(&iface, np_iface_type)?);
    }
    println!("{:?}", &np_ifaces);

    Ok(nispor::NetConf {
        ifaces: Some(np_ifaces),
    })
}

fn nmstate_iface_type_to_np(
    nms_iface_type: &InterfaceType,
) -> nispor::IfaceType {
    match nms_iface_type {
        InterfaceType::LinuxBridge => nispor::IfaceType::Bridge,
        InterfaceType::Ethernet => nispor::IfaceType::Ethernet,
        InterfaceType::Veth => nispor::IfaceType::Veth,
        _ => nispor::IfaceType::Unknown,
    }
}

fn nmstate_iface_to_np(
    nms_iface: &Interface,
    np_iface_type: nispor::IfaceType,
) -> Result<nispor::IfaceConf, NmstateError> {
    let mut np_iface = nispor::IfaceConf {
        name: nms_iface.name().to_string(),
        iface_type: Some(np_iface_type),
        state: nispor::IfaceState::Up,
        ..Default::default()
    };
    if let Some(ctrl_name) = &nms_iface.base_iface().controller {
        np_iface.controller = Some(ctrl_name.to_string())
    }
    match nms_iface {
        Interface::Veth(veth_iface) => {
            np_iface.veth = nms_veth_conf_to_np(veth_iface.veth.as_ref());
        }
        _ => {}
    }
    Ok(np_iface)
}

fn nms_veth_conf_to_np(
    nms_veth_conf: Option<&VethConfig>,
) -> Option<nispor::VethConf> {
    if let Some(nms_veth_conf) = nms_veth_conf {
        Some(nispor::VethConf {
            peer: nms_veth_conf.peer.to_string(),
        })
    } else {
        None
    }
}
