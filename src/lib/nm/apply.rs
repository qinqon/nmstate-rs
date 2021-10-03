use std::collections::HashMap;

use nm_dbus::NmApi;

use crate::{
    nm::checkpoint::nm_checkpoint_timeout_extend,
    nm::connection::iface_to_nm_connection, nm::error::nm_error_to_nmstate,
    nm::profile::delete_exist_profiles, InterfaceType, NetworkState,
    NmstateError,
};

// We only adjust timeout for every 20 profile additions.
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
    let nm_api =
        NmApi::new().map_err(|ref nm_error| nm_error_to_nmstate(nm_error))?;

    apply_single_state(&nm_api, del_net_state, checkpoint)?;
    apply_single_state(&nm_api, add_net_state, checkpoint)?;
    apply_single_state(&nm_api, chg_net_state, checkpoint)?;
    Ok(())
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
        .map_err(|ref nm_error| nm_error_to_nmstate(nm_error))?;
    let nm_acs = nm_api
        .nm_active_connections_get()
        .map_err(|ref nm_error| nm_error_to_nmstate(nm_error))?;
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

    for (index, iface) in ifaces.iter().enumerate() {
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
        if iface.iface_type() != InterfaceType::Unknown {
            let (uuid, nm_conn) =
                iface_to_nm_connection(iface, &exist_nm_conns, &nm_ac_uuids)?;
            nm_api
                .connection_add(&nm_conn)
                .map_err(|ref nm_error| nm_error_to_nmstate(nm_error))?;

            delete_exist_profiles(
                nm_api,
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
            .map_err(|ref nm_error| nm_error_to_nmstate(nm_error))?;
    }
    Ok(())
}
