use audit_telemetry::{AuditEvent, AuditTelemetry, RetentionPolicy};
use desktop_ui::{DesktopUiState, DeviceCard, DeviceStatus, TransferItem, TransferState};
use discovery::Announcement;
use lan_offline::{LanOfflineGuard, LanPolicy};
use nat_traversal::{decide_route, gather_candidates, NatType, Route};
use std::collections::HashMap;
use std::net::SocketAddr;
use transfer::{
    decrypt_chunk_frame, encrypt_chunk_frame, Ack, EncryptionFlag, TransferChunk, TransferChunkV2,
    TransferSession,
};

pub fn wire_discovery_to_ui_and_transfer() -> Result<bool, String> {
    let ann = Announcement {
        device_id: "peer-a".into(),
        public_key_b64: "PUBKEYBASE64".into(),
        display_name: "Aarav iPhone".into(),
        port: 7777,
    };

    // Discovery packet decode path
    let decoded = Announcement::decode(&ann.encode()).map_err(|e| e.to_string())?;

    // LAN policy gate
    let guard = LanOfflineGuard::new(LanPolicy::default());
    let peer_addr: SocketAddr = "192.168.1.12:7777"
        .parse()
        .map_err(|e: std::net::AddrParseError| e.to_string())?;
    guard
        .validate_peer_set([peer_addr].iter())
        .map_err(|e| e.to_string())?;

    // UI device card update
    let mut ui = DesktopUiState::new();
    ui.upsert_device_card(DeviceCard {
        device_id: decoded.device_id.clone(),
        display_name: decoded.display_name,
        status: DeviceStatus::Online,
    });

    // Transfer path + checkpoint/ack
    let mut session = TransferSession::new(101, b"hello-world".to_vec(), 4, ["peer-a".to_string()])
        .map_err(|e| e.to_string())?;

    ui.add_transfer(TransferItem {
        transfer_id: 101,
        target_device_id: "peer-a".into(),
        file_name: "hello.txt".into(),
        progress_percent: 0,
        state: TransferState::InProgress,
    });

    session
        .apply_ack(&Ack {
            transfer_id: 101,
            receiver_id: "peer-a".into(),
            next_expected_chunk: session.total_chunks(),
        })
        .map_err(|e| e.to_string())?;

    ui.update_transfer_progress(101, 100)
        .map_err(|e| e.to_string())?;

    Ok(session.all_complete())
}

pub fn e2e_route_for_lan_and_relay() -> (Route, Route) {
    let lan_a = gather_candidates(
        "192.168.1.10:5000".parse().expect("addr"),
        Some("203.0.113.10:5000".parse().expect("addr")),
        Some("198.51.100.1:7000".parse().expect("addr")),
    );
    let lan_b = gather_candidates(
        "10.0.0.2:5001".parse().expect("addr"),
        Some("203.0.113.20:5001".parse().expect("addr")),
        Some("198.51.100.2:7000".parse().expect("addr")),
    );
    let lan_plan = decide_route(NatType::RestrictedCone, NatType::FullCone, &lan_a, &lan_b);

    let relay_a = gather_candidates(
        "10.1.1.2:5000".parse().expect("addr"),
        None,
        Some("198.51.100.9:7000".parse().expect("addr")),
    );
    let relay_b = gather_candidates("10.2.2.2:5001".parse().expect("addr"), None, None);
    let relay_plan = decide_route(NatType::Symmetric, NatType::Symmetric, &relay_a, &relay_b);

    (lan_plan.route, relay_plan.route)
}

pub fn lifecycle_security_and_telemetry_validation() -> Result<(usize, bool, bool), String> {
    let mut telemetry = AuditTelemetry::new(RetentionPolicy { max_events: 20 });

    let mut md = HashMap::new();
    md.insert("file_name".to_string(), "secret-plan.pdf".to_string());
    md.insert("transfer_id".to_string(), "101".to_string());
    telemetry.record_event(AuditEvent {
        timestamp_ms: 1,
        category: "security".to_string(),
        action: "peer_trusted".to_string(),
        metadata: md,
    });

    telemetry.record_event(AuditEvent {
        timestamp_ms: 2,
        category: "security".to_string(),
        action: "encryption.negotiated".to_string(),
        metadata: HashMap::new(),
    });
    telemetry.record_event(AuditEvent {
        timestamp_ms: 3,
        category: "security".to_string(),
        action: "encryption.required_rejected_peer".to_string(),
        metadata: HashMap::new(),
    });

    telemetry.increment_counter("security.peer_trusted");
    telemetry.increment_counter("transfer.completed");

    let first = telemetry.events().first().ok_or("missing event")?;
    let redacted = first
        .metadata
        .get("file_name")
        .map(|v| v == "[REDACTED]")
        .unwrap_or(false);

    let has_negotiated_event = telemetry
        .events()
        .iter()
        .any(|e| e.action == "encryption.negotiated");

    Ok((telemetry.events().len(), redacted, has_negotiated_event))
}

pub fn plaintext_and_encrypted_paths_coexist() -> Result<(bool, bool), String> {
    let plaintext_chunk = TransferChunk {
        transfer_id: 501,
        chunk_index: 0,
        total_chunks: 1,
        payload: b"plaintext-ok".to_vec(),
    };

    let decoded_plain =
        TransferChunk::decode(&plaintext_chunk.encode()).map_err(|e| e.to_string())?;
    let plaintext_ok = decoded_plain == plaintext_chunk;

    let session_key = [21u8; 32];
    let encrypted_frame =
        encrypt_chunk_frame(&plaintext_chunk, &session_key).map_err(|e| e.to_string())?;
    let decrypted =
        decrypt_chunk_frame(&encrypted_frame, &session_key).map_err(|e| e.to_string())?;
    let encrypted_ok = decrypted == plaintext_chunk;

    Ok((plaintext_ok, encrypted_ok))
}

pub fn required_mode_rejects_plaintext_frame() -> Result<&'static str, String> {
    let plaintext_frame = TransferChunkV2 {
        protocol_version: 2,
        encryption_flag: EncryptionFlag::Plaintext,
        transfer_id: 900,
        chunk_index: 0,
        total_chunks: 1,
        nonce: [0u8; 12],
        aad: Vec::new(),
        payload: b"legacy".to_vec(),
    };

    let key = [31u8; 32];
    let err = decrypt_chunk_frame(&plaintext_frame, &key)
        .expect_err("required-mode path must reject plaintext frame");

    if err.to_string().contains("expected encrypted frame") {
        Ok("rejected")
    } else {
        Err(format!("unexpected error: {err}"))
    }
}
