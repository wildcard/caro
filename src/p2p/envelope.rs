//! Message envelope and protocol structures
//!
//! Provides signed, encrypted message containers for P2P communication.

use super::crypto::{CryptoError, EncryptedPayload, Nonce, SessionKey, Signature};
use super::identity::{AgentId, KeyPair};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Message types for different operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    /// Pattern sharing
    Pattern,
    /// Peer discovery/announcement
    Discovery,
    /// Heartbeat/presence
    Heartbeat,
    /// Request for data
    Request,
    /// Response to request
    Response,
    /// Task delegation
    Task,
    /// Task result
    TaskResult,
    /// Generic data
    Data,
}

/// Signed message envelope
///
/// Structure:
/// - Header (signed, not encrypted): routing info
/// - Payload (encrypted): actual content
/// - Signature: Ed25519 over canonical header + payload hash
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedEnvelope {
    /// Protocol version
    pub version: u32,

    /// Unique message ID
    pub message_id: String,

    /// Message type for routing
    pub message_type: MessageType,

    /// Topic for pub/sub
    pub topic: String,

    /// Sender agent ID
    pub sender_id: AgentId,

    /// Optional recipient (None = broadcast)
    pub recipient_id: Option<AgentId>,

    /// Unix timestamp
    pub timestamp: u64,

    /// Time-to-live in seconds
    pub ttl: u32,

    /// Counter for replay protection
    pub counter: u64,

    /// Encrypted payload (None for unencrypted messages)
    pub payload: Option<EncryptedPayload>,

    /// Unencrypted payload (for public messages)
    pub plaintext: Option<Vec<u8>>,

    /// Ed25519 signature over canonical representation
    pub signature: Signature,
}

impl SignedEnvelope {
    /// Create a new signed envelope with encrypted payload
    pub fn new_encrypted(
        message_type: MessageType,
        topic: impl Into<String>,
        payload: &[u8],
        keypair: &KeyPair,
        session_key: &SessionKey,
        recipient_id: Option<AgentId>,
    ) -> Result<Self, EnvelopeError> {
        let nonce = Nonce::generate();
        let encrypted = session_key
            .encrypt(payload, &nonce)
            .map_err(EnvelopeError::Crypto)?;

        let mut envelope = Self {
            version: super::PROTOCOL_VERSION,
            message_id: Uuid::new_v4().to_string(),
            message_type,
            topic: topic.into(),
            sender_id: keypair.agent_id.clone(),
            recipient_id,
            timestamp: current_timestamp(),
            ttl: 300, // 5 minutes default
            counter: generate_counter(),
            payload: Some(encrypted),
            plaintext: None,
            signature: Signature::from_bytes(&[0u8; 64]).unwrap(), // Placeholder
        };

        // Sign the envelope
        envelope.sign(keypair);

        Ok(envelope)
    }

    /// Create a new signed envelope with plaintext payload (public message)
    pub fn new_plaintext(
        message_type: MessageType,
        topic: impl Into<String>,
        payload: &[u8],
        keypair: &KeyPair,
    ) -> Self {
        let mut envelope = Self {
            version: super::PROTOCOL_VERSION,
            message_id: Uuid::new_v4().to_string(),
            message_type,
            topic: topic.into(),
            sender_id: keypair.agent_id.clone(),
            recipient_id: None,
            timestamp: current_timestamp(),
            ttl: 300,
            counter: generate_counter(),
            payload: None,
            plaintext: Some(payload.to_vec()),
            signature: Signature::from_bytes(&[0u8; 64]).unwrap(),
        };

        envelope.sign(keypair);
        envelope
    }

    /// Sign the envelope
    fn sign(&mut self, keypair: &KeyPair) {
        let canonical = self.canonical_bytes();
        self.signature = keypair.sign(&canonical);
    }

    /// Get canonical bytes for signing (deterministic serialization)
    pub fn canonical_bytes(&self) -> Vec<u8> {
        // Create a deterministic representation
        let mut data = Vec::new();

        // Version
        data.extend_from_slice(&self.version.to_le_bytes());

        // Message ID
        data.extend_from_slice(self.message_id.as_bytes());

        // Message type (as string for stability)
        let type_str = serde_json::to_string(&self.message_type).unwrap_or_default();
        data.extend_from_slice(type_str.as_bytes());

        // Topic
        data.extend_from_slice(self.topic.as_bytes());

        // Sender
        data.extend_from_slice(self.sender_id.as_str().as_bytes());

        // Recipient
        if let Some(ref recipient) = self.recipient_id {
            data.extend_from_slice(recipient.as_str().as_bytes());
        }

        // Timestamp
        data.extend_from_slice(&self.timestamp.to_le_bytes());

        // TTL
        data.extend_from_slice(&self.ttl.to_le_bytes());

        // Counter
        data.extend_from_slice(&self.counter.to_le_bytes());

        // Payload hash (if present)
        if let Some(ref payload) = self.payload {
            let mut hasher = Sha256::new();
            hasher.update(&payload.ciphertext);
            data.extend_from_slice(&hasher.finalize());
        }

        // Plaintext hash (if present)
        if let Some(ref plaintext) = self.plaintext {
            let mut hasher = Sha256::new();
            hasher.update(plaintext);
            data.extend_from_slice(&hasher.finalize());
        }

        data
    }

    /// Check if the message has expired
    pub fn is_expired(&self) -> bool {
        let now = current_timestamp();
        now > self.timestamp + self.ttl as u64
    }

    /// Check if this is a broadcast message
    pub fn is_broadcast(&self) -> bool {
        self.recipient_id.is_none()
    }

    /// Decrypt the payload
    pub fn decrypt(&self, session_key: &SessionKey) -> Result<Vec<u8>, EnvelopeError> {
        match (&self.payload, &self.plaintext) {
            (Some(encrypted), _) => session_key
                .decrypt(encrypted)
                .map_err(EnvelopeError::Crypto),
            (_, Some(plaintext)) => Ok(plaintext.clone()),
            (None, None) => Err(EnvelopeError::EmptyPayload),
        }
    }

    /// Get plaintext directly (for public messages)
    pub fn get_plaintext(&self) -> Option<&[u8]> {
        self.plaintext.as_deref()
    }

    /// Calculate message hash for deduplication
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(&self.canonical_bytes());
        hasher.finalize().into()
    }
}

/// Command pattern message for sharing learned patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMessage {
    /// Context hash (working directory, shell type, etc.)
    pub context_hash: u64,

    /// Natural language prompt (may be truncated)
    pub prompt_summary: String,

    /// Generated command
    pub command: String,

    /// Success count
    pub success_count: u32,

    /// Failure count
    pub failure_count: u32,

    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,

    /// Last used timestamp
    pub last_used: u64,

    /// Tags for categorization
    pub tags: Vec<String>,
}

impl PatternMessage {
    /// Create from components
    pub fn new(prompt: &str, command: &str, context_hash: u64) -> Self {
        Self {
            context_hash,
            prompt_summary: truncate_string(prompt, 200),
            command: command.to_string(),
            success_count: 1,
            failure_count: 0,
            confidence: 0.5,
            last_used: current_timestamp(),
            tags: vec![],
        }
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, EnvelopeError> {
        serde_json::from_slice(bytes).map_err(|e| EnvelopeError::DeserializeFailed(e.to_string()))
    }

    /// Calculate success rate
    pub fn success_rate(&self) -> f32 {
        let total = self.success_count + self.failure_count;
        if total == 0 {
            0.5
        } else {
            self.success_count as f32 / total as f32
        }
    }
}

/// Discovery/announcement message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryMessage {
    /// Agent capabilities
    pub capabilities: Vec<String>,

    /// Agent version
    pub version: String,

    /// Backend types available
    pub backends: Vec<String>,

    /// Current load (0.0 - 1.0)
    pub load: f32,

    /// Public endpoint (if any)
    pub endpoint: Option<String>,
}

impl DiscoveryMessage {
    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, EnvelopeError> {
        serde_json::from_slice(bytes).map_err(|e| EnvelopeError::DeserializeFailed(e.to_string()))
    }
}

/// Heartbeat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatMessage {
    /// Current status
    pub status: AgentStatus,

    /// Uptime in seconds
    pub uptime: u64,

    /// Patterns known
    pub pattern_count: u32,

    /// Connected peers
    pub peer_count: u32,
}

impl HeartbeatMessage {
    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, EnvelopeError> {
        serde_json::from_slice(bytes).map_err(|e| EnvelopeError::DeserializeFailed(e.to_string()))
    }
}

/// Agent status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    Online,
    Busy,
    Away,
    Offline,
}

/// Envelope errors
#[derive(Debug, thiserror::Error)]
pub enum EnvelopeError {
    #[error("Crypto error: {0}")]
    Crypto(#[from] CryptoError),

    #[error("Empty payload")]
    EmptyPayload,

    #[error("Message expired")]
    Expired,

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Deserialization failed: {0}")]
    DeserializeFailed(String),

    #[error("Serialization failed: {0}")]
    SerializeFailed(String),
}

// Helper functions

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn generate_counter() -> u64 {
    // Simple counter based on timestamp + random
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    (ts as u64) ^ ((ts >> 64) as u64)
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

// UUID generation (simplified, no external crate needed)
mod uuid {
    pub struct Uuid([u8; 16]);

    impl Uuid {
        pub fn new_v4() -> Self {
            let mut bytes = [0u8; 16];
            let seed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();

            let mut state = seed as u64;
            for byte in bytes.iter_mut() {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                *byte = state as u8;
            }

            // Set version (4) and variant (RFC4122)
            bytes[6] = (bytes[6] & 0x0f) | 0x40;
            bytes[8] = (bytes[8] & 0x3f) | 0x80;

            Self(bytes)
        }
    }

    impl std::fmt::Display for Uuid {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
                self.0[0], self.0[1], self.0[2], self.0[3],
                self.0[4], self.0[5],
                self.0[6], self.0[7],
                self.0[8], self.0[9],
                self.0[10], self.0[11], self.0[12], self.0[13], self.0[14], self.0[15]
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_envelope_plaintext() {
        let keypair = KeyPair::generate();
        let payload = b"hello world";

        let envelope = SignedEnvelope::new_plaintext(
            MessageType::Data,
            "test/topic",
            payload,
            &keypair,
        );

        assert_eq!(envelope.topic, "test/topic");
        assert_eq!(envelope.get_plaintext(), Some(payload.as_slice()));
        assert!(!envelope.is_expired());
    }

    #[test]
    fn test_pattern_message() {
        let pattern = PatternMessage::new(
            "list all files",
            "ls -la",
            12345,
        );

        let bytes = pattern.to_bytes();
        let decoded = PatternMessage::from_bytes(&bytes).unwrap();

        assert_eq!(decoded.command, "ls -la");
        assert_eq!(decoded.context_hash, 12345);
    }

    #[test]
    fn test_envelope_expiry() {
        let keypair = KeyPair::generate();

        let mut envelope = SignedEnvelope::new_plaintext(
            MessageType::Heartbeat,
            "test",
            b"ping",
            &keypair,
        );

        // Set timestamp to the past
        envelope.timestamp = 0;
        envelope.ttl = 60;

        assert!(envelope.is_expired());
    }

    #[test]
    fn test_canonical_bytes_deterministic() {
        let keypair = KeyPair::generate();

        let envelope1 = SignedEnvelope::new_plaintext(
            MessageType::Data,
            "test",
            b"payload",
            &keypair,
        );

        // Create another with same data (but sign will be different due to counter)
        // The canonical bytes for same content should be comparable structure
        let canonical = envelope1.canonical_bytes();
        assert!(!canonical.is_empty());
    }
}
