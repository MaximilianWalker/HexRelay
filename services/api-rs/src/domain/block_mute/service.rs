use crate::{shared::errors::ApiResult, state::AppState};

/// Returns true if `blocker` has blocked `target`.
pub fn is_blocked(state: &AppState, blocker: &str, target: &str) -> ApiResult<bool> {
    Ok(state
        .blocked_users
        .read()
        .expect("acquire blocked_users read lock")
        .get(blocker)
        .map(|blocked| blocked.contains_key(target))
        .unwrap_or(false))
}

/// Returns true if either party has blocked the other.
pub fn is_blocked_bidirectional(state: &AppState, a: &str, b: &str) -> ApiResult<bool> {
    let guard = state
        .blocked_users
        .read()
        .expect("acquire blocked_users read lock");

    let a_blocked_b = guard
        .get(a)
        .map(|blocked| blocked.contains_key(b))
        .unwrap_or(false);

    if a_blocked_b {
        return Ok(true);
    }

    let b_blocked_a = guard
        .get(b)
        .map(|blocked| blocked.contains_key(a))
        .unwrap_or(false);

    Ok(b_blocked_a)
}

/// Returns true if `muter` has muted `target`.
pub fn is_muted(state: &AppState, muter: &str, target: &str) -> ApiResult<bool> {
    Ok(state
        .muted_users
        .read()
        .expect("acquire muted_users read lock")
        .get(muter)
        .map(|muted| muted.contains_key(target))
        .unwrap_or(false))
}
