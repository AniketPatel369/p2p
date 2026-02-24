use large_file_manager::{
    assemble_file, integrity_tag, verify_integrity, LargeFileManager, TransferState,
};
use std::collections::BTreeMap;

#[test]
fn chunk_index_is_built_correctly() {
    let mgr = LargeFileManager::new(1, 10, 4).expect("manager");
    let index = mgr.build_chunk_index(10);

    assert_eq!(index.len(), 3);
    assert_eq!(index[0].offset, 0);
    assert_eq!(index[0].length, 4);
    assert_eq!(index[2].offset, 8);
    assert_eq!(index[2].length, 2);
}

#[test]
fn checkpoint_roundtrip_works() {
    let mut mgr = LargeFileManager::new(7, 100, 16).expect("manager");
    mgr.update_next_chunk(3).expect("update");
    mgr.pause().expect("pause");

    let temp = std::env::temp_dir().join("p2p_large_file_checkpoint_test.chk");
    mgr.save_checkpoint(&temp).expect("save");

    let loaded = LargeFileManager::load_checkpoint(&temp).expect("load");
    std::fs::remove_file(temp).ok();

    assert_eq!(loaded.transfer_id, 7);
    assert_eq!(loaded.next_chunk, 3);
    assert_eq!(loaded.state, TransferState::Paused);
}

#[test]
fn pause_resume_cancel_state_machine() {
    let mut mgr = LargeFileManager::new(8, 20, 4).expect("manager");
    assert_eq!(mgr.checkpoint().state, TransferState::Running);

    mgr.pause().expect("pause");
    assert_eq!(mgr.checkpoint().state, TransferState::Paused);

    mgr.resume().expect("resume");
    assert_eq!(mgr.checkpoint().state, TransferState::Running);

    mgr.cancel();
    assert_eq!(mgr.checkpoint().state, TransferState::Cancelled);
    assert!(mgr.resume().is_err());
}

#[test]
fn assemble_and_verify_integrity() {
    let mut chunks = BTreeMap::new();
    chunks.insert(0, b"hello ".to_vec());
    chunks.insert(1, b"world".to_vec());

    let file = assemble_file(2, &chunks).expect("assemble");
    let tag = integrity_tag(&file);

    assert_eq!(file, b"hello world".to_vec());
    assert!(verify_integrity(&file, tag));
    assert!(!verify_integrity(&file, tag.wrapping_add(1)));
}

#[test]
fn missing_chunk_fails_assembly() {
    let mut chunks = BTreeMap::new();
    chunks.insert(0, b"only first".to_vec());

    let err = assemble_file(2, &chunks).expect_err("should fail");
    assert_eq!(err.to_string(), "missing chunk 1");
}
