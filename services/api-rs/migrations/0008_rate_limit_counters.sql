CREATE TABLE IF NOT EXISTS rate_limit_counters (
    scope TEXT NOT NULL,
    limiter_key TEXT NOT NULL,
    window_start BIGINT NOT NULL,
    count INTEGER NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (scope, limiter_key, window_start)
);

CREATE INDEX IF NOT EXISTS idx_rate_limit_counters_window_start
    ON rate_limit_counters (window_start);
