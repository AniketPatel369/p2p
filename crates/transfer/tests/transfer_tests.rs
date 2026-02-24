use transfer::{
    decrypt_chunk_frame, encrypt_chunk_frame, transfer_chunk_aad, Ack, EncryptionFlag,
    TransferChunk, TransferChunkV2, TransferSession, VersionedTransferChunk,
};

#[test]
fn chunk_frame_roundtrip() {
    let chunk = TransferChunk {
        transfer_id: 42,
        chunk_index: 0,
        total_chunks: 2,
        payload: b"hello".to_vec(),
    };

    let decoded = TransferChunk::decode(&chunk.encode()).expect("decode chunk");
    assert_eq!(decoded, chunk);
}

#[test]
fn versioned_decoder_accepts_v1_and_v2() {
    let v1 = TransferChunk {
        transfer_id: 1,
        chunk_index: 0,
        total_chunks: 1,
        payload: b"v1".to_vec(),
    }
    .encode();

    let v2 = TransferChunkV2 {
        protocol_version: 2,
        encryption_flag: EncryptionFlag::Encrypted,
        transfer_id: 2,
        chunk_index: 0,
        total_chunks: 1,
        nonce: [8u8; 12],
        aad: b"meta".to_vec(),
        payload: b"v2-cipher".to_vec(),
    }
    .encode();

    assert!(matches!(
        VersionedTransferChunk::decode(&v1).expect("decode v1"),
        VersionedTransferChunk::V1(_)
    ));
    assert!(matches!(
        VersionedTransferChunk::decode(&v2).expect("decode v2"),
        VersionedTransferChunk::V2(_)
    ));
}

#[test]
fn v2_frame_roundtrip_with_metadata() {
    let chunk = TransferChunkV2 {
        protocol_version: 2,
        encryption_flag: EncryptionFlag::Encrypted,
        transfer_id: 91,
        chunk_index: 3,
        total_chunks: 10,
        nonce: [5u8; 12],
        aad: b"header-v2".to_vec(),
        payload: vec![11, 22, 33, 44],
    };

    let decoded = TransferChunkV2::decode(&chunk.encode()).expect("decode v2 frame");
    assert_eq!(decoded, chunk);
}

#[test]
fn encrypt_adapter_wraps_chunk_and_decrypt_adapter_recovers_payload() {
    let key = [13u8; 32];
    let chunk = TransferChunk {
        transfer_id: 77,
        chunk_index: 2,
        total_chunks: 5,
        payload: b"payload-for-e4".to_vec(),
    };

    let encrypted_frame = encrypt_chunk_frame(&chunk, &key).expect("encrypt adapter");
    assert_eq!(encrypted_frame.protocol_version, 2);
    assert_eq!(encrypted_frame.encryption_flag, EncryptionFlag::Encrypted);
    assert_eq!(encrypted_frame.aad, transfer_chunk_aad(&chunk));
    assert_ne!(encrypted_frame.payload, chunk.payload);

    let decrypted = decrypt_chunk_frame(&encrypted_frame, &key).expect("decrypt adapter");
    assert_eq!(decrypted, chunk);
}

#[test]
fn decrypt_adapter_fails_with_wrong_key() {
    let good_key = [1u8; 32];
    let bad_key = [2u8; 32];
    let chunk = TransferChunk {
        transfer_id: 9,
        chunk_index: 0,
        total_chunks: 1,
        payload: b"secret".to_vec(),
    };

    let frame = encrypt_chunk_frame(&chunk, &good_key).expect("encrypt");
    let err = decrypt_chunk_frame(&frame, &bad_key).expect_err("wrong key should fail");
    assert_eq!(
        err.to_string(),
        "crypto error: failed to decrypt chunk payload"
    );
}

#[test]
fn session_creates_expected_total_chunks() {
    let data = vec![1u8; 10];
    let session = TransferSession::new(10, data, 4, ["r1".to_string()]).expect("new session");
    assert_eq!(session.total_chunks(), 3);
}

#[test]
fn resume_checkpoint_moves_forward_per_receiver() {
    let data = vec![5u8; 12];
    let mut session = TransferSession::new(11, data, 4, ["r1".to_string(), "r2".to_string()])
        .expect("new session");

    session
        .apply_ack(&Ack {
            transfer_id: 11,
            receiver_id: "r1".to_string(),
            next_expected_chunk: 2,
        })
        .expect("ack 1");

    session
        .apply_ack(&Ack {
            transfer_id: 11,
            receiver_id: "r1".to_string(),
            next_expected_chunk: 1,
        })
        .expect("stale ack ignored monotonic");

    assert_eq!(
        session.resume_from_for_receiver("r1").expect("checkpoint"),
        2
    );
    assert_eq!(
        session.resume_from_for_receiver("r2").expect("checkpoint"),
        0
    );
}

#[test]
fn multi_receiver_completion_tracks_independently() {
    let mut session =
        TransferSession::new(77, vec![1u8; 8], 4, ["a".to_string(), "b".to_string()]).expect("new");

    assert!(!session.all_complete());

    session
        .apply_ack(&Ack {
            transfer_id: 77,
            receiver_id: "a".to_string(),
            next_expected_chunk: 2,
        })
        .expect("ack a done");

    assert!(!session.all_complete());

    session
        .apply_ack(&Ack {
            transfer_id: 77,
            receiver_id: "b".to_string(),
            next_expected_chunk: 2,
        })
        .expect("ack b done");

    assert!(session.all_complete());
}

#[test]
fn invalid_ack_out_of_range_fails() {
    let mut session = TransferSession::new(99, vec![1u8; 5], 2, ["r".to_string()]).expect("new");
    let err = session
        .apply_ack(&Ack {
            transfer_id: 99,
            receiver_id: "r".to_string(),
            next_expected_chunk: 10,
        })
        .expect_err("should reject out-of-range ack");
    assert_eq!(err.to_string(), "ack next_expected_chunk out of range");
}
