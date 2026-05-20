use std::{
    collections::{HashMap, HashSet},
    env, fmt, fs,
    path::PathBuf,
};

use chrono::{Duration, Utc};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, Transaction};

use crate::infra::crypto::session_token::issue_session_token;

const DEFAULT_PROFILE: &str = "dm-basic";
const SESSION_COOKIE_NAME: &str = "hexrelay_session";
const CSRF_COOKIE_NAME: &str = "hexrelay_csrf";
const DEV_CSRF_TOKEN: &str = "dev-seed-csrf";

#[derive(Debug)]
pub enum DevSeedError {
    InvalidArgs(String),
    Config(String),
    Io(std::io::Error),
    Json(serde_json::Error),
    Db(sqlx::Error),
    Safety(String),
}

impl fmt::Display for DevSeedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidArgs(message) => write!(f, "{message}"),
            Self::Config(message) => write!(f, "{message}"),
            Self::Io(error) => write!(f, "failed to read fixture file: {error}"),
            Self::Json(error) => write!(f, "failed to parse fixture file: {error}"),
            Self::Db(error) => write!(f, "database seed failed: {error}"),
            Self::Safety(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for DevSeedError {}

impl From<std::io::Error> for DevSeedError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for DevSeedError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl From<sqlx::Error> for DevSeedError {
    fn from(value: sqlx::Error) -> Self {
        Self::Db(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeedCliOptions {
    pub profile: String,
    pub fixtures_root: Option<PathBuf>,
    pub json: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResetCliOptions {
    pub profile: Option<String>,
    pub fixtures_root: Option<PathBuf>,
    pub json: bool,
    pub yes: bool,
}

impl SeedCliOptions {
    pub fn parse<I>(args: I) -> Result<Self, DevSeedError>
    where
        I: IntoIterator<Item = String>,
    {
        let mut profile = DEFAULT_PROFILE.to_string();
        let mut fixtures_root = None;
        let mut json = false;
        let mut args = args.into_iter();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--profile" | "-p" => {
                    profile = args.next().ok_or_else(|| {
                        DevSeedError::InvalidArgs("--profile requires a value".to_string())
                    })?;
                }
                "--fixtures-root" => {
                    let value = args.next().ok_or_else(|| {
                        DevSeedError::InvalidArgs("--fixtures-root requires a value".to_string())
                    })?;
                    fixtures_root = Some(PathBuf::from(value));
                }
                "--json" => json = true,
                "--help" | "-h" => {
                    return Err(DevSeedError::InvalidArgs(seed_usage().to_string()));
                }
                value if value.starts_with('-') => {
                    return Err(DevSeedError::InvalidArgs(format!(
                        "unknown seed option: {value}\n{}",
                        seed_usage()
                    )));
                }
                value => {
                    return Err(DevSeedError::InvalidArgs(format!(
                        "unexpected positional argument: {value}\n{}",
                        seed_usage()
                    )));
                }
            }
        }

        if profile.trim().is_empty() {
            return Err(DevSeedError::InvalidArgs(
                "--profile must not be empty".to_string(),
            ));
        }

        Ok(Self {
            profile,
            fixtures_root,
            json,
        })
    }
}

pub fn seed_usage() -> &'static str {
    "Usage: seed_dev [--profile dm-basic] [--fixtures-root scripts/fixtures] [--json]"
}

impl ResetCliOptions {
    pub fn parse<I>(args: I) -> Result<Self, DevSeedError>
    where
        I: IntoIterator<Item = String>,
    {
        let mut profile = None;
        let mut fixtures_root = None;
        let mut json = false;
        let mut yes = false;
        let mut args = args.into_iter();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--profile" | "-p" => {
                    let value = args.next().ok_or_else(|| {
                        DevSeedError::InvalidArgs("--profile requires a value".to_string())
                    })?;
                    if value.trim().is_empty() {
                        return Err(DevSeedError::InvalidArgs(
                            "--profile must not be empty".to_string(),
                        ));
                    }
                    profile = Some(value);
                }
                "--fixtures-root" => {
                    let value = args.next().ok_or_else(|| {
                        DevSeedError::InvalidArgs("--fixtures-root requires a value".to_string())
                    })?;
                    fixtures_root = Some(PathBuf::from(value));
                }
                "--json" => json = true,
                "--yes" | "-y" => yes = true,
                "--help" | "-h" => {
                    return Err(DevSeedError::InvalidArgs(reset_usage().to_string()));
                }
                value if value.starts_with('-') => {
                    return Err(DevSeedError::InvalidArgs(format!(
                        "unknown reset option: {value}\n{}",
                        reset_usage()
                    )));
                }
                value => {
                    return Err(DevSeedError::InvalidArgs(format!(
                        "unexpected positional argument: {value}\n{}",
                        reset_usage()
                    )));
                }
            }
        }

        Ok(Self {
            profile,
            fixtures_root,
            json,
            yes,
        })
    }

    pub fn seed_options(&self) -> Option<SeedCliOptions> {
        self.profile.as_ref().map(|profile| SeedCliOptions {
            profile: profile.clone(),
            fixtures_root: self.fixtures_root.clone(),
            json: self.json,
        })
    }
}

pub fn reset_usage() -> &'static str {
    "Usage: reset_dev_db --yes [--profile dm-basic] [--fixtures-root scripts/fixtures] [--json]\nOmit --profile to reset schema without seeding fixture data."
}

#[derive(Clone, Debug, Deserialize)]
struct SeedScenario {
    scenario_id: String,
    description: String,
    identities: Vec<IdentityFixture>,
    sessions: Vec<SessionFixture>,
    friend_requests: Vec<FriendRequestFixture>,
    dm_policies: Vec<DmPolicyFixture>,
    devices: Vec<DeviceFixture>,
    #[serde(default)]
    invites: Vec<InviteFixture>,
    #[serde(default)]
    servers: Vec<ServerFixture>,
    #[serde(default)]
    server_memberships: Vec<ServerMembershipFixture>,
    #[serde(default)]
    server_channels: Vec<ServerChannelFixture>,
    #[serde(default)]
    server_channel_messages: Vec<ServerChannelMessageFixture>,
    dm_threads: Vec<DmThreadFixture>,
}

#[derive(Clone, Debug, Deserialize)]
struct IdentityFixture {
    profile_id: String,
    identity_id: String,
    public_key: String,
    algorithm: String,
}

#[derive(Clone, Debug, Deserialize)]
struct SessionFixture {
    profile_id: String,
    identity_id: String,
    session_id: String,
    expires_in_days: i64,
}

#[derive(Clone, Debug, Deserialize)]
struct FriendRequestFixture {
    request_id: String,
    requester_identity_id: String,
    target_identity_id: String,
    status: String,
}

#[derive(Clone, Debug, Deserialize)]
struct DmPolicyFixture {
    identity_id: String,
    inbound_policy: String,
    offline_delivery_mode: String,
}

#[derive(Clone, Debug, Deserialize)]
struct DeviceFixture {
    identity_id: String,
    device_id: String,
    active: bool,
    last_seen_offset_seconds: i64,
}

#[derive(Clone, Debug, Deserialize)]
struct DmThreadFixture {
    thread_id: String,
    kind: String,
    title: String,
    participants: Vec<DmParticipantFixture>,
    messages: Vec<DmMessageFixture>,
}

#[derive(Clone, Debug, Deserialize)]
struct DmParticipantFixture {
    identity_id: String,
    last_read_seq: u64,
}

#[derive(Clone, Debug, Deserialize)]
struct DmMessageFixture {
    message_id: String,
    author_id: String,
    seq: u64,
    ciphertext: String,
    created_at: String,
    edited_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct InviteFixture {
    invite_id: String,
    token_hash: String,
    mode: String,
    server_id: String,
    expires_at: Option<String>,
    max_uses: Option<i32>,
    uses: i32,
    created_at: String,
}

#[derive(Clone, Debug, Deserialize)]
struct ServerFixture {
    name: String,
    created_at: String,
}

#[derive(Clone, Debug, Deserialize)]
struct ServerMembershipFixture {
    identity_id: String,
    pinned: bool,
    muted: bool,
    unread_count: i32,
    joined_at: String,
}

#[derive(Clone, Debug, Deserialize)]
struct ServerChannelFixture {
    channel_id: String,
    name: String,
    kind: String,
    last_message_seq: u64,
    created_at: String,
}

#[derive(Clone, Debug, Deserialize)]
struct ServerChannelMessageFixture {
    message_id: String,
    channel_id: String,
    author_id: String,
    channel_seq: u64,
    content: String,
    reply_to_message_id: Option<String>,
    #[serde(default)]
    mention_identity_ids: Vec<String>,
    created_at: String,
    edited_at: Option<String>,
    deleted_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SeedSummary {
    pub profile: String,
    pub description: String,
    pub counts: SeedCounts,
    pub sessions: Vec<SeededSession>,
}

#[derive(Debug, Serialize)]
pub struct SeedCounts {
    pub identities: usize,
    pub sessions: usize,
    pub friend_requests: usize,
    pub dm_policies: usize,
    pub devices: usize,
    pub invites: usize,
    pub servers: usize,
    pub server_memberships: usize,
    pub server_channels: usize,
    pub server_channel_messages: usize,
    pub dm_threads: usize,
    pub dm_messages: usize,
}

#[derive(Debug, Serialize)]
pub struct SeededSession {
    pub profile_id: String,
    pub identity_id: String,
    pub session_id: String,
    pub expires_at: String,
    pub authorization_header: String,
    pub cookie_header: String,
    pub csrf_header: String,
}

pub fn assert_safe_seed_target(database_url: &str) -> Result<(), DevSeedError> {
    let environment = env::var("API_ENVIRONMENT")
        .unwrap_or_else(|_| "development".to_string())
        .trim()
        .to_ascii_lowercase();

    validate_seed_target(database_url, &environment)
}

pub fn assert_safe_reset_target(database_url: &str) -> Result<(), DevSeedError> {
    let environment = env::var("API_ENVIRONMENT")
        .unwrap_or_else(|_| "development".to_string())
        .trim()
        .to_ascii_lowercase();

    validate_reset_target(database_url, &environment)
}

pub fn validate_seed_target(database_url: &str, environment: &str) -> Result<(), DevSeedError> {
    if environment == "production" {
        return Err(DevSeedError::Safety(
            "refusing to seed when API_ENVIRONMENT=production".to_string(),
        ));
    }

    if environment != "development" {
        return Err(DevSeedError::Safety(format!(
            "refusing to seed with unsupported API_ENVIRONMENT={environment}; expected development"
        )));
    }

    let url = Url::parse(database_url).map_err(|_| {
        DevSeedError::Safety("refusing to seed because API_DATABASE_URL is invalid".to_string())
    })?;

    let host = url.host_str().unwrap_or_default().to_ascii_lowercase();
    let allowed_host = matches!(
        host.as_str(),
        "localhost" | "127.0.0.1" | "::1" | "postgres" | "host.docker.internal"
    );
    if !allowed_host {
        return Err(DevSeedError::Safety(format!(
            "refusing to seed non-local database host '{host}'"
        )));
    }

    let database_name = url.path().trim_start_matches('/');
    let allowed_database = database_name == "hexrelay" || database_name.starts_with("hexrelay_");
    if !allowed_database {
        return Err(DevSeedError::Safety(format!(
            "refusing to seed database '{database_name}'; expected hexrelay or hexrelay_*"
        )));
    }

    Ok(())
}

pub fn validate_reset_target(database_url: &str, environment: &str) -> Result<(), DevSeedError> {
    validate_seed_target(database_url, environment)?;

    let url = Url::parse(database_url).map_err(|_| {
        DevSeedError::Safety("refusing to reset because API_DATABASE_URL is invalid".to_string())
    })?;
    let host = url.host_str().unwrap_or_default().to_ascii_lowercase();
    let loopback_host = matches!(host.as_str(), "localhost" | "127.0.0.1" | "::1");
    if !loopback_host {
        return Err(DevSeedError::Safety(format!(
            "refusing to reset non-loopback database host '{host}'"
        )));
    }

    Ok(())
}

fn load_scenario(options: &SeedCliOptions) -> Result<SeedScenario, DevSeedError> {
    let fixtures_root = match &options.fixtures_root {
        Some(path) => path.clone(),
        None => default_fixtures_root()?,
    };
    let path = fixtures_root
        .join("scenarios")
        .join(format!("{}.json", options.profile));
    let content = fs::read_to_string(path)?;
    let scenario = serde_json::from_str::<SeedScenario>(&content)?;

    if scenario.scenario_id != options.profile {
        return Err(DevSeedError::Config(format!(
            "fixture scenario_id '{}' does not match requested profile '{}'",
            scenario.scenario_id, options.profile
        )));
    }

    validate_scenario(&scenario)?;
    Ok(scenario)
}

pub async fn seed_profile(
    pool: &PgPool,
    options: &SeedCliOptions,
    active_signing_key_id: &str,
    signing_key: &str,
) -> Result<SeedSummary, DevSeedError> {
    let scenario = load_scenario(options)?;
    seed_scenario(pool, scenario, active_signing_key_id, signing_key).await
}

pub fn validate_seed_profile(options: &SeedCliOptions) -> Result<(), DevSeedError> {
    load_scenario(options).map(|_| ())
}

pub async fn reset_local_database(
    database_url: &str,
    options: &ResetCliOptions,
) -> Result<(), DevSeedError> {
    if !options.yes {
        return Err(DevSeedError::Safety(
            "refusing to reset without --yes confirmation".to_string(),
        ));
    }

    assert_safe_reset_target(database_url)?;

    let pool = PgPool::connect(database_url).await?;
    sqlx::query("DROP SCHEMA IF EXISTS public CASCADE")
        .execute(&pool)
        .await?;
    sqlx::query("CREATE SCHEMA public").execute(&pool).await?;
    sqlx::query("GRANT ALL ON SCHEMA public TO public")
        .execute(&pool)
        .await?;
    pool.close().await;

    Ok(())
}

async fn seed_scenario(
    pool: &PgPool,
    scenario: SeedScenario,
    active_signing_key_id: &str,
    signing_key: &str,
) -> Result<SeedSummary, DevSeedError> {
    let now = Utc::now();
    let mut tx = pool.begin().await?;

    for identity in &scenario.identities {
        seed_identity(&mut tx, identity).await?;
    }

    let mut sessions = Vec::new();
    for session in &scenario.sessions {
        let expires_at = now + Duration::days(session.expires_in_days);
        seed_session(&mut tx, session, expires_at).await?;

        let token = issue_session_token(
            &session.session_id,
            &session.identity_id,
            expires_at.timestamp(),
            active_signing_key_id,
            signing_key,
        );
        sessions.push(SeededSession {
            profile_id: session.profile_id.clone(),
            identity_id: session.identity_id.clone(),
            session_id: session.session_id.clone(),
            expires_at: expires_at.to_rfc3339(),
            authorization_header: format!("Bearer {token}"),
            cookie_header: format!(
                "{SESSION_COOKIE_NAME}={token}; {CSRF_COOKIE_NAME}={DEV_CSRF_TOKEN}"
            ),
            csrf_header: DEV_CSRF_TOKEN.to_string(),
        });
    }

    for request in &scenario.friend_requests {
        seed_friend_request(&mut tx, request).await?;
    }

    for policy in &scenario.dm_policies {
        seed_dm_policy(&mut tx, policy).await?;
    }

    for device in &scenario.devices {
        seed_device(&mut tx, device, now.timestamp()).await?;
    }

    for invite in &scenario.invites {
        seed_invite(&mut tx, invite).await?;
    }

    for server in &scenario.servers {
        seed_server(&mut tx, server).await?;
    }

    for membership in &scenario.server_memberships {
        seed_server_membership(&mut tx, membership).await?;
    }

    for channel in &scenario.server_channels {
        seed_server_channel(&mut tx, channel).await?;
    }

    for message in &scenario.server_channel_messages {
        seed_server_channel_message(&mut tx, message).await?;
    }

    for thread in &scenario.dm_threads {
        seed_dm_thread(&mut tx, thread).await?;
    }

    tx.commit().await?;

    let counts = SeedCounts {
        identities: scenario.identities.len(),
        sessions: scenario.sessions.len(),
        friend_requests: scenario.friend_requests.len(),
        dm_policies: scenario.dm_policies.len(),
        devices: scenario.devices.len(),
        invites: scenario.invites.len(),
        servers: scenario.servers.len(),
        server_memberships: scenario.server_memberships.len(),
        server_channels: scenario.server_channels.len(),
        server_channel_messages: scenario.server_channel_messages.len(),
        dm_threads: scenario.dm_threads.len(),
        dm_messages: scenario
            .dm_threads
            .iter()
            .map(|thread| thread.messages.len())
            .sum(),
    };

    Ok(SeedSummary {
        profile: scenario.scenario_id,
        description: scenario.description,
        counts,
        sessions,
    })
}

pub fn format_seed_summary(summary: &SeedSummary) -> String {
    let mut output = format!(
        "[seed] Seeded profile '{}'\n[seed] {}\n[seed] identities={} sessions={} friend_requests={} dm_policies={} devices={} invites={} servers={} server_memberships={} server_channels={} server_channel_messages={} dm_threads={} dm_messages={}",
        summary.profile,
        summary.description,
        summary.counts.identities,
        summary.counts.sessions,
        summary.counts.friend_requests,
        summary.counts.dm_policies,
        summary.counts.devices,
        summary.counts.invites,
        summary.counts.servers,
        summary.counts.server_memberships,
        summary.counts.server_channels,
        summary.counts.server_channel_messages,
        summary.counts.dm_threads,
        summary.counts.dm_messages
    );

    for session in &summary.sessions {
        output.push_str(&format!(
            "\n[seed] {} ({})\n  identity_id: {}\n  session_id: {}\n  expires_at: {}\n  authorization: {}\n  cookie: {}\n  x-csrf-token: {}",
            session.profile_id,
            session.identity_id,
            session.identity_id,
            session.session_id,
            session.expires_at,
            session.authorization_header,
            session.cookie_header,
            session.csrf_header
        ));
    }

    output
}

fn default_fixtures_root() -> Result<PathBuf, DevSeedError> {
    let mut current = env::current_dir().map_err(DevSeedError::Io)?;
    loop {
        let candidate = current.join("scripts").join("fixtures");
        if candidate.join("scenarios").is_dir() {
            return Ok(candidate);
        }

        if !current.pop() {
            return Err(DevSeedError::Config(
                "could not locate scripts/fixtures from current directory".to_string(),
            ));
        }
    }
}

fn validate_scenario(scenario: &SeedScenario) -> Result<(), DevSeedError> {
    if scenario.scenario_id.trim().is_empty() {
        return Err(DevSeedError::Config(
            "fixture scenario_id must not be empty".to_string(),
        ));
    }

    let mut identity_ids = HashSet::new();
    for identity in &scenario.identities {
        if identity.profile_id.trim().is_empty() || identity.identity_id.trim().is_empty() {
            return Err(DevSeedError::Config(
                "identity fixtures require profile_id and identity_id".to_string(),
            ));
        }
        if !identity.identity_id.starts_with("usr-test-") {
            return Err(DevSeedError::Config(format!(
                "fixture identity '{}' must use usr-test-* prefix",
                identity.identity_id
            )));
        }
        if !identity_ids.insert(identity.identity_id.as_str()) {
            return Err(DevSeedError::Config(format!(
                "duplicate fixture identity '{}'",
                identity.identity_id
            )));
        }
        if identity.algorithm != "ed25519" {
            return Err(DevSeedError::Config(format!(
                "unsupported fixture key algorithm '{}' for '{}'",
                identity.algorithm, identity.identity_id
            )));
        }
        if identity.public_key.len() != 64
            || !identity.public_key.chars().all(|c| c.is_ascii_hexdigit())
        {
            return Err(DevSeedError::Config(format!(
                "fixture public key for '{}' must be 64 hex characters",
                identity.identity_id
            )));
        }
    }

    for session in &scenario.sessions {
        require_identity(&identity_ids, &session.identity_id, "session")?;
        if session.session_id.trim().is_empty() || session.expires_in_days <= 0 {
            return Err(DevSeedError::Config(format!(
                "session fixture for '{}' requires session_id and positive expires_in_days",
                session.identity_id
            )));
        }
    }

    for request in &scenario.friend_requests {
        require_identity(
            &identity_ids,
            &request.requester_identity_id,
            "friend request requester",
        )?;
        require_identity(
            &identity_ids,
            &request.target_identity_id,
            "friend request target",
        )?;
        if request.requester_identity_id == request.target_identity_id {
            return Err(DevSeedError::Config(format!(
                "friend request '{}' cannot target the requester",
                request.request_id
            )));
        }
        if !matches!(
            request.status.as_str(),
            "accepted" | "pending" | "declined" | "cancelled"
        ) {
            return Err(DevSeedError::Config(format!(
                "unsupported friend request status '{}'",
                request.status
            )));
        }
    }

    for policy in &scenario.dm_policies {
        require_identity(&identity_ids, &policy.identity_id, "dm policy")?;
        if policy.offline_delivery_mode != "encrypted_envelope_catchup" {
            return Err(DevSeedError::Config(format!(
                "unsupported offline delivery mode '{}'",
                policy.offline_delivery_mode
            )));
        }
        if !matches!(
            policy.inbound_policy.as_str(),
            "friends_only" | "same_server" | "anyone"
        ) {
            return Err(DevSeedError::Config(format!(
                "unsupported inbound DM policy '{}'",
                policy.inbound_policy
            )));
        }
    }

    for device in &scenario.devices {
        require_identity(&identity_ids, &device.identity_id, "device")?;
    }

    for invite in &scenario.invites {
        if !invite.invite_id.starts_with("fixture-invite-") {
            return Err(DevSeedError::Config(format!(
                "invite '{}' must use fixture-invite-* prefix",
                invite.invite_id
            )));
        }
        if invite.token_hash.len() != 64
            || !invite.token_hash.chars().all(|c| c.is_ascii_hexdigit())
        {
            return Err(DevSeedError::Config(format!(
                "invite '{}' token_hash must be 64 hex characters",
                invite.invite_id
            )));
        }
        if !matches!(invite.mode.as_str(), "one_time" | "multi_use") {
            return Err(DevSeedError::Config(format!(
                "unsupported invite mode '{}'",
                invite.mode
            )));
        }
        if invite.server_id.trim().is_empty() {
            return Err(DevSeedError::Config(format!(
                "invite '{}' requires server_id",
                invite.invite_id
            )));
        }
        if invite.uses < 0 || invite.max_uses.is_some_and(|max_uses| max_uses <= 0) {
            return Err(DevSeedError::Config(format!(
                "invite '{}' requires non-negative uses and positive max_uses",
                invite.invite_id
            )));
        }
        if invite
            .max_uses
            .is_some_and(|max_uses| invite.uses > max_uses)
        {
            return Err(DevSeedError::Config(format!(
                "invite '{}' uses cannot exceed max_uses",
                invite.invite_id
            )));
        }
        validate_timestamp(&invite.created_at, "invite created_at")?;
        if let Some(expires_at) = &invite.expires_at {
            validate_timestamp(expires_at, "invite expires_at")?;
        }
    }

    if scenario.servers.len() > 1 {
        return Err(DevSeedError::Config(
            "server fixtures seed one local server at a time".to_string(),
        ));
    }

    if scenario.servers.is_empty()
        && (!scenario.server_memberships.is_empty()
            || !scenario.server_channels.is_empty()
            || !scenario.server_channel_messages.is_empty())
    {
        return Err(DevSeedError::Config(
            "server fixtures require one local server definition".to_string(),
        ));
    }

    for server in &scenario.servers {
        if server.name.trim().is_empty() {
            return Err(DevSeedError::Config(
                "local server fixture requires name".to_string(),
            ));
        }
        validate_timestamp(&server.created_at, "server created_at")?;
    }

    let mut memberships = HashSet::new();
    for membership in &scenario.server_memberships {
        require_identity(&identity_ids, &membership.identity_id, "server membership")?;
        if membership.unread_count < 0 {
            return Err(DevSeedError::Config(format!(
                "server membership for '{}' requires non-negative unread_count",
                membership.identity_id
            )));
        }
        if !memberships.insert(membership.identity_id.as_str()) {
            return Err(DevSeedError::Config(format!(
                "duplicate server membership '{}'",
                membership.identity_id
            )));
        }
        validate_timestamp(&membership.joined_at, "server membership joined_at")?;
    }

    let mut channel_ids = HashSet::new();
    let mut channel_last_seqs: HashMap<&str, u64> = HashMap::new();
    for channel in &scenario.server_channels {
        if !channel.channel_id.starts_with("fixture-channel-") {
            return Err(DevSeedError::Config(format!(
                "server channel '{}' must use fixture-channel-* prefix",
                channel.channel_id
            )));
        }
        if !channel_ids.insert(channel.channel_id.as_str()) {
            return Err(DevSeedError::Config(format!(
                "duplicate server channel '{}'",
                channel.channel_id
            )));
        }
        channel_last_seqs.insert(channel.channel_id.as_str(), channel.last_message_seq);
        if channel.name.trim().is_empty() || channel.kind != "text" {
            return Err(DevSeedError::Config(format!(
                "server channel '{}' requires name and text kind",
                channel.channel_id
            )));
        }
        validate_timestamp(&channel.created_at, "server channel created_at")?;
    }

    let mut message_channels = HashMap::new();
    let mut channel_message_seqs = HashSet::new();
    let mut channel_max_message_seqs: HashMap<&str, u64> = HashMap::new();
    for message in &scenario.server_channel_messages {
        if !message.message_id.starts_with("fixture-") {
            return Err(DevSeedError::Config(format!(
                "server message '{}' must use fixture-* prefix",
                message.message_id
            )));
        }
        if !channel_ids.contains(message.channel_id.as_str()) {
            return Err(DevSeedError::Config(format!(
                "server message '{}' references unknown channel '{}'",
                message.message_id, message.channel_id
            )));
        }
        require_identity(&identity_ids, &message.author_id, "server message author")?;
        if !memberships.contains(message.author_id.as_str()) {
            return Err(DevSeedError::Config(format!(
                "server message '{}' author is not a local server member",
                message.message_id
            )));
        }
        if message.channel_seq == 0 || message.content.trim().is_empty() {
            return Err(DevSeedError::Config(format!(
                "server message '{}' requires positive channel_seq and content",
                message.message_id
            )));
        }
        let seq_key = format!("{}:{}", message.channel_id, message.channel_seq);
        if !channel_message_seqs.insert(seq_key) {
            return Err(DevSeedError::Config(format!(
                "server message '{}' duplicates channel_seq {} in channel '{}'",
                message.message_id, message.channel_seq, message.channel_id
            )));
        }
        channel_max_message_seqs
            .entry(message.channel_id.as_str())
            .and_modify(|max_seq| *max_seq = (*max_seq).max(message.channel_seq))
            .or_insert(message.channel_seq);
        if let Some(reply_to_message_id) = &message.reply_to_message_id {
            let Some(reply_channel_id) = message_channels.get(reply_to_message_id.as_str()) else {
                return Err(DevSeedError::Config(format!(
                    "server message '{}' replies to unknown earlier message '{}'",
                    message.message_id, reply_to_message_id
                )));
            };
            if *reply_channel_id != message.channel_id.as_str() {
                return Err(DevSeedError::Config(format!(
                    "server message '{}' replies across channels",
                    message.message_id
                )));
            }
        }
        for mentioned_identity_id in &message.mention_identity_ids {
            require_identity(
                &identity_ids,
                mentioned_identity_id,
                "server message mention",
            )?;
            if !memberships.contains(mentioned_identity_id.as_str()) {
                return Err(DevSeedError::Config(format!(
                    "server message '{}' mentions non-member '{}'",
                    message.message_id, mentioned_identity_id
                )));
            }
        }
        validate_timestamp(&message.created_at, "server message created_at")?;
        if let Some(edited_at) = &message.edited_at {
            validate_timestamp(edited_at, "server message edited_at")?;
        }
        if let Some(deleted_at) = &message.deleted_at {
            validate_timestamp(deleted_at, "server message deleted_at")?;
        }
        if message_channels
            .insert(message.message_id.as_str(), message.channel_id.as_str())
            .is_some()
        {
            return Err(DevSeedError::Config(format!(
                "duplicate server message '{}'",
                message.message_id
            )));
        }
    }

    for (channel_id, last_message_seq) in channel_last_seqs {
        let max_message_seq = channel_max_message_seqs
            .get(channel_id)
            .copied()
            .unwrap_or(0);
        if last_message_seq != max_message_seq {
            return Err(DevSeedError::Config(format!(
                "server channel '{}' last_message_seq must equal max seeded message seq {}",
                channel_id, max_message_seq
            )));
        }
    }

    let mut dm_message_ids = HashSet::new();
    for thread in &scenario.dm_threads {
        if !is_seedable_dm_thread_id(&thread.thread_id) {
            return Err(DevSeedError::Config(format!(
                "DM thread '{}' must use a fixture-compatible thread id prefix",
                thread.thread_id
            )));
        }
        if thread.title.trim().is_empty() {
            return Err(DevSeedError::Config(format!(
                "DM thread '{}' requires title",
                thread.thread_id
            )));
        }
        if thread.kind != "dm" && thread.kind != "group_dm" {
            return Err(DevSeedError::Config(format!(
                "unsupported DM thread kind '{}'",
                thread.kind
            )));
        }
        if thread.kind == "dm" && thread.participants.len() != 2 {
            return Err(DevSeedError::Config(format!(
                "DM thread '{}' requires exactly two participants",
                thread.thread_id
            )));
        }
        if thread.kind == "group_dm" && thread.participants.len() < 3 {
            return Err(DevSeedError::Config(format!(
                "group DM thread '{}' requires at least three participants",
                thread.thread_id
            )));
        }
        let mut participant_ids = HashSet::new();
        for participant in &thread.participants {
            require_identity(&identity_ids, &participant.identity_id, "dm participant")?;
            if !participant_ids.insert(participant.identity_id.as_str()) {
                return Err(DevSeedError::Config(format!(
                    "DM thread '{}' has duplicate participant '{}'",
                    thread.thread_id, participant.identity_id
                )));
            }
        }
        let mut message_seqs = HashSet::new();
        for message in &thread.messages {
            require_identity(&identity_ids, &message.author_id, "dm message author")?;
            if !participant_ids.contains(message.author_id.as_str()) {
                return Err(DevSeedError::Config(format!(
                    "DM message '{}' author is not a thread participant",
                    message.message_id
                )));
            }
            if !message.message_id.starts_with("fixture-") {
                return Err(DevSeedError::Config(format!(
                    "DM message '{}' must use fixture-* prefix",
                    message.message_id
                )));
            }
            if !dm_message_ids.insert(message.message_id.as_str()) {
                return Err(DevSeedError::Config(format!(
                    "duplicate DM message '{}'",
                    message.message_id
                )));
            }
            if message.seq == 0 || !message_seqs.insert(message.seq) {
                return Err(DevSeedError::Config(format!(
                    "DM message '{}' requires positive unique seq",
                    message.message_id
                )));
            }
            if message.ciphertext.trim().is_empty() {
                return Err(DevSeedError::Config(format!(
                    "DM message '{}' requires ciphertext",
                    message.message_id
                )));
            }
            validate_timestamp(&message.created_at, "DM message created_at")?;
            if let Some(edited_at) = &message.edited_at {
                validate_timestamp(edited_at, "DM message edited_at")?;
            }
        }
    }

    Ok(())
}

fn is_seedable_dm_thread_id(thread_id: &str) -> bool {
    thread_id.starts_with("fixture-") || thread_id.starts_with("dm-usr-test-")
}

fn require_identity(
    identity_ids: &HashSet<&str>,
    identity_id: &str,
    label: &str,
) -> Result<(), DevSeedError> {
    if identity_ids.contains(identity_id) {
        return Ok(());
    }

    Err(DevSeedError::Config(format!(
        "{label} references unknown identity '{identity_id}'"
    )))
}

fn validate_timestamp(value: &str, label: &str) -> Result<(), DevSeedError> {
    chrono::DateTime::parse_from_rfc3339(value).map_err(|_| {
        DevSeedError::Config(format!(
            "{label} must be an RFC3339 timestamp, got '{value}'"
        ))
    })?;

    Ok(())
}

async fn seed_identity(
    tx: &mut Transaction<'_, Postgres>,
    identity: &IdentityFixture,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "
        INSERT INTO identity_keys (identity_id, public_key, algorithm)
        VALUES ($1, $2, $3)
        ON CONFLICT (identity_id) DO UPDATE
        SET public_key = EXCLUDED.public_key,
            algorithm = EXCLUDED.algorithm
        ",
    )
    .bind(&identity.identity_id)
    .bind(&identity.public_key)
    .bind(&identity.algorithm)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

async fn seed_session(
    tx: &mut Transaction<'_, Postgres>,
    session: &SessionFixture,
    expires_at: chrono::DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "
        INSERT INTO sessions (session_id, identity_id, expires_at, revoked_at)
        VALUES ($1, $2, $3, NULL)
        ON CONFLICT (session_id) DO UPDATE
        SET identity_id = EXCLUDED.identity_id,
            expires_at = EXCLUDED.expires_at,
            revoked_at = NULL
        ",
    )
    .bind(&session.session_id)
    .bind(&session.identity_id)
    .bind(expires_at)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

async fn seed_friend_request(
    tx: &mut Transaction<'_, Postgres>,
    request: &FriendRequestFixture,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "
        INSERT INTO friend_requests (request_id, requester_identity_id, target_identity_id, status)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (request_id) DO UPDATE
        SET requester_identity_id = EXCLUDED.requester_identity_id,
            target_identity_id = EXCLUDED.target_identity_id,
            status = EXCLUDED.status
        ",
    )
    .bind(&request.request_id)
    .bind(&request.requester_identity_id)
    .bind(&request.target_identity_id)
    .bind(&request.status)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

async fn seed_dm_policy(
    tx: &mut Transaction<'_, Postgres>,
    policy: &DmPolicyFixture,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "
        INSERT INTO dm_policies (identity_id, inbound_policy, offline_delivery_mode, updated_at)
        VALUES ($1, $2, $3, NOW())
        ON CONFLICT (identity_id) DO UPDATE
        SET inbound_policy = EXCLUDED.inbound_policy,
            offline_delivery_mode = EXCLUDED.offline_delivery_mode,
            updated_at = NOW()
        ",
    )
    .bind(&policy.identity_id)
    .bind(&policy.inbound_policy)
    .bind(&policy.offline_delivery_mode)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

async fn seed_device(
    tx: &mut Transaction<'_, Postgres>,
    device: &DeviceFixture,
    now_epoch: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "
        INSERT INTO dm_profile_devices (identity_id, device_id, active, last_seen_epoch, updated_at)
        VALUES ($1, $2, $3, $4, NOW())
        ON CONFLICT (identity_id, device_id) DO UPDATE
        SET active = EXCLUDED.active,
            last_seen_epoch = EXCLUDED.last_seen_epoch,
            updated_at = NOW()
        ",
    )
    .bind(&device.identity_id)
    .bind(&device.device_id)
    .bind(device.active)
    .bind(now_epoch + device.last_seen_offset_seconds)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

async fn seed_invite(
    tx: &mut Transaction<'_, Postgres>,
    invite: &InviteFixture,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "
        INSERT INTO invites (
            invite_id,
            token,
            mode,
            server_id,
            expires_at,
            max_uses,
            uses,
            created_at
        )
        VALUES ($1, $2, $3, $4, $5::timestamptz, $6, $7, $8::timestamptz)
        ON CONFLICT (token) DO UPDATE
        SET invite_id = EXCLUDED.invite_id,
            mode = EXCLUDED.mode,
            server_id = EXCLUDED.server_id,
            expires_at = EXCLUDED.expires_at,
            max_uses = EXCLUDED.max_uses,
            uses = EXCLUDED.uses,
            created_at = EXCLUDED.created_at
        ",
    )
    .bind(&invite.invite_id)
    .bind(&invite.token_hash)
    .bind(&invite.mode)
    .bind(&invite.server_id)
    .bind(invite.expires_at.as_deref())
    .bind(invite.max_uses)
    .bind(invite.uses)
    .bind(&invite.created_at)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

async fn seed_server(
    tx: &mut Transaction<'_, Postgres>,
    server: &ServerFixture,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "
        INSERT INTO local_server (singleton, name, created_at)
        VALUES (TRUE, $1, $2::timestamptz)
        ON CONFLICT (singleton) DO UPDATE
        SET name = EXCLUDED.name,
            created_at = EXCLUDED.created_at
        ",
    )
    .bind(&server.name)
    .bind(&server.created_at)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

async fn seed_server_membership(
    tx: &mut Transaction<'_, Postgres>,
    membership: &ServerMembershipFixture,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "
        INSERT INTO server_memberships (
            identity_id,
            pinned,
            muted,
            unread_count,
            joined_at
        )
        VALUES ($1, $2, $3, $4, $5::timestamptz)
        ON CONFLICT (identity_id) DO UPDATE
        SET pinned = EXCLUDED.pinned,
            muted = EXCLUDED.muted,
            unread_count = EXCLUDED.unread_count,
            joined_at = EXCLUDED.joined_at
        ",
    )
    .bind(&membership.identity_id)
    .bind(membership.pinned)
    .bind(membership.muted)
    .bind(membership.unread_count)
    .bind(&membership.joined_at)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

async fn seed_server_channel(
    tx: &mut Transaction<'_, Postgres>,
    channel: &ServerChannelFixture,
) -> Result<(), sqlx::Error> {
    prune_server_channel_fixture_rows(tx, &channel.channel_id).await?;

    sqlx::query(
        "
        INSERT INTO server_channels (
            channel_id,
            name,
            kind,
            last_message_seq,
            created_at
        )
        VALUES ($1, $2, $3, $4, $5::timestamptz)
        ON CONFLICT (channel_id) DO UPDATE
        SET name = EXCLUDED.name,
            kind = EXCLUDED.kind,
            last_message_seq = EXCLUDED.last_message_seq,
            created_at = EXCLUDED.created_at
        ",
    )
    .bind(&channel.channel_id)
    .bind(&channel.name)
    .bind(&channel.kind)
    .bind(
        i64::try_from(channel.last_message_seq)
            .map_err(|_| sqlx::Error::Protocol("last_message_seq too large for storage".into()))?,
    )
    .bind(&channel.created_at)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

async fn seed_server_channel_message(
    tx: &mut Transaction<'_, Postgres>,
    message: &ServerChannelMessageFixture,
) -> Result<(), sqlx::Error> {
    let channel_seq = i64::try_from(message.channel_seq)
        .map_err(|_| sqlx::Error::Protocol("channel_seq too large for storage".into()))?;

    sqlx::query(
        "
        UPDATE server_channels
        SET last_message_seq = GREATEST(last_message_seq, $2)
        WHERE channel_id = $1
        ",
    )
    .bind(&message.channel_id)
    .bind(channel_seq)
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        "
        INSERT INTO server_channel_messages (
            message_id,
            channel_id,
            author_id,
            channel_seq,
            content,
            reply_to_message_id,
            created_at,
            edited_at,
            deleted_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7::timestamptz, $8::timestamptz, $9::timestamptz)
        ON CONFLICT (message_id) DO UPDATE
        SET channel_id = EXCLUDED.channel_id,
            author_id = EXCLUDED.author_id,
            channel_seq = EXCLUDED.channel_seq,
            content = EXCLUDED.content,
            reply_to_message_id = EXCLUDED.reply_to_message_id,
            created_at = EXCLUDED.created_at,
            edited_at = EXCLUDED.edited_at,
            deleted_at = EXCLUDED.deleted_at
        ",
    )
    .bind(&message.message_id)
    .bind(&message.channel_id)
    .bind(&message.author_id)
    .bind(channel_seq)
    .bind(&message.content)
    .bind(message.reply_to_message_id.as_deref())
    .bind(&message.created_at)
    .bind(message.edited_at.as_deref())
    .bind(message.deleted_at.as_deref())
    .execute(&mut **tx)
    .await?;

    sqlx::query("DELETE FROM server_channel_message_mentions WHERE message_id = $1")
        .bind(&message.message_id)
        .execute(&mut **tx)
        .await?;

    for mentioned_identity_id in &message.mention_identity_ids {
        sqlx::query(
            "
            INSERT INTO server_channel_message_mentions (message_id, mentioned_identity_id)
            VALUES ($1, $2)
            ON CONFLICT (message_id, mentioned_identity_id) DO NOTHING
            ",
        )
        .bind(&message.message_id)
        .bind(mentioned_identity_id)
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}

async fn prune_server_channel_fixture_rows(
    tx: &mut Transaction<'_, Postgres>,
    channel_id: &str,
) -> Result<(), sqlx::Error> {
    if !channel_id.starts_with("fixture-channel-") {
        return Err(sqlx::Error::Protocol(
            "refusing to prune non-fixture server channel".into(),
        ));
    }

    sqlx::query("DELETE FROM server_channel_messages WHERE channel_id = $1")
        .bind(channel_id)
        .execute(&mut **tx)
        .await?;

    Ok(())
}

async fn seed_dm_thread(
    tx: &mut Transaction<'_, Postgres>,
    thread: &DmThreadFixture,
) -> Result<(), sqlx::Error> {
    prune_dm_thread_fixture_rows(tx, &thread.thread_id).await?;

    sqlx::query(
        "
        INSERT INTO dm_threads (thread_id, kind, title)
        VALUES ($1, $2, $3)
        ON CONFLICT (thread_id) DO UPDATE
        SET kind = EXCLUDED.kind,
            title = EXCLUDED.title
        ",
    )
    .bind(&thread.thread_id)
    .bind(&thread.kind)
    .bind(&thread.title)
    .execute(&mut **tx)
    .await?;

    for participant in &thread.participants {
        sqlx::query(
            "
            INSERT INTO dm_thread_participants (thread_id, identity_id, last_read_seq)
            VALUES ($1, $2, $3)
            ON CONFLICT (thread_id, identity_id) DO UPDATE
            SET last_read_seq = EXCLUDED.last_read_seq
            ",
        )
        .bind(&thread.thread_id)
        .bind(&participant.identity_id)
        .bind(
            i64::try_from(participant.last_read_seq)
                .map_err(|_| sqlx::Error::Protocol("last_read_seq too large for storage".into()))?,
        )
        .execute(&mut **tx)
        .await?;
    }

    for message in &thread.messages {
        sqlx::query(
            "
            INSERT INTO dm_messages (message_id, thread_id, author_id, seq, ciphertext, created_at, edited_at)
            VALUES ($1, $2, $3, $4, $5, $6::timestamptz, $7::timestamptz)
            ON CONFLICT (message_id) DO UPDATE
            SET thread_id = EXCLUDED.thread_id,
                author_id = EXCLUDED.author_id,
                seq = EXCLUDED.seq,
                ciphertext = EXCLUDED.ciphertext,
                created_at = EXCLUDED.created_at,
                edited_at = EXCLUDED.edited_at
            ",
        )
        .bind(&message.message_id)
        .bind(&thread.thread_id)
        .bind(&message.author_id)
        .bind(i64::try_from(message.seq).map_err(|_| {
            sqlx::Error::Protocol("seq too large for storage".into())
        })?)
        .bind(&message.ciphertext)
        .bind(&message.created_at)
        .bind(message.edited_at.as_deref())
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}

async fn prune_dm_thread_fixture_rows(
    tx: &mut Transaction<'_, Postgres>,
    thread_id: &str,
) -> Result<(), sqlx::Error> {
    if !is_seedable_dm_thread_id(thread_id) {
        return Err(sqlx::Error::Protocol(
            "refusing to prune non-fixture DM thread".into(),
        ));
    }

    if thread_id.starts_with("fixture-") {
        sqlx::query("DELETE FROM dm_messages WHERE thread_id = $1")
            .bind(thread_id)
            .execute(&mut **tx)
            .await?;

        sqlx::query("DELETE FROM dm_thread_participants WHERE thread_id = $1")
            .bind(thread_id)
            .execute(&mut **tx)
            .await?;
    } else {
        sqlx::query("DELETE FROM dm_messages WHERE thread_id = $1 AND message_id LIKE 'fixture-%'")
            .bind(thread_id)
            .execute(&mut **tx)
            .await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dm_basic() -> SeedScenario {
        serde_json::from_str(include_str!(
            "../../../scripts/fixtures/scenarios/dm-basic.json"
        ))
        .expect("parse dm-basic fixture")
    }

    fn contacts_edge() -> SeedScenario {
        serde_json::from_str(include_str!(
            "../../../scripts/fixtures/scenarios/contacts-edge.json"
        ))
        .expect("parse contacts-edge fixture")
    }

    fn server_chat() -> SeedScenario {
        serde_json::from_str(include_str!(
            "../../../scripts/fixtures/scenarios/server-chat.json"
        ))
        .expect("parse server-chat fixture")
    }

    #[test]
    fn parses_and_validates_dm_basic_fixture() {
        let scenario = dm_basic();

        validate_scenario(&scenario).expect("dm-basic fixture validates");

        assert_eq!(scenario.scenario_id, "dm-basic");
        assert_eq!(scenario.identities.len(), 2);
        assert_eq!(scenario.sessions.len(), 2);
        assert_eq!(scenario.dm_threads.len(), 1);
        assert_eq!(scenario.dm_threads[0].messages.len(), 5);
    }

    #[test]
    fn parses_and_validates_contacts_edge_fixture() {
        let scenario = contacts_edge();

        validate_scenario(&scenario).expect("contacts-edge fixture validates");

        assert_eq!(scenario.scenario_id, "contacts-edge");
        assert_eq!(scenario.identities.len(), 3);
        assert_eq!(scenario.sessions.len(), 3);
        assert_eq!(scenario.friend_requests.len(), 2);
        assert!(scenario.invites.is_empty());
    }

    #[test]
    fn parses_and_validates_server_chat_fixture() {
        let scenario = server_chat();

        validate_scenario(&scenario).expect("server-chat fixture validates");

        assert_eq!(scenario.scenario_id, "server-chat");
        assert_eq!(scenario.identities.len(), 3);
        assert_eq!(scenario.servers.len(), 1);
        assert_eq!(scenario.server_memberships.len(), 3);
        assert_eq!(scenario.server_channels.len(), 2);
        assert_eq!(scenario.server_channel_messages.len(), 5);
    }

    #[test]
    fn dm_basic_fixture_matches_seeded_chat_invariants() {
        let scenario = dm_basic();

        validate_scenario(&scenario).expect("dm-basic fixture validates");

        assert_fixture_identity(&scenario, "alice.primary", "usr-test-alice");
        assert_fixture_identity(&scenario, "bob.primary", "usr-test-bob");
        assert_fixture_session(&scenario, "alice.primary", "sess-test-alice-primary");
        assert_fixture_session(&scenario, "bob.primary", "sess-test-bob-primary");
        assert_friend_request(
            &scenario,
            "fixture-fr-alice-bob",
            "usr-test-alice",
            "usr-test-bob",
            "accepted",
        );
        assert_dm_policy(&scenario, "usr-test-alice", "friends_only");
        assert_dm_policy(&scenario, "usr-test-bob", "friends_only");

        let thread = scenario
            .dm_threads
            .iter()
            .find(|thread| thread.thread_id == "dm-usr-test-alice-usr-test-bob")
            .expect("alice/bob DM thread exists");
        assert_eq!(thread.kind, "dm");
        assert_eq!(thread.participants.len(), 2);
        assert!(thread
            .participants
            .iter()
            .any(|participant| participant.identity_id == "usr-test-alice"
                && participant.last_read_seq == 3));
        assert!(thread
            .participants
            .iter()
            .any(|participant| participant.identity_id == "usr-test-bob"
                && participant.last_read_seq == 5));
        assert_eq!(thread.messages.len(), 5);
        for (index, message) in thread.messages.iter().enumerate() {
            assert_eq!(message.seq, (index + 1) as u64);
            assert!(message.message_id.starts_with("fixture-dm-alice-bob-"));
            assert!(matches!(
                message.author_id.as_str(),
                "usr-test-alice" | "usr-test-bob"
            ));
        }
        assert_eq!(
            scenario
                .devices
                .iter()
                .filter(|device| device.identity_id == "usr-test-bob" && device.active)
                .count(),
            2
        );
    }

    #[test]
    fn contacts_edge_fixture_matches_pending_and_restricted_invariants() {
        let scenario = contacts_edge();

        validate_scenario(&scenario).expect("contacts-edge fixture validates");

        assert_fixture_identity(&scenario, "alice.primary", "usr-test-alice");
        assert_fixture_identity(&scenario, "carol.pending", "usr-test-carol");
        assert_fixture_identity(&scenario, "dave.restricted", "usr-test-dave");
        assert_friend_request(
            &scenario,
            "fixture-fr-alice-carol-pending",
            "usr-test-alice",
            "usr-test-carol",
            "pending",
        );
        assert_friend_request(
            &scenario,
            "fixture-fr-dave-alice-pending",
            "usr-test-dave",
            "usr-test-alice",
            "pending",
        );
        assert_dm_policy(&scenario, "usr-test-carol", "same_server");
        assert_dm_policy(&scenario, "usr-test-dave", "friends_only");
        assert!(scenario.invites.is_empty());
        assert!(scenario.dm_threads.is_empty());
    }

    #[test]
    fn server_chat_fixture_matches_shared_server_invariants() {
        let scenario = server_chat();

        validate_scenario(&scenario).expect("server-chat fixture validates");

        assert_eq!(scenario.servers[0].name, "Atlas Test Server");
        assert_eq!(scenario.server_memberships.len(), 3);
        assert!(scenario.server_memberships.iter().any(|membership| {
            membership.identity_id == "usr-test-alice" && membership.pinned && !membership.muted
        }));
        assert!(scenario.server_memberships.iter().any(|membership| {
            membership.identity_id == "usr-test-carol" && !membership.pinned && membership.muted
        }));

        let general = scenario
            .server_channels
            .iter()
            .find(|channel| channel.channel_id == "fixture-channel-atlas-general")
            .expect("general channel exists");
        let ops = scenario
            .server_channels
            .iter()
            .find(|channel| channel.channel_id == "fixture-channel-atlas-ops-lab")
            .expect("ops-lab channel exists");
        assert_eq!(general.last_message_seq, 3);
        assert_eq!(ops.last_message_seq, 2);
        assert!(scenario.server_channel_messages.iter().any(|message| {
            message.reply_to_message_id.as_deref() == Some("fixture-server-message-general-002")
                && message.mention_identity_ids.as_slice() == ["usr-test-bob"]
        }));
        assert!(scenario.server_channel_messages.iter().any(|message| {
            message.channel_id == "fixture-channel-atlas-ops-lab"
                && message.reply_to_message_id.as_deref() == Some("fixture-server-message-ops-001")
                && message.mention_identity_ids.as_slice() == ["usr-test-alice"]
        }));
    }

    #[test]
    fn rejects_production_seed_target() {
        let error = validate_seed_target(
            "postgres://hexrelay:pw@127.0.0.1:5432/hexrelay",
            "production",
        )
        .expect_err("production target rejected");

        assert!(error.to_string().contains("API_ENVIRONMENT=production"));
    }

    #[test]
    fn rejects_non_local_database_host() {
        let error = validate_seed_target(
            "postgres://hexrelay:pw@db.example.com:5432/hexrelay",
            "development",
        )
        .expect_err("remote db rejected");

        assert!(error.to_string().contains("non-local database host"));
    }

    #[test]
    fn rejects_non_development_environment() {
        let error = validate_seed_target("postgres://hexrelay:pw@127.0.0.1:5432/hexrelay", "test")
            .expect_err("non-development env rejected");

        assert!(error.to_string().contains("unsupported API_ENVIRONMENT"));
    }

    #[test]
    fn rejects_non_local_database_name() {
        let error = validate_seed_target(
            "postgres://hexrelay:pw@127.0.0.1:5432/customer_data",
            "development",
        )
        .expect_err("non-local db name rejected");

        assert!(error.to_string().contains("refusing to seed database"));
    }

    #[test]
    fn rejects_reset_against_non_loopback_host() {
        let error = validate_reset_target(
            "postgres://hexrelay:pw@postgres:5432/hexrelay",
            "development",
        )
        .expect_err("non-loopback reset rejected");

        assert!(error.to_string().contains("non-loopback database host"));
    }

    #[test]
    fn accepts_default_local_database_target() {
        validate_seed_target(
            "postgres://hexrelay:pw@127.0.0.1:5432/hexrelay",
            "development",
        )
        .expect("default local db accepted");
    }

    #[test]
    fn rejects_non_fixture_thread_ids() {
        let mut scenario = dm_basic();
        scenario.dm_threads[0].thread_id = "private-thread-1".to_string();

        let error = validate_scenario(&scenario).expect_err("non-fixture thread rejected");

        assert!(error.to_string().contains("fixture-compatible thread id"));
    }

    #[test]
    fn rejects_dm_messages_from_non_participants() {
        let mut scenario = dm_basic();
        scenario.identities.push(IdentityFixture {
            profile_id: "carol.pending".to_string(),
            identity_id: "usr-test-carol".to_string(),
            public_key: "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
                .to_string(),
            algorithm: "ed25519".to_string(),
        });
        scenario.dm_threads[0].messages[0].author_id = "usr-test-carol".to_string();

        let error = validate_scenario(&scenario).expect_err("non-participant author rejected");

        assert!(error.to_string().contains("not a thread participant"));
    }

    #[test]
    fn rejects_duplicate_dm_message_ids_across_threads() {
        let mut scenario = dm_basic();
        let mut extra_thread = scenario.dm_threads[0].clone();
        extra_thread.thread_id = "fixture-dm-extra".to_string();
        scenario.dm_threads.push(extra_thread);

        let error = validate_scenario(&scenario).expect_err("cross-thread duplicate rejected");

        assert!(error.to_string().contains("duplicate DM message"));
    }

    #[test]
    fn rejects_server_duplicate_channel_seq() {
        let mut scenario = server_chat();
        scenario.server_channel_messages[1].channel_seq = 1;

        let error = validate_scenario(&scenario).expect_err("duplicate channel seq rejected");

        assert!(error.to_string().contains("duplicates channel_seq"));
    }

    #[test]
    fn rejects_invite_uses_above_max() {
        let mut scenario = contacts_edge();
        scenario.invites.push(InviteFixture {
            invite_id: "fixture-invite-overused-server".to_string(),
            token_hash: "1".repeat(64),
            mode: "one_time".to_string(),
            server_id: "dev-server-alice".to_string(),
            expires_at: None,
            max_uses: Some(1),
            uses: 2,
            created_at: "2026-05-04T10:00:00Z".to_string(),
        });

        let error = validate_scenario(&scenario).expect_err("overused invite rejected");

        assert!(error.to_string().contains("uses cannot exceed max_uses"));
    }

    #[test]
    fn reset_requires_yes_confirmation() {
        let options = ResetCliOptions::parse(["--profile".to_string(), "dm-basic".to_string()])
            .expect("parse reset options");

        assert!(!options.yes);
    }

    #[test]
    fn reset_parses_yes_and_seed_options() {
        let options = ResetCliOptions::parse([
            "--yes".to_string(),
            "--profile".to_string(),
            "dm-basic".to_string(),
            "--json".to_string(),
        ])
        .expect("parse reset options");
        let seed_options = options.seed_options().expect("seed options exist");

        assert!(options.yes);
        assert_eq!(seed_options.profile, "dm-basic");
        assert!(seed_options.json);
    }

    #[test]
    fn reset_defaults_to_no_seed_profile() {
        let options = ResetCliOptions::parse(["--yes".to_string()]).expect("parse reset options");

        assert!(options.yes);
        assert_eq!(options.seed_options(), None);
    }

    #[tokio::test]
    async fn reset_refuses_without_yes_before_target_validation() {
        let options = ResetCliOptions::parse(["--profile".to_string(), "dm-basic".to_string()])
            .expect("parse reset options");

        let error = reset_local_database(
            "postgres://hexrelay:pw@db.example.com:5432/customer_data",
            &options,
        )
        .await
        .expect_err("reset without --yes rejected");

        assert!(error.to_string().contains("without --yes"));
    }

    fn assert_fixture_identity(scenario: &SeedScenario, profile_id: &str, identity_id: &str) {
        let identity = scenario
            .identities
            .iter()
            .find(|identity| identity.profile_id == profile_id)
            .expect("fixture identity exists");
        assert_eq!(identity.identity_id, identity_id);
    }

    fn assert_fixture_session(scenario: &SeedScenario, profile_id: &str, session_id: &str) {
        let session = scenario
            .sessions
            .iter()
            .find(|session| session.profile_id == profile_id)
            .expect("fixture session exists");
        assert_eq!(session.session_id, session_id);
    }

    fn assert_friend_request(
        scenario: &SeedScenario,
        request_id: &str,
        requester_identity_id: &str,
        target_identity_id: &str,
        status: &str,
    ) {
        let request = scenario
            .friend_requests
            .iter()
            .find(|request| request.request_id == request_id)
            .expect("fixture friend request exists");
        assert_eq!(request.requester_identity_id, requester_identity_id);
        assert_eq!(request.target_identity_id, target_identity_id);
        assert_eq!(request.status, status);
    }

    fn assert_dm_policy(scenario: &SeedScenario, identity_id: &str, inbound_policy: &str) {
        let policy = scenario
            .dm_policies
            .iter()
            .find(|policy| policy.identity_id == identity_id)
            .expect("fixture DM policy exists");
        assert_eq!(policy.inbound_policy, inbound_policy);
        assert_eq!(policy.offline_delivery_mode, "encrypted_envelope_catchup");
    }
}
