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
                sqlx::query("SELECT pg_advisory_unlock(9176412301)")
                    .execute(pool)
                    .await?;
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

    sqlx::query("SELECT pg_advisory_unlock(9176412301)")
        .execute(pool)
        .await?;

    Ok(())
}
