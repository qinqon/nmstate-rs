use crate::io_nmstate_plugin::NetIface;
use crate::nm::NmrsSettingConnection;
use nmstate::NmstateError;
use std::convert::TryFrom;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NmrsProfile {
    connection: NmrsSettingConnection,
}

impl NmrsProfile {
    pub(crate) fn to_keyfile_string(&self) -> String {
        let mut keyfile_str = String::new();
        keyfile_str.push_str(&self.connection.to_keyfile_string());

        keyfile_str
    }
}

impl TryFrom<&NetIface> for NmrsProfile {
    type Error = NmstateError;
    fn try_from(iface: &NetIface) -> Result<Self, Self::Error> {
        Ok(NmrsProfile {
            connection: NmrsSettingConnection::new(iface)?,
        })
    }
}
