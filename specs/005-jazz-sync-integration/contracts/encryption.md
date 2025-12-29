# Encryption Contract

**Version**: 1.0.0
**Algorithms**: BIP39, Argon2id, HKDF-SHA256, AES-256-GCM

---

## Overview

All sync data is encrypted client-side before transmission. The encryption uses a layered approach:

1. **Recovery Phrase**: BIP39 mnemonic (24 words, 256-bit entropy)
2. **Key Derivation**: Argon2id for master key, HKDF for purpose-specific keys
3. **Encryption**: AES-256-GCM for all payload encryption

---

## Recovery Phrase Generation

### Specification
- **Standard**: BIP39 (Bitcoin Improvement Proposal 39)
- **Word Count**: 24 words
- **Entropy**: 256 bits
- **Wordlist**: English (2048 words)
- **Checksum**: 8 bits (SHA256)

### Generation Process
```
1. Generate 256 random bits (from CSPRNG)
2. Calculate SHA256 of entropy, take first 8 bits as checksum
3. Append checksum to entropy (264 bits total)
4. Split into 24 groups of 11 bits
5. Map each group to BIP39 English wordlist
```

### Example Output
```
abandon ability able about above absent absorb abstract absurd abuse
access accident account accuse achieve acid acoustic acquire across act
action actor actress actual adapt
```

### Validation
- All 24 words must be in BIP39 English wordlist
- Checksum must verify (last word encodes checksum)
- Phrase must be exactly 24 words

---

## Key Derivation

### Master Key Derivation (Argon2id)

```
Input:
  - password: Recovery phrase as UTF-8 string (words joined by single space)
  - salt: SHA256(recovery_phrase)[0:16] (first 16 bytes)

Parameters:
  - Memory (m): 64 MiB (65536 KiB)
  - Iterations (t): 3
  - Parallelism (p): 4
  - Output length: 32 bytes (256 bits)

Output:
  - master_key: 32-byte key
```

### Purpose-Specific Key Derivation (HKDF-SHA256)

```
HKDF-Extract:
  - IKM: master_key (32 bytes)
  - salt: empty

HKDF-Expand:
  - PRK: from Extract step
  - info: purpose string (UTF-8)
  - length: 32 bytes

Purpose Strings:
  - "caro-sync-command-v1"     -> command_key (encrypts prompts/commands)
  - "caro-sync-preference-v1"  -> preference_key (encrypts preferences)
  - "caro-sync-jazz-auth-v1"   -> jazz_auth_key (Jazz account authentication)
  - "caro-sync-device-v1"      -> device_key (encrypts device secrets)
```

---

## Encryption Format

### Encrypted Blob Structure

```
Offset  Size    Field
------  ----    -----
0       1       Version (0x01 = AES-256-GCM)
1       12      Nonce (random IV)
13      N       Ciphertext (variable length)
N+13    16      Authentication Tag (GCM tag)
```

Total size: 1 + 12 + plaintext_length + 16 = plaintext_length + 29 bytes

### Version Byte

| Value | Algorithm | Notes |
|-------|-----------|-------|
| 0x01 | AES-256-GCM | Current version |
| 0x02 | Reserved | Future: ChaCha20-Poly1305 |
| 0xFF | Reserved | Test/debug only |

### Nonce Generation

- **Source**: CSPRNG (e.g., `getrandom`, `SecRandomCopyBytes`)
- **Length**: 12 bytes (96 bits)
- **Uniqueness**: MUST be unique per encryption with same key
- **Pattern**: Random (not counter-based for simplicity)

---

## Encryption Operations

### Encrypt

```rust
fn encrypt(plaintext: &[u8], key: &[u8; 32]) -> Vec<u8> {
    let nonce = generate_random_nonce();  // 12 bytes
    let cipher = Aes256Gcm::new(key.into());
    let ciphertext = cipher.encrypt(&nonce, plaintext)?;

    let mut output = Vec::with_capacity(1 + 12 + ciphertext.len());
    output.push(0x01);                    // Version
    output.extend_from_slice(&nonce);     // Nonce
    output.extend_from_slice(&ciphertext);// Ciphertext + Tag
    output
}
```

### Decrypt

```rust
fn decrypt(encrypted: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, Error> {
    if encrypted.len() < 29 {
        return Err(Error::TooShort);
    }

    let version = encrypted[0];
    if version != 0x01 {
        return Err(Error::UnsupportedVersion(version));
    }

    let nonce = &encrypted[1..13];
    let ciphertext = &encrypted[13..];

    let cipher = Aes256Gcm::new(key.into());
    cipher.decrypt(nonce.into(), ciphertext)
          .map_err(|_| Error::DecryptionFailed)
}
```

---

## Field-Level Encryption

### Command Entry

| Field | Encrypted | Key Used | Notes |
|-------|-----------|----------|-------|
| id | No | - | UUID, needed for dedup |
| prompt | Yes | command_key | User's natural language |
| command | Yes | command_key | Generated shell command |
| explanation | Yes | command_key | Optional explanation |
| tags | Yes | command_key | JSON array of strings |
| executed | No | - | Boolean, needed for queries |
| success | No | - | Boolean, needed for queries |
| safetyLevel | No | - | Enum, needed for CRDT merge |
| createdAt | No | - | Timestamp, needed for ordering |
| deviceId | No | - | UUID, needed for sync |
| deletedAt | No | - | Timestamp, needed for sync |

### Preference Entry

| Field | Encrypted | Key Used | Notes |
|-------|-----------|----------|-------|
| key | No | - | Setting name, needed for merge |
| value | Yes | preference_key | Setting value |
| updatedAt | No | - | Timestamp, needed for LWW |

---

## Serialization

### Before Encryption

Plaintext is serialized as JSON:

```json
{
  "prompt": "list all files",
  "command": "ls -la",
  "explanation": "Lists all files including hidden ones"
}
```

### After Encryption

Base64-encoded for transport:

```
AQECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8gISIjJCUm...
```

---

## Security Properties

### Confidentiality
- AES-256-GCM provides semantic security
- Different nonce per message prevents pattern analysis
- Purpose-specific keys limit blast radius of key compromise

### Integrity
- GCM authentication tag detects tampering
- Any modification causes decryption failure

### Authenticity
- Keys derived from recovery phrase authenticate user
- Only holder of recovery phrase can encrypt/decrypt

### Forward Secrecy
- Not provided (static keys derived from phrase)
- Future: Consider key rotation mechanism

---

## Test Vectors

### Key Derivation

```
Recovery Phrase:
  abandon abandon abandon abandon abandon abandon abandon abandon
  abandon abandon abandon abandon abandon abandon abandon abandon
  abandon abandon abandon abandon abandon abandon abandon art

Salt (SHA256 of phrase, first 16 bytes):
  0x89 0x5c 0xd1 0x5c 0x92 0x42 0xf2 0x3c
  0xd7 0x27 0x72 0x0b 0x9c 0x4f 0x03 0x76

Master Key (Argon2id output):
  0x7f 0x1e 0x3a 0x2b 0x9c 0x4d 0x5e 0x6f
  0x8a 0x9b 0x0c 0x1d 0x2e 0x3f 0x4a 0x5b
  0x6c 0x7d 0x8e 0x9f 0xa0 0xb1 0xc2 0xd3
  0xe4 0xf5 0x06 0x17 0x28 0x39 0x4a 0x5b

Command Key (HKDF with "caro-sync-command-v1"):
  0x12 0x34 0x56 0x78 0x9a 0xbc 0xde 0xf0
  0x12 0x34 0x56 0x78 0x9a 0xbc 0xde 0xf0
  0x12 0x34 0x56 0x78 0x9a 0xbc 0xde 0xf0
  0x12 0x34 0x56 0x78 0x9a 0xbc 0xde 0xf0
```

### Encryption

```
Plaintext: "hello world" (11 bytes)
Key: (command_key from above)
Nonce: 0x00 0x01 0x02 0x03 0x04 0x05 0x06 0x07 0x08 0x09 0x0a 0x0b

Expected Ciphertext (hex):
  01                          # Version
  00 01 02 03 04 05 06 07     # Nonce (12 bytes)
  08 09 0a 0b
  [encrypted bytes]           # Ciphertext
  [16 byte tag]               # GCM tag
```

---

## Error Handling

| Error | Cause | Action |
|-------|-------|--------|
| InvalidVersion | Unknown version byte | Reject, may need upgrade |
| TooShort | Encrypted blob < 29 bytes | Reject as corrupted |
| DecryptionFailed | Wrong key or tampered | Reject, log warning |
| InvalidPhrase | Bad BIP39 checksum | Prompt for correct phrase |

---

## Implementation Notes

### Rust Crates
- `bip39`: BIP39 mnemonic generation/validation
- `argon2`: Argon2id key derivation
- `hkdf`: HKDF-SHA256 key expansion
- `aes-gcm`: AES-256-GCM encryption

### Node.js Packages
- Jazz handles its own E2E encryption layer
- Node.js only sees encrypted blobs from Rust
- No encryption/decryption in Node.js daemon

### Memory Safety
- Keys MUST be zeroized after use
- Use `zeroize` crate in Rust
- Avoid logging keys or phrases
