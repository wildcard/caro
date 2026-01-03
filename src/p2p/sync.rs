//! Pattern synchronization layer
//!
//! High-level API for synchronizing command patterns across peers.
//! This module ties together identity, envelopes, and transports.

use super::envelope::{
    DiscoveryMessage, HeartbeatMessage, MessageType, PatternMessage, SignedEnvelope,
};
use super::identity::{AgentId, IdentityManager};
use super::transport::{Transport, TransportConfig, TransportError, TransportFactory};
use super::topics;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::sync::mpsc;

/// Pattern sync configuration
#[derive(Debug, Clone)]
pub struct SyncConfig {
    /// Transport configuration
    pub transport: TransportConfig,

    /// Heartbeat interval
    pub heartbeat_interval: Duration,

    /// Pattern sync interval
    pub sync_interval: Duration,

    /// Maximum patterns to sync at once
    pub batch_size: usize,

    /// Minimum confidence to share patterns
    pub min_confidence: f32,

    /// Agent capabilities to advertise
    pub capabilities: Vec<String>,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            transport: TransportConfig::local(),
            heartbeat_interval: Duration::from_secs(30),
            sync_interval: Duration::from_secs(60),
            batch_size: 100,
            min_confidence: 0.7,
            capabilities: vec!["patterns".to_string()],
        }
    }
}

impl SyncConfig {
    /// Create with GUN transport
    pub fn with_gun() -> Self {
        Self {
            transport: TransportConfig::gun(),
            ..Default::default()
        }
    }

    /// Create with Nostr transport
    pub fn with_nostr() -> Self {
        Self {
            transport: TransportConfig::nostr(),
            ..Default::default()
        }
    }

    /// Create with gRPC transport
    pub fn with_grpc(endpoint: impl Into<String>) -> Self {
        Self {
            transport: TransportConfig::grpc(endpoint),
            ..Default::default()
        }
    }

    /// Builder: set capabilities
    pub fn capabilities(mut self, caps: Vec<String>) -> Self {
        self.capabilities = caps;
        self
    }

    /// Builder: set sync interval
    pub fn sync_interval(mut self, interval: Duration) -> Self {
        self.sync_interval = interval;
        self
    }
}

/// Pattern store for local patterns
#[derive(Debug, Default)]
pub struct PatternStore {
    patterns: HashMap<u64, PatternMessage>,
    version: u64,
}

impl PatternStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add or update a pattern
    pub fn upsert(&mut self, pattern: PatternMessage) {
        let key = pattern.context_hash;
        if let Some(existing) = self.patterns.get_mut(&key) {
            // Merge pattern stats
            existing.success_count += pattern.success_count;
            existing.failure_count += pattern.failure_count;
            existing.confidence = (existing.confidence + pattern.confidence) / 2.0;
            if pattern.last_used > existing.last_used {
                existing.last_used = pattern.last_used;
                existing.command = pattern.command;
            }
        } else {
            self.patterns.insert(key, pattern);
        }
        self.version += 1;
    }

    /// Get a pattern by context hash
    pub fn get(&self, context_hash: u64) -> Option<&PatternMessage> {
        self.patterns.get(&context_hash)
    }

    /// Get patterns above confidence threshold
    pub fn get_shareable(&self, min_confidence: f32) -> Vec<&PatternMessage> {
        self.patterns
            .values()
            .filter(|p| p.confidence >= min_confidence)
            .collect()
    }

    /// Get all patterns
    pub fn all(&self) -> Vec<&PatternMessage> {
        self.patterns.values().collect()
    }

    /// Get pattern count
    pub fn len(&self) -> usize {
        self.patterns.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }

    /// Get current version
    pub fn version(&self) -> u64 {
        self.version
    }
}

/// Pattern sync manager
pub struct PatternSync {
    config: SyncConfig,
    identity: IdentityManager,
    transport: Box<dyn Transport>,
    store: Arc<RwLock<PatternStore>>,
    peers: Arc<RwLock<HashMap<AgentId, PeerState>>>,
    running: Arc<RwLock<bool>>,
}

/// Peer state tracking
#[derive(Debug, Clone)]
struct PeerState {
    last_seen: u64,
    pattern_count: u32,
    capabilities: Vec<String>,
}

impl PatternSync {
    /// Create a new pattern sync manager
    pub fn new(config: SyncConfig) -> Result<Self, TransportError> {
        let transport = TransportFactory::create(config.transport.clone())?;
        let identity = IdentityManager::new();

        Ok(Self {
            config,
            identity,
            transport,
            store: Arc::new(RwLock::new(PatternStore::new())),
            peers: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
        })
    }

    /// Get agent ID
    pub fn agent_id(&self) -> &AgentId {
        self.identity.agent_id()
    }

    /// Get pattern store
    pub fn store(&self) -> Arc<RwLock<PatternStore>> {
        self.store.clone()
    }

    /// Start the sync service
    pub async fn start(&mut self) -> Result<(), SyncError> {
        // Connect transport
        self.transport
            .connect()
            .await
            .map_err(SyncError::Transport)?;

        // Subscribe to topics
        self.transport
            .subscribe(topics::PATTERNS)
            .await
            .map_err(SyncError::Transport)?;
        self.transport
            .subscribe(topics::DISCOVERY)
            .await
            .map_err(SyncError::Transport)?;
        self.transport
            .subscribe(topics::HEARTBEAT)
            .await
            .map_err(SyncError::Transport)?;

        // Register ourselves
        let registration = self.identity.create_registration(self.config.capabilities.clone());
        self.identity
            .register_member(registration)
            .map_err(|e| SyncError::Identity(e.to_string()))?;

        // Mark as running
        if let Ok(mut running) = self.running.write() {
            *running = true;
        }

        // Send discovery announcement
        self.announce().await?;

        Ok(())
    }

    /// Stop the sync service
    pub async fn stop(&mut self) -> Result<(), SyncError> {
        if let Ok(mut running) = self.running.write() {
            *running = false;
        }

        self.transport
            .disconnect()
            .await
            .map_err(SyncError::Transport)?;

        Ok(())
    }

    /// Check if running
    pub fn is_running(&self) -> bool {
        self.running.read().map(|r| *r).unwrap_or(false)
    }

    /// Announce presence to the network
    pub async fn announce(&mut self) -> Result<(), SyncError> {
        let discovery = DiscoveryMessage {
            capabilities: self.config.capabilities.clone(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            backends: vec![], // Could be populated from config
            load: 0.0,
            endpoint: None,
        };

        let envelope = SignedEnvelope::new_plaintext(
            MessageType::Discovery,
            topics::DISCOVERY,
            &discovery.to_bytes(),
            self.identity.keypair(),
        );

        self.transport
            .publish(envelope)
            .await
            .map_err(SyncError::Transport)?;

        Ok(())
    }

    /// Send heartbeat
    pub async fn heartbeat(&mut self) -> Result<(), SyncError> {
        let pattern_count = self
            .store
            .read()
            .map(|s| s.len() as u32)
            .unwrap_or(0);

        let peer_count = self
            .peers
            .read()
            .map(|p| p.len() as u32)
            .unwrap_or(0);

        let heartbeat = HeartbeatMessage {
            status: super::envelope::AgentStatus::Online,
            uptime: 0, // Could track actual uptime
            pattern_count,
            peer_count,
        };

        let envelope = SignedEnvelope::new_plaintext(
            MessageType::Heartbeat,
            topics::HEARTBEAT,
            &heartbeat.to_bytes(),
            self.identity.keypair(),
        );

        self.transport
            .publish(envelope)
            .await
            .map_err(SyncError::Transport)?;

        Ok(())
    }

    /// Share a pattern with the network
    pub async fn share_pattern(&mut self, pattern: PatternMessage) -> Result<(), SyncError> {
        // Store locally first
        if let Ok(mut store) = self.store.write() {
            store.upsert(pattern.clone());
        }

        // Share if above threshold
        if pattern.confidence >= self.config.min_confidence {
            let envelope = SignedEnvelope::new_plaintext(
                MessageType::Pattern,
                topics::PATTERNS,
                &pattern.to_bytes(),
                self.identity.keypair(),
            );

            self.transport
                .publish(envelope)
                .await
                .map_err(SyncError::Transport)?;
        }

        Ok(())
    }

    /// Process incoming messages
    pub async fn process_messages(&mut self) -> Result<usize, SyncError> {
        let mut count = 0;

        while let Ok(Some(envelope)) = self.transport.try_receive().await {
            self.handle_envelope(envelope)?;
            count += 1;
        }

        Ok(count)
    }

    /// Handle a received envelope
    fn handle_envelope(&mut self, envelope: SignedEnvelope) -> Result<(), SyncError> {
        // Skip our own messages
        if envelope.sender_id == *self.identity.agent_id() {
            return Ok(());
        }

        // Check expiry
        if envelope.is_expired() {
            return Ok(());
        }

        match envelope.message_type {
            MessageType::Pattern => {
                if let Some(payload) = envelope.get_plaintext() {
                    if let Ok(pattern) = PatternMessage::from_bytes(payload) {
                        if let Ok(mut store) = self.store.write() {
                            store.upsert(pattern);
                        }
                    }
                }
            }
            MessageType::Discovery => {
                if let Some(payload) = envelope.get_plaintext() {
                    if let Ok(discovery) = DiscoveryMessage::from_bytes(payload) {
                        self.handle_discovery(&envelope.sender_id, discovery);
                    }
                }
            }
            MessageType::Heartbeat => {
                if let Some(payload) = envelope.get_plaintext() {
                    if let Ok(heartbeat) = HeartbeatMessage::from_bytes(payload) {
                        self.handle_heartbeat(&envelope.sender_id, heartbeat);
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn handle_discovery(&mut self, sender: &AgentId, discovery: DiscoveryMessage) {
        if let Ok(mut peers) = self.peers.write() {
            peers.insert(
                sender.clone(),
                PeerState {
                    last_seen: current_timestamp(),
                    pattern_count: 0,
                    capabilities: discovery.capabilities,
                },
            );
        }
    }

    fn handle_heartbeat(&mut self, sender: &AgentId, heartbeat: HeartbeatMessage) {
        if let Ok(mut peers) = self.peers.write() {
            if let Some(peer) = peers.get_mut(sender) {
                peer.last_seen = current_timestamp();
                peer.pattern_count = heartbeat.pattern_count;
            }
        }
    }

    /// Get connected peer count
    pub fn peer_count(&self) -> usize {
        self.peers.read().map(|p| p.len()).unwrap_or(0)
    }

    /// Get network stats
    pub fn stats(&self) -> super::NetworkStats {
        self.transport.stats()
    }

    /// Run the sync loop (blocking)
    pub async fn run(&mut self) -> Result<(), SyncError> {
        self.start().await?;

        let heartbeat_interval = self.config.heartbeat_interval;
        let sync_interval = self.config.sync_interval;

        let mut last_heartbeat = std::time::Instant::now();
        let mut last_sync = std::time::Instant::now();

        while self.is_running() {
            // Process incoming messages
            self.process_messages().await?;

            // Send heartbeat if needed
            if last_heartbeat.elapsed() >= heartbeat_interval {
                self.heartbeat().await?;
                last_heartbeat = std::time::Instant::now();
            }

            // Sync patterns if needed
            if last_sync.elapsed() >= sync_interval {
                self.sync_patterns().await?;
                last_sync = std::time::Instant::now();
            }

            // Small sleep to prevent busy loop
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Ok(())
    }

    /// Sync all shareable patterns
    async fn sync_patterns(&mut self) -> Result<(), SyncError> {
        let patterns: Vec<PatternMessage> = {
            let store = self.store.read().map_err(|_| SyncError::LockError)?;
            store
                .get_shareable(self.config.min_confidence)
                .into_iter()
                .take(self.config.batch_size)
                .cloned()
                .collect()
        };

        for pattern in patterns {
            let envelope = SignedEnvelope::new_plaintext(
                MessageType::Pattern,
                topics::PATTERNS,
                &pattern.to_bytes(),
                self.identity.keypair(),
            );

            self.transport
                .publish(envelope)
                .await
                .map_err(SyncError::Transport)?;
        }

        Ok(())
    }
}

/// Sync errors
#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("Transport error: {0}")]
    Transport(#[from] TransportError),

    #[error("Identity error: {0}")]
    Identity(String),

    #[error("Lock error")]
    LockError,

    #[error("Not running")]
    NotRunning,
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_store() {
        let mut store = PatternStore::new();

        let pattern = PatternMessage::new("list files", "ls -la", 12345);
        store.upsert(pattern);

        assert_eq!(store.len(), 1);
        assert!(store.get(12345).is_some());
    }

    #[test]
    fn test_pattern_store_merge() {
        let mut store = PatternStore::new();

        let pattern1 = PatternMessage {
            context_hash: 100,
            prompt_summary: "test".into(),
            command: "cmd1".into(),
            success_count: 5,
            failure_count: 1,
            confidence: 0.8,
            last_used: 1000,
            tags: vec![],
        };

        let pattern2 = PatternMessage {
            context_hash: 100,
            prompt_summary: "test".into(),
            command: "cmd2".into(),
            success_count: 3,
            failure_count: 0,
            confidence: 0.9,
            last_used: 2000,
            tags: vec![],
        };

        store.upsert(pattern1);
        store.upsert(pattern2);

        let merged = store.get(100).unwrap();
        assert_eq!(merged.success_count, 8);
        assert_eq!(merged.failure_count, 1);
        assert_eq!(merged.command, "cmd2"); // Newer command
        assert_eq!(merged.last_used, 2000);
    }

    #[test]
    fn test_sync_config_builders() {
        let gun = SyncConfig::with_gun();
        assert!(matches!(
            gun.transport.transport_type,
            super::super::transport::TransportType::Gun
        ));

        let nostr = SyncConfig::with_nostr();
        assert!(matches!(
            nostr.transport.transport_type,
            super::super::transport::TransportType::Nostr
        ));
    }

    #[tokio::test]
    async fn test_pattern_sync_create() {
        let config = SyncConfig::default();
        let sync = PatternSync::new(config).unwrap();

        assert!(!sync.is_running());
        assert_eq!(sync.peer_count(), 0);
    }
}
