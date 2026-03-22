use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize)]
struct PresenceSnapshot {
    status: String,
}

pub async fn list_presence_statuses(
    client: &redis::Client,
    identity_ids: &[String],
) -> Result<HashMap<String, String>, redis::RedisError> {
    if identity_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let keys = identity_ids
        .iter()
        .map(|identity_id| format!("presence:v1:snapshot:{identity_id}"))
        .collect::<Vec<_>>();

    let mut connection = client.get_multiplexed_tokio_connection().await?;
    let values: Vec<Option<String>> = redis::cmd("MGET")
        .arg(keys)
        .query_async(&mut connection)
        .await?;

    let mut statuses = HashMap::with_capacity(identity_ids.len());
    for (identity_id, raw_value) in identity_ids.iter().zip(values.into_iter()) {
        let Some(raw_value) = raw_value else {
            continue;
        };

        let Ok(snapshot) = serde_json::from_str::<PresenceSnapshot>(&raw_value) else {
            continue;
        };

        statuses.insert(identity_id.clone(), snapshot.status);
    }

    Ok(statuses)
}
