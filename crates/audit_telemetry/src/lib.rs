use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEvent {
    pub timestamp_ms: u64,
    pub category: String,
    pub action: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetentionPolicy {
    pub max_events: usize,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self { max_events: 1000 }
    }
}

#[derive(Debug, Clone)]
pub struct AuditTelemetry {
    events: Vec<AuditEvent>,
    counters: HashMap<String, u64>,
    retention: RetentionPolicy,
}

impl AuditTelemetry {
    pub fn new(retention: RetentionPolicy) -> Self {
        Self {
            events: Vec::new(),
            counters: HashMap::new(),
            retention,
        }
    }

    /// Records a structured event and applies redaction + retention.
    pub fn record_event(&mut self, mut event: AuditEvent) {
        redact_sensitive_metadata(&mut event.metadata);
        self.events.push(event);
        self.enforce_retention();
    }

    /// Increment a telemetry counter without payload contents.
    pub fn increment_counter(&mut self, metric: &str) {
        let value = self.counters.entry(metric.to_string()).or_insert(0);
        *value += 1;
    }

    pub fn counter_value(&self, metric: &str) -> u64 {
        *self.counters.get(metric).unwrap_or(&0)
    }

    pub fn events(&self) -> &[AuditEvent] {
        &self.events
    }

    /// Export local logs in line-oriented simple format.
    pub fn export_events(&self, path: impl AsRef<Path>) -> Result<(), AuditError> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut out = String::new();
        for e in &self.events {
            let mut metadata_parts: Vec<String> = e
                .metadata
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect();
            metadata_parts.sort();
            let metadata = metadata_parts.join(",");
            out.push_str(&format!(
                "{}|{}|{}|{}\n",
                e.timestamp_ms, e.category, e.action, metadata
            ));
        }

        fs::write(path, out)?;
        Ok(())
    }

    fn enforce_retention(&mut self) {
        if self.events.len() > self.retention.max_events {
            let drop_n = self.events.len() - self.retention.max_events;
            self.events.drain(0..drop_n);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuditError {
    Io(String),
}

impl std::fmt::Display for AuditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditError::Io(m) => write!(f, "io error: {m}"),
        }
    }
}

impl std::error::Error for AuditError {}

impl From<std::io::Error> for AuditError {
    fn from(value: std::io::Error) -> Self {
        AuditError::Io(value.to_string())
    }
}

pub fn redact_sensitive_metadata(metadata: &mut HashMap<String, String>) {
    const REDACT_KEYS: &[&str] = &[
        "file_name",
        "file_path",
        "receiver_name",
        "sender_name",
        "payload",
    ];

    for key in REDACT_KEYS {
        if metadata.contains_key(*key) {
            metadata.insert((*key).to_string(), "[REDACTED]".to_string());
        }
    }
}
