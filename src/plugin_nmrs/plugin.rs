use crate::io_nmstate_plugin::NetState;
use crate::nm::NmrsProfile;
use nmstate::NmstateError;
use std::convert::TryInto;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NmPlugin {}

impl NmPlugin {
    pub(crate) fn new() -> Result<Self, NmstateError> {
        Ok(Self {})
    }

    pub(crate) fn generate_configs(
        &self,
        net_state: &NetState,
    ) -> Result<Vec<String>, NmstateError> {
        if let Some(ifaces) = &net_state.interfaces {
            let mut confs = Vec::new();
            for iface in ifaces {
                let profile: NmrsProfile = iface.try_into()?;
                confs.push(profile.to_keyfile_string());
            }
            Ok(confs)
        } else {
            Ok(Vec::new())
        }
    }
}
