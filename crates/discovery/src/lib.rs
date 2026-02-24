use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};

const MAGIC: &[u8; 4] = b"P2PD";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Announcement {
    pub device_id: String,
    pub public_key_b64: String,
    pub display_name: String,
    pub port: u16,
}

impl Announcement {
    pub fn encode(&self) -> Vec<u8> {
        // Simple length-prefixed binary format:
        // MAGIC | port(u16 be) | len+device_id | len+public_key | len+display_name
        let mut out = Vec::with_capacity(4 + 2 + 2 + self.device_id.len() + 2 + self.public_key_b64.len() + 2 + self.display_name.len());
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&self.port.to_be_bytes());
        push_str(&mut out, &self.device_id);
        push_str(&mut out, &self.public_key_b64);
        push_str(&mut out, &self.display_name);
        out
    }

    pub fn decode(input: &[u8]) -> Result<Self, DiscoveryError> {
        if input.len() < 6 || &input[..4] != MAGIC {
            return Err(DiscoveryError::InvalidPacket("bad magic/header"));
        }

        let port = u16::from_be_bytes([input[4], input[5]]);
        let mut idx = 6;
        let device_id = read_str(input, &mut idx)?;
        let public_key_b64 = read_str(input, &mut idx)?;
        let display_name = read_str(input, &mut idx)?;

        if idx != input.len() {
            return Err(DiscoveryError::InvalidPacket("trailing bytes"));
        }

        Ok(Self {
            device_id,
            public_key_b64,
            display_name,
            port,
        })
    }
}

#[derive(Debug, Clone)]
pub struct PeerEntry {
    pub announcement: Announcement,
    pub source: SocketAddr,
    pub last_seen: Instant,
}

#[derive(Debug)]
pub struct PeerRegistry {
    peers: HashMap<String, PeerEntry>,
    ttl: Duration,
}

impl PeerRegistry {
    pub fn new(ttl: Duration) -> Self {
        Self {
            peers: HashMap::new(),
            ttl,
        }
    }

    pub fn upsert(&mut self, announcement: Announcement, source: SocketAddr, now: Instant) {
        self.peers.insert(
            announcement.device_id.clone(),
            PeerEntry {
                announcement,
                source,
                last_seen: now,
            },
        );
    }

    pub fn expire(&mut self, now: Instant) {
        let ttl = self.ttl;
        self.peers.retain(|_, p| now.duration_since(p.last_seen) <= ttl);
    }

    pub fn peers(&self) -> Vec<&PeerEntry> {
        self.peers.values().collect()
    }

    pub fn len(&self) -> usize {
        self.peers.len()
    }
}

#[derive(Debug)]
pub struct DiscoveryService {
    socket: UdpSocket,
}

impl DiscoveryService {
    pub fn bind(bind_addr: SocketAddr) -> Result<Self, DiscoveryError> {
        let socket = UdpSocket::bind(bind_addr)?;
        socket.set_nonblocking(false)?;
        Ok(Self { socket })
    }

    pub fn local_addr(&self) -> Result<SocketAddr, DiscoveryError> {
        Ok(self.socket.local_addr()?)
    }

    pub fn send_announcement(&self, target: SocketAddr, announcement: &Announcement) -> Result<usize, DiscoveryError> {
        Ok(self.socket.send_to(&announcement.encode(), target)?)
    }

    pub fn recv_announcement(&self, max_size: usize) -> Result<(Announcement, SocketAddr), DiscoveryError> {
        let mut buf = vec![0u8; max_size];
        let (n, src) = self.socket.recv_from(&mut buf)?;
        let ann = Announcement::decode(&buf[..n])?;
        Ok((ann, src))
    }
}

#[derive(Debug)]
pub enum DiscoveryError {
    Io(std::io::Error),
    InvalidPacket(&'static str),
    InvalidLength,
}

impl std::fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiscoveryError::Io(e) => write!(f, "I/O error: {e}"),
            DiscoveryError::InvalidPacket(msg) => write!(f, "invalid packet: {msg}"),
            DiscoveryError::InvalidLength => write!(f, "invalid string length"),
        }
    }
}

impl std::error::Error for DiscoveryError {}

impl From<std::io::Error> for DiscoveryError {
    fn from(value: std::io::Error) -> Self {
        DiscoveryError::Io(value)
    }
}

fn push_str(out: &mut Vec<u8>, value: &str) {
    let bytes = value.as_bytes();
    let len = u16::try_from(bytes.len()).unwrap_or(u16::MAX);
    out.extend_from_slice(&len.to_be_bytes());
    out.extend_from_slice(&bytes[..usize::from(len)]);
}

fn read_str(input: &[u8], idx: &mut usize) -> Result<String, DiscoveryError> {
    if *idx + 2 > input.len() {
        return Err(DiscoveryError::InvalidLength);
    }
    let len = u16::from_be_bytes([input[*idx], input[*idx + 1]]) as usize;
    *idx += 2;
    if *idx + len > input.len() {
        return Err(DiscoveryError::InvalidLength);
    }
    let s = std::str::from_utf8(&input[*idx..*idx + len])
        .map_err(|_| DiscoveryError::InvalidPacket("utf8 error"))?
        .to_string();
    *idx += len;
    Ok(s)
}
