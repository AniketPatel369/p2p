use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkIndexEntry {
    pub chunk_index: u32,
    pub offset: u64,
    pub length: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferState {
    Running,
    Paused,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferCheckpoint {
    pub transfer_id: u64,
    pub next_chunk: u32,
    pub state: TransferState,
}

#[derive(Debug, Clone)]
pub struct LargeFileManager {
    pub transfer_id: u64,
    pub total_chunks: u32,
    pub chunk_size: usize,
    checkpoint: TransferCheckpoint,
}

impl LargeFileManager {
    pub fn new(transfer_id: u64, file_size: usize, chunk_size: usize) -> Result<Self, ManagerError> {
        if chunk_size == 0 {
            return Err(ManagerError::InvalidConfig("chunk_size must be > 0"));
        }

        let total_chunks = if file_size == 0 {
            1
        } else {
            file_size.div_ceil(chunk_size) as u32
        };

        Ok(Self {
            transfer_id,
            total_chunks,
            chunk_size,
            checkpoint: TransferCheckpoint {
                transfer_id,
                next_chunk: 0,
                state: TransferState::Running,
            },
        })
    }

    pub fn build_chunk_index(&self, file_size: usize) -> Vec<ChunkIndexEntry> {
        let mut index = Vec::with_capacity(self.total_chunks as usize);
        for chunk_idx in 0..self.total_chunks {
            let offset = chunk_idx as usize * self.chunk_size;
            let remaining = file_size.saturating_sub(offset);
            let length = remaining.min(self.chunk_size) as u32;
            index.push(ChunkIndexEntry {
                chunk_index: chunk_idx,
                offset: offset as u64,
                length,
            });
        }
        index
    }

    pub fn save_checkpoint(&self, path: impl AsRef<Path>) -> Result<(), ManagerError> {
        let p = path.as_ref();
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent)?;
        }
        let state = match self.checkpoint.state {
            TransferState::Running => "running",
            TransferState::Paused => "paused",
            TransferState::Cancelled => "cancelled",
        };
        let content = format!("{}\n{}\n{}\n", self.transfer_id, self.checkpoint.next_chunk, state);
        fs::write(p, content)?;
        Ok(())
    }

    pub fn load_checkpoint(path: impl AsRef<Path>) -> Result<TransferCheckpoint, ManagerError> {
        let content = fs::read_to_string(path)?;
        let mut lines = content.lines();

        let transfer_id = lines
            .next()
            .ok_or(ManagerError::CheckpointFormat)?
            .parse::<u64>()
            .map_err(|_| ManagerError::CheckpointFormat)?;
        let next_chunk = lines
            .next()
            .ok_or(ManagerError::CheckpointFormat)?
            .parse::<u32>()
            .map_err(|_| ManagerError::CheckpointFormat)?;
        let state = match lines.next().ok_or(ManagerError::CheckpointFormat)? {
            "running" => TransferState::Running,
            "paused" => TransferState::Paused,
            "cancelled" => TransferState::Cancelled,
            _ => return Err(ManagerError::CheckpointFormat),
        };

        Ok(TransferCheckpoint {
            transfer_id,
            next_chunk,
            state,
        })
    }

    pub fn checkpoint(&self) -> &TransferCheckpoint {
        &self.checkpoint
    }

    pub fn update_next_chunk(&mut self, next_chunk: u32) -> Result<(), ManagerError> {
        if next_chunk > self.total_chunks {
            return Err(ManagerError::ChunkOutOfRange);
        }
        if self.checkpoint.state == TransferState::Cancelled {
            return Err(ManagerError::InvalidState("cannot update cancelled transfer"));
        }
        if next_chunk > self.checkpoint.next_chunk {
            self.checkpoint.next_chunk = next_chunk;
        }
        Ok(())
    }

    pub fn pause(&mut self) -> Result<(), ManagerError> {
        match self.checkpoint.state {
            TransferState::Running => {
                self.checkpoint.state = TransferState::Paused;
                Ok(())
            }
            TransferState::Paused => Ok(()),
            TransferState::Cancelled => Err(ManagerError::InvalidState("cannot pause cancelled transfer")),
        }
    }

    pub fn resume(&mut self) -> Result<(), ManagerError> {
        match self.checkpoint.state {
            TransferState::Paused => {
                self.checkpoint.state = TransferState::Running;
                Ok(())
            }
            TransferState::Running => Ok(()),
            TransferState::Cancelled => Err(ManagerError::InvalidState("cannot resume cancelled transfer")),
        }
    }

    pub fn cancel(&mut self) {
        self.checkpoint.state = TransferState::Cancelled;
    }
}

pub fn assemble_file(total_chunks: u32, chunks: &BTreeMap<u32, Vec<u8>>) -> Result<Vec<u8>, ManagerError> {
    let mut out = Vec::new();
    for i in 0..total_chunks {
        let chunk = chunks.get(&i).ok_or(ManagerError::MissingChunk(i))?;
        out.extend_from_slice(chunk);
    }
    Ok(out)
}

/// Stable FNV-1a 64-bit integrity tag (lightweight checkpoint validation).
pub fn integrity_tag(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in data {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

pub fn verify_integrity(data: &[u8], expected_tag: u64) -> bool {
    integrity_tag(data) == expected_tag
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManagerError {
    InvalidConfig(&'static str),
    CheckpointFormat,
    ChunkOutOfRange,
    InvalidState(&'static str),
    MissingChunk(u32),
    Io(String),
}

impl std::fmt::Display for ManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ManagerError::InvalidConfig(m) => write!(f, "invalid config: {m}"),
            ManagerError::CheckpointFormat => write!(f, "invalid checkpoint format"),
            ManagerError::ChunkOutOfRange => write!(f, "chunk out of range"),
            ManagerError::InvalidState(m) => write!(f, "invalid state: {m}"),
            ManagerError::MissingChunk(i) => write!(f, "missing chunk {i}"),
            ManagerError::Io(m) => write!(f, "io error: {m}"),
        }
    }
}

impl std::error::Error for ManagerError {}

impl From<std::io::Error> for ManagerError {
    fn from(value: std::io::Error) -> Self {
        ManagerError::Io(value.to_string())
    }
}
