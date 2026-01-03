//! P2P Networking Layer for Caro
//!
//! This module provides distributed communication capabilities for sharing
//! command patterns, coordinating backends, and enabling team collaboration.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                    Application Layer                     │
//! │         (PatternSync, BackendCoordinator)               │
//! ├─────────────────────────────────────────────────────────┤
//! │                    Envelope Layer                        │
//! │    SignedEnvelope → Message authentication              │
//! ├─────────────────────────────────────────────────────────┤
//! │                    Crypto Layer                          │
//! │    Ed25519 (signing) + X25519 (key exchange)            │
//! ├─────────────────────────────────────────────────────────┤
//! │                    Transport Layer                       │
//! │   ┌─────────┬─────────────┬──────────┬─────────────┐   │
//! │   │   GUN   │   Nostr     │   gRPC   │   Local     │   │
//! │   └─────────┴─────────────┴──────────┴─────────────┘   │
//! └─────────────────────────────────────────────────────────┘
//! ```

pub mod crypto;
pub mod envelope;
pub mod identity;
pub mod transport;

#[cfg(feature = "p2p-sync")]
pub mod sync;

// Re-export core types
pub use crypto::{CryptoError, EncryptedPayload, SessionKey};
pub use envelope::{MessageType, SignedEnvelope};
pub use identity::{AgentId, IdentityManager, KeyPair, RegisteredMember};
pub use transport::{Transport, TransportConfig, TransportError, TransportType};

use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Protocol version for compatibility checking
pub const PROTOCOL_VERSION: u32 = 1;

/// Default message TTL (5 minutes)
pub const DEFAULT_MESSAGE_TTL: Duration = Duration::from_secs(300);

/// Maximum message size (1MB)
pub const MAX_MESSAGE_SIZE: usize = 1024 * 1024;

/// Topics for pub/sub messaging
pub mod topics {
    /// Command pattern sharing
    pub const PATTERNS: &str = "caro/patterns/v1";
    /// Backend availability announcements
    pub const BACKENDS: &str = "caro/backends/v1";
    /// Peer discovery
    pub const DISCOVERY: &str = "caro/discovery/v1";
    /// Heartbeat/presence
    pub const HEARTBEAT: &str = "caro/heartbeat/v1";
}

/// Network statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkStats {
    pub messages_sent: u64,
    pub messages_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub connected_peers: usize,
    pub active_since: Option<u64>,
}

impl NetworkStats {
    pub fn new() -> Self {
        Self {
            active_since: Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            ),
            ..Default::default()
        }
    }

    pub fn record_sent(&mut self, bytes: usize) {
        self.messages_sent += 1;
        self.bytes_sent += bytes as u64;
    }

    pub fn record_received(&mut self, bytes: usize) {
        self.messages_received += 1;
        self.bytes_received += bytes as u64;
    }
}

/// Peer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub agent_id: AgentId,
    pub public_key: Vec<u8>,
    pub capabilities: Vec<String>,
    pub last_seen: u64,
    pub latency_ms: Option<u32>,
}

impl PeerInfo {
    pub fn is_stale(&self, timeout: Duration) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now.saturating_sub(self.last_seen) > timeout.as_secs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_stats() {
        let mut stats = NetworkStats::new();
        stats.record_sent(100);
        stats.record_received(200);

        assert_eq!(stats.messages_sent, 1);
        assert_eq!(stats.messages_received, 1);
        assert_eq!(stats.bytes_sent, 100);
        assert_eq!(stats.bytes_received, 200);
    }

    #[test]
    fn test_peer_staleness() {
        let peer = PeerInfo {
            agent_id: AgentId::new(),
            public_key: vec![],
            capabilities: vec![],
            last_seen: 0, // Very old
            latency_ms: None,
        };

        assert!(peer.is_stale(Duration::from_secs(60)));
    }
}
