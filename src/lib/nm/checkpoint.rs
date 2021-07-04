use nm_dbus::NmApi;

use crate::{nm::error::nm_error_to_nmstate, NmstateError};

pub(crate) fn nm_checkpoint_create() -> Result<String, NmstateError> {
    let nm_api = NmApi::new()
        .or_else(|ref nm_error| Err(nm_error_to_nmstate(nm_error)))?;
    nm_api
        .checkpoint_create()
        .map_err(|ref nm_error| nm_error_to_nmstate(nm_error))
}

pub(crate) fn nm_checkpoint_rollback(
    checkpoint: &str,
) -> Result<(), NmstateError> {
    let nm_api = NmApi::new()
        .or_else(|ref nm_error| Err(nm_error_to_nmstate(nm_error)))?;
    nm_api
        .checkpoint_rollback(checkpoint)
        .map_err(|ref nm_error| nm_error_to_nmstate(nm_error))
}

pub(crate) fn nm_checkpoint_destroy(
    checkpoint: &str,
) -> Result<(), NmstateError> {
    let nm_api = NmApi::new()
        .or_else(|ref nm_error| Err(nm_error_to_nmstate(nm_error)))?;
    nm_api
        .checkpoint_destroy(checkpoint)
        .map_err(|ref nm_error| nm_error_to_nmstate(nm_error))
}
