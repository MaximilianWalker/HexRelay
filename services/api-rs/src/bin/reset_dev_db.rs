use api_rs::{
    config::ApiConfig,
    db::connect_and_prepare,
    dev_seed::{
        format_seed_summary, reset_local_database, reset_usage, seed_profile,
        validate_seed_profile, DevSeedError, ResetCliOptions,
    },
};

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("[reset-dev-db] ERROR: {error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), DevSeedError> {
    let raw_args = std::env::args().skip(1).collect::<Vec<_>>();
    if raw_args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("{}", reset_usage());
        return Ok(());
    }

    let options = ResetCliOptions::parse(raw_args)?;
    let config = ApiConfig::from_env().map_err(DevSeedError::Config)?;
    let signing_key = config
        .session_signing_keys
        .get(&config.active_signing_key_id)
        .ok_or_else(|| {
            DevSeedError::Config(format!(
                "active signing key '{}' is missing from API_SESSION_SIGNING_KEYS",
                config.active_signing_key_id
            ))
        })?;

    let seed_options = options.seed_options();
    if let Some(seed_options) = seed_options.as_ref() {
        validate_seed_profile(seed_options)?;
    }

    reset_local_database(&config.database_url, &options).await?;
    let pool = connect_and_prepare(&config.database_url).await?;
    if let Some(seed_options) = seed_options.as_ref() {
        let summary = seed_profile(
            &pool,
            seed_options,
            &config.active_signing_key_id,
            signing_key,
        )
        .await?;

        if options.json {
            println!("{}", serde_json::to_string_pretty(&summary)?);
        } else {
            println!("[reset-dev-db] Local development database reset complete");
            println!("{}", format_seed_summary(&summary));
        }

        return Ok(());
    }

    if options.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "reset": true,
                "seeded": false
            }))?
        );
    } else {
        println!("[reset-dev-db] Local development database reset complete");
        println!("[reset-dev-db] No seed profile requested");
    }

    Ok(())
}
