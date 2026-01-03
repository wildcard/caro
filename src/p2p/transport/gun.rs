//! GUN.js transport adapter
//!
//! GUN is a decentralized, offline-first, real-time database.
//! Features:
//! - Offline-first: Works without internet
//! - Real-time sync: Changes propagate instantly
//! - Decentralized: No single point of failure
//! - HAM conflict resolution: Automatic merge
//!
//! This implementation provides a Rust interface to GUN relays.

use super::{Transport, TransportConfig, TransportError, TransportType};
use crate::p2p::envelope::SignedEnvelope;
use crate::p2p::NetworkStats;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::Duration;

/// GUN relay health status
#[derive(Debug, Clone)]
struct RelayHealth {
    url: String,
    healthy: bool,
    latency_ms: Option<u32>,
    last_check: u64,
    failure_count: u32,
}

impl RelayHealth {
    fn new(url: String) -> Self {
        Self {
            url,
            healthy: true,
            latency_ms: None,
            last_check: 0,
            failure_count: 0,
        }
    }

    fn mark_healthy(&mut self, latency_ms: u32) {
        self.healthy = true;
        self.latency_ms = Some(latency_ms);
        self.failure_count = 0;
        self.last_check = current_timestamp();
    }

    fn mark_unhealthy(&mut self) {
        self.failure_count += 1;
        if self.failure_count >= 3 {
            self.healthy = false;
        }
        self.last_check = current_timestamp();
    }
}

/// GUN transport state
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GunState {
    /// Key-value store (path -> data)
    data: HashMap<String, serde_json::Value>,
    /// Version vectors for conflict resolution
    versions: HashMap<String, u64>,
}

impl Default for GunState {
    fn default() -> Self {
        Self {
            data: HashMap::new(),
            versions: HashMap::new(),
        }
    }
}

/// GUN transport implementation
#[derive(Debug)]
pub struct GunTransport {
    config: TransportConfig,
    connected: bool,
    subscriptions: HashSet<String>,
    relays: Vec<RelayHealth>,
    state: Arc<RwLock<GunState>>,
    inbox: Arc<RwLock<VecDeque<SignedEnvelope>>>,
    stats: NetworkStats,
    #[cfg(feature = "reqwest")]
    client: Option<reqwest::Client>,
}

impl GunTransport {
    pub fn new(config: TransportConfig) -> Self {
        let relays = config
            .relays
            .iter()
            .map(|url| RelayHealth::new(url.clone()))
            .collect();

        Self {
            config,
            connected: false,
            subscriptions: HashSet::new(),
            relays,
            state: Arc::new(RwLock::new(GunState::default())),
            inbox: Arc::new(RwLock::new(VecDeque::new())),
            stats: NetworkStats::new(),
            #[cfg(feature = "reqwest")]
            client: None,
        }
    }

    /// Get healthy relays
    fn healthy_relays(&self) -> Vec<&RelayHealth> {
        self.relays.iter().filter(|r| r.healthy).collect()
    }

    /// Get the best relay (lowest latency)
    fn best_relay(&self) -> Option<&RelayHealth> {
        self.healthy_relays()
            .into_iter()
            .min_by_key(|r| r.latency_ms.unwrap_or(u32::MAX))
    }

    /// Build GUN path from topic
    fn topic_to_path(topic: &str) -> String {
        // Convert topic like "caro/patterns/v1" to "caro.patterns.v1"
        topic.replace('/', ".")
    }

    /// PUT data to GUN
    #[cfg(feature = "reqwest")]
    async fn gun_put(&self, path: &str, data: &serde_json::Value) -> Result<(), TransportError> {
        let client = self.client.as_ref().ok_or(TransportError::NotConnected)?;

        let relay = self.best_relay().ok_or(TransportError::RelayError(
            "No healthy relays available".into(),
        ))?;

        // GUN PUT format: /gun/<path>
        let url = format!("{}/{}", relay.url, path);

        let response = client
            .put(&url)
            .json(data)
            .timeout(self.config.request_timeout)
            .send()
            .await
            .map_err(|e| TransportError::SendFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(TransportError::SendFailed(format!(
                "GUN PUT failed: {}",
                response.status()
            )));
        }

        Ok(())
    }

    /// GET data from GUN
    #[cfg(feature = "reqwest")]
    async fn gun_get(&self, path: &str) -> Result<Option<serde_json::Value>, TransportError> {
        let client = self.client.as_ref().ok_or(TransportError::NotConnected)?;

        let relay = self.best_relay().ok_or(TransportError::RelayError(
            "No healthy relays available".into(),
        ))?;

        let url = format!("{}/{}", relay.url, path);

        let response = client
            .get(&url)
            .timeout(self.config.request_timeout)
            .send()
            .await
            .map_err(|e| TransportError::ReceiveFailed(e.to_string()))?;

        if response.status().is_success() {
            let data = response
                .json()
                .await
                .map_err(|e| TransportError::Serialization(e.to_string()))?;
            Ok(Some(data))
        } else if response.status().as_u16() == 404 {
            Ok(None)
        } else {
            Err(TransportError::ReceiveFailed(format!(
                "GUN GET failed: {}",
                response.status()
            )))
        }
    }

    /// Simulate GUN operations (when reqwest not available)
    #[cfg(not(feature = "reqwest"))]
    async fn gun_put(&self, path: &str, data: &serde_json::Value) -> Result<(), TransportError> {
        // Local-only simulation
        if let Ok(mut state) = self.state.write() {
            let version = state.versions.get(path).copied().unwrap_or(0) + 1;
            state.data.insert(path.to_string(), data.clone());
            state.versions.insert(path.to_string(), version);
        }
        Ok(())
    }

    #[cfg(not(feature = "reqwest"))]
    async fn gun_get(&self, path: &str) -> Result<Option<serde_json::Value>, TransportError> {
        if let Ok(state) = self.state.read() {
            Ok(state.data.get(path).cloned())
        } else {
            Ok(None)
        }
    }

    /// Check relay health
    async fn check_relay_health(&mut self, relay_idx: usize) {
        #[cfg(feature = "reqwest")]
        if let Some(client) = &self.client {
            if let Some(relay) = self.relays.get_mut(relay_idx) {
                let start = std::time::Instant::now();
                let result = client
                    .get(&relay.url)
                    .timeout(Duration::from_secs(5))
                    .send()
                    .await;

                match result {
                    Ok(_) => {
                        let latency = start.elapsed().as_millis() as u32;
                        relay.mark_healthy(latency);
                    }
                    Err(_) => {
                        relay.mark_unhealthy();
                    }
                }
            }
        }
    }

    /// HAM (Hypothetical Amnesia Machine) conflict resolution
    fn ham_merge(
        &self,
        local: &serde_json::Value,
        local_version: u64,
        remote: &serde_json::Value,
        remote_version: u64,
    ) -> (serde_json::Value, u64) {
        // GUN's HAM uses state-based CRDTs
        // Higher version wins; on tie, lexicographically larger value wins
        if remote_version > local_version {
            (remote.clone(), remote_version)
        } else if remote_version < local_version {
            (local.clone(), local_version)
        } else {
            // Same version: use deterministic tie-breaker
            let local_str = serde_json::to_string(local).unwrap_or_default();
            let remote_str = serde_json::to_string(remote).unwrap_or_default();
            if remote_str > local_str {
                (remote.clone(), remote_version)
            } else {
                (local.clone(), local_version)
            }
        }
    }
}

#[async_trait]
impl Transport for GunTransport {
    async fn connect(&mut self) -> Result<(), TransportError> {
        #[cfg(feature = "reqwest")]
        {
            self.client = Some(
                reqwest::Client::builder()
                    .timeout(self.config.connect_timeout)
                    .build()
                    .map_err(|e| TransportError::ConnectionFailed(e.to_string()))?,
            );
        }

        // Check health of all relays
        for i in 0..self.relays.len() {
            self.check_relay_health(i).await;
        }

        // Need at least one healthy relay
        if self.healthy_relays().is_empty() {
            #[cfg(feature = "reqwest")]
            return Err(TransportError::ConnectionFailed(
                "No healthy GUN relays available".into(),
            ));
        }

        self.connected = true;
        self.stats = NetworkStats::new();

        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), TransportError> {
        self.connected = false;
        self.subscriptions.clear();
        #[cfg(feature = "reqwest")]
        {
            self.client = None;
        }
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    async fn subscribe(&mut self, topic: &str) -> Result<(), TransportError> {
        if !self.connected {
            return Err(TransportError::NotConnected);
        }

        self.subscriptions.insert(topic.to_string());

        // In a full implementation, this would set up a WebSocket subscription
        // For now, we poll in receive()

        Ok(())
    }

    async fn unsubscribe(&mut self, topic: &str) -> Result<(), TransportError> {
        self.subscriptions.remove(topic);
        Ok(())
    }

    async fn publish(&mut self, envelope: SignedEnvelope) -> Result<(), TransportError> {
        if !self.connected {
            return Err(TransportError::NotConnected);
        }

        let path = Self::topic_to_path(&envelope.topic);

        // Serialize envelope
        let data = serde_json::to_value(&envelope)
            .map_err(|e| TransportError::Serialization(e.to_string()))?;

        // PUT to GUN
        self.gun_put(&path, &data).await?;

        let size = serde_json::to_vec(&envelope)
            .map(|v| v.len())
            .unwrap_or(0);
        self.stats.record_sent(size);

        Ok(())
    }

    async fn send(&mut self, envelope: SignedEnvelope) -> Result<(), TransportError> {
        // For GUN, send to specific peer uses a peer-addressed path
        if let Some(ref recipient) = envelope.recipient_id {
            let path = format!("peers.{}.inbox", recipient.as_str().replace('-', "_"));
            let data = serde_json::to_value(&envelope)
                .map_err(|e| TransportError::Serialization(e.to_string()))?;
            self.gun_put(&path, &data).await
        } else {
            self.publish(envelope).await
        }
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

        // Check inbox first
        if let Ok(mut inbox) = self.inbox.write() {
            if let Some(envelope) = inbox.pop_front() {
                let size = serde_json::to_vec(&envelope)
                    .map(|v| v.len())
                    .unwrap_or(0);
                self.stats.record_received(size);
                return Ok(Some(envelope));
            }
        }

        // Poll subscribed topics (in production, use WebSocket)
        for topic in self.subscriptions.clone() {
            let path = Self::topic_to_path(&topic);
            if let Some(data) = self.gun_get(&path).await? {
                if let Ok(envelope) = serde_json::from_value::<SignedEnvelope>(data) {
                    // Check if we've already seen this message
                    let hash = envelope.hash();
                    // In production, track seen message hashes
                    let size = serde_json::to_vec(&envelope)
                        .map(|v| v.len())
                        .unwrap_or(0);
                    self.stats.record_received(size);
                    return Ok(Some(envelope));
                }
            }
        }

        Ok(None)
    }

    fn stats(&self) -> NetworkStats {
        let mut stats = self.stats.clone();
        stats.connected_peers = self.healthy_relays().len();
        stats
    }

    fn transport_type(&self) -> TransportType {
        TransportType::Gun
    }
}

/// GUN sync manager for pattern synchronization
pub struct GunSync {
    transport: GunTransport,
    namespace: String,
}

impl GunSync {
    pub fn new(transport: GunTransport, namespace: impl Into<String>) -> Self {
        Self {
            transport,
            namespace: namespace.into(),
        }
    }

    /// Publish a pattern to GUN
    pub async fn publish_pattern(
        &mut self,
        pattern_id: &str,
        data: serde_json::Value,
    ) -> Result<(), TransportError> {
        let path = format!("{}.patterns.{}", self.namespace, pattern_id);
        self.transport.gun_put(&path, &data).await
    }

    /// Get a pattern from GUN
    pub async fn get_pattern(
        &mut self,
        pattern_id: &str,
    ) -> Result<Option<serde_json::Value>, TransportError> {
        let path = format!("{}.patterns.{}", self.namespace, pattern_id);
        self.transport.gun_get(&path).await
    }

    /// Announce peer presence
    pub async fn announce_peer(
        &mut self,
        peer_id: &str,
        info: serde_json::Value,
    ) -> Result<(), TransportError> {
        let path = format!("{}.peers.{}", self.namespace, peer_id.replace('-', "_"));
        self.transport.gun_put(&path, &info).await
    }
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

    #[tokio::test]
    async fn test_gun_transport_create() {
        let config = TransportConfig::gun();
        let transport = GunTransport::new(config);
        assert!(!transport.is_connected());
        assert!(!transport.relays.is_empty());
    }

    #[tokio::test]
    async fn test_gun_local_state() {
        let config = TransportConfig::gun();
        let mut transport = GunTransport::new(config);
        transport.connected = true; // Simulate connection

        let data = serde_json::json!({"key": "value"});
        transport.gun_put("test.path", &data).await.unwrap();

        let retrieved = transport.gun_get("test.path").await.unwrap();
        assert_eq!(retrieved, Some(data));
    }

    #[test]
    fn test_ham_merge() {
        let config = TransportConfig::gun();
        let transport = GunTransport::new(config);

        let local = serde_json::json!({"count": 5});
        let remote = serde_json::json!({"count": 10});

        // Higher version wins
        let (result, version) = transport.ham_merge(&local, 1, &remote, 2);
        assert_eq!(result, remote);
        assert_eq!(version, 2);

        // Lower version wins (local has higher version)
        let (result, version) = transport.ham_merge(&local, 3, &remote, 2);
        assert_eq!(result, local);
        assert_eq!(version, 3);

        // Same version: lexicographic comparison on JSON strings
        // {"count":10} < {"count":5} because '1' < '5' in ASCII
        let (result, _) = transport.ham_merge(&local, 1, &remote, 1);
        assert_eq!(result, local); // local wins because "5" > "10" lexicographically
    }

    #[test]
    fn test_topic_to_path() {
        assert_eq!(GunTransport::topic_to_path("caro/patterns/v1"), "caro.patterns.v1");
        assert_eq!(GunTransport::topic_to_path("test"), "test");
    }
}
