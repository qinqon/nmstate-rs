use std::convert::TryFrom;

use crate::dbus_proxy::NetworkManagerProxy;
use crate::error::{ErrorKind, NmError};

const NM_CHECKPOINT_CREATE_FLAG_DELETE_NEW_CONNECTIONS: u32 = 0x02;
const NM_CHECKPOINT_CREATE_FLAG_DISCONNECT_NEW_DEVICES: u32 = 0x04;

const CHECKPOINT_TMO: u32 = 30;

pub(crate) struct NmDbus<'a> {
    proxy: NetworkManagerProxy<'a>,
}

impl<'a> NmDbus<'a> {
    pub(crate) fn new() -> Result<Self, NmError> {
        let connection = zbus::Connection::new_system()?;

        Ok(Self {
            proxy: NetworkManagerProxy::new(&connection)?,
        })
    }

    pub(crate) fn version(&self) -> Result<String, NmError> {
        Ok(self.proxy.version()?)
    }

    pub(crate) fn checkpoint_create(&self) -> Result<String, NmError> {
        match self.proxy.checkpoint_create(
            &[],
            CHECKPOINT_TMO,
            NM_CHECKPOINT_CREATE_FLAG_DELETE_NEW_CONNECTIONS
                | NM_CHECKPOINT_CREATE_FLAG_DISCONNECT_NEW_DEVICES,
        ) {
            Ok(cp) => Ok(cp.into_inner().as_str().to_string()),
            Err(e) => {
                Err(if let zbus::Error::MethodError(ref error_type, ..) = e {
                    if error_type
                        == "org.freedesktop.NetworkManager.InvalidArguments"
                    {
                        NmError::new(
                            ErrorKind::CheckpointConflict,
                            "Another checkpoint exists, \
                            please wait its timeout or destroy it"
                                .to_string(),
                        )
                    } else {
                        e.into()
                    }
                } else {
                    e.into()
                })
            }
        }
    }

    pub(crate) fn checkpoint_destroy(
        &self,
        checkpoint: &str,
    ) -> Result<(), NmError> {
        let checkpoint_obj = match zvariant::ObjectPath::try_from(checkpoint) {
            Ok(o) => o,
            Err(e) => {
                return Err(NmError::new(
                    ErrorKind::InvalidArgument,
                    format!("Invalid checkpoint: {}", e),
                ));
            }
        };
        Ok(self.proxy.checkpoint_destroy(&checkpoint_obj)?)
    }
}
