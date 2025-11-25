# Data Model: Production-Ready Backend System

**Phase 1 Output** | **Feature**: 005-production-backends | **Date**: 2025-10-14

## Core Entities

### 1. CommandHistoryEntry

**Purpose**: Comprehensive command storage with rich metadata for performance analysis and semantic search

```rust
pub struct CommandHistoryEntry {
    // Identity
    pub id: String,                    // UUID v4 for unique identification
    pub timestamp: DateTime<Utc>,      // ISO 8601 UTC timestamp
    
    // Command Data
    pub command: String,               // Generated shell command
    pub user_input: Option<String>,    // Original natural language input
    pub explanation: String,           // AI-generated explanation
    pub shell_type: ShellType,         // bash, zsh, fish, etc.
    pub working_directory: String,     // Execution context
    
    // Execution Metadata
    pub execution_metadata: Option<ExecutionMetadata>,
    pub safety_metadata: Option<SafetyMetadata>,
    
    // Search and Analytics
    pub tags: Vec<String>,             // User-defined and auto-generated tags
    pub embedding_vector: Option<Vec<f32>>, // Semantic search embeddings
    pub relevance_score: Option<f64>,  // Query relevance for ranking
}

pub struct ExecutionMetadata {
    pub exit_code: Option<i32>,        // Command execution result
    pub execution_time: Option<Duration>, // Wall clock time
    pub output_size: Option<usize>,    // Bytes of output produced
    pub backend_used: String,          // mlx, ollama, vllm, etc.
    pub generation_time: Duration,     // AI inference duration
    pub validation_time: Duration,     // Safety check duration
}

pub struct SafetyMetadata {
    pub risk_level: RiskLevel,         // Safe, Moderate, High, Critical
    pub patterns_matched: Vec<String>, // Triggered safety patterns
    pub user_confirmed: bool,          // Whether user approved risky command
    pub safety_score: f64,            // Numerical risk assessment (0.0-1.0)
}
```

### 2. InteractiveConfigUI

**Purpose**: Full-screen configuration management with real-time validation and persistence

```rust
pub struct InteractiveConfigUI {
    pub config: ConfigurationState,
    pub validation_rules: ValidationRules,
    pub ui_state: UIState,
}

pub struct ConfigurationState {
    // Backend Configuration
    pub preferred_backend: BackendType,
    pub backend_configs: HashMap<String, BackendConfig>,
    pub fallback_chain: Vec<BackendType>,
    
    // History Settings
    pub history_enabled: bool,
    pub retention_policy: RetentionPolicy,
    pub privacy_mode: PrivacyLevel,
    pub auto_cleanup_days: u32,
    
    // Safety Configuration
    pub safety_level: SafetyLevel,
    pub confirmation_required: Vec<RiskLevel>,
    pub custom_safety_patterns: Vec<String>,
    
    // UI Preferences
    pub streaming_enabled: bool,
    pub color_output: bool,
    pub verbosity_level: VerbosityLevel,
}

pub struct RetentionPolicy {
    pub max_entries: Option<usize>,
    pub max_age_days: Option<u32>,
    pub preserve_favorites: bool,
    pub preserve_frequently_used: bool,
}
```

### 3. AdvancedSafetyValidator

**Purpose**: Multi-modal validation engine with behavioral analysis and context-aware risk assessment

```rust
pub struct AdvancedSafetyValidator {
    pub pattern_engine: PatternEngine,
    pub behavioral_analyzer: BehavioralAnalyzer,
    pub context_analyzer: ContextAnalyzer,
    pub risk_assessor: RiskAssessor,
}

pub struct PatternEngine {
    pub dangerous_patterns: CompiledPatterns,
    pub safe_patterns: CompiledPatterns,
    pub context_patterns: CompiledPatterns,
}

pub struct BehavioralAnalyzer {
    pub command_graph: CommandDependencyGraph,
    pub execution_patterns: Vec<ExecutionPattern>,
    pub anomaly_detector: AnomalyDetector,
}

pub struct ValidationResult {
    pub is_safe: bool,
    pub risk_level: RiskLevel,
    pub confidence: f64,
    pub explanation: String,
    pub suggested_alternatives: Vec<String>,
    pub required_confirmations: Vec<ConfirmationType>,
}
```

### 4. StreamingGenerator

**Purpose**: Real-time command generation with cancellation, progress feedback, and partial results

```rust
pub struct StreamingGenerator {
    pub backend: Box<dyn ModelBackend>,
    pub progress_tracker: ProgressTracker,
    pub cancellation_token: CancellationToken,
    pub partial_results: PartialResultHandler,
}

pub struct GenerationStream {
    pub request_id: String,
    pub stream: Pin<Box<dyn Stream<Item = StreamEvent> + Send>>,
    pub cancellation_token: CancellationToken,
}

pub enum StreamEvent {
    Progress { percentage: f64, message: String },
    PartialResult { command: String, confidence: f64 },
    Completed { result: GenerationResult },
    Error { error: String, recoverable: bool },
    Cancelled,
}

pub struct GenerationResult {
    pub command: String,
    pub explanation: String,
    pub confidence: f64,
    pub generation_time: Duration,
    pub tokens_generated: usize,
    pub safety_validated: bool,
}
```

### 5. BackendSelector

**Purpose**: Intelligent backend routing with performance monitoring and preference integration

```rust
pub struct BackendSelector {
    pub available_backends: HashMap<String, Box<dyn ModelBackend>>,
    pub performance_monitor: PerformanceMonitor,
    pub health_checker: HealthChecker,
    pub selection_strategy: SelectionStrategy,
}

pub struct PerformanceMonitor {
    pub metrics: HashMap<String, BackendMetrics>,
    pub historical_data: RingBuffer<PerformanceSnapshot>,
    pub real_time_stats: RealTimeStats,
}

pub struct BackendMetrics {
    pub avg_response_time: Duration,
    pub success_rate: f64,
    pub availability: f64,
    pub queue_length: usize,
    pub last_health_check: DateTime<Utc>,
}

pub struct SelectionResult {
    pub selected_backend: String,
    pub selection_reason: String,
    pub fallback_chain: Vec<String>,
    pub estimated_response_time: Duration,
}
```

### 6. SemanticSearchEngine

**Purpose**: Local semantic understanding with embedding cache and privacy-preserving search

```rust
pub struct SemanticSearchEngine {
    pub embedding_model: Box<dyn EmbeddingModel>,
    pub embedding_cache: LocalEmbeddingCache,
    pub similarity_engine: SimilarityEngine,
    pub query_processor: QueryProcessor,
}

pub struct LocalEmbeddingCache {
    pub cache_directory: PathBuf,
    pub embeddings: HashMap<String, Vec<f32>>,
    pub metadata: EmbeddingMetadata,
    pub cleanup_policy: CacheCleanupPolicy,
}

pub struct SearchQuery {
    pub text: String,
    pub filters: SearchFilters,
    pub max_results: usize,
    pub similarity_threshold: f64,
}

pub struct SearchResult {
    pub entry: CommandHistoryEntry,
    pub similarity_score: f64,
    pub matching_context: String,
    pub explanation: String,
}
```

## Entity Relationships

### Primary Relationships:
- `CommandHistoryEntry` → `SafetyMetadata` (1:0..1)
- `CommandHistoryEntry` → `ExecutionMetadata` (1:0..1)
- `InteractiveConfigUI` → `ConfigurationState` (1:1)
- `BackendSelector` → `PerformanceMonitor` (1:1)
- `SemanticSearchEngine` → `LocalEmbeddingCache` (1:1)

### Data Flow:
1. User input → `StreamingGenerator` → `BackendSelector` → Backend
2. Generated command → `AdvancedSafetyValidator` → Validation result
3. Validated command → `CommandHistoryEntry` → SQLite storage
4. Search query → `SemanticSearchEngine` → Ranked results
5. Configuration changes → `InteractiveConfigUI` → Persistent storage

## Database Schema

### SQLite Tables:

```sql
-- Command history with FTS5 support
CREATE TABLE command_history (
    id TEXT PRIMARY KEY,
    command TEXT NOT NULL,
    user_input TEXT,
    explanation TEXT NOT NULL,
    shell_type TEXT NOT NULL,
    working_directory TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    execution_metadata TEXT, -- JSON
    safety_metadata TEXT,    -- JSON
    tags TEXT,              -- JSON array
    embedding_vector BLOB   -- Serialized f32 vector
);

-- FTS5 virtual table for full-text search
CREATE VIRTUAL TABLE command_history_fts USING fts5(
    command, user_input, explanation, tags,
    content='command_history',
    content_rowid='rowid'
);

-- Performance metrics for backend selection
CREATE TABLE backend_metrics (
    backend_name TEXT PRIMARY KEY,
    avg_response_time_ms INTEGER,
    success_rate REAL,
    availability REAL,
    last_health_check TEXT,
    metrics_json TEXT
);

-- Configuration persistence
CREATE TABLE configuration (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    last_updated TEXT NOT NULL
);

-- Embedding cache for semantic search
CREATE TABLE embedding_cache (
    content_hash TEXT PRIMARY KEY,
    embedding_vector BLOB NOT NULL,
    created_at TEXT NOT NULL,
    access_count INTEGER DEFAULT 0,
    last_accessed TEXT NOT NULL
);
```

## Validation Rules

### Data Integrity:
- `CommandHistoryEntry.id` must be valid UUID v4
- `timestamp` must be valid ISO 8601 UTC format
- `embedding_vector` length must match model dimensions (384 for SentenceT5-Base)
- `safety_score` must be in range [0.0, 1.0]

### Business Rules:
- Commands with `RiskLevel::Critical` require explicit user confirmation
- History entries older than retention policy are automatically purged
- Embedding cache size limited to prevent disk space exhaustion
- Backend selection considers availability, performance, and user preferences

### Performance Constraints:
- History write operations must complete in <10ms
- Search operations must complete in <50ms for 10K entries
- Configuration changes take effect immediately without restart
- Embedding computation cached to avoid repeated calculation