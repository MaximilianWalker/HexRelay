use redis::aio::MultiplexedConnection;

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct ReplayEntry {
    pub cursor: u64,
    pub payload: String,
}

pub async fn persist_replay_entries<TCursor, F>(
    connection: &mut MultiplexedConnection,
    identities: &[String],
    client_payload: &str,
    replay_log_max_entries: usize,
    stream_head_key: fn(&str) -> String,
    replay_log_key: fn(&str) -> String,
    build_cursor: F,
) -> Result<Vec<TCursor>, redis::RedisError>
where
    F: Fn(&str, u64) -> TCursor,
{
    let mut cursors = Vec::with_capacity(identities.len());
    for identity_id in identities {
        let cursor = advance_stream_head(connection, stream_head_key, identity_id).await?;
        let replay_entry = serde_json::to_string(&ReplayEntry {
            cursor,
            payload: client_payload.to_string(),
        })
        .map_err(serialize_replay_error)?;

        let log_key = replay_log_key(identity_id);
        let _: () = redis::cmd("LPUSH")
            .arg(&log_key)
            .arg(replay_entry)
            .query_async(connection)
            .await?;
        let _: () = redis::cmd("LTRIM")
            .arg(&log_key)
            .arg(0)
            .arg((replay_log_max_entries - 1) as i64)
            .query_async(connection)
            .await?;

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
    stream_head_key: fn(&str) -> String,
    identity_id: &str,
) -> Result<u64, redis::RedisError> {
    redis::cmd("INCR")
        .arg(stream_head_key(identity_id))
        .query_async(connection)
        .await
}

fn serialize_replay_error(error: serde_json::Error) -> redis::RedisError {
    redis::RedisError::from((
        redis::ErrorKind::TypeError,
        "serialize replay entry",
        error.to_string(),
    ))
}
