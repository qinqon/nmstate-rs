use nm_dbus::{NmApi, NmConnection, NmError, NmSettingIp, NmSettingIpMethod};

use crate::{
    BaseInterface, ErrorKind, Interface, InterfaceIp, InterfaceState,
    InterfaceType, LinuxBridgeInterface, NetworkState, NmstateError,
};

pub(crate) fn nm_error_to_nmstate(nm_error: &NmError) -> NmstateError {
    NmstateError::new(
        ErrorKind::Bug,
        format!("{}: {}", nm_error.kind, nm_error.msg),
    )
}
