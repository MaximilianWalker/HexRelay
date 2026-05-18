use axum::{extract::State, http::HeaderMap, Json};

use crate::{
    infra::db::repos::account_data_repo,
    models::{
        AccountDataExportPackage, AccountDataImportCounts, AccountDataImportReport,
        AccountDataImportRequest,
    },
    shared::errors::{bad_request, internal_error, ApiResult},
    state::AppState,
    transport::http::middleware::auth::{enforce_csrf_for_cookie_auth, AuthSession},
};

pub async fn export_account_data(
    State(state): State<AppState>,
    auth: AuthSession,
) -> ApiResult<Json<AccountDataExportPackage>> {
    let Some(pool) = state.db_pool.as_ref() else {
        return Err(internal_error(
            "storage_unavailable",
            "account data export requires configured database pool",
        ));
    };

    let package = account_data_repo::export_account_data(pool, &auth.identity_id, &auth.expires_at)
        .await
        .map_err(|_| internal_error("storage_unavailable", "failed to export account data"))?;

    Ok(Json(package))
}

pub async fn import_account_data(
    auth: AuthSession,
    headers: HeaderMap,
    Json(payload): Json<AccountDataImportRequest>,
) -> ApiResult<Json<AccountDataImportReport>> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;

    if payload.package.identity.identity_id != auth.identity_id {
        return Err(bad_request(
            "account_import_identity_mismatch",
            "import package identity_id must match the authenticated identity",
        ));
    }

    if !payload.dry_run {
        return Err(bad_request(
            "account_import_write_unavailable",
            "account data import is dry-run only in this runtime",
        ));
    }

    Ok(Json(AccountDataImportReport {
        status: "dry_run".to_string(),
        identity_id: auth.identity_id,
        mutating_import_available: false,
        planned_counts: import_counts(&payload.package),
        warnings: payload.package.limitations.clone(),
        blocked_actions: vec![
            "No database rows are inserted, updated, or deleted by dry-run import.".to_string(),
            "Session ids, session tokens, private keys, DM device secrets, and transport endpoint hints are never imported.".to_string(),
        ],
    }))
}

fn import_counts(package: &AccountDataExportPackage) -> AccountDataImportCounts {
    AccountDataImportCounts {
        contacts: saturating_len(package.contacts.len()),
        servers: saturating_len(package.servers.len()),
        dm_profile_devices: saturating_len(package.dm_profile_devices.len()),
        dm_threads: saturating_len(package.dm_threads.len()),
        dm_messages: saturating_len(package.dm_messages.len()),
        server_channel_messages: saturating_len(package.server_channel_messages.len()),
    }
}

fn saturating_len(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}
