use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use ring::{rand::SystemRandom, signature::Ed25519KeyPair};

use crate::domain::{
    ed25519_public_key_base64, sign_dm_session_bootstrap_ed25519_pkcs8,
    verify_dm_session_bootstrap_ed25519, DmE2eeError, DmEphemeralPublicKey, DmEphemeralSecret,
    DmGroupSecret, DmSessionContext, DmSessionRotationState, DM_SESSION_KEY_BYTES,
    DM_SESSION_ROTATE_AFTER_MESSAGES, DM_SESSION_ROTATE_AFTER_SECONDS,
};

fn generated_identity_key() -> Vec<u8> {
    Ed25519KeyPair::generate_pkcs8(&SystemRandom::new())
        .expect("generate ed25519 key")
        .as_ref()
        .to_vec()
}

#[test]
fn one_to_one_bootstrap_binds_x25519_key_to_identity_signature() {
    let alice_identity_key = generated_identity_key();
    let alice_identity_public_key =
        ed25519_public_key_base64(&alice_identity_key).expect("derive alice identity public key");
    let bob_identity_key = generated_identity_key();
    let bob_identity_public_key =
        ed25519_public_key_base64(&bob_identity_key).expect("derive bob identity public key");
    let (alice_secret, alice_public_key) =
        DmEphemeralSecret::generate().expect("generate alice x25519 key");
    let (bob_secret, bob_public_key) =
        DmEphemeralSecret::generate().expect("generate bob x25519 key");
    let context = DmSessionContext::one_to_one("dm-thread-1", "usr-alice", "usr-bob", 0, 1_000)
        .expect("build one-to-one context");

    let alice_signature = sign_dm_session_bootstrap_ed25519_pkcs8(
        "usr-alice",
        &alice_identity_key,
        &alice_public_key,
        &context,
    )
    .expect("sign alice bootstrap");
    let bob_signature = sign_dm_session_bootstrap_ed25519_pkcs8(
        "usr-bob",
        &bob_identity_key,
        &bob_public_key,
        &context,
    )
    .expect("sign bob bootstrap");

    assert_eq!(
        verify_dm_session_bootstrap_ed25519(
            "usr-alice",
            &alice_identity_public_key,
            &alice_public_key,
            &context,
            &alice_signature,
        ),
        Ok(())
    );
    assert_eq!(
        verify_dm_session_bootstrap_ed25519(
            "usr-bob",
            &bob_identity_public_key,
            &bob_public_key,
            &context,
            &bob_signature,
        ),
        Ok(())
    );
    let alice_identity_hex = hex::encode(
        BASE64
            .decode(alice_identity_public_key.as_bytes())
            .expect("decode alice public key"),
    );
    assert_eq!(
        verify_dm_session_bootstrap_ed25519(
            "usr-alice",
            &alice_identity_hex,
            &alice_public_key,
            &context,
            &alice_signature,
        ),
        Ok(())
    );
    let alice_signature_hex = hex::encode(
        BASE64
            .decode(alice_signature.as_bytes())
            .expect("decode alice signature"),
    );
    assert_eq!(
        verify_dm_session_bootstrap_ed25519(
            "usr-alice",
            &alice_identity_public_key,
            &alice_public_key,
            &context,
            &alice_signature_hex,
        ),
        Ok(())
    );

    let tampered_public_key = DmEphemeralPublicKey::from_bytes([7_u8; DM_SESSION_KEY_BYTES]);
    assert_eq!(
        verify_dm_session_bootstrap_ed25519(
            "usr-alice",
            &alice_identity_public_key,
            &tampered_public_key,
            &context,
            &alice_signature,
        ),
        Err(DmE2eeError::SignatureInvalid)
    );

    let alice_session_key = alice_secret
        .derive_one_to_one_session_key(&bob_public_key, &context)
        .expect("derive alice session key");
    let bob_session_key = bob_secret
        .derive_one_to_one_session_key(&alice_public_key, &context)
        .expect("derive bob session key");
    let plaintext = b"client-only plaintext message";
    let envelope = alice_session_key
        .encrypt_message(&context, "msg-1", "usr-alice", plaintext)
        .expect("encrypt message");
    let ciphertext = BASE64
        .decode(envelope.ciphertext_base64.as_bytes())
        .expect("decode ciphertext");

    assert_ne!(ciphertext, plaintext);
    assert_eq!(
        bob_session_key
            .decrypt_message(&context, &envelope)
            .expect("decrypt message"),
        plaintext
    );
    assert!(format!("{alice_session_key:?}").contains("<redacted>"));
}

#[test]
fn rejects_nonparticipant_bootstrap_and_sender_metadata() {
    let outsider_identity_key = generated_identity_key();
    let outsider_identity_public_key = ed25519_public_key_base64(&outsider_identity_key)
        .expect("derive outsider identity public key");
    let (_outsider_secret, outsider_public_key) =
        DmEphemeralSecret::generate().expect("generate outsider key");
    let (alice_secret, _alice_public_key) =
        DmEphemeralSecret::generate().expect("generate alice key");
    let context = DmSessionContext::one_to_one("dm-thread-3", "usr-alice", "usr-bob", 0, 1_000)
        .expect("build context");

    assert_eq!(
        sign_dm_session_bootstrap_ed25519_pkcs8(
            "usr-mallory",
            &outsider_identity_key,
            &outsider_public_key,
            &context,
        ),
        Err(DmE2eeError::ContextInvalid)
    );
    assert_eq!(
        verify_dm_session_bootstrap_ed25519(
            "usr-mallory",
            &outsider_identity_public_key,
            &outsider_public_key,
            &context,
            BASE64.encode([9_u8; 64]),
        ),
        Err(DmE2eeError::ContextInvalid)
    );

    let alice_key = alice_secret
        .derive_one_to_one_session_key(&outsider_public_key, &context)
        .expect("derive alice key for metadata validation");
    assert_eq!(
        alice_key.encrypt_message(&context, "msg-3", "usr-mallory", b"forged sender"),
        Err(DmE2eeError::EnvelopeInvalid)
    );
}

#[test]
fn xchacha_envelopes_reject_tampered_ciphertext_and_metadata() {
    let (alice_secret, alice_public_key) =
        DmEphemeralSecret::generate().expect("generate alice key");
    let (bob_secret, bob_public_key) = DmEphemeralSecret::generate().expect("generate bob key");
    let context = DmSessionContext::one_to_one("dm-thread-2", "usr-alice", "usr-bob", 0, 1_000)
        .expect("build context");
    let alice_key = alice_secret
        .derive_one_to_one_session_key(&bob_public_key, &context)
        .expect("derive alice key");
    let bob_key = bob_secret
        .derive_one_to_one_session_key(&alice_public_key, &context)
        .expect("derive bob key");
    let envelope = alice_key
        .encrypt_message(&context, "msg-2", "usr-alice", b"authenticated payload")
        .expect("encrypt payload");

    let mut tampered_ciphertext = envelope.clone();
    tampered_ciphertext.ciphertext_base64 = BASE64.encode([3_u8; 48]);
    assert_eq!(
        bob_key.decrypt_message(&context, &tampered_ciphertext),
        Err(DmE2eeError::DecryptInvalid)
    );

    let mut tampered_metadata = envelope.clone();
    tampered_metadata.message_id = "msg-3".to_string();
    assert_eq!(
        bob_key.decrypt_message(&context, &tampered_metadata),
        Err(DmE2eeError::DecryptInvalid)
    );
}

#[test]
fn deserialized_context_must_preserve_derived_session_id() {
    let context = DmSessionContext::group(
        "group-thread-2",
        ["usr-alice", "usr-bob", "usr-cara"],
        0,
        1_000,
    )
    .expect("build context");
    let mut value = serde_json::to_value(&context).expect("serialize context");
    value["session_id"] = serde_json::Value::String("forged-session-id".to_string());

    let result = serde_json::from_value::<DmSessionContext>(value);

    assert!(result.is_err());
}

#[test]
fn group_session_rekey_excludes_removed_member() {
    let initial_context = DmSessionContext::group(
        "group-thread-1",
        ["usr-alice", "usr-bob", "usr-cara"],
        0,
        1_000,
    )
    .expect("build initial group context");
    let initial_secret = DmGroupSecret::from_bytes([11_u8; DM_SESSION_KEY_BYTES]);
    let initial_key = initial_secret
        .derive_session_key(&initial_context)
        .expect("derive initial group key");
    let initial_envelope = initial_key
        .encrypt_message(
            &initial_context,
            "msg-group-1",
            "usr-alice",
            b"group payload",
        )
        .expect("encrypt initial group payload");

    assert_eq!(
        initial_key
            .decrypt_message(&initial_context, &initial_envelope)
            .expect("decrypt initial group payload"),
        b"group payload"
    );

    let rekeyed_context =
        DmSessionContext::group("group-thread-1", ["usr-alice", "usr-cara"], 1, 1_100)
            .expect("build rekeyed group context");
    let rekeyed_secret = DmGroupSecret::from_bytes([22_u8; DM_SESSION_KEY_BYTES]);
    let rekeyed_key = rekeyed_secret
        .derive_session_key(&rekeyed_context)
        .expect("derive rekeyed group key");
    let rekeyed_envelope = rekeyed_key
        .encrypt_message(
            &rekeyed_context,
            "msg-group-2",
            "usr-cara",
            b"post-removal payload",
        )
        .expect("encrypt rekeyed group payload");

    assert_eq!(
        rekeyed_key
            .decrypt_message(&rekeyed_context, &rekeyed_envelope)
            .expect("decrypt rekeyed group payload"),
        b"post-removal payload"
    );
    assert_eq!(
        initial_key.decrypt_message(&rekeyed_context, &rekeyed_envelope),
        Err(DmE2eeError::DecryptInvalid)
    );
}

#[test]
fn rotation_triggers_at_message_count_or_session_age_boundary() {
    let fresh = DmSessionRotationState {
        created_at_epoch_seconds: 1_000,
        messages_encrypted: DM_SESSION_ROTATE_AFTER_MESSAGES - 1,
    };
    assert!(!fresh.requires_rotation(1_000 + DM_SESSION_ROTATE_AFTER_SECONDS - 1));

    let count_boundary = DmSessionRotationState {
        created_at_epoch_seconds: 1_000,
        messages_encrypted: DM_SESSION_ROTATE_AFTER_MESSAGES,
    };
    assert!(count_boundary.requires_rotation(1_100));

    let age_boundary = DmSessionRotationState {
        created_at_epoch_seconds: 1_000,
        messages_encrypted: 0,
    };
    assert!(age_boundary.requires_rotation(1_000 + DM_SESSION_ROTATE_AFTER_SECONDS));
}
