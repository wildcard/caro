# Security and Privacy Design

## Threat Model

### Assets to Protect

1. **User Commands**: Shell command history and content
2. **Secrets**: API keys, passwords, tokens in commands/environment
3. **System Integrity**: Prevent dangerous command execution
4. **User Privacy**: Command patterns, working directories, usage data

### Threat Actors

| Actor | Capability | Motivation |
|-------|------------|------------|
| Malicious software on same system | Read/write access to Caro files | Data exfiltration |
| Network attacker | If telemetry enabled, intercept data | Reconnaissance |
| Other system users | Access to shared /tmp | Privilege escalation |
| Caro itself (bugs) | Unintended data exposure | N/A |

### Attack Vectors

1. **Socket Hijacking**: Attacker creates fake socket
2. **Config Injection**: Malicious config file modifications
3. **Log Exposure**: Sensitive data in log files
4. **Memory Exposure**: Secrets in process memory
5. **Supply Chain**: Compromised Caro binary

---

## Security Controls

### 1. Socket Security

**Threat**: Another user or process impersonates Caro daemon

**Controls**:
```rust
// Socket file permissions
fn create_socket(path: &Path) -> Result<UnixListener> {
    // Remove existing socket
    let _ = std::fs::remove_file(path);

    // Create with restrictive permissions
    let listener = UnixListener::bind(path)?;

    // Set socket permissions to user-only
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;

    // Verify ownership
    let metadata = std::fs::metadata(path)?;
    if metadata.uid() != nix::unistd::getuid().as_raw() {
        return Err(anyhow!("Socket ownership mismatch"));
    }

    Ok(listener)
}
```

**Socket Location Priority**:
1. `$XDG_RUNTIME_DIR/caro-$UID.sock` (protected by runtime dir permissions)
2. `/tmp/caro-$UID.sock` (less secure, but socket has 0600)
3. `~/.config/caro/caro.sock` (fallback)

**Verification on Connect**:
```rust
fn verify_peer(stream: &UnixStream) -> Result<()> {
    // Verify peer is same user
    let cred = stream.peer_cred()?;
    if cred.uid() != nix::unistd::getuid().as_raw() {
        return Err(anyhow!("Peer UID mismatch"));
    }
    Ok(())
}
```

### 2. Secret Redaction

**Threat**: Secrets captured in logs or transmitted to external services

**Controls**:

```rust
/// Patterns that indicate sensitive data
const REDACTION_PATTERNS: &[&str] = &[
    // API Keys / Tokens
    r"(?i)(api[_-]?key|apikey|access[_-]?token|auth[_-]?token|bearer)[=:\s]+['\"]?[\w\-\.]+",
    r"(?i)(sk|pk|rk|ak)[_-][a-zA-Z0-9]{20,}",  // Stripe-like keys

    // Passwords
    r"(?i)(password|passwd|pwd|secret)[=:\s]+['\"]?[^\s'\"]+",
    r"--password[=\s]+[^\s]+",
    r"-p\s+[^\s]+(?=\s|$)",  // mysql -p

    // AWS
    r"AKIA[0-9A-Z]{16}",  // AWS Access Key ID
    r"(?i)aws[_-]?secret[_-]?access[_-]?key[=:\s]+['\"]?[\w/\+]+",

    // Private Keys
    r"-----BEGIN (RSA |EC |DSA |OPENSSH )?PRIVATE KEY-----",

    // URLs with credentials
    r"[a-zA-Z]+://[^:]+:[^@]+@",

    // Credit Cards (basic pattern)
    r"\b\d{4}[- ]?\d{4}[- ]?\d{4}[- ]?\d{4}\b",

    // Environment exports
    r"export\s+\w*(KEY|SECRET|TOKEN|PASSWORD|CREDENTIAL)\w*=",
];

pub fn redact_secrets(input: &str) -> String {
    let mut result = input.to_string();

    for pattern in REDACTION_PATTERNS {
        let re = Regex::new(pattern).unwrap();
        result = re.replace_all(&result, "[REDACTED]").to_string();
    }

    result
}
```

**Where Redaction Applies**:
- Log files (always)
- Session history storage (always)
- Debug output (always)
- Telemetry (if ever enabled)
- Fix-it suggestions display (optional)

### 3. Environment Variable Filtering

**Threat**: Secrets in environment captured by execution context

**Controls**:

```rust
/// Environment variables never captured
const BLOCKED_ENV_VARS: &[&str] = &[
    // Authentication
    "AWS_SECRET_ACCESS_KEY",
    "AWS_SESSION_TOKEN",
    "GITHUB_TOKEN",
    "GITLAB_TOKEN",
    "NPM_TOKEN",
    "DOCKER_PASSWORD",

    // API Keys
    "OPENAI_API_KEY",
    "ANTHROPIC_API_KEY",
    "STRIPE_SECRET_KEY",
    "SENDGRID_API_KEY",

    // Database credentials
    "DATABASE_URL",
    "DATABASE_PASSWORD",
    "PGPASSWORD",
    "MYSQL_PWD",
    "MONGO_PASSWORD",

    // Generic secrets
    "SECRET_KEY",
    "ENCRYPTION_KEY",
    "PRIVATE_KEY",
    "API_KEY",
];

/// Patterns for additional env var filtering
const BLOCKED_ENV_PATTERNS: &[&str] = &[
    r".*_(SECRET|KEY|TOKEN|PASSWORD|CREDENTIAL|PASSWD|PWD)$",
    r".*_(AUTH|PRIVATE)_.*",
];

pub fn filter_environment(env: &HashMap<String, String>) -> HashMap<String, String> {
    env.iter()
        .filter(|(key, _)| {
            // Explicit blocklist
            if BLOCKED_ENV_VARS.contains(&key.as_str()) {
                return false;
            }

            // Pattern-based filtering
            for pattern in BLOCKED_ENV_PATTERNS {
                if Regex::new(pattern).unwrap().is_match(key) {
                    return false;
                }
            }

            true
        })
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}
```

### 4. File Permissions

**Threat**: Other users access Caro data files

**Controls**:

```rust
/// Ensure directory has correct permissions
fn secure_directory(path: &Path) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }

    // Set directory permissions to 700 (user only)
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))?;

    // Verify no world/group readable
    let metadata = std::fs::metadata(path)?;
    let mode = metadata.permissions().mode();
    if mode & 0o077 != 0 {
        return Err(anyhow!("Insecure directory permissions: {:o}", mode));
    }

    Ok(())
}

/// Ensure file has correct permissions
fn secure_file(path: &Path) -> Result<()> {
    // Set file permissions to 600 (user only)
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    Ok(())
}
```

**Permission Matrix**:

| Path | Permissions | Contents |
|------|-------------|----------|
| `~/.config/caro/` | 700 | Configuration |
| `~/.config/caro/config.toml` | 600 | Settings |
| `~/.local/share/caro/` | 700 | User data |
| `~/.local/share/caro/history/` | 700 | Command history |
| `~/.cache/caro/` | 700 | Cache files |
| Socket | 600 | IPC |

### 5. History Encryption (Optional)

**Threat**: Command history accessed by attacker with filesystem access

**Controls**:

```rust
use aes_gcm::{Aes256Gcm, Key, Nonce};
use argon2::Argon2;

/// Encrypted history storage
pub struct EncryptedHistory {
    cipher: Aes256Gcm,
    nonce_counter: u64,
}

impl EncryptedHistory {
    /// Initialize with user passphrase
    pub fn new(passphrase: &str) -> Result<Self> {
        // Derive key from passphrase
        let salt = Self::get_or_create_salt()?;
        let mut key_bytes = [0u8; 32];

        Argon2::default()
            .hash_password_into(
                passphrase.as_bytes(),
                &salt,
                &mut key_bytes,
            )?;

        let cipher = Aes256Gcm::new(Key::from_slice(&key_bytes));

        Ok(Self {
            cipher,
            nonce_counter: 0,
        })
    }

    /// Encrypt command for storage
    pub fn encrypt(&mut self, plaintext: &str) -> Result<Vec<u8>> {
        let nonce = self.next_nonce();
        let ciphertext = self.cipher.encrypt(&nonce, plaintext.as_bytes())?;

        // Prepend nonce to ciphertext
        let mut result = nonce.to_vec();
        result.extend(ciphertext);
        Ok(result)
    }

    /// Decrypt command from storage
    pub fn decrypt(&self, data: &[u8]) -> Result<String> {
        if data.len() < 12 {
            return Err(anyhow!("Invalid encrypted data"));
        }

        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        let plaintext = self.cipher.decrypt(nonce, ciphertext)?;

        String::from_utf8(plaintext)
            .map_err(|e| anyhow!("Invalid UTF-8 in decrypted data: {}", e))
    }
}
```

---

## Privacy Controls

### 1. Data Collection Defaults

**Principle**: Minimal data collection by default

| Data Type | Default | User Can Enable |
|-----------|---------|-----------------|
| Command text | Not logged | Yes |
| Exit codes | Logged | N/A |
| Timing | Logged | N/A |
| Working directory | Logged | N/A |
| Environment | Never | No |
| Telemetry | Off | Yes |

### 2. User Consent Model

```rust
/// Privacy consent state
pub struct PrivacyConsent {
    /// User has reviewed privacy settings
    reviewed: bool,
    /// Timestamp of last review
    reviewed_at: Option<DateTime<Utc>>,
    /// Explicit consent for each feature
    telemetry_consent: bool,
    history_consent: bool,
    learning_consent: bool,
}

impl PrivacyConsent {
    /// First-run consent flow
    pub fn first_run_consent() -> Result<Self> {
        println!("Caro Privacy Settings");
        println!("====================");
        println!();
        println!("Caro can optionally:");
        println!("1. Store command history locally (for fix suggestions)");
        println!("2. Learn from your corrections (to improve suggestions)");
        println!("3. Send anonymous usage statistics (to improve Caro)");
        println!();
        println!("All features are OFF by default.");
        println!();

        let history = Confirm::new()
            .with_prompt("Store command history locally?")
            .default(false)
            .interact()?;

        let learning = Confirm::new()
            .with_prompt("Learn from your corrections?")
            .default(false)
            .interact()?;

        let telemetry = Confirm::new()
            .with_prompt("Send anonymous usage statistics?")
            .default(false)
            .interact()?;

        Ok(Self {
            reviewed: true,
            reviewed_at: Some(Utc::now()),
            telemetry_consent: telemetry,
            history_consent: history,
            learning_consent: learning,
        })
    }
}
```

### 3. Data Retention

```rust
/// Automatic data cleanup
pub struct DataRetention {
    /// Days to keep history (0 = forever, -1 = don't store)
    history_days: i32,
    /// Days to keep logs
    log_days: i32,
    /// Days to keep cached suggestions
    cache_days: i32,
}

impl DataRetention {
    pub fn cleanup(&self) -> Result<()> {
        let now = Utc::now();

        if self.history_days > 0 {
            self.cleanup_older_than(
                &history_dir(),
                now - Duration::days(self.history_days as i64),
            )?;
        }

        if self.log_days > 0 {
            self.cleanup_older_than(
                &log_dir(),
                now - Duration::days(self.log_days as i64),
            )?;
        }

        if self.cache_days > 0 {
            self.cleanup_older_than(
                &cache_dir(),
                now - Duration::days(self.cache_days as i64),
            )?;
        }

        Ok(())
    }
}
```

### 4. Export and Delete

```bash
# Export all user data
caro privacy export --output ~/caro-data-export.json

# Delete all stored data
caro privacy delete-all --confirm

# Delete specific data types
caro privacy delete --history
caro privacy delete --corrections
caro privacy delete --cache
```

---

## Threat Mitigations

### 1. Command Injection Prevention

**Threat**: Attacker crafts command that escapes quoting

**Controls**:

```rust
/// Safely quote a string for shell execution
pub fn shell_quote(s: &str) -> String {
    // If string is safe (alphanumeric + limited chars), no quoting needed
    if s.chars().all(|c| c.is_alphanumeric() || "._-/".contains(c)) {
        return s.to_string();
    }

    // Use single quotes, escaping embedded single quotes
    format!("'{}'", s.replace('\'', "'\"'\"'"))
}

/// Never use eval or similar dangerous constructs
pub fn execute_safely(command: &str) -> Result<Output> {
    // NEVER: eval "$command"
    // NEVER: sh -c "$command"  (with variable interpolation)

    // Use exec array form
    Command::new("sh")
        .arg("-c")
        .arg(command)  // Single argument, no interpolation
        .output()
}
```

### 2. Path Traversal Prevention

**Threat**: Attacker manipulates paths to access sensitive files

**Controls**:

```rust
/// Validate path is within expected directory
pub fn safe_path_join(base: &Path, user_input: &str) -> Result<PathBuf> {
    let joined = base.join(user_input);
    let canonical = joined.canonicalize()?;

    // Ensure result is still under base
    if !canonical.starts_with(base) {
        return Err(anyhow!("Path traversal attempt detected"));
    }

    Ok(canonical)
}
```

### 3. Denial of Service Prevention

**Threat**: Malicious input causes excessive resource usage

**Controls**:

```rust
/// Limits for various inputs
const MAX_COMMAND_LENGTH: usize = 65536;
const MAX_HISTORY_SIZE: usize = 10000;
const MAX_SESSIONS: usize = 1000;
const IPC_TIMEOUT_MS: u64 = 50;

pub fn validate_input(command: &str) -> Result<()> {
    if command.len() > MAX_COMMAND_LENGTH {
        return Err(anyhow!("Command too long"));
    }

    // Check for potential regex DoS patterns
    if contains_redos_pattern(command) {
        return Err(anyhow!("Potentially malicious pattern"));
    }

    Ok(())
}
```

### 4. IPC Security

**Threat**: Unauthorized process connects to Caro daemon

**Controls**:

```rust
impl CaroDaemon {
    async fn handle_connection(&self, stream: UnixStream) -> Result<()> {
        // 1. Verify peer credentials
        let cred = stream.peer_cred()?;
        if cred.uid() != nix::unistd::getuid().as_raw() {
            tracing::warn!("Rejected connection from UID {}", cred.uid());
            return Err(anyhow!("Unauthorized connection"));
        }

        // 2. Rate limiting
        if !self.rate_limiter.check(cred.pid()) {
            tracing::warn!("Rate limit exceeded for PID {}", cred.pid());
            return Err(anyhow!("Rate limit exceeded"));
        }

        // 3. Timeout for all operations
        tokio::time::timeout(
            Duration::from_millis(IPC_TIMEOUT_MS),
            self.process_message(&stream),
        ).await??;

        Ok(())
    }
}
```

---

## Security Logging

```rust
/// Security-relevant events to log
#[derive(Debug, Serialize)]
pub enum SecurityEvent {
    DaemonStarted { pid: u32 },
    DaemonStopped { pid: u32 },
    UnauthorizedConnection { peer_uid: u32, peer_pid: u32 },
    CommandBlocked { command_hash: String, reason: String },
    RateLimitExceeded { pid: u32 },
    ConfigChanged { changed_keys: Vec<String> },
    SuspiciousPattern { pattern: String },
}

impl SecurityEvent {
    pub fn log(&self) {
        // Always log to security log (separate from main logs)
        let log_path = data_dir().join("security.log");
        // ... append to log
    }
}
```

---

## Incident Response

### Detection

```bash
# Check for suspicious activity
caro security audit

# Output:
# - Unauthorized connection attempts
# - Blocked commands (with redacted content)
# - Configuration changes
# - Rate limit events
```

### Recovery

```bash
# If Caro is compromised, users can:

# 1. Disable immediately
export CARO_DISABLE=1

# 2. Uninstall
caro shell uninstall

# 3. Purge all data
rm -rf ~/.config/caro ~/.local/share/caro ~/.cache/caro

# 4. Kill any running daemons
pkill -f 'caro daemon'
```

---

## Compliance Considerations

### GDPR

- **Data minimization**: Minimal data collection by default
- **Right to access**: `caro privacy export`
- **Right to erasure**: `caro privacy delete-all`
- **Consent**: Explicit opt-in for data collection

### SOC 2

- **Access controls**: User-only file permissions
- **Encryption**: Optional history encryption
- **Logging**: Security event logging
- **Monitoring**: Audit capabilities

---

## Security Checklist

For every release:

- [ ] No new environment variables captured without explicit filtering
- [ ] All user inputs validated before use
- [ ] All file operations use restrictive permissions
- [ ] Socket operations verify peer credentials
- [ ] Timeouts on all IPC operations
- [ ] Secrets redacted from all logs
- [ ] Security audit (`cargo audit`) passes
- [ ] No eval or dynamic code execution
- [ ] Path traversal protections in place
- [ ] Rate limiting enabled on daemon
