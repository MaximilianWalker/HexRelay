use ring::digest::{digest, SHA256};
use sqlx::{Executor, PgPool, Postgres, Transaction};

type Migration = (&'static str, &'static str);

const MIGRATIONS: &[Migration] = &[
    (
        "0001_friend_requests",
        include_str!("../migrations/0001_friend_requests.sql"),
    ),
    (
        "0002_friend_requests_transition_index",
        include_str!("../migrations/0002_friend_requests_transition_index.sql"),
    ),
    (
        "0003_sessions",
        include_str!("../migrations/0003_sessions.sql"),
    ),
    (
        "0004_identity_keys",
        include_str!("../migrations/0004_identity_keys.sql"),
    ),
    (
        "0005_auth_challenges",
        include_str!("../migrations/0005_auth_challenges.sql"),
    ),
    (
        "0006_invites",
        include_str!("../migrations/0006_invites.sql"),
    ),
    (
        "0007_invites_hash_backfill",
        include_str!("../migrations/0007_invites_hash_backfill.sql"),
    ),
    (
        "0008_rate_limit_counters",
        include_str!("../migrations/0008_rate_limit_counters.sql"),
    ),
    (
        "0009_relational_constraints",
        include_str!("../migrations/0009_relational_constraints.sql"),
    ),
];

pub async fn connect_and_prepare(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPool::connect(database_url).await?;

    run_migrations(&pool).await?;
    backfill_legacy_invite_tokens(&pool).await?;

    Ok(pool)
}

async fn backfill_legacy_invite_tokens(pool: &PgPool) -> Result<(), sqlx::Error> {
    const RUNTIME_BACKFILL_MARKER: &str = "0007_invites_hash_backfill_runtime_v1";

    let legacy_tokens = sqlx::query_scalar::<_, String>(
        "SELECT token FROM invites WHERE token !~ '^[0-9a-f]{64}$'",
    )
    .fetch_all(pool)
    .await?;

    let mut tx = pool.begin().await?;

    for token in legacy_tokens {
        let hashed = hex::encode(digest(&SHA256, token.as_bytes()).as_ref());
        sqlx::query("UPDATE invites SET token = $1 WHERE token = $2")
            .bind(hashed)
            .bind(token)
            .execute(&mut *tx)
            .await?;
    }

    sqlx::query(
        "INSERT INTO schema_migrations (version, checksum) VALUES ($1, $2) ON CONFLICT (version) DO NOTHING",
    )
    .bind(RUNTIME_BACKFILL_MARKER)
    .bind("runtime-backfill-v1")
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

async fn ensure_migration_table(tx: &mut Transaction<'_, Postgres>) -> Result<(), sqlx::Error> {
    tx.execute(
        "
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version TEXT PRIMARY KEY,
            checksum TEXT NOT NULL,
            applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        ",
    )
    .await?;

    tx.execute(
        "ALTER TABLE schema_migrations ADD COLUMN IF NOT EXISTS checksum TEXT NOT NULL DEFAULT ''",
    )
    .await?;

    Ok(())
}

async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    const MIGRATION_LOCK_KEY: i64 = 0x4845_5852_454c_4159;

    let mut tx = pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock($1)")
        .bind(MIGRATION_LOCK_KEY)
        .execute(&mut *tx)
        .await?;
    ensure_migration_table(&mut tx).await?;
    run_migrations_inner(&mut tx).await?;
    tx.commit().await
}

async fn run_migrations_inner(tx: &mut Transaction<'_, Postgres>) -> Result<(), sqlx::Error> {
    for (version, sql) in MIGRATIONS {
        let checksum = format!("{:016x}", seahash::hash(sql.as_bytes()));

        let existing_checksum = sqlx::query_scalar::<_, String>(
            "SELECT checksum FROM schema_migrations WHERE version = $1",
        )
        .bind(*version)
        .fetch_optional(&mut **tx)
        .await?;

        if let Some(existing_checksum) = existing_checksum {
            if existing_checksum != checksum {
                return Err(sqlx::Error::Protocol(format!(
                    "migration checksum mismatch for version {version}"
                )));
            }

            continue;
        }

        // Migration 0007 is intentionally backfilled by runtime code to avoid
        // requiring DB extension install privileges on startup.
        if *version != "0007_invites_hash_backfill" {
            tx.execute(*sql).await?;
        }
        sqlx::query("INSERT INTO schema_migrations (version, checksum) VALUES ($1, $2)")
            .bind(*version)
            .bind(checksum)
            .execute(&mut **tx)
            .await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        env,
        time::{SystemTime, UNIX_EPOCH},
    };

    use ring::digest::{digest, SHA256};

    use super::{backfill_legacy_invite_tokens, connect_and_prepare, run_migrations};

    async fn prepare_test_pool() -> Option<PgPool> {
        let url = match env::var("API_DATABASE_URL") {
            Ok(value) if !value.trim().is_empty() => value,
            _ => {
                assert!(
                    env::var("CI").is_err(),
                    "API_DATABASE_URL must be set in CI"
                );
                return None;
            }
        };

        match connect_and_prepare(&url).await {
            Ok(pool) => Some(pool),
            Err(error) => {
                assert!(
                    env::var("CI").is_err(),
                    "failed to prepare DB in CI: {error}"
                );
                None
            }
        }
    }

    use sqlx::{Connection, Executor, PgConnection, PgPool};

    fn split_database_url(url: &str) -> Option<(&str, &str)> {
        url.rsplit_once('/')
    }

    fn temporary_db_name() -> String {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|value| value.as_millis())
            .unwrap_or(0);
        format!("hexrelay_ci_bootstrap_{}_{}", std::process::id(), millis)
    }

    async fn create_temp_database(admin_url: &str, name: &str) -> Result<(), sqlx::Error> {
        let mut connection = PgConnection::connect(admin_url).await?;
        connection
            .execute(format!("CREATE DATABASE \"{name}\";").as_str())
            .await?;
        connection.close().await
    }

    async fn drop_temp_database(admin_url: &str, name: &str) -> Result<(), sqlx::Error> {
        let mut connection = PgConnection::connect(admin_url).await?;
        connection
            .execute(format!("DROP DATABASE IF EXISTS \"{name}\" WITH (FORCE);").as_str())
            .await?;
        connection.close().await
    }

    #[tokio::test]
    async fn migration_checksum_mismatch_is_detected_and_lock_is_released() {
        let Some(pool) = prepare_test_pool().await else {
            return;
        };

        let update = sqlx::query("UPDATE schema_migrations SET checksum = $1 WHERE version = $2")
            .bind("force-mismatch")
            .bind("0001_friend_requests")
            .execute(&pool)
            .await;

        if update.is_err() {
            assert!(
                env::var("CI").is_err(),
                "failed to prepare checksum mismatch setup in CI"
            );
            return;
        }

        let mismatch = run_migrations(&pool).await;
        assert!(mismatch.is_err());

        let message = mismatch
            .err()
            .map(|value| value.to_string())
            .unwrap_or_default();
        assert!(message.contains("migration checksum mismatch"));

        sqlx::query("UPDATE schema_migrations SET checksum = $1 WHERE version = $2")
            .bind(format!(
                "{:016x}",
                seahash::hash(include_str!("../migrations/0001_friend_requests.sql").as_bytes())
            ))
            .bind("0001_friend_requests")
            .execute(&pool)
            .await
            .expect("restore checksum");

        run_migrations(&pool)
            .await
            .expect("lock should be released after mismatch");
    }

    #[tokio::test]
    async fn invite_backfill_hashes_legacy_plaintext_tokens() {
        let Some(pool) = prepare_test_pool().await else {
            return;
        };

        let token_suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let plaintext_token = format!("legacy-token-backfill-test-{token_suffix}");
        sqlx::query("DELETE FROM schema_migrations WHERE version = $1")
            .bind("0007_invites_hash_backfill_runtime_v1")
            .execute(&pool)
            .await
            .expect("clear runtime backfill marker");

        let expected_hash = hex::encode(digest(&SHA256, plaintext_token.as_bytes()).as_ref());
        sqlx::query("DELETE FROM invites WHERE token = $1 OR token = $2")
            .bind(&plaintext_token)
            .bind(&expected_hash)
            .execute(&pool)
            .await
            .expect("clear legacy and hashed invite test rows");

        let insert = sqlx::query(
            "
            INSERT INTO invites (token, mode, node_fingerprint, uses)
            VALUES ($1, 'one_time', 'hexrelay-local-fingerprint', 0)
            ON CONFLICT (token) DO NOTHING
            ",
        )
        .bind(&plaintext_token)
        .execute(&pool)
        .await;

        if insert.is_err() {
            assert!(
                env::var("CI").is_err(),
                "failed to seed legacy invite token in CI"
            );
            return;
        }

        backfill_legacy_invite_tokens(&pool)
            .await
            .expect("run invite hash backfill");

        let legacy_exists =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM invites WHERE token = $1")
                .bind(plaintext_token)
                .fetch_one(&pool)
                .await
                .expect("count plaintext invite token");
        assert_eq!(legacy_exists, 0);

        let hashed_exists =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM invites WHERE token = $1")
                .bind(expected_hash)
                .fetch_one(&pool)
                .await
                .expect("count hashed invite token");
        assert_eq!(hashed_exists, 1);
    }

    #[tokio::test]
    async fn concurrent_prepare_on_fresh_database_is_serialized() {
        let url = match env::var("API_DATABASE_URL") {
            Ok(value) if !value.trim().is_empty() => value,
            _ => {
                assert!(
                    env::var("CI").is_err(),
                    "API_DATABASE_URL must be set in CI"
                );
                return;
            }
        };

        let Some((url_prefix, _)) = split_database_url(&url) else {
            assert!(env::var("CI").is_err(), "invalid API_DATABASE_URL in CI");
            return;
        };

        let admin_url = format!("{url_prefix}/postgres");
        let db_name = temporary_db_name();
        let test_url = format!("{url_prefix}/{db_name}");

        if let Err(error) = create_temp_database(&admin_url, &db_name).await {
            assert!(
                env::var("CI").is_err(),
                "failed to create temporary database in CI: {error}"
            );
            return;
        }

        let (first, second) = tokio::join!(
            connect_and_prepare(&test_url),
            connect_and_prepare(&test_url)
        );

        if let Ok(pool) = first.as_ref() {
            pool.close().await;
        }
        if let Ok(pool) = second.as_ref() {
            pool.close().await;
        }

        drop_temp_database(&admin_url, &db_name)
            .await
            .expect("drop temporary database");

        assert!(first.is_ok(), "first prepare failed: {first:?}");
        assert!(second.is_ok(), "second prepare failed: {second:?}");
    }
}
