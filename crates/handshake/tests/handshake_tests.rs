use handshake::{
    create_client_hello, create_client_hello_with_capabilities, create_server_hello,
    create_server_hello_with_capabilities, derive_session_keys, negotiate_encryption,
    verify_client_hello, verify_server_hello, EncryptionMode, HandshakeCapabilities,
    HandshakeError, ReplayGuard,
};
use identity::DeviceIdentity;
use std::time::{Duration, Instant};

#[test]
fn client_hello_verification_succeeds() {
    let client = DeviceIdentity::generate();
    let hello = create_client_hello("client-1", &client);
    let now = hello.timestamp_secs;
    verify_client_hello(&hello, 30, now).expect("valid client hello");
}

#[test]
fn server_hello_verification_succeeds() {
    let client = DeviceIdentity::generate();
    let server = DeviceIdentity::generate();

    let ch = create_client_hello("client-1", &client);
    let sh = create_server_hello("server-1", &server, &ch);

    verify_server_hello(ch.nonce, &sh, 30, sh.timestamp_secs).expect("valid server hello");
}

#[test]
fn client_hello_signature_covers_capabilities() {
    let client = DeviceIdentity::generate();
    let mut hello = create_client_hello_with_capabilities(
        "client-1",
        &client,
        HandshakeCapabilities {
            supports_encryption: true,
            preferred_encryption_mode: EncryptionMode::Optional,
        },
    );

    hello.capabilities.preferred_encryption_mode = EncryptionMode::Required;

    let err = verify_client_hello(&hello, 30, hello.timestamp_secs).expect_err("tamper fails");
    assert!(matches!(err, HandshakeError::InvalidSignature));
}

#[test]
fn server_hello_signature_covers_capabilities() {
    let client = DeviceIdentity::generate();
    let server = DeviceIdentity::generate();
    let ch = create_client_hello("client-1", &client);

    let mut sh = create_server_hello_with_capabilities(
        "server-1",
        &server,
        &ch,
        HandshakeCapabilities {
            supports_encryption: true,
            preferred_encryption_mode: EncryptionMode::Optional,
        },
    );

    sh.capabilities.supports_encryption = false;

    let err = verify_server_hello(ch.nonce, &sh, 30, sh.timestamp_secs).expect_err("tamper fails");
    assert!(matches!(err, HandshakeError::InvalidSignature));
}

#[test]
fn negotiation_optional_falls_back_to_plaintext_when_peer_lacks_support() {
    let negotiated = negotiate_encryption(
        HandshakeCapabilities {
            supports_encryption: true,
            preferred_encryption_mode: EncryptionMode::Optional,
        },
        HandshakeCapabilities {
            supports_encryption: false,
            preferred_encryption_mode: EncryptionMode::Off,
        },
    )
    .expect("fallback allowed");

    assert!(!negotiated.enabled);
    assert_eq!(negotiated.mode, EncryptionMode::Off);
}

#[test]
fn negotiation_required_rejects_non_supporting_peer() {
    let err = negotiate_encryption(
        HandshakeCapabilities {
            supports_encryption: true,
            preferred_encryption_mode: EncryptionMode::Required,
        },
        HandshakeCapabilities {
            supports_encryption: false,
            preferred_encryption_mode: EncryptionMode::Off,
        },
    )
    .expect_err("required should fail closed");

    assert!(matches!(
        err,
        HandshakeError::EncryptionRequiredButUnsupported
    ));
}

#[test]
fn negotiation_enables_optional_when_both_support_it() {
    let negotiated = negotiate_encryption(
        HandshakeCapabilities {
            supports_encryption: true,
            preferred_encryption_mode: EncryptionMode::Optional,
        },
        HandshakeCapabilities {
            supports_encryption: true,
            preferred_encryption_mode: EncryptionMode::Off,
        },
    )
    .expect("optional succeeds");

    assert!(negotiated.enabled);
    assert_eq!(negotiated.mode, EncryptionMode::Optional);
}

#[test]
fn session_keys_are_directional_and_consistent() {
    let client = DeviceIdentity::generate();
    let server = DeviceIdentity::generate();

    let ch = create_client_hello("client-1", &client);
    let sh = create_server_hello("server-1", &server, &ch);

    let client_keys = derive_session_keys(
        &ch.public_key_b64,
        &sh.public_key_b64,
        ch.nonce,
        sh.server_nonce,
        true,
    );

    let server_keys = derive_session_keys(
        &ch.public_key_b64,
        &sh.public_key_b64,
        ch.nonce,
        sh.server_nonce,
        false,
    );

    assert_eq!(client_keys.tx_key, server_keys.rx_key);
    assert_eq!(client_keys.rx_key, server_keys.tx_key);
    assert_ne!(client_keys.tx_key, client_keys.rx_key);
}

#[test]
fn replay_guard_blocks_reused_nonce() {
    let mut guard = ReplayGuard::new(Duration::from_secs(10));
    let nonce = [7u8; 32];
    let now = Instant::now();

    assert!(guard.check_and_remember(nonce, now));
    assert!(!guard.check_and_remember(nonce, now + Duration::from_secs(1)));
    assert!(guard.check_and_remember(nonce, now + Duration::from_secs(11)));
}
