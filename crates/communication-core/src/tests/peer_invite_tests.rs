use crate::{
    ed25519_public_key_hex, sign_descriptor_ed25519_pkcs8, sign_peer_invite_ed25519_pkcs8,
    verify_peer_invite_ed25519, DescriptorValidationContext, DiscoveryPath, DiscoveryPolicy,
    DmForwardingPolicy, Ed25519DescriptorVerifier, NetworkMode, PeerInvite, PeerInviteEnvelope,
    PeerInviteSignatureError, PeerInviteValidationContext, PeerInviteValidationError,
    PeeringPolicy, RelayPolicy, ServerDescriptor, ServerSignature, ServerSignatureAlgorithm,
    StoragePolicy,
};
use ring::rand::SystemRandom;
use ring::signature::Ed25519KeyPair;

struct SignedDescriptor {
    descriptor: ServerDescriptor,
    private_key_pkcs8: Vec<u8>,
}

fn signed_descriptor(peering_policy: PeeringPolicy) -> SignedDescriptor {
    let pkcs8 = Ed25519KeyPair::generate_pkcs8(&SystemRandom::new()).expect("generate ed25519 key");
    let public_key = ed25519_public_key_hex(pkcs8.as_ref()).expect("derive public key");
    let mut descriptor = ServerDescriptor {
        server_id: "server-inviter".to_string(),
        server_public_key: public_key,
        descriptor_id: "descriptor-inviter".to_string(),
        issued_at_epoch_seconds: 1_700_000_000,
        expires_at_epoch_seconds: 1_700_000_600,
        network_mode: NetworkMode::PrivatePeers,
        discovery_policy: DiscoveryPolicy::PrivateAllowlist,
        peering_policy,
        relay_policy: RelayPolicy::None,
        dm_forwarding_policy: DmForwardingPolicy::LocalRecipientsOnly,
        storage_policy: StoragePolicy::DurableEncryptedEnvelopes,
        addresses: vec!["https://server-inviter.example".to_string()],
        supported_protocols: vec!["hexrelay-server-http".to_string()],
        rate_limits: Vec::new(),
        trust_labels: Vec::new(),
        revocation_pointer: None,
        signature: ServerSignature {
            algorithm: ServerSignatureAlgorithm::Ed25519,
            value: String::new(),
        },
    };
    descriptor.signature.value =
        sign_descriptor_ed25519_pkcs8(&descriptor, pkcs8.as_ref()).expect("sign descriptor");

    SignedDescriptor {
        descriptor,
        private_key_pkcs8: pkcs8.as_ref().to_vec(),
    }
}

fn unsigned_invite(subject_server_id: Option<&str>) -> PeerInvite {
    PeerInvite {
        invite_id: "peer-invite-1".to_string(),
        issuer_server_id: "server-inviter".to_string(),
        issuer_descriptor_id: "descriptor-inviter".to_string(),
        subject_server_id: subject_server_id.map(str::to_string),
        issued_at_epoch_seconds: 1_700_000_010,
        expires_at_epoch_seconds: 1_700_000_310,
        discovery_path: DiscoveryPath::PrivateAllowlist,
        peering_policy: PeeringPolicy::InviteToken,
        max_uses: Some(1),
        signature: ServerSignature {
            algorithm: ServerSignatureAlgorithm::Ed25519,
            value: String::new(),
        },
    }
}

fn signed_invite(issuer: &SignedDescriptor, subject_server_id: Option<&str>) -> PeerInvite {
    let mut invite = unsigned_invite(subject_server_id);
    invite.signature.value = sign_peer_invite_ed25519_pkcs8(&invite, &issuer.private_key_pkcs8)
        .expect("sign peer invite");
    invite
}

fn invite_context(expected_subject_server_id: Option<&str>) -> PeerInviteValidationContext {
    PeerInviteValidationContext {
        now_epoch_seconds: 1_700_000_020,
        max_ttl_seconds: 600,
        revoked_invite_ids: Vec::new(),
        expected_subject_server_id: expected_subject_server_id.map(str::to_string),
    }
}

fn descriptor_context() -> DescriptorValidationContext {
    DescriptorValidationContext {
        now_epoch_seconds: 1_700_000_020,
        max_ttl_seconds: 600,
        revoked_descriptor_ids: Vec::new(),
    }
}

#[test]
fn validates_subject_bound_signed_peer_invite() {
    let issuer = signed_descriptor(PeeringPolicy::InviteToken);
    let envelope = PeerInviteEnvelope {
        issuer_descriptor: issuer.descriptor.clone(),
        invite: signed_invite(&issuer, Some("server-local")),
    };

    envelope
        .issuer_descriptor
        .validate_with_signature(&descriptor_context(), &Ed25519DescriptorVerifier)
        .expect("issuer descriptor is valid");
    envelope
        .invite
        .validate(
            &envelope.issuer_descriptor,
            &invite_context(Some("server-local")),
        )
        .expect("invite policy is valid");
    verify_peer_invite_ed25519(&envelope.invite, &envelope.issuer_descriptor)
        .expect("invite signature is valid");
}

#[test]
fn rejects_subject_bound_invite_for_different_local_server() {
    let issuer = signed_descriptor(PeeringPolicy::InviteToken);
    let invite = signed_invite(&issuer, Some("server-other"));

    let error = invite
        .validate(&issuer.descriptor, &invite_context(Some("server-local")))
        .expect_err("subject mismatch should fail");

    assert_eq!(
        error,
        PeerInviteValidationError::SubjectServerMismatch {
            expected_subject_server_id: Some("server-local".to_string()),
            invite_subject_server_id: "server-other".to_string(),
        }
    );
}

#[test]
fn rejects_invite_when_issuer_descriptor_refuses_invite_token_peering() {
    let issuer = signed_descriptor(PeeringPolicy::StaticAllowlist);
    let invite = signed_invite(&issuer, Some("server-local"));

    let error = invite
        .validate(&issuer.descriptor, &invite_context(Some("server-local")))
        .expect_err("static allowlist descriptor should not validate invite token");

    assert_eq!(
        error,
        PeerInviteValidationError::PeeringPolicyRefused {
            issuer_peering_policy: PeeringPolicy::StaticAllowlist,
            invite_peering_policy: PeeringPolicy::InviteToken,
        }
    );
}

#[test]
fn rejects_tampered_invite_signature() {
    let issuer = signed_descriptor(PeeringPolicy::InviteToken);
    let mut invite = signed_invite(&issuer, Some("server-local"));
    invite.invite_id = "peer-invite-tampered".to_string();

    let error = verify_peer_invite_ed25519(&invite, &issuer.descriptor)
        .expect_err("tampered invite should fail signature verification");

    assert_eq!(error, PeerInviteSignatureError::SignatureVerificationFailed);
}

#[test]
fn rejects_expired_and_revoked_invites() {
    let issuer = signed_descriptor(PeeringPolicy::InviteToken);
    let mut expired = signed_invite(&issuer, Some("server-local"));
    expired.issued_at_epoch_seconds = 1_699_999_900;
    expired.expires_at_epoch_seconds = 1_700_000_010;
    let error = expired
        .validate(&issuer.descriptor, &invite_context(Some("server-local")))
        .expect_err("expired invite should fail");
    assert_eq!(error, PeerInviteValidationError::InviteExpired);

    let mut revoked_context = invite_context(Some("server-local"));
    revoked_context
        .revoked_invite_ids
        .push("peer-invite-1".to_string());
    let revoked = signed_invite(&issuer, Some("server-local"));
    let error = revoked
        .validate(&issuer.descriptor, &revoked_context)
        .expect_err("revoked invite should fail");
    assert_eq!(error, PeerInviteValidationError::InviteRevoked);
}
