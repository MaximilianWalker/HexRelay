use api_rs::{
    config::ApiConfig,
    db::connect_and_prepare,
    dev_seed::{
        assert_safe_seed_target, format_seed_summary, seed_profile, seed_usage, DevSeedError,
        SeedCliOptions,
    },
};

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("[seed] ERROR: {error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), DevSeedError> {
    let raw_args = std::env::args().skip(1).collect::<Vec<_>>();
    if raw_args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("{}", seed_usage());
        return Ok(());
    }

    let options = SeedCliOptions::parse(raw_args)?;
    let config = ApiConfig::from_env().map_err(DevSeedError::Config)?;
    assert_safe_seed_target(&config.database_url)?;

    let signing_key = config
        .session_signing_keys
        .get(&config.active_signing_key_id)
        .ok_or_else(|| {
            DevSeedError::Config(format!(
                "active signing key '{}' is missing from API_SESSION_SIGNING_KEYS",
                config.active_signing_key_id
            ))
        })?;

    let pool = connect_and_prepare(&config.database_url).await?;
    let summary = seed_profile(&pool, &options, &config.active_signing_key_id, signing_key).await?;

    if options.json {
        println!("{}", serde_json::to_string_pretty(&summary)?);
    } else {
        println!("{}", format_seed_summary(&summary));
    }

    Ok(())
}
