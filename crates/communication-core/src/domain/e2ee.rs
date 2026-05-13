use std::collections::BTreeMap;
use std::fmt;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    Key, XChaCha20Poly1305, XNonce,
};
use ring::{
    agreement, digest, hkdf,
    rand::{SecureRandom, SystemRandom},
    signature::{Ed25519KeyPair, KeyPair, UnparsedPublicKey, ED25519},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

const BOOTSTRAP_SIGNING_DOMAIN: &str = "hexrelay.dm_e2ee_bootstrap";
const SESSION_DERIVATION_DOMAIN: &[u8] = b"hexrelay.dm_e2ee_session";
const MESSAGE_AAD_DOMAIN: &str = "hexrelay.dm_e2ee_message";
pub const DM_SESSION_KEY_BYTES: usize = 32;
pub const DM_SESSION_NONCE_BYTES: usize = 24;
pub const DM_SESSION_ROTATE_AFTER_MESSAGES: u64 = 100;
pub const DM_SESSION_ROTATE_AFTER_SECONDS: i64 = 24 * 60 * 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DmSessionKind {
    OneToOne,
    Group,
}

impl DmSessionKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OneToOne => "one_to_one",
            Self::Group => "group",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DmE2eeError {
    ContextInvalid,
    IdentityKeyInvalid,
    PrivateKeyInvalid,
    PeerKeyInvalid,
    SignatureInvalid,
    KeyAgreementInvalid,
    KeyDerivationInvalid,
    NonceInvalid,
    EnvelopeInvalid,
    EncryptInvalid,
    DecryptInvalid,
}

impl DmE2eeError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::ContextInvalid => "context_invalid",
            Self::IdentityKeyInvalid => "identity_key_invalid",
            Self::PrivateKeyInvalid => "private_key_invalid",
            Self::PeerKeyInvalid => "peer_key_invalid",
            Self::SignatureInvalid => "signature_invalid",
            Self::KeyAgreementInvalid => "key_agreement_invalid",
            Self::KeyDerivationInvalid => "key_derivation_invalid",
            Self::NonceInvalid => "nonce_invalid",
            Self::EnvelopeInvalid => "envelope_invalid",
            Self::EncryptInvalid => "encrypt_invalid",
            Self::DecryptInvalid => "decrypt_invalid",
        }
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DmSessionContext {
    kind: DmSessionKind,
    thread_id: String,
    participant_identity_ids: Vec<String>,
    generation: u64,
    created_at_epoch_seconds: i64,
    session_id: String,
}

impl fmt::Debug for DmSessionContext {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DmSessionContext")
            .field("kind", &self.kind)
            .field("thread_id", &self.thread_id)
            .field("participant_identity_ids", &self.participant_identity_ids)
            .field("generation", &self.generation)
            .field("created_at_epoch_seconds", &self.created_at_epoch_seconds)
            .field("session_id", &self.session_id)
            .finish()
    }
}

impl DmSessionContext {
    pub fn new(
        kind: DmSessionKind,
        thread_id: impl Into<String>,
        participant_identity_ids: impl IntoIterator<Item = impl Into<String>>,
        generation: u64,
        created_at_epoch_seconds: i64,
    ) -> Result<Self, DmE2eeError> {
        let thread_id = normalize_required_id(thread_id.into())?;
        let mut participant_identity_ids = participant_identity_ids
            .into_iter()
            .map(|value| normalize_required_id(value.into()))
            .collect::<Result<Vec<_>, _>>()?;

        participant_identity_ids.sort();
        participant_identity_ids.dedup();

        match kind {
            DmSessionKind::OneToOne if participant_identity_ids.len() != 2 => {
                return Err(DmE2eeError::ContextInvalid);
            }
            DmSessionKind::Group if participant_identity_ids.len() < 2 => {
                return Err(DmE2eeError::ContextInvalid);
            }
            _ => {}
        }

        let session_id = derive_session_id(
            kind,
            &thread_id,
            &participant_identity_ids,
            generation,
            created_at_epoch_seconds,
        )?;

        Ok(Self {
            kind,
            thread_id,
            participant_identity_ids,
            generation,
            created_at_epoch_seconds,
            session_id,
        })
    }

    pub fn one_to_one(
        thread_id: impl Into<String>,
        first_identity_id: impl Into<String>,
        second_identity_id: impl Into<String>,
        generation: u64,
        created_at_epoch_seconds: i64,
    ) -> Result<Self, DmE2eeError> {
        Self::new(
            DmSessionKind::OneToOne,
            thread_id,
            [first_identity_id.into(), second_identity_id.into()],
            generation,
            created_at_epoch_seconds,
        )
    }

    pub fn group(
        thread_id: impl Into<String>,
        participant_identity_ids: impl IntoIterator<Item = impl Into<String>>,
        generation: u64,
        created_at_epoch_seconds: i64,
    ) -> Result<Self, DmE2eeError> {
        Self::new(
            DmSessionKind::Group,
            thread_id,
            participant_identity_ids,
            generation,
            created_at_epoch_seconds,
        )
    }

    pub fn kind(&self) -> DmSessionKind {
        self.kind
    }

    pub fn thread_id(&self) -> &str {
        &self.thread_id
    }

    pub fn participant_identity_ids(&self) -> &[String] {
        &self.participant_identity_ids
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn created_at_epoch_seconds(&self) -> i64 {
        self.created_at_epoch_seconds
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, DmE2eeError> {
        canonical_json_bytes(session_context_value(
            self.kind,
            &self.thread_id,
            &self.participant_identity_ids,
            self.generation,
            self.created_at_epoch_seconds,
        ))
    }

    fn message_aad(
        &self,
        message_id: &str,
        sender_identity_id: &str,
    ) -> Result<Vec<u8>, DmE2eeError> {
        let mut fields = BTreeMap::new();
        fields.insert("domain", json!(MESSAGE_AAD_DOMAIN));
        fields.insert("generation", json!(self.generation));
        fields.insert("kind", json!(self.kind.as_str()));
        fields.insert("message_id", json!(normalize_required_id(message_id)?));
        fields.insert(
            "participant_identity_ids",
            json!(self.participant_identity_ids),
        );
        fields.insert(
            "sender_identity_id",
            json!(normalize_required_id(sender_identity_id)?),
        );
        fields.insert("session_id", json!(self.session_id));
        fields.insert("thread_id", json!(self.thread_id));

        canonical_json_bytes(fields)
    }
}

pub struct DmEphemeralSecret {
    inner: agreement::EphemeralPrivateKey,
}

impl fmt::Debug for DmEphemeralSecret {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DmEphemeralSecret")
            .field("inner", &"<redacted>")
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DmEphemeralPublicKey {
    bytes_base64: String,
}

impl DmEphemeralPublicKey {
    pub fn from_bytes(bytes: [u8; DM_SESSION_KEY_BYTES]) -> Self {
        Self {
            bytes_base64: BASE64.encode(bytes),
        }
    }

    pub fn bytes(&self) -> Result<[u8; DM_SESSION_KEY_BYTES], DmE2eeError> {
        decode_fixed_base64(&self.bytes_base64).ok_or(DmE2eeError::PeerKeyInvalid)
    }

    pub fn as_base64(&self) -> &str {
        &self.bytes_base64
    }
}

impl DmEphemeralSecret {
    pub fn generate() -> Result<(Self, DmEphemeralPublicKey), DmE2eeError> {
        let rng = SystemRandom::new();
        let inner = agreement::EphemeralPrivateKey::generate(&agreement::X25519, &rng)
            .map_err(|_| DmE2eeError::KeyAgreementInvalid)?;
        let public_key = inner
            .compute_public_key()
            .map_err(|_| DmE2eeError::KeyAgreementInvalid)?;
        let public_key = fixed_array::<DM_SESSION_KEY_BYTES>(public_key.as_ref())
            .ok_or(DmE2eeError::KeyAgreementInvalid)?;

        Ok((Self { inner }, DmEphemeralPublicKey::from_bytes(public_key)))
    }

    pub fn derive_one_to_one_session_key(
        self,
        peer_public_key: &DmEphemeralPublicKey,
        context: &DmSessionContext,
    ) -> Result<DmSessionKey, DmE2eeError> {
        if context.kind != DmSessionKind::OneToOne {
            return Err(DmE2eeError::ContextInvalid);
        }

        let peer_public_key = peer_public_key.bytes()?;
        let peer_public_key =
            agreement::UnparsedPublicKey::new(&agreement::X25519, peer_public_key);

        agreement::agree_ephemeral(self.inner, &peer_public_key, |shared_secret| {
            derive_session_key(shared_secret, context)
        })
        .map_err(|_| DmE2eeError::KeyAgreementInvalid)?
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct DmGroupSecret {
    bytes: [u8; DM_SESSION_KEY_BYTES],
}

impl fmt::Debug for DmGroupSecret {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DmGroupSecret")
            .field("bytes", &"<redacted>")
            .finish()
    }
}

impl DmGroupSecret {
    pub fn generate() -> Result<Self, DmE2eeError> {
        let rng = SystemRandom::new();
        let mut bytes = [0_u8; DM_SESSION_KEY_BYTES];
        rng.fill(&mut bytes)
            .map_err(|_| DmE2eeError::KeyDerivationInvalid)?;

        Ok(Self { bytes })
    }

    pub fn from_bytes(bytes: [u8; DM_SESSION_KEY_BYTES]) -> Self {
        Self { bytes }
    }

    pub fn derive_session_key(
        &self,
        context: &DmSessionContext,
    ) -> Result<DmSessionKey, DmE2eeError> {
        if context.kind != DmSessionKind::Group {
            return Err(DmE2eeError::ContextInvalid);
        }

        derive_session_key(&self.bytes, context)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct DmSessionKey {
    bytes: [u8; DM_SESSION_KEY_BYTES],
}

impl fmt::Debug for DmSessionKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DmSessionKey")
            .field("bytes", &"<redacted>")
            .finish()
    }
}

impl DmSessionKey {
    pub fn encrypt_message(
        &self,
        context: &DmSessionContext,
        message_id: impl AsRef<str>,
        sender_identity_id: impl AsRef<str>,
        plaintext: &[u8],
    ) -> Result<DmCiphertextEnvelope, DmE2eeError> {
        let message_id = normalize_required_id(message_id.as_ref())?;
        let sender_identity_id = normalize_required_id(sender_identity_id.as_ref())?;
        let mut nonce = [0_u8; DM_SESSION_NONCE_BYTES];
        SystemRandom::new()
            .fill(&mut nonce)
            .map_err(|_| DmE2eeError::NonceInvalid)?;

        let aad = context.message_aad(&message_id, &sender_identity_id)?;
        let cipher = XChaCha20Poly1305::new(Key::from_slice(&self.bytes));
        let ciphertext = cipher
            .encrypt(
                XNonce::from_slice(&nonce),
                Payload {
                    msg: plaintext,
                    aad: &aad,
                },
            )
            .map_err(|_| DmE2eeError::EncryptInvalid)?;

        Ok(DmCiphertextEnvelope {
            session_id: context.session_id.clone(),
            generation: context.generation,
            message_id,
            sender_identity_id,
            nonce_base64: BASE64.encode(nonce),
            ciphertext_base64: BASE64.encode(ciphertext),
        })
    }

    pub fn decrypt_message(
        &self,
        context: &DmSessionContext,
        envelope: &DmCiphertextEnvelope,
    ) -> Result<Vec<u8>, DmE2eeError> {
        if envelope.session_id != context.session_id || envelope.generation != context.generation {
            return Err(DmE2eeError::EnvelopeInvalid);
        }

        let nonce: [u8; DM_SESSION_NONCE_BYTES] =
            decode_fixed_base64(&envelope.nonce_base64).ok_or(DmE2eeError::NonceInvalid)?;
        let ciphertext = BASE64
            .decode(envelope.ciphertext_base64.as_bytes())
            .map_err(|_| DmE2eeError::EnvelopeInvalid)?;
        let aad = context.message_aad(&envelope.message_id, &envelope.sender_identity_id)?;
        let cipher = XChaCha20Poly1305::new(Key::from_slice(&self.bytes));

        cipher
            .decrypt(
                XNonce::from_slice(&nonce),
                Payload {
                    msg: &ciphertext,
                    aad: &aad,
                },
            )
            .map_err(|_| DmE2eeError::DecryptInvalid)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DmCiphertextEnvelope {
    pub session_id: String,
    pub generation: u64,
    pub message_id: String,
    pub sender_identity_id: String,
    pub nonce_base64: String,
    pub ciphertext_base64: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DmSessionRotationState {
    pub created_at_epoch_seconds: i64,
    pub messages_encrypted: u64,
}

impl DmSessionRotationState {
    pub fn requires_rotation(&self, now_epoch_seconds: i64) -> bool {
        self.messages_encrypted >= DM_SESSION_ROTATE_AFTER_MESSAGES
            || now_epoch_seconds.saturating_sub(self.created_at_epoch_seconds)
                >= DM_SESSION_ROTATE_AFTER_SECONDS
    }
}

pub fn sign_dm_session_bootstrap_ed25519_pkcs8(
    identity_id: impl AsRef<str>,
    identity_private_key_pkcs8: &[u8],
    session_public_key: &DmEphemeralPublicKey,
    context: &DmSessionContext,
) -> Result<String, DmE2eeError> {
    let key_pair = Ed25519KeyPair::from_pkcs8(identity_private_key_pkcs8)
        .map_err(|_| DmE2eeError::PrivateKeyInvalid)?;
    let public_key = BASE64.encode(key_pair.public_key().as_ref());
    let payload = canonical_bootstrap_payload(
        identity_id.as_ref(),
        &public_key,
        session_public_key,
        context,
    )?;
    let signature = key_pair.sign(&payload);

    Ok(BASE64.encode(signature.as_ref()))
}

pub fn verify_dm_session_bootstrap_ed25519(
    identity_id: impl AsRef<str>,
    identity_public_key: impl AsRef<str>,
    session_public_key: &DmEphemeralPublicKey,
    context: &DmSessionContext,
    signature_base64: impl AsRef<str>,
) -> Result<(), DmE2eeError> {
    let identity_public_key = normalize_public_key(identity_public_key.as_ref())?;
    let signature: [u8; 64] =
        decode_fixed_base64(signature_base64.as_ref()).ok_or(DmE2eeError::SignatureInvalid)?;
    let payload = canonical_bootstrap_payload(
        identity_id.as_ref(),
        &identity_public_key,
        session_public_key,
        context,
    )?;

    let key = UnparsedPublicKey::new(&ED25519, decode_public_key(&identity_public_key)?);
    key.verify(&payload, &signature)
        .map_err(|_| DmE2eeError::SignatureInvalid)
}

pub fn ed25519_public_key_base64(private_key_pkcs8: &[u8]) -> Result<String, DmE2eeError> {
    let key_pair = Ed25519KeyPair::from_pkcs8(private_key_pkcs8)
        .map_err(|_| DmE2eeError::PrivateKeyInvalid)?;

    Ok(BASE64.encode(key_pair.public_key().as_ref()))
}

fn derive_session_key(
    secret: &[u8],
    context: &DmSessionContext,
) -> Result<DmSessionKey, DmE2eeError> {
    let salt = hkdf::Salt::new(hkdf::HKDF_SHA256, SESSION_DERIVATION_DOMAIN);
    let prk = salt.extract(secret);
    let context_bytes = context.canonical_bytes()?;
    let info = [SESSION_DERIVATION_DOMAIN, context_bytes.as_slice()];
    let mut bytes = [0_u8; DM_SESSION_KEY_BYTES];

    prk.expand(&info, HkdfOutputLen(DM_SESSION_KEY_BYTES))
        .map_err(|_| DmE2eeError::KeyDerivationInvalid)?
        .fill(&mut bytes)
        .map_err(|_| DmE2eeError::KeyDerivationInvalid)?;

    Ok(DmSessionKey { bytes })
}

struct HkdfOutputLen(usize);

impl hkdf::KeyType for HkdfOutputLen {
    fn len(&self) -> usize {
        self.0
    }
}

fn derive_session_id(
    kind: DmSessionKind,
    thread_id: &str,
    participant_identity_ids: &[String],
    generation: u64,
    created_at_epoch_seconds: i64,
) -> Result<String, DmE2eeError> {
    let bytes = canonical_json_bytes(session_context_value(
        kind,
        thread_id,
        participant_identity_ids,
        generation,
        created_at_epoch_seconds,
    ))?;
    let digest = digest::digest(&digest::SHA256, &bytes);

    Ok(hex::encode(digest.as_ref()))
}

fn session_context_value(
    kind: DmSessionKind,
    thread_id: &str,
    participant_identity_ids: &[String],
    generation: u64,
    created_at_epoch_seconds: i64,
) -> BTreeMap<&'static str, Value> {
    let mut fields = BTreeMap::new();
    fields.insert("created_at_epoch_seconds", json!(created_at_epoch_seconds));
    fields.insert("domain", json!("hexrelay.dm_e2ee_session_context"));
    fields.insert("generation", json!(generation));
    fields.insert("kind", json!(kind.as_str()));
    fields.insert("participant_identity_ids", json!(participant_identity_ids));
    fields.insert("thread_id", json!(thread_id));
    fields
}

fn canonical_bootstrap_payload(
    identity_id: &str,
    identity_public_key: &str,
    session_public_key: &DmEphemeralPublicKey,
    context: &DmSessionContext,
) -> Result<Vec<u8>, DmE2eeError> {
    let mut fields = BTreeMap::new();
    fields.insert("domain", json!(BOOTSTRAP_SIGNING_DOMAIN));
    fields.insert("identity_id", json!(normalize_required_id(identity_id)?));
    fields.insert(
        "identity_public_key",
        json!(normalize_public_key(identity_public_key)?),
    );
    fields.insert(
        "session_context",
        json!(session_context_signing_value(context)),
    );
    fields.insert("session_public_key", json!(session_public_key.as_base64()));

    canonical_json_bytes(fields)
}

fn session_context_signing_value(context: &DmSessionContext) -> BTreeMap<&'static str, Value> {
    let mut fields = session_context_value(
        context.kind,
        &context.thread_id,
        &context.participant_identity_ids,
        context.generation,
        context.created_at_epoch_seconds,
    );
    fields.insert("session_id", json!(context.session_id));
    fields
}

fn canonical_json_bytes(fields: BTreeMap<&'static str, Value>) -> Result<Vec<u8>, DmE2eeError> {
    serde_json::to_vec(&fields).map_err(|_| DmE2eeError::ContextInvalid)
}

fn normalize_required_id(value: impl AsRef<str>) -> Result<String, DmE2eeError> {
    let trimmed = value.as_ref().trim();
    if trimmed.is_empty() {
        return Err(DmE2eeError::ContextInvalid);
    }

    Ok(trimmed.to_string())
}

fn normalize_public_key(value: &str) -> Result<String, DmE2eeError> {
    let trimmed = value.trim();
    let decoded = decode_public_key(trimmed).map_err(|_| DmE2eeError::IdentityKeyInvalid)?;

    Ok(BASE64.encode(decoded))
}

fn decode_public_key(value: &str) -> Result<[u8; 32], DmE2eeError> {
    if value.len() == 64 && value.chars().all(|character| character.is_ascii_hexdigit()) {
        return fixed_array(&hex::decode(value).map_err(|_| DmE2eeError::IdentityKeyInvalid)?)
            .ok_or(DmE2eeError::IdentityKeyInvalid);
    }

    decode_fixed_base64(value).ok_or(DmE2eeError::IdentityKeyInvalid)
}

fn decode_fixed_base64<const N: usize>(value: &str) -> Option<[u8; N]> {
    let decoded = BASE64.decode(value.trim().as_bytes()).ok()?;
    fixed_array(&decoded)
}

fn fixed_array<const N: usize>(bytes: &[u8]) -> Option<[u8; N]> {
    if bytes.len() != N {
        return None;
    }

    let mut output = [0_u8; N];
    output.copy_from_slice(bytes);
    Some(output)
}
