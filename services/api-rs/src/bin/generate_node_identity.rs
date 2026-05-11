use api_rs::{
    config::{current_epoch_seconds, parse_i64_env},
    domain::node_identity::{
        generate_node_identity, node_identity_generate_usage, NodeIdentityGenerateCliOptions,
        DEFAULT_NODE_DESCRIPTOR_MAX_TTL_SECONDS,
    },
};

fn main() {
    if let Err(error) = run() {
        eprintln!("[generate-node-identity] ERROR: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let raw_args = std::env::args().skip(1).collect::<Vec<_>>();
    if raw_args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("{}", node_identity_generate_usage());
        return Ok(());
    }

    let mut options =
        NodeIdentityGenerateCliOptions::parse(raw_args).map_err(|error| error.to_string())?;
    let configured_max_ttl_seconds = parse_i64_env(
        "API_STATIC_PEER_DESCRIPTOR_MAX_TTL_SECONDS",
        DEFAULT_NODE_DESCRIPTOR_MAX_TTL_SECONDS,
    )?;
    let effective_max_ttl_seconds = options
        .max_ttl_seconds_override
        .unwrap_or(configured_max_ttl_seconds);
    if effective_max_ttl_seconds > configured_max_ttl_seconds {
        return Err(format!(
            "--max-ttl-seconds must be less than or equal to API_STATIC_PEER_DESCRIPTOR_MAX_TTL_SECONDS ({configured_max_ttl_seconds})"
        ));
    }
    options.generate_options.max_ttl_seconds = effective_max_ttl_seconds;
    if options.ttl_seconds_override.is_none()
        && options.generate_options.ttl_seconds > effective_max_ttl_seconds
    {
        options.generate_options.ttl_seconds = effective_max_ttl_seconds;
    }
    if options.generate_options.ttl_seconds > options.generate_options.max_ttl_seconds {
        return Err(format!(
            "--ttl-seconds must be less than or equal to the node descriptor TTL ceiling ({})",
            options.generate_options.max_ttl_seconds
        ));
    }

    let issued_at_epoch_seconds = current_epoch_seconds()?;
    let (output, _) = generate_node_identity(&options.generate_options, issued_at_epoch_seconds)
        .map_err(|error| error.to_string())?;

    if options.json_compact {
        println!(
            "{}",
            serde_json::to_string(&output)
                .map_err(|error| format!("failed to serialize generated node identity: {error}"))?
        );
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(&output)
                .map_err(|error| format!("failed to serialize generated node identity: {error}"))?
        );
    }

    Ok(())
}
