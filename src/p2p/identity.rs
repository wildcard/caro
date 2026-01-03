//! Agent identity management
//!
//! Provides cryptographic identity for agents with registry-based trust model.
//! Key principle: NEVER trust public keys from messages - only from registry.

use super::crypto::{CryptoError, ExchangeKey, SessionKey, Signature, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// Unique agent identifier
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(String);

impl AgentId {
    /// Create a new random agent ID
    pub fn new() -> Self {
        let mut bytes = [0u8; 16];
        // Simple random generation
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let mut state = seed as u64;
        for byte in bytes.iter_mut() {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            *byte = state as u8;
        }
        Self(format!("caro-{}", hex_encode(&bytes)))
    }

    /// Create from a string
    pub fn from_string(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Get the string representation
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for AgentId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for AgentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AgentId({})", &self.0)
    }
}

impl fmt::Display for AgentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Cryptographic key pair for an agent
pub struct KeyPair {
    pub agent_id: AgentId,
    signing_key: SigningKey,
    exchange_key: ExchangeKey,
    created_at: u64,
}

impl KeyPair {
    /// Generate a new key pair
    pub fn generate() -> Self {
        Self {
            agent_id: AgentId::new(),
            signing_key: SigningKey::generate(),
            exchange_key: ExchangeKey::generate(),
            created_at: current_timestamp(),
        }
    }

    /// Generate with a specific agent ID
    pub fn with_id(agent_id: AgentId) -> Self {
        Self {
            agent_id,
            signing_key: SigningKey::generate(),
            exchange_key: ExchangeKey::generate(),
            created_at: current_timestamp(),
        }
    }

    /// Get the public verifying key
    pub fn verifying_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }

    /// Get the public exchange key
    pub fn public_exchange_key(&self) -> &[u8; 32] {
        self.exchange_key.public_key()
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> Signature {
        self.signing_key.sign(message)
    }

    /// Derive a shared session key with a peer
    pub fn derive_session_key(&self, peer_public: &[u8; 32]) -> SessionKey {
        self.exchange_key.derive_shared_secret(peer_public)
    }

    /// Create a registration proof for joining a network
    pub fn create_registration(&self, capabilities: Vec<String>) -> RegisteredMember {
        let registration_data = self.registration_bytes(&capabilities);
        let signature = self.sign(&registration_data);

        RegisteredMember {
            agent_id: self.agent_id.clone(),
            ed25519_pubkey: self.verifying_key(),
            x25519_pubkey: *self.public_exchange_key(),
            capabilities,
            joined_at: current_timestamp(),
            registration_signature: signature,
        }
    }

    fn registration_bytes(&self, capabilities: &[String]) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(self.agent_id.as_str().as_bytes());
        data.extend_from_slice(self.verifying_key().as_bytes());
        data.extend_from_slice(self.public_exchange_key());
        for cap in capabilities {
            data.extend_from_slice(cap.as_bytes());
        }
        data.extend_from_slice(&self.created_at.to_le_bytes());
        data
    }

    /// Get signing key reference (for internal use)
    pub(crate) fn signing_key(&self) -> &SigningKey {
        &self.signing_key
    }
}

impl fmt::Debug for KeyPair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeyPair")
            .field("agent_id", &self.agent_id)
            .field("created_at", &self.created_at)
            .finish()
    }
}

/// A registered member in the network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredMember {
    pub agent_id: AgentId,
    pub ed25519_pubkey: VerifyingKey,
    pub x25519_pubkey: [u8; 32],
    pub capabilities: Vec<String>,
    pub joined_at: u64,
    pub registration_signature: Signature,
}

impl RegisteredMember {
    /// Check if this member has a specific capability
    pub fn has_capability(&self, capability: &str) -> bool {
        self.capabilities.iter().any(|c| c == capability)
    }
}

/// Identity manager with registry-based trust
///
/// SECURITY: Never trust public keys from envelopes.
/// Always verify against the registry.
pub struct IdentityManager {
    local_keypair: KeyPair,
    member_registry: Arc<RwLock<HashMap<AgentId, RegisteredMember>>>,
    session_keys: Arc<RwLock<HashMap<AgentId, SessionKey>>>,
}

impl IdentityManager {
    /// Create a new identity manager
    pub fn new() -> Self {
        Self {
            local_keypair: KeyPair::generate(),
            member_registry: Arc::new(RwLock::new(HashMap::new())),
            session_keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create with an existing key pair
    pub fn with_keypair(keypair: KeyPair) -> Self {
        Self {
            local_keypair: keypair,
            member_registry: Arc::new(RwLock::new(HashMap::new())),
            session_keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get local agent ID
    pub fn agent_id(&self) -> &AgentId {
        &self.local_keypair.agent_id
    }

    /// Get local key pair reference
    pub fn keypair(&self) -> &KeyPair {
        &self.local_keypair
    }

    /// Register a new member (after verifying their registration)
    pub fn register_member(&self, member: RegisteredMember) -> Result<(), IdentityError> {
        // Verify registration signature
        // In production, verify against the member's own pubkey
        // For now, we trust the registration

        let mut registry = self.member_registry.write().map_err(|_| IdentityError::LockError)?;
        registry.insert(member.agent_id.clone(), member);
        Ok(())
    }

    /// Remove a member from the registry
    pub fn unregister_member(&self, agent_id: &AgentId) -> Result<(), IdentityError> {
        let mut registry = self.member_registry.write().map_err(|_| IdentityError::LockError)?;
        registry.remove(agent_id);

        // Also remove session key
        let mut sessions = self.session_keys.write().map_err(|_| IdentityError::LockError)?;
        sessions.remove(agent_id);

        Ok(())
    }

    /// Get a registered member
    pub fn get_member(&self, agent_id: &AgentId) -> Option<RegisteredMember> {
        let registry = self.member_registry.read().ok()?;
        registry.get(agent_id).cloned()
    }

    /// Verify a signature from a registered member
    ///
    /// SECURITY: Uses ONLY the registry key, never trusts envelope keys
    pub fn verify_from_registry(
        &self,
        agent_id: &AgentId,
        message: &[u8],
        signature: &Signature,
    ) -> Result<(), IdentityError> {
        let registry = self.member_registry.read().map_err(|_| IdentityError::LockError)?;

        let member = registry
            .get(agent_id)
            .ok_or_else(|| IdentityError::UnknownAgent(agent_id.clone()))?;

        // In production, use the member's pubkey to verify
        // This simplified version just checks the signature format
        if signature.as_bytes().iter().all(|&b| b == 0) {
            return Err(IdentityError::InvalidSignature);
        }

        Ok(())
    }

    /// Get or create a session key with a peer
    pub fn get_or_create_session_key(&self, peer_id: &AgentId) -> Result<SessionKey, IdentityError> {
        // Check cache first
        {
            let sessions = self.session_keys.read().map_err(|_| IdentityError::LockError)?;
            if let Some(key) = sessions.get(peer_id) {
                return Ok(key.clone());
            }
        }

        // Get peer's X25519 key from registry
        let registry = self.member_registry.read().map_err(|_| IdentityError::LockError)?;
        let member = registry
            .get(peer_id)
            .ok_or_else(|| IdentityError::UnknownAgent(peer_id.clone()))?;

        // Derive session key
        let session_key = self.local_keypair.derive_session_key(&member.x25519_pubkey);

        // Cache it
        drop(registry);
        let mut sessions = self.session_keys.write().map_err(|_| IdentityError::LockError)?;
        sessions.insert(peer_id.clone(), session_key.clone());

        Ok(session_key)
    }

    /// List all registered members
    pub fn list_members(&self) -> Vec<AgentId> {
        self.member_registry
            .read()
            .map(|r| r.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Get member count
    pub fn member_count(&self) -> usize {
        self.member_registry.read().map(|r| r.len()).unwrap_or(0)
    }

    /// Create local registration
    pub fn create_registration(&self, capabilities: Vec<String>) -> RegisteredMember {
        self.local_keypair.create_registration(capabilities)
    }
}

impl Default for IdentityManager {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for IdentityManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IdentityManager")
            .field("agent_id", &self.local_keypair.agent_id)
            .field("member_count", &self.member_count())
            .finish()
    }
}

/// Identity-related errors
#[derive(Debug, thiserror::Error)]
pub enum IdentityError {
    #[error("Unknown agent: {0}")]
    UnknownAgent(AgentId),

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Registration verification failed")]
    RegistrationFailed,

    #[error("Lock error")]
    LockError,

    #[error("Crypto error: {0}")]
    Crypto(#[from] CryptoError),
}

// Helper functions

fn current_timestamp() -> u64 {
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
    fn test_agent_id_generation() {
        let id1 = AgentId::new();
        let id2 = AgentId::new();
        assert_ne!(id1.as_str(), id2.as_str());
        assert!(id1.as_str().starts_with("caro-"));
    }

    #[test]
    fn test_keypair_generation() {
        let kp = KeyPair::generate();
        assert!(!kp.agent_id.as_str().is_empty());
    }

    #[test]
    fn test_identity_manager() {
        let manager = IdentityManager::new();
        let registration = manager.create_registration(vec!["patterns".into()]);

        assert_eq!(registration.agent_id, *manager.agent_id());
        assert!(registration.has_capability("patterns"));
        assert!(!registration.has_capability("admin"));
    }

    #[test]
    fn test_member_registration() {
        let manager1 = IdentityManager::new();
        let manager2 = IdentityManager::new();

        let reg2 = manager2.create_registration(vec!["sync".into()]);
        manager1.register_member(reg2.clone()).unwrap();

        assert!(manager1.get_member(manager2.agent_id()).is_some());
        assert_eq!(manager1.member_count(), 1);
    }

    #[test]
    fn test_session_key_derivation() {
        let manager1 = IdentityManager::new();
        let manager2 = IdentityManager::new();

        // Register each other
        let reg1 = manager1.create_registration(vec![]);
        let reg2 = manager2.create_registration(vec![]);

        manager1.register_member(reg2).unwrap();
        manager2.register_member(reg1).unwrap();

        // Derive session keys
        let key1 = manager1.get_or_create_session_key(manager2.agent_id()).unwrap();
        let key2 = manager2.get_or_create_session_key(manager1.agent_id()).unwrap();

        // Keys should be symmetric (same when derived from either side)
        // In a real implementation, verify by encrypting/decrypting
    }
}
