use crate::io_nmstate_plugin::NetIface;
use ini::{EscapePolicy, Ini};
use nmstate::{IfaceType, NmstateError};
use std::io::Cursor;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct NmrsSettingConnection {
    id: String,
    uuid: String,
    iface_type: String,
    iface_name: String,
}

impl NmrsSettingConnection {
    pub(crate) fn new(iface: &NetIface) -> Result<Self, NmstateError> {
        Ok(Self {
            id: iface.name.clone(),
            uuid: Uuid::new_v4().to_string(),
            iface_type: nmstate_iface_type_to_nm(iface.r#type.as_str().into())?,
            iface_name: iface.name.clone(),
        })
    }

    pub(crate) fn to_keyfile_string(&self) -> String {
        let mut key_file: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        let mut ini_obj = Ini::new();
        ini_obj
            .with_section(Some("connection"))
            .set("id", &self.id)
            .set("uuid", &self.uuid)
            .set("type", &self.iface_type);
        ini_obj
            .write_to_policy(&mut key_file, EscapePolicy::Basics)
            .unwrap();
        std::string::String::from_utf8(key_file.into_inner()).unwrap()
    }
}

fn nmstate_iface_type_to_nm(
    nmstate_iface_type: IfaceType,
) -> Result<String, NmstateError> {
    match nmstate_iface_type {
        IfaceType::Ethernet => Ok("ethernet".into()),
        _ => Err(NmstateError::invalid_argument(format!(
            "Unsupported interface type: {:?}",
            nmstate_iface_type
        ))),
    }
}
