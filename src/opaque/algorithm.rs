//! Pseudo-algorithms for opaque data

use crate::algorithm::{AlgorithmError, AlgorithmErrorKind::TagInvalid};

/// Valid algorithms for opaque data
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum Algorithm {
    /// Arbitrary opaque data
    DATA = 0x1e,

    /// X.509 certificates
    X509_CERTIFICATE = 0x1f,
}

impl Algorithm {
    /// Convert an unsigned byte tag into an `Algorithmorithm` (if valid)
    pub fn from_u8(tag: u8) -> Result<Self, AlgorithmError> {
        Ok(match tag {
            0x1e => Algorithm::DATA,
            0x1f => Algorithm::X509_CERTIFICATE,
            _ => fail!(TagInvalid, "unknown opaque data ID: 0x{:02x}", tag),
        })
    }

    /// Serialize algorithm ID as a byte
    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

impl_algorithm_serializers!(Algorithm);
