# Trust & Cryptography Model

**Document**: Caro Distributed Mesh Security Architecture
**Version**: 1.0.0
**Date**: December 2025
**Status**: Design Complete

---

## Executive Summary

This document defines the cryptographic foundations and trust model for Caro's distributed mesh system. The design prioritizes:

1. **Zero-trust default**: All nodes are untrusted until explicitly verified
2. **Cryptographic identity**: Every node has a unique Ed25519 keypair
3. **End-to-end encryption**: All inter-node communication uses TLS 1.3
4. **Minimal trust requirements**: No central CA, no external PKI
5. **Air-gap compatibility**: No external dependencies for trust establishment

---

## 1. Cryptographic Primitives

### 1.1 Algorithm Selection

| Purpose | Algorithm | Crate | Justification |
|---------|-----------|-------|---------------|
| **Identity keys** | Ed25519 | `ring` | Fast, compact, audited |
| **Key exchange** | X25519 | `ring` | ECDH compatible with Ed25519 |
| **Transport** | TLS 1.3 | `rustls` | Modern, no legacy baggage |
| **Symmetric encryption** | ChaCha20-Poly1305 | `ring` | Fast on ARM, AEAD |
| **Hashing** | BLAKE3 | `blake3` | Fast, parallelizable |
| **Key derivation** | HKDF-SHA256 | `ring` | Standard, well-analyzed |

### 1.2 Why These Choices?

**Ed25519 over RSA/ECDSA:**
- 32-byte public keys (vs 256+ for RSA)
- 64-byte signatures (vs 256+ for RSA)
- Deterministic signatures (no nonce reuse vulnerability)
- Fast verification (~10x faster than RSA-2048)
- Audited implementation in `ring`

**X25519 over ECDH-P256:**
- Designed for ECDH (Ed25519 keys convertible to X25519)
- No point validation vulnerabilities
- Constant-time implementation

**BLAKE3 over SHA-256:**
- 2-4x faster on all platforms
- Parallelizable (benefits multi-core)
- No length extension attacks
- Keyed and incremental modes built-in

**ChaCha20-Poly1305 over AES-GCM:**
- No timing side channels without hardware AES
- Faster on ARM without AES extensions
- Simpler implementation (less attack surface)

---

## 2. Node Identity

### 2.1 Key Generation

```
┌─────────────────────────────────────────────────────────────────┐
│                    IDENTITY GENERATION                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1. Generate 32 bytes from OS CSPRNG                           │
│     └── ring::rand::SystemRandom                               │
│                                                                 │
│  2. Derive Ed25519 keypair                                      │
│     ├── Private key: 32 bytes (secret scalar)                  │
│     └── Public key: 32 bytes (curve point)                     │
│                                                                 │
│  3. Compute fingerprint                                         │
│     └── BLAKE3(public_key)[0:8] → 16 hex chars                 │
│                                                                 │
│  4. Format node ID                                              │
│     └── "caro:ed25519:<base64(public_key)>"                    │
│                                                                 │
│  5. Store securely                                              │
│     └── ~/.local/share/caro/identity.key (mode 0600)           │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 Key Storage Format

```toml
# ~/.local/share/caro/identity.key
# This file contains the node's private key
# Permissions MUST be 0600 (owner read/write only)

[identity]
version = 1
created_at = "2025-12-28T10:30:00Z"

# Ed25519 private key (32 bytes, base64)
# NEVER share this value
private_key = "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx="

# Derived public key (for reference)
public_key = "yyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyy="

# Human-readable fingerprint
fingerprint = "a1b2c3d4e5f6g7h8"
```

### 2.3 Key Derivation for X25519

Ed25519 and X25519 use the same curve (Curve25519), allowing key conversion:

```
Ed25519 private key → clamp → X25519 private key
Ed25519 public key → birational map → X25519 public key
```

This allows a single identity key to be used for both signing and key exchange.

### 2.4 Key Rotation

```
┌─────────────────────────────────────────────────────────────────┐
│                    KEY ROTATION PROTOCOL                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1. Generate new Ed25519 keypair                                │
│     └── new_public_key, new_private_key                         │
│                                                                 │
│  2. Create rotation endorsement                                  │
│     ┌──────────────────────────────────────────────────────┐   │
│     │ KeyRotation {                                         │   │
│     │   old_public_key: "caro:ed25519:OLD...",             │   │
│     │   new_public_key: "caro:ed25519:NEW...",             │   │
│     │   valid_from: "2025-12-28T12:00:00Z",                │   │
│     │   expires_at: "2025-12-29T12:00:00Z",                │   │
│     │   old_key_signature: sign(old_key, endorsement),     │   │
│     │   new_key_signature: sign(new_key, endorsement),     │   │
│     │ }                                                     │   │
│     └──────────────────────────────────────────────────────┘   │
│                                                                 │
│  3. Broadcast KeyRotation to all peers                          │
│                                                                 │
│  4. Peers update trust mappings                                 │
│     └── old_key trust → new_key trust                          │
│                                                                 │
│  5. Grace period (24 hours by default)                          │
│     └── Both keys accepted during transition                   │
│                                                                 │
│  6. Old key deprecated after grace period                       │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 3. Transport Security

### 3.1 Connection Establishment

```
┌─────────────────────────────────────────────────────────────────┐
│                 TLS 1.3 MUTUAL AUTHENTICATION                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Node A (Initiator)              Node B (Responder)             │
│       │                                │                        │
│       │──── ClientHello ──────────────►│                        │
│       │     + key_share (X25519)       │                        │
│       │     + signature_algorithms     │                        │
│       │                                │                        │
│       │◄─── ServerHello ───────────────│                        │
│       │     + key_share (X25519)       │                        │
│       │     + EncryptedExtensions      │                        │
│       │     + CertificateRequest       │                        │
│       │     + Certificate (self-signed)│                        │
│       │     + CertificateVerify        │                        │
│       │     + Finished                 │                        │
│       │                                │                        │
│       │──── Certificate ──────────────►│                        │
│       │     (self-signed Ed25519)      │                        │
│       │──── CertificateVerify ────────►│                        │
│       │──── Finished ─────────────────►│                        │
│       │                                │                        │
│       │◄═══ Application Data ═════════►│                        │
│       │     (ChaCha20-Poly1305)        │                        │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 3.2 Self-Signed Certificates

Each node generates a self-signed X.509 certificate containing:

```
Certificate:
    Subject: CN=<fingerprint>
    Issuer: CN=<fingerprint> (self-signed)
    Not Before: <creation_time>
    Not After: <creation_time + 1 year>
    Public Key: Ed25519 <public_key>
    Extensions:
        Subject Alternative Name:
            DNS: caro-<fingerprint>.local
            IP: (bound IP addresses)
        Key Usage: digitalSignature, keyAgreement
```

### 3.3 Certificate Validation

Caro does NOT use traditional PKI trust chains. Instead:

1. **First connection**: Store peer's public key fingerprint
2. **Subsequent connections**: Verify certificate matches stored fingerprint
3. **Trust decision**: Based on explicit configuration, not certificate chain

This is similar to SSH's `known_hosts` model (TOFU - Trust On First Use).

### 3.4 TLS Configuration

```rust
// rustls configuration for Caro nodes
let config = ServerConfig::builder()
    .with_cipher_suites(&[
        TLS13_CHACHA20_POLY1305_SHA256,
    ])
    .with_kx_groups(&[X25519])
    .with_protocol_versions(&[ProtocolVersion::TLSv1_3])
    .with_client_cert_verifier(CaroVerifier::new(trust_store))
    .with_single_cert(cert_chain, private_key)?;
```

---

## 4. Message Security

### 4.1 Message Signing

All messages include an Ed25519 signature:

```
┌─────────────────────────────────────────────────────────────────┐
│                    MESSAGE SIGNING                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Signed data = version || msg_type || sender || timestamp       │
│                || nonce || sequence || payload                  │
│                                                                 │
│  Signature = Ed25519_Sign(private_key, BLAKE3(signed_data))     │
│                                                                 │
│  Verification:                                                  │
│  1. Extract sender's public key from signed_data                │
│  2. Verify signature against public key                         │
│  3. Verify timestamp is within 5-minute window                  │
│  4. Verify nonce not in recent cache (1000 entries)            │
│  5. Verify sequence >= last seen sequence for this sender       │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 4.2 Replay Protection

Three-layer replay protection:

| Layer | Mechanism | Window |
|-------|-----------|--------|
| **Timestamp** | Reject messages > 5 min old | 5 minutes |
| **Nonce** | Cache recent nonces, reject duplicates | 1000 entries |
| **Sequence** | Monotonic sequence per sender | Infinite |

### 4.3 Message Confidentiality

After TLS handshake, all messages are encrypted with:

- **Algorithm**: ChaCha20-Poly1305
- **Key**: Derived from TLS 1.3 key schedule
- **Nonce**: 12 bytes, incrementing counter

---

## 5. Trust Model

### 5.1 Trust Levels

```
┌─────────────────────────────────────────────────────────────────┐
│                    TRUST LEVEL HIERARCHY                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Level 0: UNTRUSTED                                             │
│  └── No data exchange permitted                                 │
│  └── Connection rejected after authentication                   │
│                                                                 │
│  Level 1: SHARE_TO                                              │
│  └── Can receive our L1 summaries (push only)                  │
│  └── Cannot query our data                                      │
│                                                                 │
│  Level 2: QUERY_FROM                                            │
│  └── Can query our L1 summaries                                 │
│  └── Cannot receive proactive pushes                           │
│                                                                 │
│  Level 3: PEER                                                  │
│  └── Full bidirectional L1 exchange                             │
│  └── Can query and receive pushes                              │
│                                                                 │
│  Level 4: SUPERVISOR                                            │
│  └── Can query L2 (aggregated) data                            │
│  └── Cannot see L0 or L1 data                                  │
│  └── For security teams / admins                               │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 5.2 Trust Establishment Methods

```
┌─────────────────────────────────────────────────────────────────┐
│                TRUST ESTABLISHMENT OPTIONS                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Option A: Pre-Shared Configuration                             │
│  ──────────────────────────────────                             │
│  Administrator distributes trust config file:                   │
│                                                                 │
│  [peers.trusted]                                                │
│  "caro:ed25519:ABC123..." = { trust = "peer", name = "dev1" }  │
│  "caro:ed25519:DEF456..." = { trust = "supervisor" }           │
│                                                                 │
│  Pros: No interactive setup, enterprise-ready                   │
│  Cons: Requires key distribution mechanism                      │
│                                                                 │
│  ─────────────────────────────────────────────────────────────  │
│                                                                 │
│  Option B: Trust On First Use (TOFU)                            │
│  ───────────────────────────────────                            │
│  Interactive confirmation on first connection:                  │
│                                                                 │
│  "New peer detected:                                            │
│   Fingerprint: a1b2c3d4e5f6g7h8                                │
│   Name: dev-laptop                                              │
│   Trust as [P]eer / [S]upervisor / [U]ntrusted? "              │
│                                                                 │
│  Pros: Simple, no prior coordination                            │
│  Cons: Vulnerable to MITM on first connection                   │
│                                                                 │
│  ─────────────────────────────────────────────────────────────  │
│                                                                 │
│  Option C: Subnet-Based Trust                                   │
│  ───────────────────────────────                                │
│  Trust based on source IP range:                                │
│                                                                 │
│  [trust.subnets]                                                │
│  "10.0.1.0/24" = "peer"       # Engineering                    │
│  "10.0.2.0/24" = "supervisor" # Security team                  │
│                                                                 │
│  Pros: Works with network segmentation                          │
│  Cons: Trusts network, not cryptographic identity              │
│                                                                 │
│  ─────────────────────────────────────────────────────────────  │
│                                                                 │
│  Option D: Organization CA (Enterprise)                         │
│  ─────────────────────────────────────                          │
│  Organization runs internal CA that signs node certificates:    │
│                                                                 │
│  [trust.ca]                                                     │
│  root_cert = "/etc/caro/org-ca.crt"                            │
│  required_ou = "Engineering"                                    │
│                                                                 │
│  Pros: Leverages existing PKI                                   │
│  Cons: Requires CA infrastructure                               │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 5.3 Trust Revocation

```rust
/// Trust revocation mechanisms
enum RevocationMethod {
    /// Remove from trusted peers list
    ConfigRemoval,

    /// Add to explicit deny list
    ExplicitDeny {
        fingerprint: String,
        reason: String,
        expires: Option<DateTime<Utc>>,
    },

    /// Revoke via key rotation (old key becomes untrusted)
    KeyRotation,

    /// Subnet block
    SubnetBlock {
        cidr: String,
    },
}
```

---

## 6. Data Protection

### 6.1 Data Classification Enforcement

```
┌─────────────────────────────────────────────────────────────────┐
│               DATA CLASSIFICATION ENFORCEMENT                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Level 0 (Raw) - NEVER TRANSMITTED                              │
│  ─────────────────────────────────                              │
│  • Full command text                                            │
│  • File paths                                                   │
│  • Environment variables                                        │
│  • User prompts to Caro                                         │
│  • Working directories                                          │
│                                                                 │
│  ENFORCEMENT: These fields are not included in any             │
│  serializable struct that crosses network boundary.             │
│  TerminalEvent is ONLY stored locally.                          │
│                                                                 │
│  ─────────────────────────────────────────────────────────────  │
│                                                                 │
│  Level 1 (Summarized) - PEER CONSENT REQUIRED                   │
│  ─────────────────────────────────────────────                  │
│  • Command category counts (e.g., "git: 45")                   │
│  • Temporal patterns (hour/day distributions)                   │
│  • Risk level distributions                                     │
│  • Average durations by category                                │
│                                                                 │
│  ENFORCEMENT: NodeSummary struct explicitly excludes            │
│  raw command text. Code review required for changes.            │
│                                                                 │
│  ─────────────────────────────────────────────────────────────  │
│                                                                 │
│  Level 2 (Aggregated) - SUPERVISOR ACCESS                       │
│  ─────────────────────────────────────────                      │
│  • Cross-node aggregated patterns                               │
│  • Organization-wide risk metrics                               │
│  • Anomaly signals (no individual identification)               │
│                                                                 │
│  ENFORCEMENT: Aggregation functions produce new structs         │
│  that cannot contain per-node detail.                           │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 6.2 Privacy-Preserving Aggregation

When computing organization-wide metrics:

```rust
/// Aggregate summaries while preserving privacy
fn aggregate_summaries(
    summaries: &[NodeSummary],
    min_contributors: usize,  // Default: 5
) -> Option<AggregatedMetrics> {
    // Require minimum contributors to prevent identification
    if summaries.len() < min_contributors {
        return None;
    }

    // Aggregate without individual attribution
    let mut total_by_category: HashMap<Category, u64> = HashMap::new();

    for summary in summaries {
        for stat in &summary.category_stats {
            *total_by_category.entry(stat.category).or_default() += stat.count as u64;
        }
    }

    // Return aggregate without per-node breakdown
    Some(AggregatedMetrics {
        by_category: total_by_category,
        contributor_count: summaries.len(),
        // No individual node data included
    })
}
```

### 6.3 Data Retention

```toml
[storage.retention]
# Raw events (Level 0)
events_days = 30        # Delete after 30 days

# Summaries (Level 1)
summaries_days = 365    # Keep for 1 year

# Audit log
audit_days = 730        # Keep for 2 years (compliance)

# Aggregate cache
aggregate_hours = 24    # Cache aggregates for 24 hours
```

---

## 7. Threat Model

### 7.1 Threats Addressed

| Threat | Mitigation |
|--------|-----------|
| **Eavesdropping** | TLS 1.3 encryption |
| **MITM on first connection** | TOFU with fingerprint verification; pre-shared config option |
| **Node impersonation** | Ed25519 signatures on all messages |
| **Replay attacks** | Timestamp + nonce + sequence number |
| **Unauthorized data access** | Trust levels + policy enforcement |
| **Data exfiltration** | Level 0 never serialized; air-gap design |
| **Compromised node** | Can only expose own data; cannot forge others |
| **Key compromise** | Key rotation with signed endorsement |
| **Malicious aggregation** | Min-contributor threshold; cryptographic attribution |

### 7.2 Threats NOT Addressed

| Threat | Why Not | Mitigation |
|--------|---------|-----------|
| **Local malware** | Out of scope | OS security, endpoint protection |
| **Physical access** | Out of scope | Full disk encryption |
| **Timing attacks** | Low priority | Constant-time crypto (ring crate) |
| **Quantum attacks** | Future concern | Monitor PQC developments |

### 7.3 Attack Surface

```
┌─────────────────────────────────────────────────────────────────┐
│                    ATTACK SURFACE                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  NETWORK SURFACE                                                │
│  ├── TCP port 9238 (mesh protocol)                              │
│  │   └── Protected by: TLS 1.3, mutual auth, policy            │
│  ├── TCP port 9237 (local dashboard)                            │
│  │   └── Protected by: Bind to 127.0.0.1 only                  │
│  └── mDNS (optional discovery)                                  │
│      └── Protected by: Disabled by default; subnet scoped       │
│                                                                 │
│  LOCAL SURFACE                                                  │
│  ├── SQLite database (~/.local/share/caro/)                     │
│  │   └── Protected by: File permissions (0600)                 │
│  ├── Identity key file                                          │
│  │   └── Protected by: File permissions (0600)                 │
│  └── Config file                                                │
│      └── Protected by: File permissions (0644)                 │
│                                                                 │
│  SHELL SURFACE                                                  │
│  ├── Shell hooks (bash, zsh, fish)                              │
│  │   └── Protected by: User-initiated installation             │
│  └── Process observation                                        │
│      └── Protected by: Only own user's processes               │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 8. Security Audit Checklist

### 8.1 Implementation Checklist

- [ ] Ed25519 key generation uses OS CSPRNG
- [ ] Private key file created with 0600 permissions
- [ ] TLS 1.3 only (no fallback to older versions)
- [ ] Mutual authentication enforced on all connections
- [ ] All messages signed before transmission
- [ ] Signature verified before processing
- [ ] Timestamp within 5-minute window verified
- [ ] Nonce cache prevents replay
- [ ] Sequence numbers prevent reordering
- [ ] Trust policy evaluated before responding to queries
- [ ] Level 0 data never included in network messages
- [ ] Audit log is append-only with hash chain

### 8.2 Testing Requirements

- [ ] Fuzz testing for message parsing
- [ ] Timing analysis for crypto operations
- [ ] Invalid signature rejection test
- [ ] Replay attack prevention test
- [ ] Trust policy enforcement test
- [ ] Key rotation protocol test
- [ ] Connection timeout handling
- [ ] Malformed message handling

### 8.3 Operational Security

- [ ] Key backup procedure documented
- [ ] Key rotation procedure documented
- [ ] Trust revocation procedure documented
- [ ] Incident response procedure documented
- [ ] Audit log review procedure documented

---

## 9. Cryptographic API Design

### 9.1 Identity Module

```rust
pub mod identity {
    /// Generate a new node identity
    pub fn generate() -> Result<NodeIdentity, IdentityError>;

    /// Load identity from disk
    pub fn load(path: &Path) -> Result<NodeIdentity, IdentityError>;

    /// Save identity to disk (secure permissions)
    pub fn save(identity: &NodeIdentity, path: &Path) -> Result<(), IdentityError>;

    /// Compute fingerprint from public key
    pub fn fingerprint(public_key: &[u8; 32]) -> String;

    /// Convert Ed25519 public key to X25519 for key exchange
    pub fn to_x25519(ed_public: &[u8; 32]) -> [u8; 32];
}
```

### 9.2 Signing Module

```rust
pub mod signing {
    /// Sign a message with the node's private key
    pub fn sign(message: &[u8], private_key: &[u8; 32]) -> Signature;

    /// Verify a signature against a public key
    pub fn verify(
        message: &[u8],
        signature: &Signature,
        public_key: &[u8; 32],
    ) -> Result<(), SignatureError>;

    /// Create a signed wire message
    pub fn sign_message<T: Serialize>(
        payload: &T,
        identity: &NodeIdentity,
    ) -> Result<WireMessage, SigningError>;

    /// Verify and deserialize a wire message
    pub fn verify_message<T: DeserializeOwned>(
        message: &WireMessage,
        known_keys: &KeyStore,
    ) -> Result<(T, String), VerificationError>;
}
```

### 9.3 Transport Module

```rust
pub mod transport {
    /// Create a TLS client configuration
    pub fn client_config(
        identity: &NodeIdentity,
        trust_store: &TrustStore,
    ) -> Result<ClientConfig, TlsError>;

    /// Create a TLS server configuration
    pub fn server_config(
        identity: &NodeIdentity,
        trust_store: &TrustStore,
    ) -> Result<ServerConfig, TlsError>;

    /// Connect to a peer with mutual authentication
    pub async fn connect(
        addr: SocketAddr,
        config: &ClientConfig,
    ) -> Result<PeerConnection, ConnectError>;

    /// Accept a peer connection with mutual authentication
    pub async fn accept(
        stream: TcpStream,
        config: &ServerConfig,
    ) -> Result<PeerConnection, AcceptError>;
}
```

---

## 10. References

### 10.1 Standards

- RFC 8446: TLS 1.3
- RFC 8032: Ed25519
- RFC 7748: X25519
- RFC 8439: ChaCha20-Poly1305

### 10.2 Implementation References

- `ring` crate: https://github.com/briansmith/ring
- `rustls` crate: https://github.com/rustls/rustls
- `blake3` crate: https://github.com/BLAKE3-team/BLAKE3

### 10.3 Security Guidance

- OWASP Cryptographic Cheat Sheet
- NIST SP 800-57: Key Management Guidelines
- NIST SP 800-56A: Key Establishment Schemes

---

*This document defines the security architecture for Caro's distributed mesh. All implementations MUST follow these specifications. Deviations require security review and ADR amendment.*
