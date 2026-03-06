use crate::{
    models::FriendRequestRecord,
    shared::errors::{unauthorized, ApiResult},
};

#[cfg(test)]
use crate::shared::errors::conflict;

#[derive(Clone, Copy)]
pub enum ActorRole {
    Requester,
    Target,
}

#[cfg(test)]
pub fn apply_friend_request_transition(
    request: &mut FriendRequestRecord,
    next_status: &str,
    actor_identity: &str,
    actor_role: ActorRole,
) -> ApiResult<()> {
    assert_actor_can_transition(request, actor_identity, actor_role)?;

    if request.status == next_status {
        return Ok(());
    }

    if request.status != "pending" {
        return Err(conflict(
            "transition_invalid",
            "friend request transition is not allowed from current state",
        ));
    }

    request.status = next_status.to_string();
    Ok(())
}

pub fn assert_actor_can_transition(
    request: &FriendRequestRecord,
    actor_identity: &str,
    actor_role: ActorRole,
) -> ApiResult<()> {
    let allowed = match actor_role {
        ActorRole::Requester => request.requester_identity_id == actor_identity,
        ActorRole::Target => request.target_identity_id == actor_identity,
    };

    if !allowed {
        return Err(unauthorized(
            "identity_invalid",
            "friend request cannot be mutated by this session",
        ));
    }

    Ok(())
}
