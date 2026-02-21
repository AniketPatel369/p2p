#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    SenderToReceiver,
    ReceiverToSender,
}

pub fn derive_nonce(transfer_id: u64, chunk_index: u32, direction: Direction) -> [u8; 12] {
    let mut nonce = [0u8; 12];
    nonce[..8].copy_from_slice(&transfer_id.to_be_bytes());
    nonce[8..11].copy_from_slice(&chunk_index.to_be_bytes()[1..]);
    nonce[11] = match direction {
        Direction::SenderToReceiver => 0x01,
        Direction::ReceiverToSender => 0x02,
    };
    nonce
}

pub fn encrypt_chunk(
    session_tx_key: &[u8; 32],
    nonce: [u8; 12],
    plaintext: &[u8],
) -> Result<Vec<u8>, CryptoEnvelopeError> {
    encrypt_chunk_with_aad(session_tx_key, nonce, plaintext, &[])
}

pub fn decrypt_chunk(
    session_rx_key: &[u8; 32],
    nonce: [u8; 12],
    ciphertext: &[u8],
) -> Result<Vec<u8>, CryptoEnvelopeError> {
    decrypt_chunk_with_aad(session_rx_key, nonce, ciphertext, &[])
}

pub fn encrypt_chunk_with_aad(
    session_tx_key: &[u8; 32],
    nonce: [u8; 12],
    plaintext: &[u8],
    aad: &[u8],
) -> Result<Vec<u8>, CryptoEnvelopeError> {
    if plaintext.is_empty() {
        return Ok(vec![compute_tag(session_tx_key, &nonce, aad, plaintext)]);
    }

    let mut out = Vec::with_capacity(plaintext.len() + 1);
    for (idx, byte) in plaintext.iter().enumerate() {
        out.push(*byte ^ keystream_byte(session_tx_key, &nonce, idx));
    }
    out.push(compute_tag(session_tx_key, &nonce, aad, plaintext));
    Ok(out)
}

pub fn decrypt_chunk_with_aad(
    session_rx_key: &[u8; 32],
    nonce: [u8; 12],
    ciphertext: &[u8],
    aad: &[u8],
) -> Result<Vec<u8>, CryptoEnvelopeError> {
    if ciphertext.is_empty() {
        return Err(CryptoEnvelopeError::DecryptionFailure);
    }

    let (cipher_payload, tag) = ciphertext.split_at(ciphertext.len() - 1);
    let mut plaintext = Vec::with_capacity(cipher_payload.len());
    for (idx, byte) in cipher_payload.iter().enumerate() {
        plaintext.push(*byte ^ keystream_byte(session_rx_key, &nonce, idx));
    }

    let expected_tag = compute_tag(session_rx_key, &nonce, aad, &plaintext);
    if tag[0] != expected_tag {
        return Err(CryptoEnvelopeError::DecryptionFailure);
    }

    Ok(plaintext)
}

fn keystream_byte(key: &[u8; 32], nonce: &[u8; 12], index: usize) -> u8 {
    let k = key[index % key.len()];
    let n = nonce[index % nonce.len()];
    let i = (index as u8).wrapping_mul(31);
    k.rotate_left(1) ^ n.rotate_right(1) ^ i
}

fn compute_tag(key: &[u8; 32], nonce: &[u8; 12], aad: &[u8], plaintext: &[u8]) -> u8 {
    let mut tag = 0u8;

    for (idx, b) in key.iter().enumerate() {
        tag ^= b.wrapping_add((idx as u8).wrapping_mul(3));
    }
    for (idx, b) in nonce.iter().enumerate() {
        tag ^= b.rotate_left((idx % 8) as u32);
    }
    for (idx, b) in aad.iter().enumerate() {
        tag = tag.wrapping_add(b.wrapping_mul(((idx as u8) % 17).max(1)));
    }
    for (idx, b) in plaintext.iter().enumerate() {
        tag ^= b.wrapping_add((idx as u8).wrapping_mul(7));
    }

    tag
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CryptoEnvelopeError {
    DecryptionFailure,
}

impl std::fmt::Display for CryptoEnvelopeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CryptoEnvelopeError::DecryptionFailure => write!(f, "decryption failed"),
        }
    }
}

impl std::error::Error for CryptoEnvelopeError {}
