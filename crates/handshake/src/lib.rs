use identity::{verify_signature, DeviceIdentity, IdentityError};
use rand::rngs::OsRng;
use rand::RngCore;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncryptionMode {
    Off,
    Optional,
    Required,
}

impl EncryptionMode {
    fn as_u8(self) -> u8 {
        match self {
            EncryptionMode::Off => 0,
            EncryptionMode::Optional => 1,
            EncryptionMode::Required => 2,
        }
    }

    fn from_u8(v: u8) -> Result<Self, HandshakeError> {
        match v {
            0 => Ok(EncryptionMode::Off),
            1 => Ok(EncryptionMode::Optional),
            2 => Ok(EncryptionMode::Required),
            _ => Err(HandshakeError::InvalidCapabilities),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandshakeCapabilities {
    pub supports_encryption: bool,
    pub preferred_encryption_mode: EncryptionMode,
}

impl Default for HandshakeCapabilities {
    fn default() -> Self {
        Self {
            supports_encryption: false,
            preferred_encryption_mode: EncryptionMode::Off,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NegotiatedEncryption {
    pub enabled: bool,
    pub mode: EncryptionMode,
}

#[derive(Debug, Clone)]
pub struct ClientHello {
    pub device_id: String,
    pub public_key_b64: String,
    pub nonce: [u8; 32],
    pub timestamp_secs: u64,
    pub capabilities: HandshakeCapabilities,
    pub signature: [u8; 64],
}

#[derive(Debug, Clone)]
pub struct ServerHello {
    pub device_id: String,
    pub public_key_b64: String,
    pub client_nonce: [u8; 32],
    pub server_nonce: [u8; 32],
    pub timestamp_secs: u64,
    pub capabilities: HandshakeCapabilities,
    pub signature: [u8; 64],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionKeys {
    pub tx_key: [u8; 32],
    pub rx_key: [u8; 32],
}

#[derive(Debug)]
pub struct ReplayGuard {
    seen: HashMap<[u8; 32], Instant>,
    ttl: Duration,
}

impl ReplayGuard {
    pub fn new(ttl: Duration) -> Self {
        Self {
            seen: HashMap::new(),
            ttl,
        }
    }

    pub fn check_and_remember(&mut self, nonce: [u8; 32], now: Instant) -> bool {
        self.expire(now);
        if self.seen.contains_key(&nonce) {
            return false;
        }
        self.seen.insert(nonce, now);
        true
    }

    pub fn expire(&mut self, now: Instant) {
        let ttl = self.ttl;
        self.seen
            .retain(|_, seen_at| now.duration_since(*seen_at) <= ttl);
    }
}

pub fn create_client_hello(device_id: &str, identity: &DeviceIdentity) -> ClientHello {
    create_client_hello_with_capabilities(device_id, identity, HandshakeCapabilities::default())
}

pub fn create_client_hello_with_capabilities(
    device_id: &str,
    identity: &DeviceIdentity,
    capabilities: HandshakeCapabilities,
) -> ClientHello {
    let nonce = random_nonce();
    let timestamp_secs = now_unix();
    let public_key_b64 = identity.public_key_b64();
    let to_sign = client_hello_signing_bytes(
        device_id,
        &public_key_b64,
        nonce,
        timestamp_secs,
        capabilities,
    );
    let signature = identity.sign(&to_sign);

    ClientHello {
        device_id: device_id.to_string(),
        public_key_b64,
        nonce,
        timestamp_secs,
        capabilities,
        signature,
    }
}

pub fn verify_client_hello(
    hello: &ClientHello,
    max_skew_secs: u64,
    now_secs: u64,
) -> Result<(), HandshakeError> {
    if is_skewed(hello.timestamp_secs, now_secs, max_skew_secs) {
        return Err(HandshakeError::TimestampSkew);
    }

    let data = client_hello_signing_bytes(
        &hello.device_id,
        &hello.public_key_b64,
        hello.nonce,
        hello.timestamp_secs,
        hello.capabilities,
    );

    let valid = verify_signature(&hello.public_key_b64, &data, &hello.signature)
        .map_err(HandshakeError::Identity)?;
    if !valid {
        return Err(HandshakeError::InvalidSignature);
    }

    Ok(())
}

pub fn create_server_hello(
    device_id: &str,
    server_identity: &DeviceIdentity,
    client_hello: &ClientHello,
) -> ServerHello {
    create_server_hello_with_capabilities(
        device_id,
        server_identity,
        client_hello,
        HandshakeCapabilities::default(),
    )
}

pub fn create_server_hello_with_capabilities(
    device_id: &str,
    server_identity: &DeviceIdentity,
    client_hello: &ClientHello,
    capabilities: HandshakeCapabilities,
) -> ServerHello {
    let server_nonce = random_nonce();
    let timestamp_secs = now_unix();
    let public_key_b64 = server_identity.public_key_b64();
    let data = server_hello_signing_bytes(
        device_id,
        &public_key_b64,
        client_hello.nonce,
        server_nonce,
        timestamp_secs,
        capabilities,
    );
    let signature = server_identity.sign(&data);

    ServerHello {
        device_id: device_id.to_string(),
        public_key_b64,
        client_nonce: client_hello.nonce,
        server_nonce,
        timestamp_secs,
        capabilities,
        signature,
    }
}

pub fn verify_server_hello(
    expected_client_nonce: [u8; 32],
    hello: &ServerHello,
    max_skew_secs: u64,
    now_secs: u64,
) -> Result<(), HandshakeError> {
    if hello.client_nonce != expected_client_nonce {
        return Err(HandshakeError::NonceMismatch);
    }

    if is_skewed(hello.timestamp_secs, now_secs, max_skew_secs) {
        return Err(HandshakeError::TimestampSkew);
    }

    let data = server_hello_signing_bytes(
        &hello.device_id,
        &hello.public_key_b64,
        hello.client_nonce,
        hello.server_nonce,
        hello.timestamp_secs,
        hello.capabilities,
    );

    let valid = verify_signature(&hello.public_key_b64, &data, &hello.signature)
        .map_err(HandshakeError::Identity)?;
    if !valid {
        return Err(HandshakeError::InvalidSignature);
    }

    Ok(())
}

pub fn negotiate_encryption(
    client: HandshakeCapabilities,
    server: HandshakeCapabilities,
) -> Result<NegotiatedEncryption, HandshakeError> {
    validate_capabilities(client)?;
    validate_capabilities(server)?;

    let either_requires = client.preferred_encryption_mode == EncryptionMode::Required
        || server.preferred_encryption_mode == EncryptionMode::Required;
    let both_support = client.supports_encryption && server.supports_encryption;

    if either_requires && !both_support {
        return Err(HandshakeError::EncryptionRequiredButUnsupported);
    }

    if !both_support {
        return Ok(NegotiatedEncryption {
            enabled: false,
            mode: EncryptionMode::Off,
        });
    }

    if either_requires {
        return Ok(NegotiatedEncryption {
            enabled: true,
            mode: EncryptionMode::Required,
        });
    }

    if client.preferred_encryption_mode == EncryptionMode::Optional
        || server.preferred_encryption_mode == EncryptionMode::Optional
    {
        return Ok(NegotiatedEncryption {
            enabled: true,
            mode: EncryptionMode::Optional,
        });
    }

    Ok(NegotiatedEncryption {
        enabled: false,
        mode: EncryptionMode::Off,
    })
}

fn validate_capabilities(capabilities: HandshakeCapabilities) -> Result<(), HandshakeError> {
    // Roundtrip check so invalid discriminants are rejected if structs were built via unchecked paths.
    let _ = EncryptionMode::from_u8(capabilities.preferred_encryption_mode.as_u8())?;

    if !capabilities.supports_encryption
        && capabilities.preferred_encryption_mode != EncryptionMode::Off
    {
        return Err(HandshakeError::InvalidCapabilities);
    }

    Ok(())
}

/// Derive directional keys so each side gets tx/rx based on role.
pub fn derive_session_keys(
    client_public_key_b64: &str,
    server_public_key_b64: &str,
    client_nonce: [u8; 32],
    server_nonce: [u8; 32],
    is_client: bool,
) -> SessionKeys {
    let c2s = derive_key_material(
        b"p2p/c2s",
        client_public_key_b64,
        server_public_key_b64,
        client_nonce,
        server_nonce,
    );
    let s2c = derive_key_material(
        b"p2p/s2c",
        client_public_key_b64,
        server_public_key_b64,
        client_nonce,
        server_nonce,
    );

    if is_client {
        SessionKeys {
            tx_key: c2s,
            rx_key: s2c,
        }
    } else {
        SessionKeys {
            tx_key: s2c,
            rx_key: c2s,
        }
    }
}

#[derive(Debug, Error)]
pub enum HandshakeError {
    #[error("timestamp skew exceeded")]
    TimestampSkew,
    #[error("invalid signature")]
    InvalidSignature,
    #[error("client/server nonce mismatch")]
    NonceMismatch,
    #[error("identity error: {0}")]
    Identity(IdentityError),
    #[error("peer does not support required encryption mode")]
    EncryptionRequiredButUnsupported,
    #[error("invalid handshake capabilities")]
    InvalidCapabilities,
}

fn client_hello_signing_bytes(
    device_id: &str,
    public_key_b64: &str,
    nonce: [u8; 32],
    timestamp_secs: u64,
    capabilities: HandshakeCapabilities,
) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"p2p/client-hello/v1");
    out.extend_from_slice(device_id.as_bytes());
    out.extend_from_slice(public_key_b64.as_bytes());
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&timestamp_secs.to_be_bytes());
    out.push(capabilities.supports_encryption as u8);
    out.push(capabilities.preferred_encryption_mode.as_u8());
    out
}

fn server_hello_signing_bytes(
    device_id: &str,
    public_key_b64: &str,
    client_nonce: [u8; 32],
    server_nonce: [u8; 32],
    timestamp_secs: u64,
    capabilities: HandshakeCapabilities,
) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"p2p/server-hello/v1");
    out.extend_from_slice(device_id.as_bytes());
    out.extend_from_slice(public_key_b64.as_bytes());
    out.extend_from_slice(&client_nonce);
    out.extend_from_slice(&server_nonce);
    out.extend_from_slice(&timestamp_secs.to_be_bytes());
    out.push(capabilities.supports_encryption as u8);
    out.push(capabilities.preferred_encryption_mode.as_u8());
    out
}

fn derive_key_material(
    label: &[u8],
    client_public_key_b64: &str,
    server_public_key_b64: &str,
    client_nonce: [u8; 32],
    server_nonce: [u8; 32],
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(label);
    hasher.update(client_public_key_b64.as_bytes());
    hasher.update(server_public_key_b64.as_bytes());
    hasher.update(client_nonce);
    hasher.update(server_nonce);
    let digest = hasher.finalize();

    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}

fn random_nonce() -> [u8; 32] {
    let mut nonce = [0u8; 32];
    OsRng.fill_bytes(&mut nonce);
    nonce
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn is_skewed(msg_ts: u64, now: u64, max_skew: u64) -> bool {
    if msg_ts > now {
        msg_ts - now > max_skew
    } else {
        now - msg_ts > max_skew
    }
}
