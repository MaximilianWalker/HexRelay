CREATE INDEX IF NOT EXISTS friend_requests_target_status_created_idx
ON friend_requests (target_identity_id, status, created_at DESC);

CREATE INDEX IF NOT EXISTS friend_requests_requester_status_created_idx
ON friend_requests (requester_identity_id, status, created_at DESC);
