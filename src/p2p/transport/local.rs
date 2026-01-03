//! Local transport (in-memory, for testing)
//!
//! Provides a local-only transport that uses channels for message passing.

use super::{Transport, TransportConfig, TransportError, TransportType};
use crate::p2p::envelope::SignedEnvelope;
use crate::p2p::NetworkStats;
use async_trait::async_trait;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, RwLock};

/// Local transport for testing and single-node operation
#[derive(Debug)]
pub struct LocalTransport {
    config: TransportConfig,
    connected: bool,
    subscriptions: HashSet<String>,
    inbox: Arc<RwLock<VecDeque<SignedEnvelope>>>,
    outbox: Arc<RwLock<VecDeque<SignedEnvelope>>>,
    stats: NetworkStats,
}

impl LocalTransport {
    pub fn new(config: TransportConfig) -> Self {
        Self {
            config,
            connected: false,
            subscriptions: HashSet::new(),
            inbox: Arc::new(RwLock::new(VecDeque::new())),
            outbox: Arc::new(RwLock::new(VecDeque::new())),
            stats: NetworkStats::new(),
        }
    }

    /// Get shared inbox for testing
    pub fn inbox(&self) -> Arc<RwLock<VecDeque<SignedEnvelope>>> {
        self.inbox.clone()
    }

    /// Get shared outbox for testing
    pub fn outbox(&self) -> Arc<RwLock<VecDeque<SignedEnvelope>>> {
        self.outbox.clone()
    }

    /// Inject a message into the inbox (for testing)
    pub fn inject_message(&self, envelope: SignedEnvelope) {
        if let Ok(mut inbox) = self.inbox.write() {
            inbox.push_back(envelope);
        }
    }
}

#[async_trait]
impl Transport for LocalTransport {
    async fn connect(&mut self) -> Result<(), TransportError> {
        self.connected = true;
        self.stats.active_since = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        );
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), TransportError> {
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
        self.subscriptions.insert(topic.to_string());
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

        // Check topic subscription
        if !self.subscriptions.contains(&envelope.topic) {
            // Still allow publishing, but log
        }

        let size = serde_json::to_vec(&envelope)
            .map(|v| v.len())
            .unwrap_or(0);

        // For local transport, messages go to outbox
        if let Ok(mut outbox) = self.outbox.write() {
            outbox.push_back(envelope);
            self.stats.record_sent(size);
        }

        Ok(())
    }

    async fn send(&mut self, envelope: SignedEnvelope) -> Result<(), TransportError> {
        // For local transport, send is the same as publish
        self.publish(envelope).await
    }

    async fn receive(&mut self) -> Result<SignedEnvelope, TransportError> {
        if !self.connected {
            return Err(TransportError::NotConnected);
        }

        // Block until message available (with timeout simulation)
        loop {
            if let Some(envelope) = self.try_receive().await? {
                return Ok(envelope);
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }

    async fn try_receive(&mut self) -> Result<Option<SignedEnvelope>, TransportError> {
        if !self.connected {
            return Err(TransportError::NotConnected);
        }

        if let Ok(mut inbox) = self.inbox.write() {
            if let Some(envelope) = inbox.pop_front() {
                // Check if we're subscribed to this topic
                if self.subscriptions.contains(&envelope.topic) || envelope.topic.is_empty() {
                    let size = serde_json::to_vec(&envelope)
                        .map(|v| v.len())
                        .unwrap_or(0);
                    self.stats.record_received(size);
                    return Ok(Some(envelope));
                } else {
                    // Put it back if we're not subscribed
                    inbox.push_front(envelope);
                }
            }
        }

        Ok(None)
    }

    fn stats(&self) -> NetworkStats {
        self.stats.clone()
    }

    fn transport_type(&self) -> TransportType {
        TransportType::Local
    }
}

/// Local transport hub for connecting multiple local transports
#[derive(Debug, Default)]
pub struct LocalHub {
    transports: Arc<RwLock<HashMap<String, Arc<RwLock<VecDeque<SignedEnvelope>>>>>>,
}

impl LocalHub {
    pub fn new() -> Self {
        Self {
            transports: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a transport's inbox with the hub
    pub fn register(&self, id: &str, inbox: Arc<RwLock<VecDeque<SignedEnvelope>>>) {
        if let Ok(mut transports) = self.transports.write() {
            transports.insert(id.to_string(), inbox);
        }
    }

    /// Broadcast a message to all registered transports
    pub fn broadcast(&self, envelope: SignedEnvelope, exclude: Option<&str>) {
        if let Ok(transports) = self.transports.read() {
            for (id, inbox) in transports.iter() {
                if exclude.map(|e| e != id).unwrap_or(true) {
                    if let Ok(mut inbox) = inbox.write() {
                        inbox.push_back(envelope.clone());
                    }
                }
            }
        }
    }

    /// Send to a specific transport
    pub fn send_to(&self, id: &str, envelope: SignedEnvelope) -> Result<(), TransportError> {
        if let Ok(transports) = self.transports.read() {
            if let Some(inbox) = transports.get(id) {
                if let Ok(mut inbox) = inbox.write() {
                    inbox.push_back(envelope);
                    return Ok(());
                }
            }
        }
        Err(TransportError::SendFailed(format!("Transport {} not found", id)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::p2p::envelope::MessageType;
    use crate::p2p::identity::KeyPair;

    #[tokio::test]
    async fn test_local_transport_connect() {
        let config = TransportConfig::local();
        let mut transport = LocalTransport::new(config);

        assert!(!transport.is_connected());
        transport.connect().await.unwrap();
        assert!(transport.is_connected());
        transport.disconnect().await.unwrap();
        assert!(!transport.is_connected());
    }

    #[tokio::test]
    async fn test_local_transport_subscribe() {
        let config = TransportConfig::local();
        let mut transport = LocalTransport::new(config);

        transport.connect().await.unwrap();
        transport.subscribe("test/topic").await.unwrap();

        let keypair = KeyPair::generate();
        let envelope = SignedEnvelope::new_plaintext(
            MessageType::Data,
            "test/topic",
            b"hello",
            &keypair,
        );

        transport.inject_message(envelope.clone());

        let received = transport.try_receive().await.unwrap();
        assert!(received.is_some());
    }

    #[tokio::test]
    async fn test_local_hub() {
        let hub = LocalHub::new();

        let config = TransportConfig::local();
        let transport1 = LocalTransport::new(config.clone());
        let transport2 = LocalTransport::new(config);

        hub.register("t1", transport1.inbox());
        hub.register("t2", transport2.inbox());

        let keypair = KeyPair::generate();
        let envelope = SignedEnvelope::new_plaintext(
            MessageType::Data,
            "broadcast",
            b"hello all",
            &keypair,
        );

        hub.broadcast(envelope, Some("t1")); // Exclude t1

        // t2 should have received, t1 should not
        assert_eq!(transport1.inbox.read().unwrap().len(), 0);
        assert_eq!(transport2.inbox.read().unwrap().len(), 1);
    }
}
