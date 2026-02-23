use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceStatus {
    Online,
    Busy,
    Offline,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceCard {
    pub device_id: String,
    pub display_name: String,
    pub status: DeviceStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IncomingDecision {
    Pending,
    Accepted,
    Declined,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncomingRequestModal {
    pub from_device_id: String,
    pub file_name: String,
    pub size_bytes: u64,
    pub decision: IncomingDecision,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransferState {
    Queued,
    InProgress,
    Completed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferItem {
    pub transfer_id: u64,
    pub target_device_id: String,
    pub file_name: String,
    pub progress_percent: u8,
    pub state: TransferState,
}

#[derive(Debug, Default)]
pub struct DesktopUiState {
    devices: HashMap<String, DeviceCard>,
    incoming_modal: Option<IncomingRequestModal>,
    transfers: HashMap<u64, TransferItem>,
}

impl DesktopUiState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Device grid/cards support.
    pub fn upsert_device_card(&mut self, card: DeviceCard) {
        self.devices.insert(card.device_id.clone(), card);
    }

    pub fn remove_device_card(&mut self, device_id: &str) {
        self.devices.remove(device_id);
    }

    pub fn device_cards(&self) -> Vec<&DeviceCard> {
        let mut items: Vec<&DeviceCard> = self.devices.values().collect();
        items.sort_by(|a, b| a.display_name.cmp(&b.display_name));
        items
    }

    /// Incoming request modal flow.
    pub fn show_incoming_request(&mut self, request: IncomingRequestModal) {
        self.incoming_modal = Some(request);
    }

    pub fn decide_incoming_request(&mut self, decision: IncomingDecision) -> Result<(), UiError> {
        let modal = self.incoming_modal.as_mut().ok_or(UiError::NoIncomingRequest)?;
        modal.decision = decision;
        Ok(())
    }

    pub fn clear_incoming_request(&mut self) {
        self.incoming_modal = None;
    }

    pub fn incoming_request(&self) -> Option<&IncomingRequestModal> {
        self.incoming_modal.as_ref()
    }

    /// Transfer dashboard support.
    pub fn add_transfer(&mut self, item: TransferItem) {
        self.transfers.insert(item.transfer_id, item);
    }

    pub fn update_transfer_progress(&mut self, transfer_id: u64, progress_percent: u8) -> Result<(), UiError> {
        let item = self
            .transfers
            .get_mut(&transfer_id)
            .ok_or(UiError::TransferNotFound)?;

        let progress = progress_percent.min(100);
        item.progress_percent = progress;

        if progress == 100 && item.state == TransferState::InProgress {
            item.state = TransferState::Completed;
        }

        Ok(())
    }

    pub fn set_transfer_state(&mut self, transfer_id: u64, state: TransferState) -> Result<(), UiError> {
        let item = self
            .transfers
            .get_mut(&transfer_id)
            .ok_or(UiError::TransferNotFound)?;
        item.state = state;
        Ok(())
    }

    pub fn transfers(&self) -> Vec<&TransferItem> {
        let mut items: Vec<&TransferItem> = self.transfers.values().collect();
        items.sort_by_key(|t| t.transfer_id);
        items
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiError {
    NoIncomingRequest,
    TransferNotFound,
}

impl std::fmt::Display for UiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UiError::NoIncomingRequest => write!(f, "no incoming request modal is open"),
            UiError::TransferNotFound => write!(f, "transfer not found"),
        }
    }
}

impl std::error::Error for UiError {}
