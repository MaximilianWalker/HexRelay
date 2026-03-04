use sqlx::{Executor, PgPool};

pub async fn connect_and_prepare(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPool::connect(database_url).await?;

    pool.execute(
        "
        CREATE TABLE IF NOT EXISTS friend_requests (
            request_id TEXT PRIMARY KEY,
            requester_identity_id TEXT NOT NULL,
            target_identity_id TEXT NOT NULL,
            status TEXT NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            CHECK (status IN ('pending', 'accepted', 'declined', 'cancelled')),
            CHECK (requester_identity_id <> target_identity_id)
        );
        ",
    )
    .await?;

    pool.execute(
        "
        CREATE UNIQUE INDEX IF NOT EXISTS friend_requests_unique_pending_pair
        ON friend_requests (requester_identity_id, target_identity_id)
        WHERE status = 'pending';
        ",
    )
    .await?;

    Ok(pool)
}
