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

    Ok(parse_presence_statuses(identity_ids, values))
}

fn parse_presence_statuses(
    identity_ids: &[String],
    values: Vec<Option<String>>,
) -> HashMap<String, String> {
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

    statuses
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_presence_statuses_ignores_missing_and_malformed_entries() {
        let identities = vec![
            "usr-a".to_string(),
            "usr-b".to_string(),
            "usr-c".to_string(),
        ];
        let values = vec![
            Some(r#"{"status":"online"}"#.to_string()),
            None,
            Some("not-json".to_string()),
        ];

        let statuses = parse_presence_statuses(&identities, values);

        assert_eq!(statuses.len(), 1);
        assert_eq!(statuses.get("usr-a"), Some(&"online".to_string()));
        assert!(!statuses.contains_key("usr-b"));
        assert!(!statuses.contains_key("usr-c"));
    }
}
