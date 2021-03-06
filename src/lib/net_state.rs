use std::collections::HashMap;

use log::{debug, warn};
use serde::{Deserialize, Serialize};

use crate::{
    nispor::{nispor_apply, nispor_retrieve},
    nm::{
        nm_apply, nm_checkpoint_create, nm_checkpoint_destroy,
        nm_checkpoint_rollback, nm_checkpoint_timeout_extend, nm_retrieve,
    },
    ErrorKind, Interface, Interfaces, NmstateError,
};

const VERIFY_RETRY_INTERVAL_MILLISECONDS: u64 = 500;
const VERIFY_RETRY_COUNT: usize = 60;
const VERIFY_RETRY_COUNT_KERNEL_MODE: usize = 6;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct NetworkState {
    #[serde(default)]
    pub interfaces: Interfaces,
    #[serde(skip)]
    // Contain a list of struct member name which is defined explicitly in
    // desire state instead of generated.
    pub prop_list: Vec<&'static str>,
    #[serde(skip)]
    // TODO: Hide user space only info when serialize
    kernel_only: bool,
}

impl NetworkState {
    pub fn set_kernel_only(&mut self, value: bool) -> &mut Self {
        self.kernel_only = value;
        self
    }

    pub fn new() -> Self {
        Default::default()
    }

    // We provide this instead asking use to do serde_json::from_str(), so that
    // we could provide better error NmstateError instead of serde_json one.
    pub fn new_from_json(net_state_json: &str) -> Result<Self, NmstateError> {
        match serde_json::from_str(net_state_json) {
            Ok(s) => Ok(s),
            Err(e) => Err(NmstateError::new(
                ErrorKind::InvalidArgument,
                format!("Invalid json string: {}", e),
            )),
        }
    }

    pub fn append_interface_data(&mut self, iface: Interface) {
        self.interfaces.push(iface);
    }

    pub fn retrieve(&mut self) -> Result<&mut Self, NmstateError> {
        self.interfaces = nispor_retrieve()?.interfaces;
        if !self.kernel_only {
            let nm_state = nm_retrieve()?;
            // TODO: Priority handling
            self.update_state(&nm_state)?;
        }
        Ok(self)
    }

    pub fn apply(&self) -> Result<(), NmstateError> {
        let desire_state_to_edit = self.clone();
        let desire_state_to_verify = self.clone();
        let mut cur_net_state = NetworkState::new();
        cur_net_state.set_kernel_only(self.kernel_only);
        cur_net_state.retrieve()?;

        let (add_net_state, chg_net_state, del_net_state) =
            desire_state_to_edit.gen_state_for_apply(&cur_net_state)?;

        if !self.kernel_only {
            let checkpoint = nm_checkpoint_create()?;
            with_nm_checkpoint(&checkpoint, || {
                nm_apply(
                    &add_net_state,
                    &chg_net_state,
                    &del_net_state,
                    &cur_net_state,
                    &checkpoint,
                )?;
                nm_checkpoint_timeout_extend(
                    &checkpoint,
                    (VERIFY_RETRY_INTERVAL_MILLISECONDS
                        * VERIFY_RETRY_COUNT as u64
                        / 1000) as u32,
                )?;
                with_retry(
                    VERIFY_RETRY_INTERVAL_MILLISECONDS,
                    VERIFY_RETRY_COUNT,
                    || {
                        let mut new_cur_net_state = cur_net_state.clone();
                        new_cur_net_state.retrieve()?;
                        desire_state_to_verify.verify(&new_cur_net_state)
                    },
                )
            })
        } else {
            // TODO: Need checkpoint for kernel only mode
            nispor_apply(
                &add_net_state,
                &chg_net_state,
                &del_net_state,
                &cur_net_state,
            )?;
            with_retry(
                VERIFY_RETRY_INTERVAL_MILLISECONDS,
                VERIFY_RETRY_COUNT_KERNEL_MODE,
                || {
                    let mut new_cur_net_state = cur_net_state.clone();
                    new_cur_net_state.retrieve()?;
                    desire_state_to_verify.verify(&new_cur_net_state)
                },
            )
        }
    }

    fn update_state(&mut self, other: &Self) -> Result<(), NmstateError> {
        if other.prop_list.contains(&"interfaces") {
            self.interfaces.update(&other.interfaces)
        } else {
            Ok(())
        }
    }

    pub fn gen_conf(
        &self,
    ) -> Result<HashMap<String, Vec<String>>, NmstateError> {
        todo!()
    }

    fn verify(&self, current: &Self) -> Result<(), NmstateError> {
        self.interfaces.verify(&current.interfaces)
    }

    // Return three NetworkState:
    //  * State for addition.
    //  * State for change.
    //  * State for deletion.
    // This function is the entry point for decision making which
    // expanding complex desire network layout to flat network layout.
    fn gen_state_for_apply(
        &self,
        current: &Self,
    ) -> Result<(Self, Self, Self), NmstateError> {
        let mut add_net_state = NetworkState::new();
        let mut chg_net_state = NetworkState::new();
        let mut del_net_state = NetworkState::new();

        let (add_ifaces, chg_ifaces, del_ifaces) =
            self.interfaces.gen_state_for_apply(&current.interfaces)?;

        debug!("DEBUG, new interfaces {:?}", add_ifaces);
        debug!("DEBUG, chg interfaces {:?}", chg_ifaces);
        debug!("DEBUG, del interfaces {:?}", del_ifaces);

        add_net_state.interfaces = add_ifaces;
        add_net_state.prop_list = vec!["interfaces"];

        chg_net_state.interfaces = chg_ifaces;
        chg_net_state.prop_list = vec!["interfaces"];

        del_net_state.interfaces = del_ifaces;
        del_net_state.prop_list = vec!["interfaces"];

        Ok((add_net_state, chg_net_state, del_net_state))
    }
}

fn with_nm_checkpoint<T>(checkpoint: &str, func: T) -> Result<(), NmstateError>
where
    T: FnOnce() -> Result<(), NmstateError>,
{
    match func() {
        Ok(()) => nm_checkpoint_destroy(&checkpoint),
        Err(e) => {
            if let Err(e) = nm_checkpoint_rollback(&checkpoint) {
                warn!("nm_checkpoint_rollback() failed: {}", e);
            }
            Err(e)
        }
    }
}

fn with_retry<T>(
    interval_ms: u64,
    count: usize,
    func: T,
) -> Result<(), NmstateError>
where
    T: FnOnce() -> Result<(), NmstateError> + Copy,
{
    let mut cur_count = 0usize;
    while cur_count < count {
        if let Err(e) = func() {
            if cur_count == count - 1 {
                return Err(e);
            } else {
                warn!("Retrying on verification failure: {}", e);
                std::thread::sleep(std::time::Duration::from_millis(
                    interval_ms,
                ));
                cur_count += 1;
                continue;
            }
        } else {
            return Ok(());
        }
    }
    Ok(())
}
