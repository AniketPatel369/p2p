use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine as _};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IdentityError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid key bytes")]
    InvalidKey,
    #[error("invalid base64 input")]
    InvalidBase64,
}

#[derive(Clone, Debug)]
pub struct DeviceIdentity {
    signing_key: SigningKey,
}

impl DeviceIdentity {
    /// Generate a new Ed25519 identity.
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        Self { signing_key }
    }

    /// Load identity from a 32-byte secret key file.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, IdentityError> {
        let bytes = fs::read(path)?;
        if bytes.len() != 32 {
            return Err(IdentityError::InvalidKey);
        }

        let mut sk_bytes = [0u8; 32];
        sk_bytes.copy_from_slice(&bytes);
        Ok(Self {
            signing_key: SigningKey::from_bytes(&sk_bytes),
        })
    }

    /// Save identity as a raw 32-byte secret key file with restrictive permissions.
    ///
    /// On Unix, this function ensures mode 0o600.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), IdentityError> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path, self.secret_key_bytes())?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(0o600);
            fs::set_permissions(path, perms)?;
        }

        Ok(())
    }

    /// Returns the device public key.
    pub fn verifying_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }

    /// Returns base64-url-like (no padding) encoded public key.
    pub fn public_key_b64(&self) -> String {
        STANDARD_NO_PAD.encode(self.verifying_key().to_bytes())
    }

    /// Sign handshake or protocol bytes with this identity.
    pub fn sign(&self, message: &[u8]) -> [u8; 64] {
        self.signing_key.sign(message).to_bytes()
    }

    /// Stable fingerprint to display in trust UI.
    ///
    /// Format: SHA-256(pubkey), first 16 bytes, uppercase hex with `:` separator.
    pub fn fingerprint(&self) -> String {
        let pubkey = self.verifying_key().to_bytes();
        let digest = Sha256::digest(pubkey);
        digest[..16]
            .iter()
            .map(|b| format!("{b:02X}"))
            .collect::<Vec<_>>()
            .join(":")
    }

    fn secret_key_bytes(&self) -> [u8; 32] {
        self.signing_key.to_bytes()
    }
}

/// Verify signature bytes using a base64 (no padding) encoded public key.
pub fn verify_signature(public_key_b64: &str, message: &[u8], signature: &[u8; 64]) -> Result<bool, IdentityError> {
    let pk_bytes = STANDARD_NO_PAD
        .decode(public_key_b64)
        .map_err(|_| IdentityError::InvalidBase64)?;
    if pk_bytes.len() != 32 {
        return Err(IdentityError::InvalidKey);
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(&pk_bytes);
    let verifying_key = VerifyingKey::from_bytes(&key).map_err(|_| IdentityError::InvalidKey)?;
    let sig = Signature::from_bytes(signature);
    Ok(verifying_key.verify(message, &sig).is_ok())
}
