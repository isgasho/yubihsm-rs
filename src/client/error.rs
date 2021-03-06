//! Errors that occur during sessions

use crate::{
    connector::{ConnectionError, ConnectionErrorKind},
    device::DeviceErrorKind,
    error::Error,
    serialization::SerializationError,
    session::{SessionError, SessionErrorKind},
};
use std::{error::Error as StdError, io};

/// Session errors
pub type ClientError = Error<ClientErrorKind>;

/// Session error kinds
#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum ClientErrorKind {
    /// Couldn't authenticate session
    #[fail(display = "authentication failed")]
    AuthenticationError,

    /// Session is closed
    #[fail(display = "session closed")]
    ClosedSessionError,

    /// Errors with the connection to the HSM
    #[fail(display = "connection error")]
    ConnectionError {
        /// Connection error kind
        kind: ConnectionErrorKind,
    },

    /// Couldn't create session
    #[fail(display = "couldn't create session")]
    CreateFailed,

    /// Errors originating in the HSM device
    #[fail(display = "HSM error: {}", kind)]
    DeviceError {
        /// HSM error kind
        kind: DeviceErrorKind,
    },

    /// Protocol error occurred
    #[fail(display = "protocol error")]
    ProtocolError,

    /// Error response from HSM we can't further specify
    #[fail(display = "HSM error")]
    ResponseError,
}

impl ClientErrorKind {
    /// Get the device error, if this is a device error
    pub fn device_error(self) -> Option<DeviceErrorKind> {
        match self {
            ClientErrorKind::DeviceError { kind } => Some(kind),
            _ => None,
        }
    }
}

// TODO: capture causes?
impl From<ConnectionError> for ClientError {
    fn from(err: ConnectionError) -> Self {
        let kind = ClientErrorKind::ConnectionError { kind: err.kind() };
        err!(kind, err.description())
    }
}

// TODO: capture causes?
impl From<SessionError> for ClientError {
    fn from(err: SessionError) -> Self {
        let kind = match err.kind() {
            SessionErrorKind::AuthenticationError => ClientErrorKind::AuthenticationError,
            SessionErrorKind::ClosedSessionError => ClientErrorKind::ClosedSessionError,
            SessionErrorKind::CreateFailed => ClientErrorKind::CreateFailed,
            SessionErrorKind::DeviceError { kind } => ClientErrorKind::DeviceError { kind },
            SessionErrorKind::ProtocolError
            | SessionErrorKind::CommandLimitExceeded
            | SessionErrorKind::MismatchError
            | SessionErrorKind::VerifyFailed => ClientErrorKind::ProtocolError,
            SessionErrorKind::ResponseError => ClientErrorKind::ResponseError,
        };

        err!(kind, err.description())
    }
}

impl From<io::Error> for ClientError {
    fn from(err: io::Error) -> Self {
        err!(ClientErrorKind::ProtocolError, err.description())
    }
}

// TODO: capture causes?
impl From<SerializationError> for ClientError {
    fn from(err: SerializationError) -> Self {
        err!(ClientErrorKind::ProtocolError, err.description())
    }
}

impl From<ClientError> for signatory::Error {
    fn from(client_error: ClientError) -> signatory::Error {
        signatory::Error::new(
            signatory::ErrorKind::ProviderError,
            Some(&client_error.to_string()),
        )
    }
}
