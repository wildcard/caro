# Data Privacy & Security Architecture

**Document Version**: 1.0
**Last Updated**: 2026-01-08
**Status**: Strategic Planning
**Owner**: Security Lead & Privacy Officer

---

## Executive Summary

This document defines Caro's comprehensive privacy and security architecture, demonstrating how privacy-first principles are embedded at every layer from local inference to optional cloud sync. It serves as both a technical specification and a trust-building transparency document.

**Core Commitment**: Privacy is not a feature—it's the foundation of Caro's architecture.

**Key Principles**:
1. **Local-first by default** - No cloud dependency for core functionality
2. **Zero-knowledge architecture** - If you use sync, we can't read your data
3. **Minimal data collection** - Collect only what's necessary, anonymize everything
4. **User control** - Transparent settings, easy opt-out, complete data export
5. **Open source transparency** - Code is auditable by anyone

---

## Privacy-First Architecture

### Architectural Layers

```
┌─────────────────────────────────────────────────────┐
│              User's Machine (100% Local)            │
│                                                     │
│  ┌────────────────────────────────────────────┐   │
│  │  User Input (Natural Language)             │   │
│  └──────────────────┬─────────────────────────┘   │
│                     │                               │
│  ┌──────────────────▼─────────────────────────┐   │
│  │  Static Matcher / Embedded LLM / MLX       │   │
│  │  (All inference happens locally)           │   │
│  └──────────────────┬─────────────────────────┘   │
│                     │                               │
│  ┌──────────────────▼─────────────────────────┐   │
│  │  Safety Validator (Local)                  │   │
│  │  - Pattern matching                        │   │
│  │  - Platform checks                         │   │
│  └──────────────────┬─────────────────────────┘   │
│                     │                               │
│  ┌──────────────────▼─────────────────────────┐   │
│  │  Generated Command                         │   │
│  │  (Never leaves machine by default)         │   │
│  └────────────────────────────────────────────┘   │
│                                                     │
│  Optional (User-Controlled):                       │
│  ┌────────────────────────────────────────────┐   │
│  │  Local History Database                    │   │
│  │  - SQLite on disk                          │   │
│  │  - User owns data                          │   │
│  │  - Easy to export/delete                   │   │
│  └────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────┘
                          │
                          │ (Optional, Encrypted)
                          │
┌─────────────────────────▼─────────────────────────┐
│         Cloud Sync (Optional, E2EE)               │
│                                                   │
│  ┌──────────────────────────────────────────┐   │
│  │  Encrypted Blob Storage                  │   │
│  │  - Server cannot decrypt                 │   │
│  │  - Zero-knowledge architecture           │   │
│  │  - User controls master key              │   │
│  └──────────────────────────────────────────┘   │
└───────────────────────────────────────────────────┘
```

**Key Design Decisions**:
- **No telemetry by default** - User must explicitly opt-in
- **No account required** - Works completely offline
- **No analytics cookies** - Website doesn't track users
- **No third-party services** - No Google Analytics, no tracking pixels

---

## Data Classification

### What Data Exists

#### Tier 1: Never Collected (Guaranteed)

**User Input (Natural Language Prompts)**:
- ❌ NEVER stored on our servers
- ❌ NEVER sent to cloud (unless user chooses cloud backend plugin)
- ❌ NEVER logged
- ✅ Stays on user's machine

**Generated Commands**:
- ❌ NEVER sent to our servers
- ✅ Optionally stored locally (user-controlled)
- ✅ Optionally synced encrypted (user-controlled)

**Command Output**:
- ❌ NEVER collected
- ❌ NEVER stored by us
- ✅ Terminal handles all output

**Environment Variables**:
- ❌ NEVER collected
- ❌ NEVER sent anywhere
- ✅ Filtered from any logs

**Secrets/API Keys**:
- ❌ NEVER collected
- ❌ NEVER transmitted
- ✅ Safety validator filters these

---

#### Tier 2: Collected Locally (User Owns)

**Command History** (if enabled by user):
- ✅ Stored in `~/.caro/history.db` (SQLite)
- ✅ User has full control (view, export, delete)
- ✅ Never leaves machine unless sync enabled

**Data Structure**:
```sql
CREATE TABLE history (
    id INTEGER PRIMARY KEY,
    timestamp TEXT NOT NULL,
    user_query TEXT NOT NULL,      -- Natural language
    generated_command TEXT NOT NULL,
    backend TEXT NOT NULL,
    platform TEXT NOT NULL,
    executed BOOLEAN DEFAULT FALSE,
    success BOOLEAN DEFAULT NULL
);
```

**Privacy Controls**:
```bash
# Disable history entirely
caro config set history.enabled false

# Clear all history
caro history clear --all

# Export history (for backup/migration)
caro history export history.json

# Delete specific entries
caro history delete --before "2026-01-01"
```

---

#### Tier 3: Optionally Synced (Encrypted)

**If User Enables Cloud Sync**:

**What Gets Synced** (encrypted):
- ✅ Command history (encrypted with user's master key)
- ✅ Configuration preferences
- ✅ Custom patterns and aliases

**What DOESN'T Get Synced**:
- ❌ Command output
- ❌ Environment variables
- ❌ Secrets or API keys
- ❌ File contents
- ❌ Raw prompts (only encrypted)

**Encryption Details**:
- Algorithm: AES-256-GCM
- Key derivation: Argon2id
- User controls master key (derived from passphrase)
- Server cannot decrypt (zero-knowledge)

---

#### Tier 4: Minimal Telemetry (Opt-In Only)

**If User Explicitly Opts In**:

**What's Collected** (anonymized and aggregated):
- ✅ Platform (macOS, Linux)
- ✅ Architecture (x86_64, ARM64)
- ✅ Backend used (static, embedded, mlx)
- ✅ Latency metrics (generation time)
- ✅ Success/failure (boolean)
- ✅ Error types (generic categories)

**What's NOT Collected**:
- ❌ User queries (natural language)
- ❌ Generated commands
- ❌ IP addresses
- ❌ User identifiers
- ❌ File paths
- ❌ Any personal information

**Telemetry Payload Example**:
```json
{
  "version": "1.2.0",
  "platform": "macos",
  "arch": "arm64",
  "backend": "mlx",
  "latency_ms": 123,
  "success": true,
  "timestamp": "2026-01-15T10:30:00Z",
  "session_id": "<random-uuid>"  // Rotates daily
}
```

**User Controls**:
```bash
# View current telemetry settings
caro config get telemetry.enabled
# → false (default)

# Enable telemetry
caro config set telemetry.enabled true

# View telemetry payload before sending
caro telemetry preview

# Disable telemetry
caro config set telemetry.enabled false
```

---

## Encryption Architecture

### Local Encryption (Command History)

**At-Rest Encryption** (optional):
```bash
# Enable local encryption
caro config set history.encrypt true
```

**Implementation**:
- Encryption key derived from system keychain (macOS) or keyring (Linux)
- AES-256-GCM for symmetric encryption
- Each entry encrypted individually
- Key rotation supported

**Benefits**:
- Protection against disk theft
- Protection against malware reading history
- User doesn't need to manage keys

---

### Cloud Sync Encryption (Zero-Knowledge)

**Architecture**:
```
User's Device                     Sync Server
─────────────                     ───────────

Master Key (user's passphrase)
     │
     ▼
Derived Key (Argon2id)
     │
     ▼
Encrypt Data (AES-256-GCM)
     │
     ▼
Encrypted Blob ────────────────► Store Blob
                                  (cannot decrypt)
     │
     ◄────────────────────────── Retrieve Blob
     │
     ▼
Decrypt Data (AES-256-GCM)
     │
     ▼
Plaintext Data (on device)
```

**Key Properties**:
1. **Zero-Knowledge**: Server never sees plaintext data or encryption keys
2. **End-to-End Encrypted**: Data encrypted on device, decrypted on device
3. **User-Controlled Keys**: User's passphrase is the only way to decrypt
4. **Forward Secrecy**: Keys rotated periodically

**Setup Flow**:
```bash
# Enable sync
$ caro sync enable

Enter a passphrase to encrypt your data:
[User enters passphrase]

Confirm passphrase:
[User confirms]

✅ Sync enabled with end-to-end encryption

⚠️  IMPORTANT: Store this passphrase safely!
   - We cannot recover your data if you lose it
   - We cannot reset your passphrase
   - Your data is encrypted with this passphrase

Write down your recovery key: [shows recovery key]
```

**Key Derivation**:
```rust
// Derive encryption key from user's passphrase
fn derive_key(passphrase: &str, salt: &[u8]) -> Key {
    let config = Argon2::default();
    let mut key = [0u8; 32];  // 256 bits

    config.hash_password_into(
        passphrase.as_bytes(),
        salt,
        &mut key
    ).unwrap();

    Key::from(key)
}

// Encrypt data
fn encrypt(data: &[u8], key: &Key) -> Vec<u8> {
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    cipher.encrypt(&nonce, data).unwrap()
}
```

**Security Properties**:
- **Argon2id**: Memory-hard KDF resistant to GPU attacks
- **AES-256-GCM**: Industry-standard authenticated encryption
- **Random Nonces**: Each encryption uses unique nonce
- **Authenticated**: Tampering detected via GMAC

---

## Security Architecture

### Threat Model

**Assets to Protect**:
1. User's command history
2. User's configuration (may contain sensitive patterns)
3. User's privacy (queries, commands)
4. User's system (from dangerous commands)

**Threat Actors**:
1. **Network Attackers**: Man-in-the-middle, eavesdropping
2. **Malicious Plugins**: Rogue plugins trying to exfiltrate data
3. **Server Compromise**: If sync server hacked
4. **Malware on User's System**: Keyloggers, data theft
5. **Insider Threats**: Rogue Caro team members

---

### Security Controls

#### 1. Network Security

**TLS 1.3 Only**:
```rust
// Sync client configuration
let tls_config = ClientConfig::builder()
    .with_safe_defaults()
    .with_min_protocol_version(Some(TlsVersion::TLS13))
    .with_certificate_verification();
```

**Certificate Pinning** (for sync service):
```rust
// Pin expected certificate
const SYNC_CERT_FINGERPRINT: &str = "sha256/...";

fn verify_cert(cert: &Certificate) -> Result<()> {
    let fingerprint = sha256_fingerprint(cert);
    if fingerprint != SYNC_CERT_FINGERPRINT {
        return Err(Error::CertificateMismatch);
    }
    Ok(())
}
```

**Benefits**:
- Prevents MITM attacks
- Protects against compromised CAs
- Ensures connecting to legitimate server

---

#### 2. Plugin Sandboxing

**WebAssembly Isolation**:
```
Plugin (WASM)
└─ Limited Capabilities
   ├─ No network access (by default)
   ├─ No file system access (by default)
   ├─ No environment variables (by default)
   └─ CPU/memory limits
```

**Permission Manifest**:
```toml
# plugin.toml
[permissions]
network = ["https://api.example.com"]  # Whitelist only
filesystem = ["~/.cache/my-plugin"]     # Specific paths only
environment = []                         # No env access
```

**Runtime Enforcement**:
```rust
// WASM runtime with restricted permissions
let engine = Engine::default();
let mut linker = Wasmtime::new(&engine);

// Only grant requested permissions
if plugin.has_network_permission("https://api.example.com") {
    linker.allow_network("https://api.example.com");
}

// All other capabilities denied by default
```

**User Consent**:
```bash
$ caro plugin install risky-plugin

⚠️  Permission Request:
This plugin requests:
  - Network access: https://external-api.com
  - File system access: ~/.cache/risky-plugin
  - Environment variables: API_KEY

Grant these permissions? [y/N]
```

---

#### 3. Secret Filtering

**Automatic Secret Detection**:
```rust
const SECRET_PATTERNS: &[Regex] = &[
    // API keys
    regex!(r"[A-Za-z0-9_-]{32,}"),

    // AWS keys
    regex!(r"AKIA[0-9A-Z]{16}"),

    // GitHub tokens
    regex!(r"ghp_[A-Za-z0-9_]{36}"),

    // Private keys
    regex!(r"-----BEGIN (RSA |)PRIVATE KEY-----"),

    // Environment-like
    regex!(r"\$[A-Z_]+"),
];

fn filter_secrets(text: &str) -> String {
    let mut filtered = text.to_string();

    for pattern in SECRET_PATTERNS {
        filtered = pattern.replace_all(
            &filtered,
            "[REDACTED]"
        ).to_string();
    }

    filtered
}
```

**Application**:
- Filter command history before storage
- Filter logs before writing
- Filter error messages before display
- Filter telemetry payloads

---

#### 4. Safe Defaults

**Configuration Defaults**:
```toml
[safety]
enabled = true          # Safety validation on by default
strict_mode = false     # Allow --unsafe flag if needed

[history]
enabled = false         # Opt-in for history

[sync]
enabled = false         # Opt-in for cloud sync

[telemetry]
enabled = false         # Opt-in for telemetry

[plugins]
auto_update = false     # User controls plugin updates
unsigned_plugins = false # Only signed plugins by default
```

**Principle**: Secure by default, opt-in for everything else

---

## Privacy Compliance

### GDPR Compliance

**Legal Basis**: Legitimate interest (product improvement) + Consent (for telemetry/sync)

**User Rights**:

**1. Right to Access**:
```bash
# Export all user data
caro data export --output my-data.zip

# Contents:
# - history.json (command history)
# - config.toml (configuration)
# - patterns.json (custom patterns)
```

**2. Right to Rectification**:
```bash
# Edit configuration
caro config set <key> <value>

# Edit history
caro history delete <id>
```

**3. Right to Erasure**:
```bash
# Delete all local data
caro data delete --all --confirm

# Delete cloud sync data
caro sync delete --confirm
```

**4. Right to Data Portability**:
```bash
# Export in JSON format
caro data export --format json
```

**5. Right to Object**:
```bash
# Disable telemetry
caro config set telemetry.enabled false
```

**6. Right to Restrict Processing**:
```bash
# Disable sync
caro sync disable
```

---

### CCPA Compliance

**Do Not Sell Disclosure**: We do not sell user data. Ever.

**Data Collection Notice**:
```
Privacy Policy (Short Version):

What we collect:
- Nothing by default (100% local)
- Encrypted history if you enable sync
- Anonymous metrics if you opt-in to telemetry

What we don't collect:
- Your commands or prompts
- Personal information
- Browsing history
- Location data

Your rights:
- Access your data: caro data export
- Delete your data: caro data delete
- Opt-out of telemetry: caro config set telemetry.enabled false

Questions? privacy@caro-cli.dev
```

---

### SOC 2 Type II (Enterprise)

**For Sync Service** (required for enterprise customers):

**Security Controls**:
- Access control (2FA for all admin access)
- Encryption at rest (database encrypted)
- Encryption in transit (TLS 1.3)
- Logging and monitoring (intrusion detection)
- Incident response (documented procedures)
- Vendor management (third-party audits)

**Compliance Controls**:
- Change management (documented approval process)
- Data retention (automated deletion after N days)
- Backup and recovery (daily backups, tested)
- Business continuity (disaster recovery plan)

**Audit**:
- Annual SOC 2 Type II audit (Q4 2027 target)
- Report available to enterprise customers under NDA

---

## Transparency & Auditability

### Open Source Transparency

**Everything is Open**:
```
github.com/caro-cli/caro
├── src/               # All source code (MIT license)
├── tests/             # All tests
├── docs/              # Documentation
└── security/          # Security documentation
```

**Benefits**:
- Anyone can audit our privacy claims
- Security researchers can review code
- Users can verify what runs on their machine
- Community can contribute improvements

---

### Privacy Policy

**Short Form** (displayed in CLI):
```
Caro Privacy Policy (Summary)

1. Local-First: Everything runs on your machine by default
2. Zero-Knowledge Sync: If you enable sync, we can't read your data
3. No Telemetry by Default: Opt-in only, anonymous if enabled
4. No Selling Data: We will never sell your data
5. User Control: Export, delete, opt-out anytime

Full policy: https://caro-cli.dev/privacy
```

**Full Policy** (website):
- Legal language for compliance
- Detailed data handling practices
- User rights and procedures
- Contact information

---

### Security Disclosures

**Vulnerability Reporting**:
```
Security Policy

Found a vulnerability? We appreciate responsible disclosure.

Email: security@caro-cli.dev
PGP Key: [key fingerprint]

We commit to:
- Acknowledge within 24 hours
- Provide updates every 48 hours
- Credit in release notes (if desired)

Rewards:
- Critical: $1000
- High: $500
- Medium: $200
- Low: $100
```

**Bug Bounty Program** (v2.0+):
- Managed via HackerOne or similar
- Clear scope and rules
- Fast response and patching

---

## Security Operations

### Incident Response

**Security Incident Levels**:

**S1 - Critical** (Data breach, server compromise):
- Response: Immediate (15 min)
- Team: All hands
- Communication: Public disclosure within 72 hours

**S2 - High** (Vulnerability discovered, exploit possible):
- Response: 4 hours
- Team: Security lead + engineering
- Communication: Security advisory on release

**S3 - Medium** (Minor vulnerability, low risk):
- Response: 24 hours
- Team: Security lead
- Communication: Patch notes

**S4 - Low** (Enhancement, hardening):
- Response: 1 week
- Team: Regular development
- Communication: Changelog

---

### Security Monitoring

**What We Monitor** (for sync service):
- Failed login attempts (brute force detection)
- Unusual access patterns (anomaly detection)
- Certificate expiration (automated alerts)
- System vulnerabilities (automated scanning)

**What We DON'T Monitor**:
- User queries or commands (we can't, E2EE)
- User behavior within CLI (local only)
- User IP addresses (not logged)

---

## Privacy-Preserving Analytics

### Differential Privacy

**For Aggregate Statistics** (opt-in telemetry):

**Goal**: Learn about usage without compromising individual privacy

**Technique**: Add calibrated noise to prevent individual identification

**Example**:
```rust
// Without differential privacy (DON'T DO THIS):
"45% of users use the embedded backend"

// With differential privacy (DO THIS):
fn add_laplace_noise(value: f64, epsilon: f64) -> f64 {
    let noise = Laplace::new(0.0, 1.0/epsilon).sample();
    value + noise
}

let true_percentage = 0.45;
let epsilon = 0.1;  // Privacy budget
let noisy_percentage = add_laplace_noise(true_percentage, epsilon);
// Result: ~0.47 (noise added, individual privacy protected)
```

**Privacy Budget**:
- ε = 0.1 (strong privacy)
- Applied to all aggregates
- Prevents individual re-identification

---

### Privacy-Preserving Learning

**Federated Learning** (future):

**Goal**: Improve models without collecting data

**Approach**:
1. User trains model locally on their data
2. Only model gradients sent (not data)
3. Server aggregates gradients
4. Updated model distributed to all users

**Benefits**:
- Individual data stays private
- Model improves from collective usage
- No central data repository

**Status**: Research for v2.x

---

## User Education

### Privacy Dashboards

**In-App Privacy Dashboard**:
```bash
$ caro privacy

Caro Privacy Status:

Data Collection:
  History: ✅ Enabled (stored locally)
  Sync: ❌ Disabled (no cloud data)
  Telemetry: ❌ Disabled (no metrics sent)

Your Data Location:
  History: ~/.caro/history.db (123 entries)
  Config: ~/.config/caro/config.toml
  Sync: Not configured

Actions:
  - Export your data: caro data export
  - Delete your data: caro data delete
  - Review privacy policy: caro privacy policy
```

---

### Privacy Tips

**Educational Content**:
```bash
$ caro privacy tips

Privacy Tips:

1. History is optional
   - Disable: caro config set history.enabled false

2. Sync is encrypted
   - We cannot read your data if you use sync

3. Telemetry is opt-in
   - We never enable it without your consent

4. Plugins need permissions
   - Review carefully before granting

5. You own your data
   - Export anytime: caro data export

Learn more: https://caro-cli.dev/docs/privacy
```

---

## Conclusion

### Privacy as Competitive Advantage

> "In a world where AI tools harvest every keystroke, Caro stands alone: privacy-first, user-controlled, transparent. This isn't just ethical—it's our moat."

### Trust Through Transparency

**What Sets Caro Apart**:
1. **Architectural Privacy**: Local-first isn't optional, it's fundamental
2. **Zero-Knowledge Sync**: E2EE means we can't spy even if we wanted to
3. **Open Source**: Code is auditable, claims are verifiable
4. **No Dark Patterns**: Opt-in for everything, clear controls
5. **User Ownership**: Your data, your control, always

### Continuous Improvement

**Ongoing Efforts**:
- Annual privacy audit (external firm)
- Annual security audit (penetration testing)
- Quarterly code review (security focus)
- Bug bounty program (responsible disclosure)
- SOC 2 certification (enterprise requirement)

### Success Metrics

- ✅ Zero data breaches (always)
- ✅ Zero GDPR/CCPA violations (always)
- ✅ <1% users worried about privacy (surveys)
- ✅ Privacy trust score >90% (user surveys)
- ✅ External security audit: No critical findings

---

## Document Control

**Version**: 1.0
**Created**: 2026-01-08
**Owner**: Security Lead & Privacy Officer
**Next Review**: 2026-04-01 (quarterly)
**Distribution**: All stakeholders, public

**Related Documents**:
- Product Evolution 2026-2027
- Risk Management Strategy
- Technical Architecture
- Legal & Compliance Guide

---

**Status**: ✅ Ready for Public Distribution

**Privacy is not negotiable. It's who we are. 🔒**
