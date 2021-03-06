//! Wrap Keys for making encrypted backups of keys within the HSM, so they can be
//! imported into other devices.
//!
//! The `wrap::Key` builder type is presently only used by `yubihsm::setup`
//! functionality.

// TODO(tarcieri): use this for `yubihsm::client::put_wrap_key` in general?

use crate::{
    client::ClientError,
    device::{DeviceError, DeviceErrorKind::*},
    object, wrap, Capability, Client, Domain,
};
use rand_os::{rand_core::RngCore, OsRng};
use std::fmt::{self, Debug};
use zeroize::Zeroize;

/// Wrap key to import into the device
#[derive(Clone)]
pub struct Key {
    /// Object parameters
    pub(crate) import_params: object::import::Params,

    /// Delegated capabilities apply to objects imported by this key
    pub(crate) delegated_capabilities: Capability,

    /// Key bytes
    pub(crate) data: Vec<u8>,
}

impl Key {
    /// Generate a random wrap key with the given key size.
    pub fn generate_random(key_id: object::Id, algorithm: wrap::Algorithm) -> Self {
        let mut rand = OsRng::new().unwrap();
        let mut bytes = vec![0u8; algorithm.key_len()];
        rand.fill_bytes(&mut bytes);

        let result = Self::from_bytes(key_id, &bytes).unwrap();
        bytes.zeroize();
        result
    }

    /// Create a new `wrap::Key` instance. Must be 16, 24, or 32-bytes long.
    pub fn from_bytes(key_id: object::Id, bytes: &[u8]) -> Result<Self, DeviceError> {
        let alg = match bytes.len() {
            16 => wrap::Algorithm::AES128_CCM,
            24 => wrap::Algorithm::AES192_CCM,
            32 => wrap::Algorithm::AES256_CCM,
            other => fail!(
                WrongLength,
                "expected 16, 24, or 32-byte wrap key (got {})",
                other
            ),
        };

        let object_params = object::import::Params::new(key_id, alg.into());

        Ok(Self {
            import_params: object_params,
            delegated_capabilities: Default::default(),
            data: bytes.to_vec(),
        })
    }

    /// Set the object label on this key
    pub fn label(mut self, label: object::Label) -> Self {
        self.import_params.label = label;
        self
    }

    /// Set the domains this wrap key can be used in (default: all)
    pub fn domains(mut self, domains: Domain) -> Self {
        self.import_params.domains = domains;
        self
    }

    /// Set the capabilities of this key (what it can be used for)
    pub fn capabilities(mut self, capabilities: Capability) -> Self {
        self.import_params.capabilities = capabilities;
        self
    }

    /// Set the delegated capabilities of this key (what capabilities it can
    /// set on imported objects)
    pub fn delegated_capabilities(mut self, capabilities: Capability) -> Self {
        self.delegated_capabilities = capabilities;
        self
    }

    /// Create this key within the HSM
    pub fn create(&self, client: &mut Client) -> Result<(), ClientError> {
        let algorithm = self.import_params.algorithm.wrap().unwrap();

        client.put_wrap_key(
            self.import_params.id,
            self.import_params.label.clone(),
            self.import_params.domains,
            self.import_params.capabilities,
            self.delegated_capabilities,
            algorithm,
            self.data.clone(),
        )?;

        Ok(())
    }
}

impl Debug for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Avoid leaking secrets in debug messages
        write!(
            f,
            "yubihsm:wrap::Key {{ import_params: {:?}, delegated_capabilities: {:?}, data: ... }}",
            self.import_params, self.delegated_capabilities
        )
    }
}

impl Drop for Key {
    fn drop(&mut self) {
        self.data.zeroize();
    }
}
