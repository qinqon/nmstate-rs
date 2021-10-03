use crate::{BaseInterface, VethConfig, VethInterface};

pub(crate) fn np_veth_to_nmstate(
    np_iface: nispor::Iface,
    base_iface: BaseInterface,
) -> VethInterface {
    let veth_conf = match np_iface.veth {
        Some(np_veth_info) => Some(VethConfig {
            peer: np_veth_info.peer,
        }),
        None => None,
    };
    VethInterface {
        base: base_iface,
        veth: veth_conf,
    }
}
