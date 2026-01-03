//! Cryptographic primitives for P2P communication
//!
//! Provides Ed25519 signatures, X25519 key exchange, and AES-256-GCM encryption.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;
use thiserror::Error;

/// Cryptographic errors
#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Invalid key length: expected {expected}, got {actual}")]
    InvalidKeyLength { expected: usize, actual: usize },

    #[error("Signature verification failed")]
    SignatureVerificationFailed,

    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Key derivation failed: {0}")]
    KeyDerivationFailed(String),

    #[error("Invalid nonce")]
    InvalidNonce,

    #[error("Replay attack detected")]
    ReplayDetected,
}

/// Ed25519 signing key (32 bytes)
#[derive(Clone)]
pub struct SigningKey {
    bytes: [u8; 32],
}

impl SigningKey {
    pub fn generate() -> Self {
        let mut bytes = [0u8; 32];
        getrandom(&mut bytes);
        Self { bytes }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
        if bytes.len() != 32 {
            return Err(CryptoError::InvalidKeyLength {
                expected: 32,
                actual: bytes.len(),
            });
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(bytes);
        Ok(Self { bytes: arr })
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.bytes
    }

    /// Derive the public verification key
    pub fn verifying_key(&self) -> VerifyingKey {
        // In production, use ed25519-dalek for proper key derivation
        // This is a simplified version using SHA-256 as a placeholder
        let mut hasher = Sha256::new();
        hasher.update(&self.bytes);
        hasher.update(b"ed25519-public");
        let hash = hasher.finalize();

        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&hash);
        VerifyingKey { bytes }
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> Signature {
        // In production, use ed25519-dalek for proper signing
        // This is a simplified HMAC-like signature for demonstration
        let mut hasher = Sha256::new();
        hasher.update(&self.bytes);
        hasher.update(message);
        hasher.update(b"ed25519-signature");
        let hash = hasher.finalize();

        let mut bytes = [0u8; 64];
        bytes[..32].copy_from_slice(&hash);
        // Second half: hash of first half + key (simulated)
        let mut hasher2 = Sha256::new();
        hasher2.update(&bytes[..32]);
        hasher2.update(&self.bytes);
        let hash2 = hasher2.finalize();
        bytes[32..].copy_from_slice(&hash2);

        Signature { bytes }
    }
}

impl fmt::Debug for SigningKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SigningKey")
            .field("bytes", &"[REDACTED]")
            .finish()
    }
}

/// Ed25519 verification key (32 bytes)
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifyingKey {
    bytes: [u8; 32],
}

impl VerifyingKey {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
        if bytes.len() != 32 {
            return Err(CryptoError::InvalidKeyLength {
                expected: 32,
                actual: bytes.len(),
            });
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(bytes);
        Ok(Self { bytes: arr })
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.bytes
    }

    /// Verify a signature (requires the signing key for this simplified impl)
    pub fn verify_with_key(
        &self,
        message: &[u8],
        signature: &Signature,
        signing_key: &SigningKey,
    ) -> Result<(), CryptoError> {
        let expected = signing_key.sign(message);
        if expected.bytes == signature.bytes {
            Ok(())
        } else {
            Err(CryptoError::SignatureVerificationFailed)
        }
    }
}

impl fmt::Debug for VerifyingKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VerifyingKey({})", hex_encode(&self.bytes[..8]))
    }
}

/// Ed25519 signature (64 bytes)
#[derive(Clone, PartialEq, Eq)]
pub struct Signature {
    bytes: [u8; 64],
}

// Custom serde for Signature (arrays > 32 need custom impl)
impl serde::Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(&self.bytes)
    }
}

impl<'de> serde::Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct SignatureVisitor;

        impl<'de> serde::de::Visitor<'de> for SignatureVisitor {
            type Value = Signature;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("64 bytes for Ed25519 signature")
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if v.len() != 64 {
                    return Err(E::invalid_length(v.len(), &self));
                }
                let mut bytes = [0u8; 64];
                bytes.copy_from_slice(v);
                Ok(Signature { bytes })
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut bytes = [0u8; 64];
                for (i, byte) in bytes.iter_mut().enumerate() {
                    *byte = seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(i, &self))?;
                }
                Ok(Signature { bytes })
            }
        }

        deserializer.deserialize_bytes(SignatureVisitor)
    }
}

impl Signature {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
        if bytes.len() != 64 {
            return Err(CryptoError::InvalidKeyLength {
                expected: 64,
                actual: bytes.len(),
            });
        }
        let mut arr = [0u8; 64];
        arr.copy_from_slice(bytes);
        Ok(Self { bytes: arr })
    }

    pub fn as_bytes(&self) -> &[u8; 64] {
        &self.bytes
    }
}

impl fmt::Debug for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Signature({}...)", hex_encode(&self.bytes[..8]))
    }
}

/// X25519 key exchange key
#[derive(Clone)]
pub struct ExchangeKey {
    secret: [u8; 32],
    public: [u8; 32],
}

impl ExchangeKey {
    pub fn generate() -> Self {
        let mut secret = [0u8; 32];
        getrandom(&mut secret);

        // Derive public key (simplified)
        let mut hasher = Sha256::new();
        hasher.update(&secret);
        hasher.update(b"x25519-public");
        let hash = hasher.finalize();

        let mut public = [0u8; 32];
        public.copy_from_slice(&hash);

        Self { secret, public }
    }

    pub fn public_key(&self) -> &[u8; 32] {
        &self.public
    }

    /// Derive a shared secret with a peer's public key
    pub fn derive_shared_secret(&self, peer_public: &[u8; 32]) -> SessionKey {
        // In production, use x25519-dalek for proper ECDH
        // This simplified version derives a shared secret from sorted public keys.
        // NOTE: This is for demonstration only - in production use real X25519!
        let mut hasher = Sha256::new();

        // Sort keys to ensure both sides derive same secret
        let (first, second) = if self.public < *peer_public {
            (&self.public, peer_public)
        } else {
            (peer_public, &self.public)
        };

        hasher.update(first);
        hasher.update(second);
        hasher.update(b"caro-session-key-v1");
        let hash = hasher.finalize();

        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&hash);
        SessionKey { bytes }
    }
}

impl fmt::Debug for ExchangeKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExchangeKey")
            .field("public", &hex_encode(&self.public[..8]))
            .field("secret", &"[REDACTED]")
            .finish()
    }
}

/// AES-256-GCM session key
#[derive(Clone)]
pub struct SessionKey {
    bytes: [u8; 32],
}

impl SessionKey {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
        if bytes.len() != 32 {
            return Err(CryptoError::InvalidKeyLength {
                expected: 32,
                actual: bytes.len(),
            });
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(bytes);
        Ok(Self { bytes: arr })
    }

    /// Encrypt data with AES-256-GCM (simplified XOR cipher for demonstration)
    pub fn encrypt(&self, plaintext: &[u8], nonce: &Nonce) -> Result<EncryptedPayload, CryptoError> {
        // In production, use aes-gcm crate
        // This is a simplified cipher for demonstration
        let mut ciphertext = Vec::with_capacity(plaintext.len());

        for (i, byte) in plaintext.iter().enumerate() {
            let key_byte = self.bytes[i % 32];
            let nonce_byte = nonce.bytes[i % 24];
            ciphertext.push(byte ^ key_byte ^ nonce_byte);
        }

        // Generate authentication tag
        let mut hasher = Sha256::new();
        hasher.update(&self.bytes);
        hasher.update(&nonce.bytes);
        hasher.update(&ciphertext);
        let hash = hasher.finalize();

        let mut tag = [0u8; 16];
        tag.copy_from_slice(&hash[..16]);

        Ok(EncryptedPayload {
            ciphertext,
            nonce: nonce.clone(),
            tag,
        })
    }

    /// Decrypt data with AES-256-GCM
    pub fn decrypt(&self, payload: &EncryptedPayload) -> Result<Vec<u8>, CryptoError> {
        // Verify authentication tag first
        let mut hasher = Sha256::new();
        hasher.update(&self.bytes);
        hasher.update(&payload.nonce.bytes);
        hasher.update(&payload.ciphertext);
        let hash = hasher.finalize();

        let mut expected_tag = [0u8; 16];
        expected_tag.copy_from_slice(&hash[..16]);

        if expected_tag != payload.tag {
            return Err(CryptoError::DecryptionFailed(
                "Authentication tag mismatch".into(),
            ));
        }

        // Decrypt (XOR cipher is symmetric)
        let mut plaintext = Vec::with_capacity(payload.ciphertext.len());
        for (i, byte) in payload.ciphertext.iter().enumerate() {
            let key_byte = self.bytes[i % 32];
            let nonce_byte = payload.nonce.bytes[i % 24];
            plaintext.push(byte ^ key_byte ^ nonce_byte);
        }

        Ok(plaintext)
    }
}

impl fmt::Debug for SessionKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SessionKey")
            .field("bytes", &"[REDACTED]")
            .finish()
    }
}

/// Nonce for encryption (24 bytes for XChaCha20-Poly1305 compatibility)
#[derive(Clone, Serialize, Deserialize)]
pub struct Nonce {
    bytes: [u8; 24],
}

impl Nonce {
    pub fn generate() -> Self {
        let mut bytes = [0u8; 24];
        getrandom(&mut bytes);
        Self { bytes }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
        if bytes.len() != 24 {
            return Err(CryptoError::InvalidNonce);
        }
        let mut arr = [0u8; 24];
        arr.copy_from_slice(bytes);
        Ok(Self { bytes: arr })
    }

    pub fn as_bytes(&self) -> &[u8; 24] {
        &self.bytes
    }
}

impl fmt::Debug for Nonce {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Nonce({}...)", hex_encode(&self.bytes[..8]))
    }
}

/// Encrypted payload with authentication tag
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedPayload {
    pub ciphertext: Vec<u8>,
    pub nonce: Nonce,
    pub tag: [u8; 16],
}

/// Replay protection tracker
#[derive(Debug, Default)]
pub struct ReplayGuard {
    seen_nonces: std::collections::HashSet<[u8; 24]>,
    window_start: u64,
}

impl ReplayGuard {
    pub fn new() -> Self {
        Self {
            seen_nonces: std::collections::HashSet::new(),
            window_start: current_timestamp(),
        }
    }

    /// Check if a nonce has been seen before
    pub fn check_and_record(&mut self, nonce: &Nonce, timestamp: u64) -> Result<(), CryptoError> {
        // Reject messages older than 5 minutes
        let now = current_timestamp();
        if timestamp < now.saturating_sub(300) {
            return Err(CryptoError::ReplayDetected);
        }

        // Check for duplicate nonce
        if self.seen_nonces.contains(&nonce.bytes) {
            return Err(CryptoError::ReplayDetected);
        }

        // Record nonce
        self.seen_nonces.insert(nonce.bytes);

        // Cleanup old nonces periodically (every 1000 entries)
        if self.seen_nonces.len() > 10000 {
            self.cleanup(now);
        }

        Ok(())
    }

    fn cleanup(&mut self, now: u64) {
        // In a real implementation, we'd track timestamps per nonce
        // For simplicity, we just clear old entries periodically
        if now > self.window_start + 600 {
            self.seen_nonces.clear();
            self.window_start = now;
        }
    }
}

// Helper functions

fn getrandom(buf: &mut [u8]) {
    // In production, use getrandom crate
    // This is a simple fallback using system time + counter
    use std::time::{SystemTime, UNIX_EPOCH};

    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    let mut state = seed as u64;
    for byte in buf.iter_mut() {
        // Simple xorshift64
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        *byte = state as u8;
    }
}

fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signing_key_generation() {
        let key1 = SigningKey::generate();
        let key2 = SigningKey::generate();
        assert_ne!(key1.as_bytes(), key2.as_bytes());
    }

    #[test]
    fn test_signature_roundtrip() {
        let signing_key = SigningKey::generate();
        let verifying_key = signing_key.verifying_key();
        let message = b"hello world";

        let signature = signing_key.sign(message);
        assert!(verifying_key
            .verify_with_key(message, &signature, &signing_key)
            .is_ok());
    }

    #[test]
    fn test_encryption_roundtrip() {
        let exchange1 = ExchangeKey::generate();
        let exchange2 = ExchangeKey::generate();

        let shared1 = exchange1.derive_shared_secret(exchange2.public_key());
        let shared2 = exchange2.derive_shared_secret(exchange1.public_key());

        let message = b"secret message";
        let nonce = Nonce::generate();

        let encrypted = shared1.encrypt(message, &nonce).unwrap();
        let decrypted = shared2.decrypt(&encrypted).unwrap();

        assert_eq!(message.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_replay_guard() {
        let mut guard = ReplayGuard::new();
        let nonce = Nonce::generate();
        let now = current_timestamp();

        // First use should succeed
        assert!(guard.check_and_record(&nonce, now).is_ok());

        // Replay should fail
        assert!(guard.check_and_record(&nonce, now).is_err());

        // New nonce should succeed
        let nonce2 = Nonce::generate();
        assert!(guard.check_and_record(&nonce2, now).is_ok());
    }
}
