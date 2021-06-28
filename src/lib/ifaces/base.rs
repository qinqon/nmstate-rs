use serde::{Deserialize, Serialize};

use crate::{InterfaceState, InterfaceType};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct BaseInterface {
    pub name: String,
    #[serde(skip_serializing)] // Done by enum tag
    pub iface_type: InterfaceType,
    pub state: InterfaceState,
}
