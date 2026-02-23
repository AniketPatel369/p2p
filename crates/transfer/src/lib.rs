use crypto_envelope::{decrypt_chunk, derive_nonce, encrypt_chunk, Direction};
use std::collections::HashMap;

const MAGIC_V1: &[u8; 4] = b"P2PF";
const MAGIC_V2: &[u8; 4] = b"P2PE";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferChunk {
    pub transfer_id: u64,
    pub chunk_index: u32,
    pub total_chunks: u32,
    pub payload: Vec<u8>,
}

impl TransferChunk {
    pub fn encode(&self) -> Vec<u8> {
        let payload_len = u32::try_from(self.payload.len()).unwrap_or(u32::MAX);
        let mut out = Vec::with_capacity(4 + 8 + 4 + 4 + 4 + payload_len as usize);
        out.extend_from_slice(MAGIC_V1);
        out.extend_from_slice(&self.transfer_id.to_be_bytes());
        out.extend_from_slice(&self.chunk_index.to_be_bytes());
        out.extend_from_slice(&self.total_chunks.to_be_bytes());
        out.extend_from_slice(&payload_len.to_be_bytes());
        out.extend_from_slice(&self.payload[..payload_len as usize]);
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, TransferError> {
        if bytes.len() < 24 || &bytes[..4] != MAGIC_V1 {
            return Err(TransferError::InvalidFrame("bad header"));
        }

        let transfer_id = u64::from_be_bytes(bytes[4..12].try_into().expect("slice len"));
        let chunk_index = u32::from_be_bytes(bytes[12..16].try_into().expect("slice len"));
        let total_chunks = u32::from_be_bytes(bytes[16..20].try_into().expect("slice len"));
        let payload_len = u32::from_be_bytes(bytes[20..24].try_into().expect("slice len")) as usize;

        if bytes.len() != 24 + payload_len {
            return Err(TransferError::InvalidFrame("invalid payload length"));
        }
        if total_chunks == 0 || chunk_index >= total_chunks {
            return Err(TransferError::InvalidFrame("invalid chunk bounds"));
        }

        Ok(Self {
            transfer_id,
            chunk_index,
            total_chunks,
            payload: bytes[24..].to_vec(),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncryptionFlag {
    Plaintext,
    Encrypted,
}

impl EncryptionFlag {
    fn as_u8(self) -> u8 {
        match self {
            EncryptionFlag::Plaintext => 0,
            EncryptionFlag::Encrypted => 1,
        }
    }

    fn from_u8(v: u8) -> Result<Self, TransferError> {
        match v {
            0 => Ok(EncryptionFlag::Plaintext),
            1 => Ok(EncryptionFlag::Encrypted),
            _ => Err(TransferError::InvalidFrame("invalid encryption flag")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferChunkV2 {
    pub protocol_version: u8,
    pub encryption_flag: EncryptionFlag,
    pub transfer_id: u64,
    pub chunk_index: u32,
    pub total_chunks: u32,
    pub nonce: [u8; 12],
    pub aad: Vec<u8>,
    pub payload: Vec<u8>,
}

impl TransferChunkV2 {
    pub fn encode(&self) -> Vec<u8> {
        let aad_len = u16::try_from(self.aad.len()).unwrap_or(u16::MAX);
        let payload_len = u32::try_from(self.payload.len()).unwrap_or(u32::MAX);

        let mut out = Vec::with_capacity(
            4 + 1 + 1 + 8 + 4 + 4 + 12 + 2 + 4 + aad_len as usize + payload_len as usize,
        );
        out.extend_from_slice(MAGIC_V2);
        out.push(self.protocol_version);
        out.push(self.encryption_flag.as_u8());
        out.extend_from_slice(&self.transfer_id.to_be_bytes());
        out.extend_from_slice(&self.chunk_index.to_be_bytes());
        out.extend_from_slice(&self.total_chunks.to_be_bytes());
        out.extend_from_slice(&self.nonce);
        out.extend_from_slice(&aad_len.to_be_bytes());
        out.extend_from_slice(&payload_len.to_be_bytes());
        out.extend_from_slice(&self.aad[..aad_len as usize]);
        out.extend_from_slice(&self.payload[..payload_len as usize]);
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, TransferError> {
        let min_header = 4 + 1 + 1 + 8 + 4 + 4 + 12 + 2 + 4;
        if bytes.len() < min_header || &bytes[..4] != MAGIC_V2 {
            return Err(TransferError::InvalidFrame("bad v2 header"));
        }

        let protocol_version = bytes[4];
        let encryption_flag = EncryptionFlag::from_u8(bytes[5])?;
        let transfer_id = u64::from_be_bytes(bytes[6..14].try_into().expect("slice len"));
        let chunk_index = u32::from_be_bytes(bytes[14..18].try_into().expect("slice len"));
        let total_chunks = u32::from_be_bytes(bytes[18..22].try_into().expect("slice len"));

        if protocol_version != 2 {
            return Err(TransferError::InvalidFrame("unsupported protocol version"));
        }
        if total_chunks == 0 || chunk_index >= total_chunks {
            return Err(TransferError::InvalidFrame("invalid chunk bounds"));
        }

        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(&bytes[22..34]);

        let aad_len = u16::from_be_bytes(bytes[34..36].try_into().expect("slice len")) as usize;
        let payload_len = u32::from_be_bytes(bytes[36..40].try_into().expect("slice len")) as usize;

        let expected_len = min_header + aad_len + payload_len;
        if bytes.len() != expected_len {
            return Err(TransferError::InvalidFrame("invalid payload length"));
        }

        let aad_start = min_header;
        let payload_start = aad_start + aad_len;

        Ok(Self {
            protocol_version,
            encryption_flag,
            transfer_id,
            chunk_index,
            total_chunks,
            nonce,
            aad: bytes[aad_start..payload_start].to_vec(),
            payload: bytes[payload_start..].to_vec(),
        })
    }
}

pub fn encrypt_chunk_frame(
    chunk: &TransferChunk,
    session_tx_key: &[u8; 32],
) -> Result<TransferChunkV2, TransferError> {
    let nonce = derive_nonce(
        chunk.transfer_id,
        chunk.chunk_index,
        Direction::SenderToReceiver,
    );
    let aad = transfer_chunk_aad(chunk);
    let ciphertext = encrypt_chunk(session_tx_key, nonce, &chunk.payload)
        .map_err(|_| TransferError::Crypto("failed to encrypt chunk payload"))?;

    Ok(TransferChunkV2 {
        protocol_version: 2,
        encryption_flag: EncryptionFlag::Encrypted,
        transfer_id: chunk.transfer_id,
        chunk_index: chunk.chunk_index,
        total_chunks: chunk.total_chunks,
        nonce,
        aad,
        payload: ciphertext,
    })
}

pub fn decrypt_chunk_frame(
    frame: &TransferChunkV2,
    session_rx_key: &[u8; 32],
) -> Result<TransferChunk, TransferError> {
    if frame.encryption_flag != EncryptionFlag::Encrypted {
        return Err(TransferError::InvalidFrame("expected encrypted frame"));
    }

    let plaintext = decrypt_chunk(session_rx_key, frame.nonce, &frame.payload)
        .map_err(|_| TransferError::Crypto("failed to decrypt chunk payload"))?;

    Ok(TransferChunk {
        transfer_id: frame.transfer_id,
        chunk_index: frame.chunk_index,
        total_chunks: frame.total_chunks,
        payload: plaintext,
    })
}

pub fn transfer_chunk_aad(chunk: &TransferChunk) -> Vec<u8> {
    let mut aad = Vec::with_capacity(8 + 4 + 4);
    aad.extend_from_slice(&chunk.transfer_id.to_be_bytes());
    aad.extend_from_slice(&chunk.chunk_index.to_be_bytes());
    aad.extend_from_slice(&chunk.total_chunks.to_be_bytes());
    aad
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionedTransferChunk {
    V1(TransferChunk),
    V2(TransferChunkV2),
}

impl VersionedTransferChunk {
    pub fn decode(bytes: &[u8]) -> Result<Self, TransferError> {
        if bytes.len() < 4 {
            return Err(TransferError::InvalidFrame("bad header"));
        }

        if &bytes[..4] == MAGIC_V1 {
            Ok(VersionedTransferChunk::V1(TransferChunk::decode(bytes)?))
        } else if &bytes[..4] == MAGIC_V2 {
            Ok(VersionedTransferChunk::V2(TransferChunkV2::decode(bytes)?))
        } else {
            Err(TransferError::InvalidFrame("bad header"))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ack {
    pub transfer_id: u64,
    pub receiver_id: String,
    pub next_expected_chunk: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceiverProgress {
    pub receiver_id: String,
    pub acked_up_to_exclusive: u32,
    pub total_chunks: u32,
}

impl ReceiverProgress {
    pub fn percent(&self) -> u8 {
        if self.total_chunks == 0 {
            return 0;
        }
        let pct = (self.acked_up_to_exclusive as f64 / self.total_chunks as f64) * 100.0;
        pct.min(100.0) as u8
    }

    pub fn is_complete(&self) -> bool {
        self.acked_up_to_exclusive >= self.total_chunks
    }
}

#[derive(Debug, Clone)]
pub struct TransferSession {
    transfer_id: u64,
    total_chunks: u32,
    chunk_size: usize,
    data: Vec<u8>,
    receivers: HashMap<String, ReceiverProgress>,
}

impl TransferSession {
    pub fn new(
        transfer_id: u64,
        data: Vec<u8>,
        chunk_size: usize,
        receiver_ids: impl IntoIterator<Item = String>,
    ) -> Result<Self, TransferError> {
        if chunk_size == 0 {
            return Err(TransferError::InvalidConfig("chunk_size must be > 0"));
        }

        let total_chunks = if data.is_empty() {
            1
        } else {
            data.len().div_ceil(chunk_size) as u32
        };

        let mut receivers = HashMap::new();
        for id in receiver_ids {
            receivers.insert(
                id.clone(),
                ReceiverProgress {
                    receiver_id: id,
                    acked_up_to_exclusive: 0,
                    total_chunks,
                },
            );
        }

        Ok(Self {
            transfer_id,
            total_chunks,
            chunk_size,
            data,
            receivers,
        })
    }

    pub fn chunk_for(&self, chunk_index: u32) -> Result<TransferChunk, TransferError> {
        if chunk_index >= self.total_chunks {
            return Err(TransferError::ChunkOutOfRange);
        }

        let start = chunk_index as usize * self.chunk_size;
        let end = ((chunk_index as usize + 1) * self.chunk_size).min(self.data.len());

        let payload = if self.data.is_empty() {
            Vec::new()
        } else {
            self.data[start..end].to_vec()
        };

        Ok(TransferChunk {
            transfer_id: self.transfer_id,
            chunk_index,
            total_chunks: self.total_chunks,
            payload,
        })
    }

    pub fn apply_ack(&mut self, ack: &Ack) -> Result<(), TransferError> {
        if ack.transfer_id != self.transfer_id {
            return Err(TransferError::WrongTransfer);
        }

        let receiver = self
            .receivers
            .get_mut(&ack.receiver_id)
            .ok_or(TransferError::UnknownReceiver)?;

        if ack.next_expected_chunk > self.total_chunks {
            return Err(TransferError::AckOutOfRange);
        }

        // Monotonic forward-only checkpointing for resume safety.
        if ack.next_expected_chunk > receiver.acked_up_to_exclusive {
            receiver.acked_up_to_exclusive = ack.next_expected_chunk;
        }

        Ok(())
    }

    pub fn resume_from_for_receiver(&self, receiver_id: &str) -> Result<u32, TransferError> {
        let receiver = self
            .receivers
            .get(receiver_id)
            .ok_or(TransferError::UnknownReceiver)?;
        Ok(receiver.acked_up_to_exclusive)
    }

    pub fn progress_for(&self, receiver_id: &str) -> Result<ReceiverProgress, TransferError> {
        self.receivers
            .get(receiver_id)
            .cloned()
            .ok_or(TransferError::UnknownReceiver)
    }

    pub fn all_complete(&self) -> bool {
        self.receivers.values().all(ReceiverProgress::is_complete)
    }

    pub fn total_chunks(&self) -> u32 {
        self.total_chunks
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransferError {
    InvalidFrame(&'static str),
    InvalidConfig(&'static str),
    ChunkOutOfRange,
    WrongTransfer,
    UnknownReceiver,
    AckOutOfRange,
    Crypto(&'static str),
}

impl std::fmt::Display for TransferError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransferError::InvalidFrame(m) => write!(f, "invalid frame: {m}"),
            TransferError::InvalidConfig(m) => write!(f, "invalid config: {m}"),
            TransferError::ChunkOutOfRange => write!(f, "chunk index out of range"),
            TransferError::WrongTransfer => write!(f, "ack for wrong transfer"),
            TransferError::UnknownReceiver => write!(f, "unknown receiver"),
            TransferError::AckOutOfRange => write!(f, "ack next_expected_chunk out of range"),
            TransferError::Crypto(m) => write!(f, "crypto error: {m}"),
        }
    }
}

impl std::error::Error for TransferError {}
