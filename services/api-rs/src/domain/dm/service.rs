use crate::{
    domain::dm::validation::DM_OFFLINE_DELIVERY_MODE,
    infra::db::repos::{dm_repo, friends_repo, servers_repo},
    models::{DmPolicy, DmPolicyUpdate, DmProfileDeviceRecord},
    shared::errors::{internal_error, ApiResult},
    state::AppState,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DmInteractionPolicyDecision {
    Allowed,
    BlockedFriendsOnly,
    BlockedSameServer,
    BlockedUnknown,
}

pub fn default_dm_policy() -> DmPolicy {
    DmPolicy {
        inbound_policy: "friends_only".to_string(),
        offline_delivery_mode: DM_OFFLINE_DELIVERY_MODE.to_string(),
    }
}

pub fn dm_policy_from_update(payload: &DmPolicyUpdate) -> DmPolicy {
    DmPolicy {
        inbound_policy: payload.inbound_policy.trim().to_string(),
        offline_delivery_mode: DM_OFFLINE_DELIVERY_MODE.to_string(),
    }
}

pub async fn current_dm_policy(state: &AppState, identity_id: &str) -> ApiResult<DmPolicy> {
    if let Some(pool) = state.db_pool.as_ref() {
        return dm_repo::get_dm_policy(pool, identity_id)
            .await
            .map_err(|_| internal_error("storage_unavailable", "failed to load dm policy"))
            .map(|policy| policy.unwrap_or_else(default_dm_policy));
    }

    Ok(state
        .dm_policies
        .read()
        .expect("acquire dm policy read lock")
        .get(identity_id)
        .cloned()
        .unwrap_or_else(default_dm_policy))
}

pub async fn load_dm_profile_device(
    state: &AppState,
    identity_id: &str,
    device_id: &str,
) -> ApiResult<Option<DmProfileDeviceRecord>> {
    if let Some(pool) = state.db_pool.as_ref() {
        return dm_repo::get_dm_profile_device(pool, identity_id, device_id)
            .await
            .map_err(|_| internal_error("storage_unavailable", "failed to load profile device"));
    }

    Ok(state
        .dm_profile_devices
        .read()
        .expect("acquire dm profile devices read lock")
        .get(identity_id)
        .and_then(|devices| devices.get(device_id))
        .cloned())
}

pub async fn dm_interaction_policy_decision(
    state: &AppState,
    sender_identity_id: &str,
    recipient_identity_id: &str,
) -> ApiResult<DmInteractionPolicyDecision> {
    let policy = current_dm_policy(state, recipient_identity_id).await?;

    match policy.inbound_policy.as_str() {
        "anyone" => Ok(DmInteractionPolicyDecision::Allowed),
        "friends_only" => {
            if is_friend(state, sender_identity_id, recipient_identity_id).await? {
                Ok(DmInteractionPolicyDecision::Allowed)
            } else {
                Ok(DmInteractionPolicyDecision::BlockedFriendsOnly)
            }
        }
        "same_server" => {
            if let Some(pool) = state.db_pool.as_ref() {
                if servers_repo::identities_share_server(
                    pool,
                    sender_identity_id,
                    recipient_identity_id,
                )
                .await
                .map_err(|_| {
                    internal_error(
                        "storage_unavailable",
                        "failed to evaluate shared-server DM policy",
                    )
                })? {
                    Ok(DmInteractionPolicyDecision::Allowed)
                } else {
                    Ok(DmInteractionPolicyDecision::BlockedSameServer)
                }
            } else {
                Ok(DmInteractionPolicyDecision::BlockedSameServer)
            }
        }
        _ => Ok(DmInteractionPolicyDecision::BlockedUnknown),
    }
}

async fn is_friend(state: &AppState, a: &str, b: &str) -> ApiResult<bool> {
    if let Some(pool) = state.db_pool.as_ref() {
        return friends_repo::are_friends(pool, a, b).await.map_err(|_| {
            internal_error(
                "friendship_lookup_failed",
                "failed to evaluate friendship state for DM policy",
            )
        });
    }

    Ok(state
        .friend_requests
        .read()
        .expect("acquire friend request read lock")
        .values()
        .any(|record| {
            record.status == "accepted"
                && ((record.requester_identity_id == a && record.target_identity_id == b)
                    || (record.requester_identity_id == b && record.target_identity_id == a))
        }))
}

#[cfg(test)]
mod tests {
    use crate::models::FriendRequestRecord;

    use super::*;

    #[test]
    fn default_policy_is_friends_only_with_encrypted_catch_up() {
        let policy = default_dm_policy();

        assert_eq!(policy.inbound_policy, "friends_only");
        assert_eq!(policy.offline_delivery_mode, DM_OFFLINE_DELIVERY_MODE);
    }

    #[test]
    fn policy_update_normalizes_inbound_policy_and_preserves_delivery_mode() {
        let policy = dm_policy_from_update(&DmPolicyUpdate {
            inbound_policy: " anyone ".to_string(),
        });

        assert_eq!(policy.inbound_policy, "anyone");
        assert_eq!(policy.offline_delivery_mode, DM_OFFLINE_DELIVERY_MODE);
    }

    #[tokio::test]
    async fn friends_only_policy_allows_accepted_friend() {
        let state = AppState::default();
        state
            .friend_requests
            .write()
            .expect("acquire friend request write lock")
            .insert(
                "friend-1".to_string(),
                FriendRequestRecord {
                    request_id: "friend-1".to_string(),
                    requester_identity_id: "alice".to_string(),
                    target_identity_id: "bob".to_string(),
                    status: "accepted".to_string(),
                    created_at: "2026-05-19T00:00:00Z".to_string(),
                },
            );

        let decision = match dm_interaction_policy_decision(&state, "alice", "bob").await {
            Ok(decision) => decision,
            Err(_) => panic!("evaluate DM policy"),
        };

        assert_eq!(decision, DmInteractionPolicyDecision::Allowed);
    }

    #[tokio::test]
    async fn same_server_policy_blocks_without_server_context() {
        let state = AppState::default();
        state
            .dm_policies
            .write()
            .expect("acquire dm policy write lock")
            .insert(
                "bob".to_string(),
                DmPolicy {
                    inbound_policy: "same_server".to_string(),
                    offline_delivery_mode: DM_OFFLINE_DELIVERY_MODE.to_string(),
                },
            );

        let decision = match dm_interaction_policy_decision(&state, "alice", "bob").await {
            Ok(decision) => decision,
            Err(_) => panic!("evaluate DM policy"),
        };

        assert_eq!(decision, DmInteractionPolicyDecision::BlockedSameServer);
    }
}
