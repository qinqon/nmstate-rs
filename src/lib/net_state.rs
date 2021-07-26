use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    nispor::nispor_retrieve,
    nm::{
        nm_apply, nm_checkpoint_create, nm_checkpoint_destroy,
        nm_checkpoint_rollback, nm_checkpoint_timeout_extend, nm_retrieve,
    },
    Interface, Interfaces, NmstateError,
};

const VERIFY_RETRY_INTERVAL_MILLISECONDS: u64 = 500;
const VERIFY_RETRY_COUNT: usize = 60;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct NetworkState {
    #[serde(default)]
    pub interfaces: Interfaces,
    #[serde(skip)]
    pub prop_list: Vec<&'static str>,
}

impl NetworkState {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn append_interface_data(&mut self, iface: Interface) {
        self.interfaces.push(iface);
    }

    pub fn retrieve() -> Result<Self, NmstateError> {
        let mut state = nispor_retrieve()?;
        let nm_state = nm_retrieve()?;
        // TODO: Priority handling
        state.update_state(&nm_state)?;
        Ok(state)
    }

    pub fn apply(&self) -> Result<(), NmstateError> {
        // TODO: Verify
        let checkpoint = nm_checkpoint_create()?;
        with_nm_checkpoint(&checkpoint, || {
            let desire_state = self.clone();
            nm_apply(self, &checkpoint)?;
            nm_checkpoint_timeout_extend(
                &checkpoint,
                (VERIFY_RETRY_INTERVAL_MILLISECONDS * VERIFY_RETRY_COUNT as u64
                    / 1000) as u32,
            )?;
            with_retry(
                VERIFY_RETRY_INTERVAL_MILLISECONDS,
                VERIFY_RETRY_COUNT,
                || {
                    let cur_state = NetworkState::retrieve()?;
                    desire_state.verify(&cur_state)
                },
            )
        })
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
}

fn with_nm_checkpoint<T>(checkpoint: &str, func: T) -> Result<(), NmstateError>
where
    T: FnOnce() -> Result<(), NmstateError>,
{
    match func() {
        Ok(()) => nm_checkpoint_destroy(&checkpoint),
        Err(e) => {
            if let Err(e) = nm_checkpoint_rollback(&checkpoint) {
                eprintln!("nm_checkpoint_rollback() failed: {}", e);
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
                eprintln!("Retrying on verification failure: {}", e);
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
