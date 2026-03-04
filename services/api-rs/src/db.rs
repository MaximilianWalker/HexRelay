use sqlx::{Executor, PgPool};

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
];

pub async fn connect_and_prepare(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPool::connect(database_url).await?;

    ensure_migration_table(&pool).await?;
    run_migrations(&pool).await?;

    Ok(pool)
}

async fn ensure_migration_table(pool: &PgPool) -> Result<(), sqlx::Error> {
    pool.execute(
        "
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version TEXT PRIMARY KEY,
            checksum TEXT NOT NULL,
            applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        ",
    )
    .await?;

    pool.execute(
        "ALTER TABLE schema_migrations ADD COLUMN IF NOT EXISTS checksum TEXT NOT NULL DEFAULT ''",
    )
    .await?;

    Ok(())
}

async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT pg_advisory_lock(9176412301)")
        .execute(pool)
        .await?;

    let result = run_migrations_inner(pool).await;

    let unlock_result = sqlx::query("SELECT pg_advisory_unlock(9176412301)")
        .execute(pool)
        .await;

    match (result, unlock_result) {
        (Err(err), _) => Err(err),
        (Ok(()), Ok(_)) => Ok(()),
        (Ok(()), Err(err)) => Err(err),
    }
}

async fn run_migrations_inner(pool: &PgPool) -> Result<(), sqlx::Error> {
    for (version, sql) in MIGRATIONS {
        let checksum = format!("{:016x}", seahash::hash(sql.as_bytes()));

        let existing_checksum = sqlx::query_scalar::<_, Option<String>>(
            "SELECT checksum FROM schema_migrations WHERE version = $1",
        )
        .bind(*version)
        .fetch_one(pool)
        .await?;

        if let Some(existing_checksum) = existing_checksum {
            if existing_checksum != checksum {
                return Err(sqlx::Error::Protocol(format!(
                    "migration checksum mismatch for version {version}"
                )));
            }

            continue;
        }

        let mut tx = pool.begin().await?;
        tx.execute(*sql).await?;
        sqlx::query("INSERT INTO schema_migrations (version, checksum) VALUES ($1, $2)")
            .bind(*version)
            .bind(checksum)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::{connect_and_prepare, run_migrations};

    async fn prepare_test_pool() -> Option<PgPool> {
        let url = match env::var("API_DATABASE_URL") {
            Ok(value) if !value.trim().is_empty() => value,
            _ => return None,
        };

        connect_and_prepare(&url).await.ok()
    }

    use sqlx::PgPool;

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
}
