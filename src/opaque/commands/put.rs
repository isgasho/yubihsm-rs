//! Put an opaque object into the `YubiHSM 2`
//!
//! <https://developers.yubico.com/YubiHSM2/Commands/Put_Opaque.html>

use crate::{
    command::{self, Command},
    object,
    response::Response,
};

/// Request parameters for `command::put_opaque`
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct PutOpaqueCommand {
    /// Common parameters to all put object commands
    pub params: object::import::Params,

    /// Serialized object
    pub data: Vec<u8>,
}

impl Command for PutOpaqueCommand {
    type ResponseType = PutOpaqueResponse;
}

/// Response from `command::put_opaque`
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct PutOpaqueResponse {
    /// ID of the opaque data object
    pub object_id: object::Id,
}

impl Response for PutOpaqueResponse {
    const COMMAND_CODE: command::Code = command::Code::PutOpaqueObject;
}
