# Cache Module API Contract

**Module**: `caro::cache`
**Purpose**: Model caching with Hugging Face integration and offline support

## Public API

### CacheManager

```rust
pub struct CacheManager {
    // Private fields
}

impl CacheManager {
    /// Creates a new CacheManager with default cache directory
    /// Returns error if cache directory cannot be created or accessed
    pub fn new() -> Result<Self, CacheError>;

    /// Creates a new CacheManager with custom cache directory
    pub fn with_cache_dir(cache_dir: PathBuf) -> Result<Self, CacheError>;

    /// Gets a model from cache or downloads if not present
    /// Returns path to cached model file
    ///
    /// # Errors
    /// - CacheError::DownloadFailed if network error during download
    /// - CacheError::ChecksumMismatch if integrity validation fails
    /// - CacheError::DiskFull if insufficient disk space
    pub async fn get_model(&self, model_id: &str) -> Result<PathBuf, CacheError>;

    /// Checks if a model is already cached
    pub fn is_cached(&self, model_id: &str) -> bool;

    /// Removes a model from cache to free disk space
    pub async fn remove_model(&self, model_id: &str) -> Result<(), CacheError>;

    /// Clears entire cache
    pub async fn clear_cache(&self) -> Result<(), CacheError>;

    /// Gets cache statistics
    pub fn stats(&self) -> CacheStats;

    /// Validates integrity of all cached models
    pub async fn validate_integrity(&self) -> Result<IntegrityReport, CacheError>;
}
```

### CacheStats

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_models: usize,
    pub total_size_bytes: u64,
    pub cache_dir: PathBuf,
    pub models: Vec<CachedModelInfo>,
}
```

### CachedModelInfo

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedModelInfo {
    pub model_id: String,
    pub size_bytes: u64,
    pub downloaded_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
}
```

### IntegrityReport

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityReport {
    pub valid_models: Vec<String>,
    pub corrupted_models: Vec<String>,
    pub missing_models: Vec<String>,
}
```

### CacheError

```rust
#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Cache directory not accessible: {0}")]
    DirectoryNotAccessible(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Download failed: {0}")]
    DownloadFailed(String),

    #[error("Checksum mismatch for {model_id}: expected {expected}, got {actual}")]
    ChecksumMismatch {
        model_id: String,
        expected: String,
        actual: String,
    },

    #[error("Disk full: need {needed} bytes, available {available}")]
    DiskFull { needed: u64, available: u64 },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
```

## Behavioral Contracts

### Contract: get_model Returns Existing Cached Model

**Given**: Model "test-model" is already cached with valid checksum
**When**: `cache_manager.get_model("test-model")` is called
**Then**:
- Returns `Ok(PathBuf)` pointing to cached file
- No network request is made
- `last_accessed` timestamp is updated
- Operation completes in <100ms

### Contract: get_model Downloads Uncached Model

**Given**: Model "test-model" is not in cache
**When**: `cache_manager.get_model("test-model")` is called
**Then**:
- Downloads model from Hugging Face
- Validates checksum
- Adds to cache manifest
- Returns `Ok(PathBuf)` pointing to cached file
- Operation completes in <5s for models <1GB

### Contract: get_model Fails Offline for Uncached Model

**Given**: Model "test-model" is not in cache AND network is unavailable
**When**: `cache_manager.get_model("test-model")` is called
**Then**:
- Returns `Err(CacheError::DownloadFailed(...))`
- Error message suggests checking network connection
- No partial files left in cache directory

### Contract: get_model Detects Corrupted Cache

**Given**: Model "test-model" is cached but file is corrupted (checksum mismatch)
**When**: `cache_manager.get_model("test-model")` is called
**Then**:
- Returns `Err(CacheError::ChecksumMismatch{...})`
- Corrupted file is removed from cache
- Suggests re-running command to re-download

### Contract: is_cached Returns Accurate Status

**Given**: Model "cached-model" is in cache, "uncached-model" is not
**When**:
- `cache_manager.is_cached("cached-model")` called
- `cache_manager.is_cached("uncached-model")` called
**Then**:
- First call returns `true`
- Second call returns `false`

### Contract: remove_model Frees Disk Space

**Given**: Model "test-model" with 1GB size is cached
**When**: `cache_manager.remove_model("test-model")` is called
**Then**:
- Model file deleted from filesystem
- Removed from cache manifest
- `stats().total_size_bytes` decreased by 1GB
- Returns `Ok(())`

### Contract: clear_cache Removes All Models

**Given**: Cache contains 3 models
**When**: `cache_manager.clear_cache()` is called
**Then**:
- All model files deleted
- Cache manifest reset to empty
- Cache directory still exists
- `stats().total_models` equals 0

### Contract: stats Returns Accurate Information

**Given**: Cache contains 2 models (sizes: 500MB, 1GB)
**When**: `cache_manager.stats()` is called
**Then**:
- `total_models` equals 2
- `total_size_bytes` equals 1,500,000,000
- `models` contains both models with correct metadata

### Contract: validate_integrity Detects Corruption

**Given**: Cache has 3 models, 1 is corrupted (bad checksum)
**When**: `cache_manager.validate_integrity()` is called
**Then**:
- Returns `Ok(IntegrityReport{...})`
- `valid_models` contains 2 model IDs
- `corrupted_models` contains 1 model ID
- `missing_models` is empty

### Contract: Cache Directory Creation

**Given**: Cache directory does not exist
**When**: `CacheManager::new()` is called
**Then**:
- Cache directory created at XDG-compliant location
- Directory permissions set to 0700 (user-only)
- Manifest file initialized
- Returns `Ok(CacheManager)`

### Contract: Concurrent Access Safety

**Given**: Two CacheManager instances access same cache directory
**When**: Both call `get_model("same-model")` concurrently
**Then**:
- Only one download occurs (file locking)
- Both calls return successfully
- No corrupted state in manifest

## Test Coverage Requirements

1. **Unit Tests**:
   - Cache directory initialization
   - Checksum validation logic
   - Manifest serialization/deserialization
   - LRU eviction algorithm

2. **Integration Tests**:
   - End-to-end download and cache workflow
   - Offline operation with cached models
   - Corruption detection and recovery
   - Concurrent access scenarios

3. **Property Tests**:
   - `is_cached` always consistent with filesystem state
   - `stats().total_size_bytes` always matches sum of model sizes
   - Manifest always deserializable after any operation

## Performance Requirements

| Operation | Target Latency | Constraint |
|-----------|----------------|------------|
| `is_cached()` | <1ms | In-memory manifest lookup |
| `get_model()` (cached) | <100ms | Checksum validation included |
| `get_model()` (download, <1GB) | <5s | Network-dependent |
| `stats()` | <10ms | Aggregate manifest data |
| `validate_integrity()` | <1s per GB | CPU-bound checksum calculation |

## Thread Safety

- All public methods are `async` and thread-safe
- Internal manifest uses `Arc<RwLock<CacheManifest>>` for concurrent reads
- File operations use `tokio::fs` for async I/O

## Error Handling

All errors include:
- Context about what operation failed
- Actionable suggestion for user (e.g., "Check network connection")
- Original error source via `#[from]` where applicable
