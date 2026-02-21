use desktop_ui::{
    DesktopUiState, DeviceCard, DeviceStatus, IncomingDecision, IncomingRequestModal, TransferItem,
    TransferState,
};

#[test]
fn device_cards_are_sorted_for_grid_rendering() {
    let mut ui = DesktopUiState::new();
    ui.upsert_device_card(DeviceCard {
        device_id: "b".into(),
        display_name: "Zeta Mac".into(),
        status: DeviceStatus::Online,
    });
    ui.upsert_device_card(DeviceCard {
        device_id: "a".into(),
        display_name: "Alpha iPhone".into(),
        status: DeviceStatus::Busy,
    });

    let cards = ui.device_cards();
    assert_eq!(cards[0].display_name, "Alpha iPhone");
    assert_eq!(cards[1].display_name, "Zeta Mac");
}

#[test]
fn incoming_request_modal_accept_decline_flow() {
    let mut ui = DesktopUiState::new();
    ui.show_incoming_request(IncomingRequestModal {
        from_device_id: "peer-1".into(),
        file_name: "photo.jpg".into(),
        size_bytes: 1024,
        decision: IncomingDecision::Pending,
    });

    ui.decide_incoming_request(IncomingDecision::Accepted)
        .expect("accept should work");
    assert_eq!(
        ui.incoming_request().expect("modal").decision,
        IncomingDecision::Accepted
    );

    ui.clear_incoming_request();
    assert!(ui.incoming_request().is_none());
}

#[test]
fn transfer_dashboard_progress_completion_and_failure() {
    let mut ui = DesktopUiState::new();
    ui.add_transfer(TransferItem {
        transfer_id: 10,
        target_device_id: "peer-2".into(),
        file_name: "video.mp4".into(),
        progress_percent: 0,
        state: TransferState::InProgress,
    });

    ui.update_transfer_progress(10, 60).expect("progress update");
    assert_eq!(ui.transfers()[0].progress_percent, 60);
    assert_eq!(ui.transfers()[0].state, TransferState::InProgress);

    ui.update_transfer_progress(10, 100)
        .expect("complete progress update");
    assert_eq!(ui.transfers()[0].state, TransferState::Completed);

    ui.set_transfer_state(10, TransferState::Failed)
        .expect("manual fail state");
    assert_eq!(ui.transfers()[0].state, TransferState::Failed);
}

#[test]
fn updating_unknown_transfer_fails() {
    let mut ui = DesktopUiState::new();
    let err = ui
        .update_transfer_progress(999, 80)
        .expect_err("unknown transfer should fail");
    assert_eq!(err.to_string(), "transfer not found");
}
