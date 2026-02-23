use crypto_envelope::{
    decrypt_chunk, decrypt_chunk_with_aad, derive_nonce, encrypt_chunk, encrypt_chunk_with_aad,
    Direction,
};

#[test]
fn encrypt_then_decrypt_round_trip() {
    let key = [9u8; 32];
    let nonce = derive_nonce(42, 7, Direction::SenderToReceiver);
    let plaintext = b"hello encrypted world";

    let ciphertext = encrypt_chunk(&key, nonce, plaintext).expect("encrypt");
    assert_ne!(ciphertext, plaintext);

    let decrypted = decrypt_chunk(&key, nonce, &ciphertext).expect("decrypt");
    assert_eq!(decrypted, plaintext);
}

#[test]
fn decryption_fails_with_wrong_key() {
    let good_key = [1u8; 32];
    let bad_key = [2u8; 32];
    let nonce = derive_nonce(1001, 3, Direction::SenderToReceiver);

    let ciphertext = encrypt_chunk(&good_key, nonce, b"payload").expect("encrypt");
    let result = decrypt_chunk(&bad_key, nonce, &ciphertext);

    assert!(result.is_err());
}

#[test]
fn decryption_fails_with_wrong_aad() {
    let key = [7u8; 32];
    let nonce = derive_nonce(55, 2, Direction::SenderToReceiver);

    let ciphertext =
        encrypt_chunk_with_aad(&key, nonce, b"payload", b"header-v2").expect("encrypt");

    let result = decrypt_chunk_with_aad(&key, nonce, &ciphertext, b"header-v1");
    assert!(result.is_err());
}

#[test]
fn nonce_derivation_changes_with_direction_and_index() {
    let n1 = derive_nonce(5, 1, Direction::SenderToReceiver);
    let n2 = derive_nonce(5, 2, Direction::SenderToReceiver);
    let n3 = derive_nonce(5, 1, Direction::ReceiverToSender);

    assert_ne!(n1, n2);
    assert_ne!(n1, n3);
    assert_eq!(n1.len(), 12);
}
