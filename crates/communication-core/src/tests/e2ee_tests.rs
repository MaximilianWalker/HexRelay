use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use ring::{rand::SystemRandom, signature::Ed25519KeyPair};

use crate::domain::{
    ed25519_public_key_base64, sign_dm_session_bootstrap_ed25519_pkcs8,
    verify_dm_session_bootstrap_ed25519, DmClientSession, DmE2eeError, DmEphemeralPublicKey,
    DmEphemeralSecret, DmGroupRekeyPlan, DmGroupSecret, DmSessionBootstrap, DmSessionContext,
    DmSessionRotationState, DM_SESSION_KEY_BYTES, DM_SESSION_ROTATE_AFTER_MESSAGES,
    DM_SESSION_ROTATE_AFTER_SECONDS,
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
fn verified_one_to_one_bootstrap_builds_matching_client_sessions() {
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
    let context =
        DmSessionContext::one_to_one("dm-thread-verified", "usr-alice", "usr-bob", 0, 1_000)
            .expect("build context");

    let alice_bootstrap = DmSessionBootstrap::sign_ed25519_pkcs8(
        "usr-alice",
        &alice_identity_key,
        alice_public_key,
        &context,
    )
    .expect("sign alice bootstrap");
    let bob_bootstrap = DmSessionBootstrap::sign_ed25519_pkcs8(
        "usr-bob",
        &bob_identity_key,
        bob_public_key,
        &context,
    )
    .expect("sign bob bootstrap");

    assert_eq!(
        alice_bootstrap.verify(&context, &alice_identity_public_key),
        Ok(())
    );
    assert_eq!(
        bob_bootstrap.verify(&context, &bob_identity_public_key),
        Ok(())
    );

    let mut alice_session = DmClientSession::one_to_one_from_verified_peer_bootstrap(
        context.clone(),
        "usr-alice",
        alice_secret,
        &bob_bootstrap,
        &bob_identity_public_key,
    )
    .expect("derive alice session");
    let bob_session = DmClientSession::one_to_one_from_verified_peer_bootstrap(
        context,
        "usr-bob",
        bob_secret,
        &alice_bootstrap,
        &alice_identity_public_key,
    )
    .expect("derive bob session");
    let plaintext = b"verified bootstrap plaintext";
    let encrypted = alice_session
        .encrypt_outbound(1_000, "msg-verified-bootstrap", "usr-alice", plaintext)
        .expect("encrypt verified payload");
    let serialized_bootstrap =
        serde_json::to_string(&alice_bootstrap).expect("serialize bootstrap");

    assert_eq!(
        bob_session
            .decrypt_inbound(&encrypted.envelope)
            .expect("decrypt verified payload"),
        plaintext
    );
    assert!(!serialized_bootstrap.contains("verified bootstrap plaintext"));
}

#[test]
fn verified_one_to_one_bootstrap_rejects_untrusted_peer_material() {
    let alice_identity_key = generated_identity_key();
    let alice_identity_public_key =
        ed25519_public_key_base64(&alice_identity_key).expect("derive alice identity public key");
    let bob_identity_key = generated_identity_key();
    let bob_identity_public_key =
        ed25519_public_key_base64(&bob_identity_key).expect("derive bob identity public key");
    let mallory_identity_key = generated_identity_key();
    let (alice_secret, alice_public_key) =
        DmEphemeralSecret::generate().expect("generate alice x25519 key");
    let (_bob_secret, bob_public_key) =
        DmEphemeralSecret::generate().expect("generate bob x25519 key");
    let (mallory_secret, _mallory_public_key) =
        DmEphemeralSecret::generate().expect("generate mallory x25519 key");
    let (forged_secret, forged_public_key) =
        DmEphemeralSecret::generate().expect("generate forged x25519 key");
    let context =
        DmSessionContext::one_to_one("dm-thread-reject", "usr-alice", "usr-bob", 0, 1_000)
            .expect("build context");
    let group_context = DmSessionContext::group(
        "group-thread-reject",
        ["usr-alice", "usr-bob", "usr-cara"],
        0,
        1_000,
    )
    .expect("build group context");

    let alice_bootstrap = DmSessionBootstrap::sign_ed25519_pkcs8(
        "usr-alice",
        &alice_identity_key,
        alice_public_key,
        &context,
    )
    .expect("sign alice bootstrap");
    let bob_bootstrap = DmSessionBootstrap::sign_ed25519_pkcs8(
        "usr-bob",
        &bob_identity_key,
        bob_public_key,
        &context,
    )
    .expect("sign bob bootstrap");

    assert_eq!(
        DmClientSession::one_to_one_from_verified_peer_bootstrap(
            context.clone(),
            "usr-alice",
            alice_secret,
            &alice_bootstrap,
            &alice_identity_public_key,
        ),
        Err(DmE2eeError::ContextInvalid)
    );

    let forged_bob_bootstrap = DmSessionBootstrap::sign_ed25519_pkcs8(
        "usr-bob",
        &mallory_identity_key,
        forged_public_key,
        &context,
    )
    .expect("sign forged bob bootstrap");
    assert_eq!(
        DmClientSession::one_to_one_from_verified_peer_bootstrap(
            context.clone(),
            "usr-alice",
            forged_secret,
            &forged_bob_bootstrap,
            &bob_identity_public_key,
        ),
        Err(DmE2eeError::IdentityKeyInvalid)
    );

    let mut tampered_bootstrap = bob_bootstrap.clone();
    tampered_bootstrap.signature_base64 = BASE64.encode([3_u8; 64]);
    assert_eq!(
        DmClientSession::one_to_one_from_verified_peer_bootstrap(
            context,
            "usr-alice",
            mallory_secret,
            &tampered_bootstrap,
            &bob_identity_public_key,
        ),
        Err(DmE2eeError::SignatureInvalid)
    );
    assert_eq!(
        DmClientSession::one_to_one_from_verified_peer_bootstrap(
            group_context,
            "usr-alice",
            DmEphemeralSecret::generate()
                .expect("generate extra x25519 key")
                .0,
            &bob_bootstrap,
            &bob_identity_public_key,
        ),
        Err(DmE2eeError::ContextInvalid)
    );
}

#[test]
fn verified_one_to_one_bootstrap_rejects_wrong_session_context() {
    let alice_identity_key = generated_identity_key();
    let alice_identity_public_key =
        ed25519_public_key_base64(&alice_identity_key).expect("derive alice identity public key");
    let (_alice_secret, alice_public_key) =
        DmEphemeralSecret::generate().expect("generate alice x25519 key");
    let (bob_secret, _bob_public_key) =
        DmEphemeralSecret::generate().expect("generate bob x25519 key");
    let context = DmSessionContext::one_to_one("dm-thread-a", "usr-alice", "usr-bob", 0, 1_000)
        .expect("build original context");
    let different_thread_context =
        DmSessionContext::one_to_one("dm-thread-b", "usr-alice", "usr-bob", 0, 1_000)
            .expect("build different thread context");
    let next_generation_context =
        DmSessionContext::one_to_one("dm-thread-a", "usr-alice", "usr-bob", 1, 1_100)
            .expect("build next generation context");
    let alice_bootstrap = DmSessionBootstrap::sign_ed25519_pkcs8(
        "usr-alice",
        &alice_identity_key,
        alice_public_key,
        &context,
    )
    .expect("sign alice bootstrap");

    assert_eq!(
        alice_bootstrap.verify(&context, &alice_identity_public_key),
        Ok(())
    );
    assert_eq!(
        alice_bootstrap.verify(&different_thread_context, &alice_identity_public_key),
        Err(DmE2eeError::SignatureInvalid)
    );
    assert_eq!(
        alice_bootstrap.verify(&next_generation_context, &alice_identity_public_key),
        Err(DmE2eeError::SignatureInvalid)
    );
    assert_eq!(
        DmClientSession::one_to_one_from_verified_peer_bootstrap(
            different_thread_context,
            "usr-bob",
            bob_secret,
            &alice_bootstrap,
            &alice_identity_public_key,
        ),
        Err(DmE2eeError::SignatureInvalid)
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
fn client_session_encrypts_locally_and_reports_rotation_boundary() {
    let (alice_secret, alice_public_key) =
        DmEphemeralSecret::generate().expect("generate alice key");
    let (bob_secret, bob_public_key) = DmEphemeralSecret::generate().expect("generate bob key");
    let context = DmSessionContext::one_to_one("dm-thread-4", "usr-alice", "usr-bob", 0, 1_000)
        .expect("build context");
    let alice_key = alice_secret
        .derive_one_to_one_session_key(&bob_public_key, &context)
        .expect("derive alice key");
    let bob_key = bob_secret
        .derive_one_to_one_session_key(&alice_public_key, &context)
        .expect("derive bob key");
    let mut alice_session = DmClientSession::one_to_one(
        context.clone(),
        alice_key,
        DM_SESSION_ROTATE_AFTER_MESSAGES - 1,
    )
    .expect("build alice client session");
    let bob_session =
        DmClientSession::one_to_one(context, bob_key, 0).expect("build bob client session");
    let plaintext = b"client-only plaintext";

    let encrypted = alice_session
        .encrypt_outbound(1_000, "msg-client-1", "usr-alice", plaintext)
        .expect("encrypt client payload");

    assert_eq!(
        encrypted.messages_encrypted,
        DM_SESSION_ROTATE_AFTER_MESSAGES
    );
    assert!(encrypted.rotation_required);
    assert!(alice_session.requires_rotation(1_000));
    assert_eq!(
        bob_session
            .decrypt_inbound(&encrypted.envelope)
            .expect("decrypt client payload"),
        plaintext
    );

    let serialized_encrypt_result =
        serde_json::to_string(&encrypted).expect("serialize encrypt result");
    assert!(!serialized_encrypt_result.contains("client-only plaintext"));
    assert!(format!("{alice_session:?}").contains("<redacted>"));
    assert_eq!(
        alice_session.encrypt_outbound(1_001, "msg-client-2", "usr-alice", b"stale key"),
        Err(DmE2eeError::SessionRotationRequired)
    );
}

#[test]
fn one_to_one_rotation_plan_rekeys_client_sessions() {
    let alice_identity_key = generated_identity_key();
    let alice_identity_public_key =
        ed25519_public_key_base64(&alice_identity_key).expect("derive alice identity public key");
    let bob_identity_key = generated_identity_key();
    let bob_identity_public_key =
        ed25519_public_key_base64(&bob_identity_key).expect("derive bob identity public key");
    let (alice_initial_secret, alice_initial_public_key) =
        DmEphemeralSecret::generate().expect("generate initial alice x25519 key");
    let (bob_initial_secret, bob_initial_public_key) =
        DmEphemeralSecret::generate().expect("generate initial bob x25519 key");
    let initial_context =
        DmSessionContext::one_to_one("dm-thread-rotate", "usr-alice", "usr-bob", 0, 1_000)
            .expect("build initial context");
    let alice_initial_key = alice_initial_secret
        .derive_one_to_one_session_key(&bob_initial_public_key, &initial_context)
        .expect("derive initial alice key");
    let bob_initial_key = bob_initial_secret
        .derive_one_to_one_session_key(&alice_initial_public_key, &initial_context)
        .expect("derive initial bob key");
    let alice_initial_session = DmClientSession::one_to_one(
        initial_context.clone(),
        alice_initial_key,
        DM_SESSION_ROTATE_AFTER_MESSAGES,
    )
    .expect("build initial alice session");
    let bob_initial_session =
        DmClientSession::one_to_one(initial_context.clone(), bob_initial_key, 0)
            .expect("build initial bob session");

    assert!(alice_initial_session.requires_rotation(1_100));

    let rotation_plan = alice_initial_session
        .one_to_one_rotation_plan(1_100)
        .expect("build one-to-one rotation plan");
    assert_eq!(
        rotation_plan.previous_session_id(),
        initial_context.session_id()
    );
    assert_ne!(
        rotation_plan.next_context().session_id(),
        initial_context.session_id()
    );
    assert_eq!(rotation_plan.next_context().generation(), 1);
    assert_eq!(
        rotation_plan.next_context().participant_identity_ids(),
        ["usr-alice".to_string(), "usr-bob".to_string()]
    );

    let (alice_next_secret, alice_next_public_key) =
        DmEphemeralSecret::generate().expect("generate next alice x25519 key");
    let (bob_next_secret, bob_next_public_key) =
        DmEphemeralSecret::generate().expect("generate next bob x25519 key");
    let alice_next_bootstrap = DmSessionBootstrap::sign_ed25519_pkcs8(
        "usr-alice",
        &alice_identity_key,
        alice_next_public_key,
        rotation_plan.next_context(),
    )
    .expect("sign rotated alice bootstrap");
    let bob_next_bootstrap = DmSessionBootstrap::sign_ed25519_pkcs8(
        "usr-bob",
        &bob_identity_key,
        bob_next_public_key,
        rotation_plan.next_context(),
    )
    .expect("sign rotated bob bootstrap");

    let mut alice_next_session = rotation_plan
        .derive_next_session_from_verified_peer_bootstrap(
            "usr-alice",
            alice_next_secret,
            &bob_next_bootstrap,
            &bob_identity_public_key,
        )
        .expect("derive rotated alice session");
    let bob_next_session = rotation_plan
        .derive_next_session_from_verified_peer_bootstrap(
            "usr-bob",
            bob_next_secret,
            &alice_next_bootstrap,
            &alice_identity_public_key,
        )
        .expect("derive rotated bob session");
    let plaintext = b"rotated client-only plaintext";
    let encrypted = alice_next_session
        .encrypt_outbound(1_100, "msg-rotated-1", "usr-alice", plaintext)
        .expect("encrypt rotated payload");

    assert_eq!(
        bob_next_session
            .decrypt_inbound(&encrypted.envelope)
            .expect("decrypt rotated payload"),
        plaintext
    );
    assert_eq!(
        bob_initial_session.decrypt_inbound(&encrypted.envelope),
        Err(DmE2eeError::EnvelopeInvalid)
    );
    assert_eq!(encrypted.messages_encrypted, 1);
    assert!(!encrypted.rotation_required);

    let serialized_encrypt_result =
        serde_json::to_string(&encrypted).expect("serialize rotated encrypt result");
    assert!(!serialized_encrypt_result.contains("rotated client-only plaintext"));
}

#[test]
fn one_to_one_rotation_plan_rejects_group_context() {
    let group_context = DmSessionContext::group(
        "group-thread-no-one-to-one-rotation",
        ["usr-alice", "usr-bob", "usr-cara"],
        0,
        1_000,
    )
    .expect("build group context");
    let group_key = DmGroupSecret::from_bytes([33_u8; DM_SESSION_KEY_BYTES])
        .derive_session_key(&group_context)
        .expect("derive group key");
    let group_session =
        DmClientSession::group(group_context, group_key, 0).expect("build group session");

    assert_eq!(
        group_session.one_to_one_rotation_plan(1_100),
        Err(DmE2eeError::ContextInvalid)
    );
}

#[test]
fn group_rekey_plan_creates_new_client_session_for_membership_change() {
    let initial_context = DmSessionContext::group(
        "group-thread-3",
        ["usr-alice", "usr-bob", "usr-cara"],
        0,
        1_000,
    )
    .expect("build initial context");
    let initial_key = DmGroupSecret::from_bytes([11_u8; DM_SESSION_KEY_BYTES])
        .derive_session_key(&initial_context)
        .expect("derive initial key");
    let initial_session =
        DmClientSession::group(initial_context.clone(), initial_key, 0).expect("build session");

    let rekey_plan = DmGroupRekeyPlan::new(&initial_context, ["usr-dina", "usr-alice"], 1_100)
        .expect("build rekey plan");

    assert_eq!(
        rekey_plan.previous_session_id(),
        initial_context.session_id()
    );
    assert_eq!(rekey_plan.next_context().generation(), 1);
    assert_eq!(
        rekey_plan.next_context().participant_identity_ids(),
        ["usr-alice".to_string(), "usr-dina".to_string()]
    );
    assert_eq!(rekey_plan.added_identity_ids(), ["usr-dina".to_string()]);
    assert_eq!(
        rekey_plan.removed_identity_ids(),
        ["usr-bob".to_string(), "usr-cara".to_string()]
    );

    let rekeyed_secret = DmGroupSecret::from_bytes([22_u8; DM_SESSION_KEY_BYTES]);
    let mut rekeyed_session = rekey_plan
        .derive_next_session(&rekeyed_secret)
        .expect("derive rekeyed session");
    let encrypted = rekeyed_session
        .encrypt_outbound(
            1_100,
            "msg-group-3",
            "usr-dina",
            b"post-rekey group payload",
        )
        .expect("encrypt rekeyed payload");

    assert_eq!(
        rekeyed_session
            .decrypt_inbound(&encrypted.envelope)
            .expect("decrypt rekeyed payload"),
        b"post-rekey group payload"
    );
    assert_eq!(
        initial_session.decrypt_inbound(&encrypted.envelope),
        Err(DmE2eeError::EnvelopeInvalid)
    );
    assert_eq!(
        DmGroupRekeyPlan::new(
            &initial_context,
            ["usr-cara", "usr-bob", "usr-alice"],
            1_100,
        ),
        Err(DmE2eeError::ContextInvalid)
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
