// TODO: cleanup tests
#![allow(dead_code, unused_imports)]

#[cfg(all(feature = "secp256k1", not(feature = "mockhsm")))]
use signatory::ecdsa::curve::Secp256k1;
use signatory::{
    ecdsa::{
        curve::{NistP256, NistP384, WeierstrassCurve, WeierstrassCurveKind},
        Asn1Signature,
    },
    PublicKeyed,
};
#[cfg(all(feature = "secp256k1", not(feature = "mockhsm")))]
use signatory_secp256k1::EcdsaVerifier as Secp256k1Verifier;
use yubihsm::{
    asymmetric::{self, Signer as SignerTrait},
    ecdsa, object, Client,
};

/// Domain IDs for test key
const TEST_SIGNING_KEY_DOMAINS: yubihsm::Domain = yubihsm::Domain::DOM1;

/// Capability for test key
const TEST_SIGNING_KEY_CAPABILITIES: yubihsm::Capability = yubihsm::Capability::SIGN_ECDSA;

/// Label for test key
const TEST_SIGNING_KEY_LABEL: &str = "Signatory test key";

/// Example message to sign
const TEST_MESSAGE: &[u8] =
    b"The Elliptic Curve Digital Signature Algorithm (ECDSA) is a variant of the \
      Digital Signature Algorithm (DSA) which uses elliptic curve cryptography.";

/// Create the signer for this test
fn create_signer<C>(key_id: object::Id) -> ecdsa::Signer<C>
where
    C: WeierstrassCurve,
{
    let client = crate::get_hsm_client();
    let alg = match C::CURVE_KIND {
        WeierstrassCurveKind::NistP256 => asymmetric::Algorithm::EC_P256,
        WeierstrassCurveKind::NistP384 => asymmetric::Algorithm::EC_P384,
        WeierstrassCurveKind::Secp256k1 => asymmetric::Algorithm::EC_K256,
    };

    create_yubihsm_key(&client, key_id, alg);
    ecdsa::Signer::create(client.clone(), key_id).unwrap()
}

/// Create the key on the YubiHSM to use for this test
fn create_yubihsm_key(client: &Client, key_id: object::Id, alg: yubihsm::asymmetric::Algorithm) {
    // Delete the key in TEST_KEY_ID slot it exists
    // Ignore errors since the object may not exist yet
    let _ = client.delete_object(key_id, yubihsm::object::Type::AsymmetricKey);

    // Create a new key for testing
    client
        .generate_asymmetric_key(
            key_id,
            TEST_SIGNING_KEY_LABEL.into(),
            TEST_SIGNING_KEY_DOMAINS,
            TEST_SIGNING_KEY_CAPABILITIES,
            alg,
        )
        .unwrap();
}

// Use *ring* to verify NIST P-256 ECDSA signatures
#[test]
#[cfg(not(feature = "mockhsm"))]
fn ecdsa_nistp256_sign_test() {
    let signer = create_signer::<NistP256>(201);
    let signature: Asn1Signature<_> = signatory::sign_sha256(&signer, TEST_MESSAGE).unwrap();
    let verifier = signatory_ring::ecdsa::P256Verifier::from(&signer.public_key().unwrap());
    assert!(signatory::verify_sha256(&verifier, TEST_MESSAGE, &signature).is_ok());
}

// Use *ring* to verify NIST P-384 ECDSA signatures
#[cfg(not(feature = "mockhsm"))]
#[test]
fn ecdsa_nistp384_sign_test() {
    let signer = create_signer::<NistP384>(202);
    let signature: Asn1Signature<_> = signatory::sign_sha384(&signer, TEST_MESSAGE).unwrap();
    let verifier = signatory_ring::ecdsa::P384Verifier::from(&signer.public_key().unwrap());
    assert!(signatory::verify_sha384(&verifier, TEST_MESSAGE, &signature).is_ok());
}

// Use `secp256k1` crate to verify secp256k1 ECDSA signatures.
// The MockHSM does not presently support secp256k1
#[cfg(all(feature = "secp256k1", not(feature = "mockhsm")))]
#[test]
fn ecdsa_secp256k1_sign_test() {
    let signer = create_signer::<Secp256k1>(203);
    let signature: Asn1Signature<_> = signatory::sign_sha256(&signer, TEST_MESSAGE).unwrap();
    let verifier = Secp256k1Verifier::from(&signer.public_key().unwrap());
    assert!(signatory::verify_sha256(&verifier, TEST_MESSAGE, &signature).is_ok());
}
