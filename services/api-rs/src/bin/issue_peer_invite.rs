use api_rs::{
    config::{parse_i64_env, parse_required_local_server_identity},
    domain::peer_invites::{
        current_epoch_seconds, issue_peer_invite, peer_invite_issue_usage,
        PeerInviteIssueCliOptions, DEFAULT_PEER_INVITE_MAX_TTL_SECONDS,
    },
};

fn main() {
    if let Err(error) = run() {
        eprintln!("[issue-peer-invite] ERROR: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let raw_args = std::env::args().skip(1).collect::<Vec<_>>();
    if raw_args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("{}", peer_invite_issue_usage());
        return Ok(());
    }

    let mut options =
        PeerInviteIssueCliOptions::parse(raw_args).map_err(|error| error.to_string())?;
    let configured_max_ttl_seconds = parse_i64_env(
        "API_STATIC_PEER_DESCRIPTOR_MAX_TTL_SECONDS",
        DEFAULT_PEER_INVITE_MAX_TTL_SECONDS,
    )?;
    let effective_max_ttl_seconds = options
        .max_ttl_seconds_override
        .unwrap_or(configured_max_ttl_seconds);
    if effective_max_ttl_seconds > configured_max_ttl_seconds {
        return Err(format!(
            "--max-ttl-seconds must be less than or equal to API_STATIC_PEER_DESCRIPTOR_MAX_TTL_SECONDS ({configured_max_ttl_seconds})"
        ));
    }
    options.issue_options.max_ttl_seconds = effective_max_ttl_seconds;
    if options.issue_options.ttl_seconds > effective_max_ttl_seconds {
        return Err(format!(
            "--ttl-seconds must be less than or equal to the peer invite TTL ceiling ({effective_max_ttl_seconds})"
        ));
    }

    let expected_server_id = std::env::var("API_SERVER_ID")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let local_identity = parse_required_local_server_identity(
        "API_LOCAL_SERVER_DESCRIPTOR_JSON",
        "API_LOCAL_SERVER_PRIVATE_KEY_PKCS8_BASE64",
        configured_max_ttl_seconds,
        expected_server_id.as_deref(),
    )?;
    let issued_at_epoch_seconds = current_epoch_seconds().map_err(|error| error.to_string())?;
    let envelope = issue_peer_invite(
        &local_identity,
        &options.issue_options,
        issued_at_epoch_seconds,
    )
    .map_err(|error| error.to_string())?;

    if options.json_compact {
        println!(
            "{}",
            serde_json::to_string(&envelope)
                .map_err(|error| format!("failed to serialize peer invite envelope: {error}"))?
        );
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(&envelope)
                .map_err(|error| format!("failed to serialize peer invite envelope: {error}"))?
        );
    }

    Ok(())
}
