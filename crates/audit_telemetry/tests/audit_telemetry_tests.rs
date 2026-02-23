use audit_telemetry::{AuditEvent, AuditTelemetry, RetentionPolicy};
use std::collections::HashMap;

#[test]
fn records_event_with_redaction() {
    let mut telemetry = AuditTelemetry::new(RetentionPolicy { max_events: 100 });

    let mut metadata = HashMap::new();
    metadata.insert("file_name".to_string(), "secret.pdf".to_string());
    metadata.insert("transfer_id".to_string(), "42".to_string());

    telemetry.record_event(AuditEvent {
        timestamp_ms: 1,
        category: "transfer".to_string(),
        action: "sent".to_string(),
        metadata,
    });

    let event = &telemetry.events()[0];
    assert_eq!(event.metadata.get("file_name").expect("redacted"), "[REDACTED]");
    assert_eq!(event.metadata.get("transfer_id").expect("kept"), "42");
}

#[test]
fn counter_increments_without_payload() {
    let mut telemetry = AuditTelemetry::new(RetentionPolicy::default());
    telemetry.increment_counter("transfers.completed");
    telemetry.increment_counter("transfers.completed");
    telemetry.increment_counter("transfers.failed");

    assert_eq!(telemetry.counter_value("transfers.completed"), 2);
    assert_eq!(telemetry.counter_value("transfers.failed"), 1);
    assert_eq!(telemetry.counter_value("unknown.metric"), 0);
}

#[test]
fn retention_keeps_latest_events_only() {
    let mut telemetry = AuditTelemetry::new(RetentionPolicy { max_events: 2 });

    for i in 0..3 {
        telemetry.record_event(AuditEvent {
            timestamp_ms: i,
            category: "c".to_string(),
            action: format!("a{i}"),
            metadata: HashMap::new(),
        });
    }

    assert_eq!(telemetry.events().len(), 2);
    assert_eq!(telemetry.events()[0].action, "a1");
    assert_eq!(telemetry.events()[1].action, "a2");
}

#[test]
fn export_writes_local_log_file() {
    let mut telemetry = AuditTelemetry::new(RetentionPolicy::default());
    telemetry.record_event(AuditEvent {
        timestamp_ms: 123,
        category: "transfer".to_string(),
        action: "received".to_string(),
        metadata: HashMap::new(),
    });

    let p = std::env::temp_dir().join("p2p_audit_export.log");
    telemetry.export_events(&p).expect("export");

    let content = std::fs::read_to_string(&p).expect("read export");
    std::fs::remove_file(&p).ok();

    assert!(content.contains("123|transfer|received|"));
}
