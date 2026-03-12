use axum::{
    extract::{Path, Query},
    http::StatusCode,
    Json,
};

use crate::{
    models::{
        ApiError, DmMessagePage, DmMessageRecord, DmThreadListQuery, DmThreadMessageListQuery,
        DmThreadPage, DmThreadSummary,
    },
    shared::errors::{bad_request, ApiResult},
    transport::http::middleware::auth::AuthSession,
};

const DEFAULT_PAGE_LIMIT: usize = 20;
const MAX_PAGE_LIMIT: usize = 100;

pub async fn list_dm_threads(
    _auth: AuthSession,
    Query(query): Query<DmThreadListQuery>,
) -> ApiResult<Json<DmThreadPage>> {
    let limit = parse_limit(query.limit)?;
    let mut items = dm_thread_fixtures();

    if query.unread_only.unwrap_or(false) {
        items.retain(|item| item.unread > 0);
    }

    let start = if let Some(cursor) = query.cursor {
        items
            .iter()
            .position(|item| item.thread_id == cursor)
            .map(|idx| idx + 1)
            .ok_or_else(|| bad_request("cursor_invalid", "unknown dm thread cursor"))?
    } else {
        0
    };

    let page_items = items
        .iter()
        .skip(start)
        .take(limit)
        .cloned()
        .collect::<Vec<_>>();
    let has_more = start + page_items.len() < items.len();
    let next_cursor = if has_more {
        page_items.last().map(|item| item.thread_id.clone())
    } else {
        None
    };

    Ok(Json(DmThreadPage {
        items: page_items,
        next_cursor,
    }))
}

pub async fn list_dm_thread_messages(
    _auth: AuthSession,
    Path(thread_id): Path<String>,
    Query(query): Query<DmThreadMessageListQuery>,
) -> ApiResult<Json<DmMessagePage>> {
    let limit = parse_limit(query.limit)?;
    let cursor = parse_message_cursor(query.cursor)?;

    let mut items = dm_message_fixtures(&thread_id).ok_or({
        (
            StatusCode::NOT_FOUND,
            Json(ApiError {
                code: "thread_not_found",
                message: "dm thread was not found",
            }),
        )
    })?;

    if let Some(cursor) = cursor {
        items.retain(|item| item.seq < cursor);
    }

    let page_items = items.iter().take(limit).cloned().collect::<Vec<_>>();
    let has_more = page_items.len() < items.len();
    let next_cursor = if has_more {
        page_items.last().map(|item| item.seq.to_string())
    } else {
        None
    };

    Ok(Json(DmMessagePage {
        items: page_items,
        next_cursor,
    }))
}

fn parse_limit(value: Option<u32>) -> ApiResult<usize> {
    let raw = value.unwrap_or(DEFAULT_PAGE_LIMIT as u32);
    if raw == 0 {
        return Err(bad_request(
            "limit_invalid",
            "limit must be greater than zero",
        ));
    }
    if raw as usize > MAX_PAGE_LIMIT {
        return Err(bad_request(
            "limit_invalid",
            "limit exceeds maximum page size",
        ));
    }

    Ok(raw as usize)
}

fn parse_message_cursor(value: Option<String>) -> ApiResult<Option<u64>> {
    let Some(cursor) = value else {
        return Ok(None);
    };

    cursor
        .parse::<u64>()
        .map(Some)
        .map_err(|_| bad_request("cursor_invalid", "message cursor must be numeric"))
}

fn dm_thread_fixtures() -> Vec<DmThreadSummary> {
    vec![
        DmThreadSummary {
            thread_id: "dm-thread-nora-jules".to_string(),
            kind: "dm".to_string(),
            title: "Nora K + Jules P".to_string(),
            participant_ids: vec!["usr-nora-k".to_string(), "usr-jules-p".to_string()],
            unread: 3,
            last_read_seq: 401,
            last_message_seq: 404,
            last_message_preview: "See you in the relay standup".to_string(),
            last_message_at: "2026-03-12T09:21:11Z".to_string(),
        },
        DmThreadSummary {
            thread_id: "gdm-thread-atlas".to_string(),
            kind: "group_dm".to_string(),
            title: "Atlas Draft Squad".to_string(),
            participant_ids: vec![
                "usr-nora-k".to_string(),
                "usr-mina-s".to_string(),
                "usr-alex-r".to_string(),
            ],
            unread: 1,
            last_read_seq: 144,
            last_message_seq: 145,
            last_message_preview: "Pushed the draft, review when free".to_string(),
            last_message_at: "2026-03-12T08:10:00Z".to_string(),
        },
        DmThreadSummary {
            thread_id: "dm-thread-nora-alex".to_string(),
            kind: "dm".to_string(),
            title: "Nora K + Alex R".to_string(),
            participant_ids: vec!["usr-nora-k".to_string(), "usr-alex-r".to_string()],
            unread: 0,
            last_read_seq: 220,
            last_message_seq: 220,
            last_message_preview: "Thanks for confirming the schedule".to_string(),
            last_message_at: "2026-03-11T21:45:30Z".to_string(),
        },
    ]
}

fn dm_message_fixtures(thread_id: &str) -> Option<Vec<DmMessageRecord>> {
    match thread_id {
        "dm-thread-nora-jules" => Some(vec![
            DmMessageRecord {
                message_id: "msg-404".to_string(),
                thread_id: thread_id.to_string(),
                author_id: "usr-jules-p".to_string(),
                seq: 404,
                ciphertext: "enc:95a0f4".to_string(),
                created_at: "2026-03-12T09:21:11Z".to_string(),
                edited_at: None,
            },
            DmMessageRecord {
                message_id: "msg-403".to_string(),
                thread_id: thread_id.to_string(),
                author_id: "usr-nora-k".to_string(),
                seq: 403,
                ciphertext: "enc:4bf120".to_string(),
                created_at: "2026-03-12T09:19:24Z".to_string(),
                edited_at: None,
            },
            DmMessageRecord {
                message_id: "msg-402".to_string(),
                thread_id: thread_id.to_string(),
                author_id: "usr-jules-p".to_string(),
                seq: 402,
                ciphertext: "enc:5c8e73".to_string(),
                created_at: "2026-03-12T09:12:00Z".to_string(),
                edited_at: Some("2026-03-12T09:12:39Z".to_string()),
            },
            DmMessageRecord {
                message_id: "msg-401".to_string(),
                thread_id: thread_id.to_string(),
                author_id: "usr-nora-k".to_string(),
                seq: 401,
                ciphertext: "enc:88f0ab".to_string(),
                created_at: "2026-03-12T09:05:08Z".to_string(),
                edited_at: None,
            },
        ]),
        "gdm-thread-atlas" => Some(vec![
            DmMessageRecord {
                message_id: "msg-145".to_string(),
                thread_id: thread_id.to_string(),
                author_id: "usr-mina-s".to_string(),
                seq: 145,
                ciphertext: "enc:10beef".to_string(),
                created_at: "2026-03-12T08:10:00Z".to_string(),
                edited_at: None,
            },
            DmMessageRecord {
                message_id: "msg-144".to_string(),
                thread_id: thread_id.to_string(),
                author_id: "usr-nora-k".to_string(),
                seq: 144,
                ciphertext: "enc:bada55".to_string(),
                created_at: "2026-03-12T08:03:19Z".to_string(),
                edited_at: None,
            },
        ]),
        "dm-thread-nora-alex" => Some(vec![DmMessageRecord {
            message_id: "msg-220".to_string(),
            thread_id: thread_id.to_string(),
            author_id: "usr-alex-r".to_string(),
            seq: 220,
            ciphertext: "enc:deed01".to_string(),
            created_at: "2026-03-11T21:45:30Z".to_string(),
            edited_at: None,
        }]),
        _ => None,
    }
}
