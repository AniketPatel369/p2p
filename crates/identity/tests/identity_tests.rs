use identity::{verify_signature, DeviceIdentity};

#[test]
fn generate_has_public_key_and_fingerprint() {
    let id = DeviceIdentity::generate();
    let public_key = id.public_key_b64();
    let fp = id.fingerprint();

    assert!(!public_key.is_empty());
    assert_eq!(fp.len(), 47); // 16 bytes -> "AA:.." format => 16*2 + 15 separators
    assert!(fp.chars().all(|c| c.is_ascii_hexdigit() || c == ':'));
}

#[test]
fn save_and_load_roundtrip() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("device.key");

    let id = DeviceIdentity::generate();
    let original_pk = id.public_key_b64();
    id.save(&path).expect("save");

    let loaded = DeviceIdentity::load(&path).expect("load");
    assert_eq!(loaded.public_key_b64(), original_pk);
}

#[test]
fn sign_and_verify_roundtrip() {
    let id = DeviceIdentity::generate();
    let msg = b"handshake-message";
    let sig = id.sign(msg);
    let ok = verify_signature(&id.public_key_b64(), msg, &sig).expect("verify");
    assert!(ok);
}
