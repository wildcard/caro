//! Transport layer abstraction
//!
//! Provides a unified interface for different P2P transport mechanisms.

#[cfg(feature = "p2p-gun")]
pub mod gun;

#[cfg(feature = "p2p-nostr")]
pub mod nostr;

#[cfg(feature = "p2p-grpc")]
pub mod grpc;

pub mod local;

use crate::p2p::envelope::SignedEnvelope;
use crate::p2p::NetworkStats;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Duration;
use thiserror::Error;

/// Transport types available
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransportType {
    /// GUN.js decentralized database (offline-first, real-time sync)
    Gun,
    /// Nostr protocol (simple pub/sub via relays)
    Nostr,
    /// gRPC (low-latency, direct connections)
    Grpc,
    /// Local only (no network, for testing)
    Local,
}

impl fmt::Display for TransportType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Gun => write!(f, "GUN"),
            Self::Nostr => write!(f, "Nostr"),
            Self::Grpc => write!(f, "gRPC"),
            Self::Local => write!(f, "Local"),
        }
    }
}

/// Transport configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportConfig {
    /// Transport type to use
    pub transport_type: TransportType,

    /// Relay/server URLs
    pub relays: Vec<String>,

    /// Connection timeout
    #[serde(with = "duration_serde")]
    pub connect_timeout: Duration,

    /// Request timeout
    #[serde(with = "duration_serde")]
    pub request_timeout: Duration,

    /// Enable automatic reconnection
    pub auto_reconnect: bool,

    /// Reconnection interval
    #[serde(with = "duration_serde")]
    pub reconnect_interval: Duration,

    /// Maximum message size
    pub max_message_size: usize,

    /// Enable compression
    pub compression: bool,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            transport_type: TransportType::Local,
            relays: vec![],
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(30),
            auto_reconnect: true,
            reconnect_interval: Duration::from_secs(5),
            max_message_size: 1024 * 1024, // 1MB
            compression: true,
        }
    }
}

impl TransportConfig {
    /// Create a GUN transport config with default relays
    pub fn gun() -> Self {
        Self {
            transport_type: TransportType::Gun,
            relays: vec![
                "https://gun-manhattan.herokuapp.com/gun".into(),
                "https://gun-us.herokuapp.com/gun".into(),
            ],
            ..Default::default()
        }
    }

    /// Create a Nostr transport config with default relays
    pub fn nostr() -> Self {
        Self {
            transport_type: TransportType::Nostr,
            relays: vec![
                "wss://relay.damus.io".into(),
                "wss://relay.nostr.info".into(),
                "wss://nostr-pub.wellorder.net".into(),
            ],
            ..Default::default()
        }
    }

    /// Create a gRPC transport config
    pub fn grpc(endpoint: impl Into<String>) -> Self {
        Self {
            transport_type: TransportType::Grpc,
            relays: vec![endpoint.into()],
            ..Default::default()
        }
    }

    /// Create a local-only transport config
    pub fn local() -> Self {
        Self {
            transport_type: TransportType::Local,
            ..Default::default()
        }
    }

    /// Builder: set relays
    pub fn with_relays(mut self, relays: Vec<String>) -> Self {
        self.relays = relays;
        self
    }

    /// Builder: set timeouts
    pub fn with_timeouts(mut self, connect: Duration, request: Duration) -> Self {
        self.connect_timeout = connect;
        self.request_timeout = request;
        self
    }

    /// Builder: enable/disable compression
    pub fn with_compression(mut self, enabled: bool) -> Self {
        self.compression = enabled;
        self
    }
}

/// Core transport trait
///
/// All P2P transport mechanisms implement this trait for a unified interface.
#[async_trait]
pub trait Transport: Send + Sync + fmt::Debug {
    /// Connect to the transport network
    async fn connect(&mut self) -> Result<(), TransportError>;

    /// Disconnect from the transport network
    async fn disconnect(&mut self) -> Result<(), TransportError>;

    /// Check if connected
    fn is_connected(&self) -> bool;

    /// Subscribe to a topic
    async fn subscribe(&mut self, topic: &str) -> Result<(), TransportError>;

    /// Unsubscribe from a topic
    async fn unsubscribe(&mut self, topic: &str) -> Result<(), TransportError>;

    /// Publish a message to a topic (broadcast)
    async fn publish(&mut self, envelope: SignedEnvelope) -> Result<(), TransportError>;

    /// Send a message to a specific peer
    async fn send(&mut self, envelope: SignedEnvelope) -> Result<(), TransportError>;

    /// Receive the next message (blocking)
    async fn receive(&mut self) -> Result<SignedEnvelope, TransportError>;

    /// Try to receive a message (non-blocking)
    async fn try_receive(&mut self) -> Result<Option<SignedEnvelope>, TransportError>;

    /// Get transport statistics
    fn stats(&self) -> NetworkStats;

    /// Get transport type
    fn transport_type(&self) -> TransportType;
}

/// Transport errors
#[derive(Debug, Error)]
pub enum TransportError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Not connected")]
    NotConnected,

    #[error("Connection lost")]
    ConnectionLost,

    #[error("Timeout after {0:?}")]
    Timeout(Duration),

    #[error("Message too large: {size} > {max}")]
    MessageTooLarge { size: usize, max: usize },

    #[error("Subscription failed: {0}")]
    SubscriptionFailed(String),

    #[error("Send failed: {0}")]
    SendFailed(String),

    #[error("Receive failed: {0}")]
    ReceiveFailed(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Relay error: {0}")]
    RelayError(String),

    #[error("Channel closed")]
    ChannelClosed,

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// Transport factory for creating transports based on config
pub struct TransportFactory;

impl TransportFactory {
    /// Create a transport based on configuration
    pub fn create(config: TransportConfig) -> Result<Box<dyn Transport>, TransportError> {
        match config.transport_type {
            #[cfg(feature = "p2p-gun")]
            TransportType::Gun => Ok(Box::new(gun::GunTransport::new(config))),

            #[cfg(feature = "p2p-nostr")]
            TransportType::Nostr => Ok(Box::new(nostr::NostrTransport::new(config))),

            #[cfg(feature = "p2p-grpc")]
            TransportType::Grpc => Ok(Box::new(grpc::GrpcTransport::new(config))),

            TransportType::Local => Ok(Box::new(local::LocalTransport::new(config))),

            #[allow(unreachable_patterns)]
            _ => Err(TransportError::InvalidConfig(format!(
                "Transport type {:?} not enabled in features",
                config.transport_type
            ))),
        }
    }
}

/// Duration serialization helper
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_millis().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_config_builders() {
        let gun = TransportConfig::gun();
        assert_eq!(gun.transport_type, TransportType::Gun);
        assert!(!gun.relays.is_empty());

        let nostr = TransportConfig::nostr();
        assert_eq!(nostr.transport_type, TransportType::Nostr);

        let grpc = TransportConfig::grpc("localhost:50051");
        assert_eq!(grpc.transport_type, TransportType::Grpc);
        assert_eq!(grpc.relays[0], "localhost:50051");

        let local = TransportConfig::local();
        assert_eq!(local.transport_type, TransportType::Local);
    }

    #[test]
    fn test_config_with_builder() {
        let config = TransportConfig::gun()
            .with_relays(vec!["custom-relay.com".into()])
            .with_compression(false)
            .with_timeouts(Duration::from_secs(5), Duration::from_secs(15));

        assert_eq!(config.relays.len(), 1);
        assert!(!config.compression);
        assert_eq!(config.connect_timeout, Duration::from_secs(5));
    }
}
