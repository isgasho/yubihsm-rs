//! Responses sent by the HSM after executing a command.
//!
//! <https://developers.yubico.com/YubiHSM2/Commands/>

// TODO: this code predates the serde serializers. It could be rewritten with serde.

#[cfg(feature = "mockhsm")]
use byteorder::WriteBytesExt;
use byteorder::{BigEndian, ByteOrder};

#[cfg(feature = "mockhsm")]
use crate::device::DeviceErrorKind;
use crate::{
    command, connector, response,
    session::{
        self,
        securechannel::{Mac, MAC_SIZE},
        SessionError,
        SessionErrorKind::ProtocolError,
    },
};

/// Command responses
#[derive(Debug)]
pub(crate) struct Message {
    /// Success (for a given command type) or an error type
    pub code: response::Code,

    /// Session ID for this response
    pub session_id: Option<session::Id>,

    /// "Response Data Field"
    pub data: Vec<u8>,

    /// Optional Message Authentication Code (MAC)
    pub mac: Option<Mac>,
}

impl Message {
    /// Parse a response into a Response struct
    pub fn parse(message: connector::Message) -> Result<Self, SessionError> {
        let connector::Message(mut bytes) = message;

        if bytes.len() < 3 {
            fail!(
                ProtocolError,
                "response too short: {} (expected at least 3-bytes)",
                bytes.len()
            );
        }

        let code = response::Code::from_u8(bytes[0]).map_err(|e| err!(ProtocolError, "{}", e))?;
        let length = BigEndian::read_u16(&bytes[1..3]) as usize;

        if length.checked_add(3).unwrap() != bytes.len() {
            fail!(
                ProtocolError,
                "unexpected response length {} (expecting {})",
                bytes.len().checked_sub(3).unwrap(),
                length
            );
        }

        bytes.drain(..3);

        let session_id = if has_session_id(code) {
            if bytes.is_empty() {
                fail!(ProtocolError, "session ID missing");
            } else {
                Some(session::Id::from_u8(bytes.remove(0))?)
            }
        } else {
            None
        };

        let mac = if has_rmac(code) {
            if bytes.len() < MAC_SIZE {
                fail!(ProtocolError, "missing R-MAC for {:?}", code,);
            }

            let mac_index = bytes.len() - MAC_SIZE;
            Some(Mac::from_slice(&bytes.split_off(mac_index)))
        } else {
            None
        };

        Ok(Self {
            code,
            session_id,
            data: bytes,
            mac,
        })
    }

    /// Create a new response without an associated session
    #[cfg(feature = "mockhsm")]
    pub fn new<T>(code: response::Code, response_data: T) -> Self
    where
        T: Into<Vec<u8>>,
    {
        Self {
            code,
            session_id: None,
            data: response_data.into(),
            mac: None,
        }
    }

    /// Create a new response message with a MAC
    #[cfg(feature = "mockhsm")]
    pub fn new_with_mac<D, M>(
        code: response::Code,
        session_id: session::Id,
        response_data: D,
        mac: M,
    ) -> Self
    where
        D: Into<Vec<u8>>,
        M: Into<Mac>,
    {
        Self {
            code,
            session_id: Some(session_id),
            data: response_data.into(),
            mac: Some(mac.into()),
        }
    }

    /// Create a successful response
    #[cfg(feature = "mockhsm")]
    pub fn success<T>(command_type: command::Code, response_data: T) -> Self
    where
        T: Into<Vec<u8>>,
    {
        Self::new(response::Code::Success(command_type), response_data)
    }

    /// Did an error occur?
    pub fn is_err(&self) -> bool {
        match self.code {
            response::Code::Success(_) => false,
            _ => true,
        }
    }

    /// Get the command being responded to
    pub fn command(&self) -> Option<command::Code> {
        match self.code {
            response::Code::Success(cmd) => Some(cmd),
            _ => None,
        }
    }

    /// Total length of the response
    pub fn len(&self) -> usize {
        let mut result = self.data.len();

        if self.session_id.is_some() {
            result += 1;
        }

        if self.mac.is_some() {
            result += MAC_SIZE;
        }

        result
    }
}

#[cfg(feature = "mockhsm")]
impl From<DeviceErrorKind> for Message {
    fn from(kind: DeviceErrorKind) -> Self {
        Self::new(response::Code::MemoryError, vec![kind.to_u8()])
    }
}

#[cfg(feature = "mockhsm")]
impl Into<Vec<u8>> for Message {
    /// Serialize this response, consuming it and producing a Vec<u8>
    fn into(mut self) -> Vec<u8> {
        let mut result = Vec::with_capacity(3 + self.len());
        result.push(self.code.to_u8());
        result.write_u16::<BigEndian>(self.len() as u16).unwrap();

        if let Some(session_id) = self.session_id {
            result.push(session_id.to_u8());
        }

        result.append(&mut self.data);

        if let Some(mac) = self.mac {
            result.extend_from_slice(mac.as_slice());
        }

        result
    }
}

/// Do responses with the given code include a session ID?
fn has_session_id(code: response::Code) -> bool {
    match code {
        response::Code::Success(cmd_type) => match cmd_type {
            command::Code::CreateSession | command::Code::SessionMessage => true,
            _ => false,
        },
        _ => false,
    }
}

/// Do responses with the given code have a Response-MAC (R-MAC) value?
fn has_rmac(code: response::Code) -> bool {
    match code {
        response::Code::Success(cmd_type) => match cmd_type {
            command::Code::SessionMessage => true,
            _ => false,
        },
        _ => false,
    }
}
