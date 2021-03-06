//! Simulation of the HSM for integration testing.

// TESTING ONLY DO NOT PRODUCTIONIZE IT IS NOT SAFE!!!
#[cfg(not(debug_assertions))]
compile_error!("MockHsm is not intended for use in release builds");

use std::sync::{Arc, Mutex};

mod audit;
mod command;
mod connection;
mod object;
mod session;
mod state;

pub use self::connection::MockConnection;
use self::state::State;
use crate::connector::{Connectable, Connection, ConnectionError};

/// Mock serial number for the MockHsm
pub const MOCK_SERIAL_NUMBER: &str = "0123456789";

/// Software simulation of a `YubiHSM 2` intended for testing
/// implemented as a `yubihsm::Connection`.
///
/// This only implements a subset of the YubiHSM's functionality, and does
/// *NOT* properly enforce access control / capabilities!
///
/// It is *STRONGLY* recommended to also test live against a real device.
///
/// To enable, make sure to build yubihsm.rs with the `mockhsm` cargo feature
#[derive(Clone, Debug)]
pub struct MockHsm(Arc<Mutex<State>>);

impl MockHsm {
    /// Create a new MockHsm
    pub fn new() -> Self {
        MockHsm(Arc::new(Mutex::new(State::new())))
    }
}

impl Connectable for MockHsm {
    /// Make a clone of this connectable as boxed trait object
    fn box_clone(&self) -> Box<dyn Connectable> {
        Box::new(MockHsm(self.0.clone()))
    }

    /// Create a new connection with a clone of the MockHsm state
    fn connect(&self) -> Result<Box<Connection>, ConnectionError> {
        Ok(Box::new(MockConnection::new(self)))
    }
}

impl Default for MockHsm {
    fn default() -> Self {
        Self::new()
    }
}

impl Into<Box<dyn Connectable>> for MockHsm {
    fn into(self) -> Box<Connectable> {
        Box::new(self)
    }
}
