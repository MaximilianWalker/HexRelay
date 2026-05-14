use redis::aio::MultiplexedConnection;

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct ReplayEntry {
    pub cursor: u64,
    pub payload: String,
}

#[derive(Clone, Copy)]
pub struct ReplayRetention {
    max_entries: usize,
    key_ttl_seconds: Option<u64>,
}

impl ReplayRetention {
    pub fn with_key_ttl(max_entries: usize, ttl_seconds: u64) -> Result<Self, redis::RedisError> {
        validate_replay_key_ttl(ttl_seconds)?;
        Ok(Self {
            max_entries,
            key_ttl_seconds: Some(ttl_seconds),
        })
    }
}

pub async fn persist_replay_entries_with_retention<TCursor, F>(
    connection: &mut MultiplexedConnection,
    identities: &[String],
    client_payload: &str,
    retention: ReplayRetention,
    stream_head_key: fn(&str) -> String,
    replay_log_key: fn(&str) -> String,
    build_cursor: F,
) -> Result<Vec<TCursor>, redis::RedisError>
where
    F: Fn(&str, u64) -> TCursor,
{
    let trim_end = replay_log_trim_end(retention.max_entries)?;
    let mut cursors = Vec::with_capacity(identities.len());
    for identity_id in identities {
        let head_key = stream_head_key(identity_id);
        let cursor = advance_stream_head(connection, &head_key).await?;
        let replay_entry = serde_json::to_string(&ReplayEntry {
            cursor,
            payload: client_payload.to_string(),
        })
        .map_err(serialize_replay_error)?;

        let log_key = replay_log_key(identity_id);
        let mut pipe = redis::pipe();
        pipe.cmd("LPUSH")
            .arg(&log_key)
            .arg(replay_entry)
            .ignore()
            .cmd("LTRIM")
            .arg(&log_key)
            .arg(0)
            .arg(trim_end)
            .ignore();
        if let Some(ttl_seconds) = retention.key_ttl_seconds {
            pipe.cmd("EXPIRE")
                .arg(&head_key)
                .arg(ttl_seconds)
                .ignore()
                .cmd("EXPIRE")
                .arg(&log_key)
                .arg(ttl_seconds)
                .ignore();
        }
        let _: () = pipe.query_async(connection).await?;

        cursors.push(build_cursor(identity_id, cursor));
    }

    Ok(cursors)
}

pub async fn list_replay_entries(
    connection: &mut MultiplexedConnection,
    replay_log_key: fn(&str) -> String,
    identity_id: &str,
) -> Result<Vec<ReplayEntry>, redis::RedisError> {
    let values: Vec<String> = redis::cmd("LRANGE")
        .arg(replay_log_key(identity_id))
        .arg(0)
        .arg(-1)
        .query_async(connection)
        .await?;

    let mut entries = values
        .into_iter()
        .filter_map(|value| serde_json::from_str::<ReplayEntry>(&value).ok())
        .collect::<Vec<_>>();
    entries.reverse();
    Ok(entries)
}

pub async fn get_device_cursor(
    connection: &mut MultiplexedConnection,
    device_cursor_key: fn(&str, &str) -> String,
    identity_id: &str,
    device_id: &str,
) -> Result<u64, redis::RedisError> {
    redis::cmd("GET")
        .arg(device_cursor_key(identity_id, device_id))
        .query_async::<Option<u64>>(connection)
        .await
        .map(|value| value.unwrap_or(0))
}

pub async fn set_device_cursor(
    connection: &mut MultiplexedConnection,
    device_cursor_key: fn(&str, &str) -> String,
    ttl_seconds: u64,
    identity_id: &str,
    device_id: &str,
    cursor: u64,
) -> Result<(), redis::RedisError> {
    let _: () = redis::cmd("EVAL")
        .arg(
            r#"
            local key = KEYS[1]
            local incoming = tonumber(ARGV[1])
            local ttl = tonumber(ARGV[2])
            local current = tonumber(redis.call('GET', key) or '0')
            if incoming > current then
              current = incoming
            end
            redis.call('SET', key, current, 'EX', ttl)
            return current
            "#,
        )
        .arg(1)
        .arg(device_cursor_key(identity_id, device_id))
        .arg(cursor)
        .arg(ttl_seconds)
        .query_async(connection)
        .await?;
    Ok(())
}

async fn advance_stream_head(
    connection: &mut MultiplexedConnection,
    stream_head_key: &str,
) -> Result<u64, redis::RedisError> {
    redis::cmd("INCR")
        .arg(stream_head_key)
        .query_async(connection)
        .await
}

fn validate_replay_key_ttl(ttl_seconds: u64) -> Result<(), redis::RedisError> {
    if ttl_seconds == 0 {
        return Err(redis::RedisError::from((
            redis::ErrorKind::TypeError,
            "replay_key_ttl_seconds must be greater than 0",
        )));
    }
    Ok(())
}

fn replay_log_trim_end(replay_log_max_entries: usize) -> Result<i64, redis::RedisError> {
    replay_log_max_entries
        .checked_sub(1)
        .map(|value| value as i64)
        .ok_or_else(|| {
            redis::RedisError::from((
                redis::ErrorKind::TypeError,
                "replay_log_max_entries must be greater than 0",
            ))
        })
}

fn serialize_replay_error(error: serde_json::Error) -> redis::RedisError {
    redis::RedisError::from((
        redis::ErrorKind::TypeError,
        "serialize replay entry",
        error.to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::{replay_log_trim_end, validate_replay_key_ttl};

    #[test]
    fn replay_log_trim_end_rejects_zero_entries() {
        assert!(replay_log_trim_end(0).is_err());
    }

    #[test]
    fn replay_log_trim_end_uses_last_valid_index() {
        assert_eq!(replay_log_trim_end(3).expect("trim end"), 2);
    }

    #[test]
    fn replay_key_ttl_rejects_zero_seconds() {
        assert!(validate_replay_key_ttl(0).is_err());
    }

    #[test]
    fn replay_key_ttl_accepts_positive_seconds() {
        assert!(validate_replay_key_ttl(1).is_ok());
    }
}
