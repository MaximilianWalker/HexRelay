use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use chrono::Utc;
use communication_core::{
    parse_lan_endpoint_hint, validate_lan_endpoint_hint, LanDiscoveryAdvertisement,
    LAN_DISCOVERY_MULTICAST_HOP_LIMIT, LAN_DISCOVERY_SCOPE, LAN_DISCOVERY_TTL_SECONDS,
};
use serde::{Deserialize, Serialize};
use tokio::net::UdpSocket;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::state::AppState;

const LAN_DISCOVERY_EVENT_TYPE: &str = "dm.lan_discovery.advertise";
const LAN_DISCOVERY_MAX_PACKET_BYTES: usize = 8192;

#[derive(Deserialize)]
struct RealtimeInboundEnvelope {
    event_type: String,
    event_version: u8,
    #[serde(default)]
    correlation_id: Option<String>,
    data: serde_json::Value,
}

#[derive(Serialize)]
struct RealtimeOutboundEnvelope<T: Serialize> {
    event_id: String,
    event_type: String,
    event_version: u8,
    occurred_at: String,
    correlation_id: String,
    producer: String,
    data: T,
}

pub async fn handle_lan_discovery_ws_event(
    state: &AppState,
    session_identity_id: &str,
    raw: &str,
) -> Option<String> {
    let parsed = serde_json::from_str::<RealtimeInboundEnvelope>(raw).ok()?;
    if parsed.event_type != LAN_DISCOVERY_EVENT_TYPE {
        return None;
    }

    if !state.enable_lan_discovery {
        return Some(build_error_event(
            "lan_discovery_disabled",
            "LAN discovery is disabled for this realtime runtime",
        ));
    }

    if parsed.event_version != 1 {
        return Some(build_error_event(
            "event_version_unsupported",
            "event_version must be 1",
        ));
    }

    let advertisement = match serde_json::from_value::<LanDiscoveryAdvertisement>(parsed.data) {
        Ok(value) => value,
        Err(_) => {
            return Some(build_error_event(
                "event_invalid",
                "invalid dm.lan_discovery.advertise payload",
            ));
        }
    };

    if let Err((code, message)) = validate_advertisement_for_session(
        &advertisement,
        session_identity_id,
        Utc::now().timestamp(),
    ) {
        return Some(build_error_event(code, message));
    }

    state
        .active_lan_advertisements
        .lock()
        .await
        .insert(session_identity_id.to_string(), advertisement.clone());

    Some(build_lan_discovery_advertised_event(
        &advertisement,
        parsed.correlation_id,
    ))
}

pub fn spawn_lan_discovery_tasks(state: AppState) {
    if !state.enable_lan_discovery {
        return;
    }

    info!(
        bind_addr = %state.lan_discovery_bind_addr,
        multicast_addr = %state.lan_discovery_multicast_addr,
        "starting LAN discovery multicast tasks"
    );

    tokio::spawn(run_lan_discovery_announcer(state.clone()));
    tokio::spawn(run_lan_discovery_listener(state));
}

async fn run_lan_discovery_announcer(state: AppState) {
    let socket = match bind_announce_socket() {
        Ok(value) => value,
        Err(error) => {
            warn!(error = %error, "LAN discovery announcer disabled after socket bind failure");
            return;
        }
    };

    let mut interval = tokio::time::interval(state.lan_discovery_announce_interval);
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    loop {
        interval.tick().await;
        let advertisements = fresh_active_advertisements(&state).await;
        for advertisement in advertisements {
            match serde_json::to_vec(&advertisement) {
                Ok(packet) if packet.len() <= LAN_DISCOVERY_MAX_PACKET_BYTES => {
                    if let Err(error) = socket
                        .send_to(&packet, state.lan_discovery_multicast_addr)
                        .await
                    {
                        warn!(error = %error, "failed to send LAN discovery multicast packet");
                    }
                }
                Ok(_) => warn!("skipped oversized LAN discovery multicast packet"),
                Err(error) => warn!(error = %error, "failed to encode LAN discovery packet"),
            }
        }
    }
}

async fn run_lan_discovery_listener(state: AppState) {
    let socket = match bind_listen_socket(
        state.lan_discovery_bind_addr,
        state.lan_discovery_multicast_addr,
    ) {
        Ok(value) => value,
        Err(error) => {
            warn!(error = %error, "LAN discovery listener disabled after socket bind failure");
            return;
        }
    };

    let mut buffer = [0_u8; LAN_DISCOVERY_MAX_PACKET_BYTES];
    loop {
        let (len, source) = match socket.recv_from(&mut buffer).await {
            Ok(value) => value,
            Err(error) => {
                warn!(error = %error, "failed to receive LAN discovery multicast packet");
                continue;
            }
        };

        let packet = &buffer[..len];
        let advertisement = match serde_json::from_slice::<LanDiscoveryAdvertisement>(packet) {
            Ok(value) => value,
            Err(_) => {
                debug!(%source, "ignored invalid LAN discovery packet");
                continue;
            }
        };

        if validate_advertisement_shape(&advertisement, Utc::now().timestamp()).is_err() {
            debug!(%source, "ignored stale or invalid LAN discovery packet");
            continue;
        }
        if !advertisement_matches_source(&advertisement, source.ip()) {
            debug!(%source, "ignored LAN discovery packet with mismatched endpoint source");
            continue;
        }

        if let Err(error) =
            ingest_lan_discovery_advertisement(&state, &advertisement, source.ip()).await
        {
            warn!(error = %error, "failed to ingest LAN discovery packet through API");
        }
    }
}

async fn fresh_active_advertisements(state: &AppState) -> Vec<LanDiscoveryAdvertisement> {
    let now = Utc::now().timestamp();
    let mut guard = state.active_lan_advertisements.lock().await;
    guard.retain(|_, advertisement| validate_advertisement_shape(advertisement, now).is_ok());
    guard.values().cloned().collect()
}

async fn ingest_lan_discovery_advertisement(
    state: &AppState,
    advertisement: &LanDiscoveryAdvertisement,
    observed_source_ip: IpAddr,
) -> Result<(), String> {
    let url = format!(
        "{}/v1/internal/dm/connectivity/lan-discovery/ingest",
        state.api_base_url.trim_end_matches('/')
    );
    let response = state
        .http_client
        .post(url)
        .header(
            "x-hexrelay-internal-token",
            &state.channel_dispatch_internal_token,
        )
        .header(
            "x-hexrelay-observed-source-ip",
            observed_source_ip.to_string(),
        )
        .json(advertisement)
        .send()
        .await
        .map_err(|error| format!("API ingest request failed: {error}"))?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!("API ingest returned {}", response.status()))
    }
}

fn advertisement_matches_source(
    advertisement: &LanDiscoveryAdvertisement,
    observed_source_ip: IpAddr,
) -> bool {
    advertisement.endpoint_hints.iter().all(|hint| {
        parse_lan_endpoint_hint(hint)
            .map(|endpoint| endpoint.address.ip() == observed_source_ip)
            .unwrap_or(false)
    })
}

fn bind_announce_socket() -> Result<UdpSocket, String> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0")
        .map_err(|error| format!("bind UDP announce socket: {error}"))?;
    socket
        .set_multicast_ttl_v4(LAN_DISCOVERY_MULTICAST_HOP_LIMIT)
        .map_err(|error| format!("set multicast ttl: {error}"))?;
    socket
        .set_nonblocking(true)
        .map_err(|error| format!("set announce socket nonblocking: {error}"))?;
    UdpSocket::from_std(socket).map_err(|error| format!("create tokio announce socket: {error}"))
}

fn bind_listen_socket(
    bind_addr: SocketAddr,
    multicast_addr: SocketAddr,
) -> Result<UdpSocket, String> {
    let socket = std::net::UdpSocket::bind(bind_addr)
        .map_err(|error| format!("bind UDP listen socket: {error}"))?;
    if let SocketAddr::V4(multicast) = multicast_addr {
        socket
            .join_multicast_v4(multicast.ip(), &Ipv4Addr::UNSPECIFIED)
            .map_err(|error| format!("join IPv4 multicast group: {error}"))?;
    } else {
        return Err("LAN discovery currently requires IPv4 multicast".to_string());
    }
    socket
        .set_nonblocking(true)
        .map_err(|error| format!("set listen socket nonblocking: {error}"))?;
    UdpSocket::from_std(socket).map_err(|error| format!("create tokio listen socket: {error}"))
}

fn validate_advertisement_for_session(
    advertisement: &LanDiscoveryAdvertisement,
    session_identity_id: &str,
    now: i64,
) -> Result<(), (&'static str, &'static str)> {
    validate_advertisement_shape(advertisement, now)?;
    if advertisement.identity_id != session_identity_id {
        return Err((
            "event_identity_mismatch",
            "identity_id does not match authenticated session",
        ));
    }

    Ok(())
}

fn validate_advertisement_shape(
    advertisement: &LanDiscoveryAdvertisement,
    now: i64,
) -> Result<(), (&'static str, &'static str)> {
    if advertisement.version != 1 {
        return Err((
            "event_invalid",
            "LAN discovery advertisement version must be 1",
        ));
    }
    if advertisement.scope != LAN_DISCOVERY_SCOPE {
        return Err((
            "event_invalid",
            "LAN discovery advertisement scope must be lan_subnet",
        ));
    }
    if !is_valid_identity_id(&advertisement.identity_id) {
        return Err(("event_invalid", "LAN discovery identity_id is invalid"));
    }
    if advertisement.nonce.trim().is_empty() || advertisement.nonce.len() > 128 {
        return Err(("event_invalid", "LAN discovery nonce is invalid"));
    }
    if advertisement.signature.trim().is_empty() || advertisement.signature.len() > 256 {
        return Err(("event_invalid", "LAN discovery signature is invalid"));
    }
    if advertisement.issued_at_epoch > now + 30
        || advertisement.expires_at_epoch <= now
        || advertisement.expires_at_epoch <= advertisement.issued_at_epoch
        || advertisement.expires_at_epoch - advertisement.issued_at_epoch
            > LAN_DISCOVERY_TTL_SECONDS
    {
        return Err((
            "event_invalid",
            "LAN discovery timing is invalid or expired",
        ));
    }
    if advertisement.endpoint_hints.is_empty() || advertisement.endpoint_hints.len() > 8 {
        return Err(("event_invalid", "LAN discovery endpoint_hints are invalid"));
    }
    if !advertisement
        .endpoint_hints
        .iter()
        .all(|hint| validate_lan_endpoint_hint(hint).is_ok())
    {
        return Err((
            "event_invalid",
            "LAN discovery endpoint_hints must be local-only direct addresses",
        ));
    }

    Ok(())
}

fn is_valid_identity_id(value: &str) -> bool {
    let len = value.len();
    (3..=64).contains(&len)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
}

fn build_lan_discovery_advertised_event(
    advertisement: &LanDiscoveryAdvertisement,
    correlation_id: Option<String>,
) -> String {
    let envelope = RealtimeOutboundEnvelope {
        event_id: Uuid::new_v4().to_string(),
        event_type: LAN_DISCOVERY_EVENT_TYPE.to_string(),
        event_version: 1,
        occurred_at: Utc::now().to_rfc3339(),
        correlation_id: correlation_id.unwrap_or_else(|| Uuid::new_v4().to_string()),
        producer: "realtime-gateway".to_string(),
        data: serde_json::json!({
            "status": "advertising",
            "identity_id": &advertisement.identity_id,
            "endpoint_hints": &advertisement.endpoint_hints,
            "scope": &advertisement.scope,
            "expires_at_epoch": advertisement.expires_at_epoch,
        }),
    };

    serde_json::to_string(&envelope).unwrap_or_else(|_| {
        build_error_event("event_serialize_failed", "failed to serialize event")
    })
}

fn build_error_event(code: &'static str, message: &'static str) -> String {
    crate::domain::events::service::build_error_event(code, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn advertisement(identity_id: &str) -> LanDiscoveryAdvertisement {
        let issued_at_epoch = Utc::now().timestamp();
        LanDiscoveryAdvertisement {
            version: 1,
            identity_id: identity_id.to_string(),
            endpoint_hints: vec!["udp://192.168.1.12:4040".to_string()],
            scope: LAN_DISCOVERY_SCOPE.to_string(),
            issued_at_epoch,
            expires_at_epoch: issued_at_epoch + LAN_DISCOVERY_TTL_SECONDS,
            nonce: "nonce-1".to_string(),
            signature: "aa".repeat(64),
        }
    }

    fn test_state(enable_lan_discovery: bool) -> AppState {
        AppState::new(
            "http://127.0.0.1:8080".to_string(),
            vec!["http://127.0.0.1:3002".to_string()],
            "test-channel-dispatch-token-12345".to_string(),
            "test-presence-watcher-token-12345".to_string(),
            None,
            false,
            60,
            60,
            16_384,
            120,
            60,
            3,
            0,
            10_000,
        )
        .expect("build state")
        .with_lan_discovery_config(
            enable_lan_discovery,
            "0.0.0.0:48999".parse().unwrap(),
            "239.255.48.31:48999".parse().unwrap(),
            Duration::from_secs(10),
        )
    }

    #[tokio::test]
    async fn handles_valid_lan_discovery_ws_event() {
        let state = test_state(true);
        let raw = serde_json::json!({
            "event_type": LAN_DISCOVERY_EVENT_TYPE,
            "event_version": 1,
            "correlation_id": "corr-1",
            "data": advertisement("usr-nora-k"),
        })
        .to_string();

        let response = handle_lan_discovery_ws_event(&state, "usr-nora-k", &raw)
            .await
            .expect("LAN event handled");
        let payload: serde_json::Value = serde_json::from_str(&response).expect("decode response");

        assert_eq!(payload["event_type"], LAN_DISCOVERY_EVENT_TYPE);
        assert_eq!(payload["correlation_id"], "corr-1");
        assert_eq!(
            state.active_lan_advertisements.lock().await.len(),
            1,
            "advertisement stored for multicast announcer"
        );
    }

    #[tokio::test]
    async fn rejects_cross_identity_lan_discovery_ws_event() {
        let state = test_state(true);
        let raw = serde_json::json!({
            "event_type": LAN_DISCOVERY_EVENT_TYPE,
            "event_version": 1,
            "data": advertisement("usr-jules-p"),
        })
        .to_string();

        let response = handle_lan_discovery_ws_event(&state, "usr-nora-k", &raw)
            .await
            .expect("LAN event handled");
        let payload: serde_json::Value = serde_json::from_str(&response).expect("decode response");

        assert_eq!(payload["event_type"], "error");
        assert_eq!(payload["data"]["code"], "event_identity_mismatch");
    }

    #[tokio::test]
    async fn ignores_non_lan_discovery_ws_event() {
        let state = test_state(true);
        assert!(handle_lan_discovery_ws_event(
            &state,
            "usr-nora-k",
            r#"{"event_type":"call.signal.offer","event_version":1,"data":{}}"#,
        )
        .await
        .is_none());
    }

    #[test]
    fn rejects_lan_discovery_advertisement_source_mismatch() {
        let mut advertisement = advertisement("usr-nora-k");
        assert!(advertisement_matches_source(
            &advertisement,
            "192.168.1.12".parse().unwrap(),
        ));

        advertisement.endpoint_hints = vec!["udp://192.168.1.99:4040".to_string()];
        assert!(!advertisement_matches_source(
            &advertisement,
            "192.168.1.12".parse().unwrap(),
        ));
    }
}
