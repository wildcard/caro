//! Nostr transport adapter
//!
//! Nostr is a simple, open protocol for decentralized social networking.
//! We use it as a lightweight pub/sub transport.
//!
//! Features:
//! - Simple relay-based architecture
//! - Cryptographic identity (secp256k1)
//! - Event-based messaging
//! - NIPs (Nostr Implementation Possibilities) for extensibility
//!
//! Relevant NIPs:
//! - NIP-01: Basic protocol
//! - NIP-04: Encrypted Direct Messages
//! - NIP-28: Public Chat (channels)

use super::{Transport, TransportConfig, TransportError, TransportType};
use crate::p2p::envelope::SignedEnvelope;
use crate::p2p::NetworkStats;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::Duration;

/// Nostr event kinds
mod event_kind {
    pub const TEXT_NOTE: u32 = 1;
    pub const ENCRYPTED_DM: u32 = 4;
    pub const CHANNEL_CREATE: u32 = 40;
    pub const CHANNEL_MESSAGE: u32 = 42;
    pub const CARO_PATTERN: u32 = 30078; // Custom kind for caro patterns
    pub const CARO_DISCOVERY: u32 = 30079; // Custom kind for discovery
}

/// Nostr event (NIP-01)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NostrEvent {
    /// Event ID (SHA-256 of serialized event)
    pub id: String,

    /// Public key of the creator (hex)
    pub pubkey: String,

    /// Unix timestamp
    pub created_at: u64,

    /// Event kind
    pub kind: u32,

    /// Tags (arrays of strings)
    pub tags: Vec<Vec<String>>,

    /// Event content (JSON string for caro)
    pub content: String,

    /// Schnorr signature (hex)
    pub sig: String,
}

impl NostrEvent {
    /// Create a new event
    pub fn new(pubkey: &str, kind: u32, content: &str, tags: Vec<Vec<String>>) -> Self {
        let created_at = current_timestamp();

        let mut event = Self {
            id: String::new(),
            pubkey: pubkey.to_string(),
            created_at,
            kind,
            tags,
            content: content.to_string(),
            sig: String::new(),
        };

        event.id = event.compute_id();
        event
    }

    /// Compute event ID (NIP-01)
    fn compute_id(&self) -> String {
        // ID = SHA256([0, pubkey, created_at, kind, tags, content])
        let serialized = serde_json::json!([
            0,
            &self.pubkey,
            self.created_at,
            self.kind,
            &self.tags,
            &self.content
        ]);

        let json = serde_json::to_string(&serialized).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());
        hex_encode(&hasher.finalize())
    }

    /// Sign the event (simplified - in production use secp256k1)
    pub fn sign(&mut self, secret_key: &[u8]) {
        let mut hasher = Sha256::new();
        hasher.update(&self.id);
        hasher.update(secret_key);
        self.sig = hex_encode(&hasher.finalize());
    }

    /// Verify event signature (simplified)
    pub fn verify(&self) -> bool {
        // In production, verify secp256k1 Schnorr signature
        !self.sig.is_empty() && self.id == self.compute_id()
    }

    /// Create a caro pattern event
    pub fn caro_pattern(pubkey: &str, envelope: &SignedEnvelope) -> Self {
        let content = serde_json::to_string(envelope).unwrap_or_default();
        let tags = vec![
            vec!["d".to_string(), envelope.topic.clone()],
            vec!["t".to_string(), "caro-pattern".to_string()],
        ];
        Self::new(pubkey, event_kind::CARO_PATTERN, &content, tags)
    }

    /// Create a caro discovery event
    pub fn caro_discovery(pubkey: &str, envelope: &SignedEnvelope) -> Self {
        let content = serde_json::to_string(envelope).unwrap_or_default();
        let tags = vec![
            vec!["d".to_string(), "discovery".to_string()],
            vec!["t".to_string(), "caro-discovery".to_string()],
        ];
        Self::new(pubkey, event_kind::CARO_DISCOVERY, &content, tags)
    }
}

/// Nostr relay message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RelayMessage {
    /// ["EVENT", <subscription_id>, <event>]
    Event(String, String, NostrEvent),
    /// ["OK", <event_id>, <accepted>, <message>]
    Ok(String, String, bool, String),
    /// ["EOSE", <subscription_id>]
    EndOfStoredEvents(String, String),
    /// ["NOTICE", <message>]
    Notice(String, String),
}

/// Nostr client message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ClientMessage {
    /// ["EVENT", <event>]
    Event(String, NostrEvent),
    /// ["REQ", <subscription_id>, <filters>...]
    Request(String, String, NostrFilter),
    /// ["CLOSE", <subscription_id>]
    Close(String, String),
}

/// Nostr filter (NIP-01)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NostrFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ids: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub authors: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub kinds: Option<Vec<u32>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "#t")]
    pub tags_t: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "#d")]
    pub tags_d: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub until: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

impl NostrFilter {
    /// Filter for caro patterns
    pub fn caro_patterns() -> Self {
        Self {
            kinds: Some(vec![event_kind::CARO_PATTERN]),
            tags_t: Some(vec!["caro-pattern".to_string()]),
            ..Default::default()
        }
    }

    /// Filter for caro discovery
    pub fn caro_discovery() -> Self {
        Self {
            kinds: Some(vec![event_kind::CARO_DISCOVERY]),
            tags_t: Some(vec!["caro-discovery".to_string()]),
            ..Default::default()
        }
    }

    /// Filter by topic
    pub fn by_topic(topic: &str) -> Self {
        Self {
            tags_d: Some(vec![topic.to_string()]),
            ..Default::default()
        }
    }
}

/// Relay connection status
#[derive(Debug, Clone)]
struct RelayConnection {
    url: String,
    connected: bool,
    latency_ms: Option<u32>,
    event_count: u64,
    last_event: u64,
}

impl RelayConnection {
    fn new(url: String) -> Self {
        Self {
            url,
            connected: false,
            latency_ms: None,
            event_count: 0,
            last_event: 0,
        }
    }
}

/// Nostr transport implementation
#[derive(Debug)]
pub struct NostrTransport {
    config: TransportConfig,
    connected: bool,
    subscriptions: HashMap<String, NostrFilter>, // subscription_id -> filter
    relays: Vec<RelayConnection>,
    pubkey: String,
    secret_key: [u8; 32],
    inbox: Arc<RwLock<VecDeque<SignedEnvelope>>>,
    seen_events: HashSet<String>, // Event IDs we've already processed
    stats: NetworkStats,
    subscription_counter: u64,
}

impl NostrTransport {
    pub fn new(config: TransportConfig) -> Self {
        let relays = config
            .relays
            .iter()
            .map(|url| RelayConnection::new(url.clone()))
            .collect();

        // Generate keypair (simplified)
        let mut secret_key = [0u8; 32];
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let mut state = seed as u64;
        for byte in secret_key.iter_mut() {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            *byte = state as u8;
        }

        // Derive pubkey (simplified - in production use secp256k1)
        let mut hasher = Sha256::new();
        hasher.update(&secret_key);
        let pubkey = hex_encode(&hasher.finalize());

        Self {
            config,
            connected: false,
            subscriptions: HashMap::new(),
            relays,
            pubkey,
            secret_key,
            inbox: Arc::new(RwLock::new(VecDeque::new())),
            seen_events: HashSet::new(),
            stats: NetworkStats::new(),
            subscription_counter: 0,
        }
    }

    /// Generate a unique subscription ID
    fn next_subscription_id(&mut self) -> String {
        self.subscription_counter += 1;
        format!("caro-{}", self.subscription_counter)
    }

    /// Send event to relays
    #[cfg(feature = "reqwest")]
    async fn send_event(&mut self, event: NostrEvent) -> Result<(), TransportError> {
        // In production, use WebSocket connections
        // For now, simulate with HTTP POST to relay
        let message = ClientMessage::Event("EVENT".to_string(), event);
        let json = serde_json::to_string(&message)
            .map_err(|e| TransportError::Serialization(e.to_string()))?;

        for relay in &mut self.relays {
            if relay.connected {
                // Simulate sending (in production, use WebSocket)
                relay.event_count += 1;
            }
        }

        self.stats.record_sent(json.len());
        Ok(())
    }

    #[cfg(not(feature = "reqwest"))]
    async fn send_event(&mut self, event: NostrEvent) -> Result<(), TransportError> {
        // Local simulation
        let json = serde_json::to_string(&event)
            .map_err(|e| TransportError::Serialization(e.to_string()))?;
        self.stats.record_sent(json.len());
        Ok(())
    }

    /// Process incoming event
    fn process_event(&mut self, event: NostrEvent) -> Result<Option<SignedEnvelope>, TransportError> {
        // Check if we've seen this event
        if self.seen_events.contains(&event.id) {
            return Ok(None);
        }
        self.seen_events.insert(event.id.clone());

        // Verify event
        if !event.verify() {
            return Ok(None);
        }

        // Parse envelope from content
        match serde_json::from_str::<SignedEnvelope>(&event.content) {
            Ok(envelope) => Ok(Some(envelope)),
            Err(_) => Ok(None), // Not a caro message, ignore
        }
    }

    /// Get pubkey
    pub fn pubkey(&self) -> &str {
        &self.pubkey
    }
}

#[async_trait]
impl Transport for NostrTransport {
    async fn connect(&mut self) -> Result<(), TransportError> {
        // In production, establish WebSocket connections to relays
        for relay in &mut self.relays {
            relay.connected = true; // Simulate connection
        }

        self.connected = true;
        self.stats = NetworkStats::new();

        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), TransportError> {
        for relay in &mut self.relays {
            relay.connected = false;
        }
        self.connected = false;
        self.subscriptions.clear();
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    async fn subscribe(&mut self, topic: &str) -> Result<(), TransportError> {
        if !self.connected {
            return Err(TransportError::NotConnected);
        }

        let sub_id = self.next_subscription_id();
        let filter = NostrFilter::by_topic(topic);

        // In production, send REQ to relays
        let _message = ClientMessage::Request("REQ".to_string(), sub_id.clone(), filter.clone());

        self.subscriptions.insert(sub_id, filter);

        Ok(())
    }

    async fn unsubscribe(&mut self, topic: &str) -> Result<(), TransportError> {
        // Find and remove subscription for this topic
        let to_remove: Vec<String> = self
            .subscriptions
            .iter()
            .filter(|(_, f)| f.tags_d.as_ref().map(|t| t.contains(&topic.to_string())).unwrap_or(false))
            .map(|(id, _)| id.clone())
            .collect();

        for id in to_remove {
            self.subscriptions.remove(&id);
            // In production, send CLOSE to relays
        }

        Ok(())
    }

    async fn publish(&mut self, envelope: SignedEnvelope) -> Result<(), TransportError> {
        if !self.connected {
            return Err(TransportError::NotConnected);
        }

        // Create Nostr event
        let mut event = NostrEvent::caro_pattern(&self.pubkey, &envelope);
        event.sign(&self.secret_key);

        // Send to relays
        self.send_event(event).await
    }

    async fn send(&mut self, envelope: SignedEnvelope) -> Result<(), TransportError> {
        if !self.connected {
            return Err(TransportError::NotConnected);
        }

        if envelope.recipient_id.is_some() {
            // For direct messages, use NIP-04 encrypted DM
            // For now, treat as regular publish
        }

        self.publish(envelope).await
    }

    async fn receive(&mut self) -> Result<SignedEnvelope, TransportError> {
        if !self.connected {
            return Err(TransportError::NotConnected);
        }

        loop {
            if let Some(envelope) = self.try_receive().await? {
                return Ok(envelope);
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    async fn try_receive(&mut self) -> Result<Option<SignedEnvelope>, TransportError> {
        if !self.connected {
            return Err(TransportError::NotConnected);
        }

        // Check inbox
        if let Ok(mut inbox) = self.inbox.write() {
            if let Some(envelope) = inbox.pop_front() {
                let size = serde_json::to_vec(&envelope)
                    .map(|v| v.len())
                    .unwrap_or(0);
                self.stats.record_received(size);
                return Ok(Some(envelope));
            }
        }

        // In production, poll WebSocket connections for new events
        Ok(None)
    }

    fn stats(&self) -> NetworkStats {
        let mut stats = self.stats.clone();
        stats.connected_peers = self.relays.iter().filter(|r| r.connected).count();
        stats
    }

    fn transport_type(&self) -> TransportType {
        TransportType::Nostr
    }
}

/// Helper to inject events (for testing and WebSocket integration)
impl NostrTransport {
    pub fn inject_event(&mut self, event: NostrEvent) -> Result<(), TransportError> {
        if let Some(envelope) = self.process_event(event)? {
            if let Ok(mut inbox) = self.inbox.write() {
                inbox.push_back(envelope);
            }
        }
        Ok(())
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::p2p::envelope::MessageType;
    use crate::p2p::identity::KeyPair;

    #[test]
    fn test_nostr_event_creation() {
        let event = NostrEvent::new(
            "pubkey123",
            event_kind::TEXT_NOTE,
            "Hello Nostr!",
            vec![],
        );

        assert!(!event.id.is_empty());
        assert_eq!(event.kind, 1);
    }

    #[test]
    fn test_nostr_event_signing() {
        let mut event = NostrEvent::new(
            "pubkey123",
            event_kind::TEXT_NOTE,
            "Hello Nostr!",
            vec![],
        );

        let secret_key = [1u8; 32];
        event.sign(&secret_key);

        assert!(!event.sig.is_empty());
        assert!(event.verify());
    }

    #[test]
    fn test_caro_pattern_event() {
        let keypair = KeyPair::generate();
        let envelope = SignedEnvelope::new_plaintext(
            MessageType::Pattern,
            "caro/patterns/v1",
            b"test pattern",
            &keypair,
        );

        let event = NostrEvent::caro_pattern("pubkey123", &envelope);
        assert_eq!(event.kind, event_kind::CARO_PATTERN);
        assert!(event.tags.iter().any(|t| t.get(0) == Some(&"t".to_string())));
    }

    #[tokio::test]
    async fn test_nostr_transport() {
        let config = TransportConfig::nostr();
        let mut transport = NostrTransport::new(config);

        transport.connect().await.unwrap();
        assert!(transport.is_connected());

        transport.subscribe("caro/patterns/v1").await.unwrap();
        assert_eq!(transport.subscriptions.len(), 1);
    }

    #[test]
    fn test_nostr_filter() {
        let filter = NostrFilter::caro_patterns();
        assert_eq!(filter.kinds, Some(vec![event_kind::CARO_PATTERN]));

        let topic_filter = NostrFilter::by_topic("test/topic");
        assert_eq!(topic_filter.tags_d, Some(vec!["test/topic".to_string()]));
    }
}
