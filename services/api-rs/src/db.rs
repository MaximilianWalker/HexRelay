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
            applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        ",
    )
    .await?;

    Ok(())
}

async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    for (version, sql) in MIGRATIONS {
        let already_applied = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM schema_migrations WHERE version = $1",
        )
        .bind(*version)
        .fetch_one(pool)
        .await?
            > 0;

        if already_applied {
            continue;
        }

        let mut tx = pool.begin().await?;
        tx.execute(*sql).await?;
        sqlx::query("INSERT INTO schema_migrations (version) VALUES ($1)")
            .bind(*version)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
    }

    Ok(())
}
