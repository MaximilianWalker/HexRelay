use ring::rand::SystemRandom;
use ring::signature::Ed25519KeyPair;

use crate::domain::{
    canonical_descriptor_signing_payload, ed25519_public_key_hex, sign_descriptor_ed25519_pkcs8,
    verify_descriptor_ed25519, DescriptorValidationContext, DiscoveryPolicy,
    Ed25519DescriptorVerifier, NetworkMode, PeeringPolicy, RelayPolicy, ServerDescriptor,
    ServerDescriptorSignatureError, ServerDescriptorValidationError, ServerSignature,
    ServerSignatureAlgorithm, StoragePolicy,
};

use crate::domain::DmForwardingPolicy;

fn validation_context() -> DescriptorValidationContext {
    DescriptorValidationContext {
        now_epoch_seconds: 1_000,
        max_ttl_seconds: 600,
        revoked_descriptor_ids: Vec::new(),
    }
}

fn descriptor(public_key: String) -> ServerDescriptor {
    ServerDescriptor {
        server_id: "server-a".to_string(),
        server_public_key: public_key,
        descriptor_id: "descriptor-a".to_string(),
        issued_at_epoch_seconds: 1_000,
        expires_at_epoch_seconds: 1_300,
        network_mode: NetworkMode::PrivatePeers,
        discovery_policy: DiscoveryPolicy::PrivateAllowlist,
        peering_policy: PeeringPolicy::StaticAllowlist,
        relay_policy: RelayPolicy::None,
        dm_forwarding_policy: DmForwardingPolicy::LocalRecipientsOnly,
        storage_policy: StoragePolicy::DurableEncryptedEnvelopes,
        addresses: vec!["https://server-a.example".to_string()],
        supported_protocols: vec!["hexrelay-server-http".to_string()],
        rate_limits: Vec::new(),
        trust_labels: Vec::new(),
        revocation_pointer: Some("https://server-a.example/revocations".to_string()),
        signature: ServerSignature {
            algorithm: ServerSignatureAlgorithm::Ed25519,
            value: String::new(),
        },
    }
}

fn generated_pkcs8() -> Vec<u8> {
    Ed25519KeyPair::generate_pkcs8(&SystemRandom::new())
        .expect("generate ed25519 key")
        .as_ref()
        .to_vec()
}

fn signed_descriptor() -> (ServerDescriptor, Vec<u8>) {
    let pkcs8 = generated_pkcs8();
    let public_key = ed25519_public_key_hex(&pkcs8).expect("derive public key");
    let mut descriptor = descriptor(public_key);
    descriptor.signature.value =
        sign_descriptor_ed25519_pkcs8(&descriptor, &pkcs8).expect("sign descriptor");

    (descriptor, pkcs8)
}

#[test]
fn signs_and_verifies_descriptor_ed25519_signature() {
    let (descriptor, _) = signed_descriptor();

    let result =
        descriptor.validate_with_signature(&validation_context(), &Ed25519DescriptorVerifier);

    assert_eq!(result, Ok(()));
    assert_eq!(verify_descriptor_ed25519(&descriptor), Ok(()));
}

#[test]
fn canonical_payload_is_stable_and_excludes_signature_value() {
    let (mut descriptor, _) = signed_descriptor();
    let first = canonical_descriptor_signing_payload(&descriptor);

    descriptor.signature.value = "different-signature-value".to_string();
    let second = canonical_descriptor_signing_payload(&descriptor);

    assert_eq!(first, second);
}

#[test]
fn canonical_payload_changes_when_policy_changes() {
    let (mut descriptor, _) = signed_descriptor();
    let first = canonical_descriptor_signing_payload(&descriptor);

    descriptor.discovery_policy = DiscoveryPolicy::UserConsentedIntroduction;
    let second = canonical_descriptor_signing_payload(&descriptor);

    assert_ne!(first, second);
}

#[test]
fn rejects_signature_after_descriptor_mutation() {
    let (mut descriptor, _) = signed_descriptor();
    descriptor.discovery_policy = DiscoveryPolicy::UserConsentedIntroduction;

    let result = verify_descriptor_ed25519(&descriptor);

    assert_eq!(
        result,
        Err(ServerDescriptorSignatureError::SignatureVerificationFailed)
    );
}

#[test]
fn rejects_invalid_public_key_encoding() {
    let (mut descriptor, _) = signed_descriptor();
    descriptor.server_public_key = "not-a-public-key".to_string();

    let result = verify_descriptor_ed25519(&descriptor);

    assert_eq!(
        result,
        Err(ServerDescriptorSignatureError::InvalidPublicKeyEncoding)
    );
}

#[test]
fn rejects_invalid_signature_encoding() {
    let (mut descriptor, _) = signed_descriptor();
    descriptor.signature.value = "not-a-signature".to_string();

    let result = verify_descriptor_ed25519(&descriptor);

    assert_eq!(
        result,
        Err(ServerDescriptorSignatureError::InvalidSignatureEncoding)
    );
}

#[test]
fn descriptor_validation_maps_ed25519_failure_to_existing_error() {
    let (mut descriptor, _) = signed_descriptor();
    descriptor.server_id = "server-mutated".to_string();

    let result =
        descriptor.validate_with_signature(&validation_context(), &Ed25519DescriptorVerifier);

    assert_eq!(
        result,
        Err(ServerDescriptorValidationError::SignatureVerificationFailed)
    );
}
