# cmdai V2 - Technical Specification
## "The Command Intelligence Platform"

**Version**: 2.0.0
**Status**: Planning
**Target Release**: Q2 2026
**Author**: Architecture Team
**Last Updated**: 2025-11-19

---

## Executive Summary

cmdai V2 transforms from a command generator into an intelligent platform that understands context, learns from interactions, provides enterprise-grade safety, and connects developers through collective intelligence.

**Core Thesis**: Command-line tools are the last major developer workflow without AI-native intelligence. cmdai V2 becomes the intelligent layer between human intent and shell execution.

**Market Position**:
- V1: "AI command generator" (commodity)
- V2: "Command intelligence platform" (category defining)

**Differentiation**: Context awareness + ML-powered safety + learning engine + community knowledge

---

## Table of Contents

1. [Architecture Overview](#1-architecture-overview)
2. [Feature Specifications](#2-feature-specifications)
3. [Technical Implementation](#3-technical-implementation)
4. [Data Models](#4-data-models)
5. [API Contracts](#5-api-contracts)
6. [Success Metrics](#6-success-metrics)
7. [Implementation Roadmap](#7-implementation-roadmap)

---

## 1. Architecture Overview

### 1.1 System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     cmdai V2 Application                         │
└─────────────────────────────────────────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        ▼                     ▼                     ▼
┌──────────────────┐  ┌──────────────┐  ┌──────────────────┐
│   Intelligence   │  │  Safety ML   │  │    Learning      │
│     Engine       │  │    Engine    │  │     Engine       │
│                  │  │              │  │                  │
│ • Context Graph  │  │ • Risk ML    │  │ • Pattern DB     │
│ • Intent AI      │  │ • Sandbox    │  │ • Explainer      │
│ • Project Parser │  │ • Audit Log  │  │ • Tutorial Sys   │
│ • Tool Detector  │  │ • Rollback   │  │ • Achievement    │
└──────────────────┘  └──────────────┘  └──────────────────┘
        │                     │                     │
        └─────────────────────┼─────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        ▼                     ▼                     ▼
┌──────────────────┐  ┌──────────────┐  ┌──────────────────┐
│ Backend System   │  │  Community   │  │   Data Layer     │
│   (from V1)      │  │  Marketplace │  │                  │
│                  │  │              │  │                  │
│ • Embedded Model │  │ • Registry   │  │ • SQLite (local) │
│ • Ollama         │  │ • Playbooks  │  │ • Cloud Sync     │
│ • vLLM           │  │ • Reputation │  │ • Analytics      │
└──────────────────┘  └──────────────┘  └──────────────────┘
```

### 1.2 Data Flow

**V1 Flow (Simple)**:
```
User Prompt → Backend → Safety Check → Execute
```

**V2 Flow (Intelligent)**:
```
User Prompt
  ↓
[Intelligence Engine]
  ├─ Parse project context (Git, package managers, tools)
  ├─ Analyze shell history patterns
  ├─ Classify intent (deploy, search, cleanup, etc.)
  └─ Build augmented prompt
  ↓
[Backend System]
  ├─ Send context-enriched prompt to LLM
  └─ Receive generated command
  ↓
[Safety ML Engine]
  ├─ Extract command features
  ├─ ML risk prediction (0-10 score)
  ├─ Estimate impact (files, reversibility)
  └─ Suggest mitigations
  ↓
[Community Intelligence] (if enabled)
  ├─ Search similar commands
  ├─ Show success rates
  └─ Offer community alternatives
  ↓
[Learning Engine]
  ├─ Generate explanation
  ├─ Check user history for patterns
  └─ Suggest improvements
  ↓
[Execution Options]
  ├─ Execute directly
  ├─ Run in sandbox (preview changes)
  ├─ View explanation
  └─ Search community
  ↓
[Feedback Loop]
  ├─ Record interaction
  ├─ Learn from user edits
  ├─ Update ML models
  └─ Improve future suggestions
```

### 1.3 Module Dependencies

```
┌─────────────────────────────────────────────────────┐
│ src/                                                │
├─────────────────────────────────────────────────────┤
│                                                     │
│ intelligence/                                       │
│   ├─ mod.rs                (Public API)            │
│   ├─ context_graph.rs      (Project analysis)      │
│   ├─ intent_classifier.rs  (Intent detection)      │
│   ├─ project_parser.rs     (Multi-language)        │
│   ├─ tool_detector.rs      (Docker/K8s/etc)        │
│   └─ history_analyzer.rs   (Shell history AI)      │
│                                                     │
│ safety/                                             │
│   ├─ mod.rs                (Public API)            │
│   ├─ ml_predictor.rs       (TFLite risk model)     │
│   ├─ feature_extractor.rs  (Command → features)    │
│   ├─ sandbox.rs            (Overlay FS execution)  │
│   ├─ impact_estimator.rs   (Predict changes)       │
│   └─ audit_logger.rs       (Compliance logs)       │
│                                                     │
│ learning/                                           │
│   ├─ mod.rs                (Public API)            │
│   ├─ pattern_db.rs         (SQLite + embeddings)   │
│   ├─ explainer.rs          (AST → explanations)    │
│   ├─ tutorial_engine.rs    (Interactive lessons)   │
│   └─ achievement_system.rs (Gamification)          │
│                                                     │
│ community/                                          │
│   ├─ mod.rs                (Public API)            │
│   ├─ marketplace.rs        (Command registry)      │
│   ├─ playbooks.rs          (Team workflows)        │
│   ├─ reputation.rs         (User/command scores)   │
│   └─ sync_client.rs        (Cloud API client)      │
│                                                     │
│ backends/                  (FROM V1 - unchanged)    │
│ cli/                       (FROM V1 - enhanced)     │
│ config/                    (FROM V1 - extended)     │
│ models/                    (FROM V1 - extended)     │
│                                                     │
└─────────────────────────────────────────────────────┘
```

---

## 2. Feature Specifications

### 2.1 Context Intelligence Engine

**FR-CI-001**: System MUST detect project type from filesystem markers
- **Markers**: `package.json` (Node), `Cargo.toml` (Rust), `pyproject.toml` (Python), `go.mod` (Go), `pom.xml` (Java)
- **Performance**: Detection must complete in < 100ms
- **Accuracy**: 95%+ correct project type identification

**FR-CI-002**: System MUST analyze Git repository state
- **Data**: Current branch, remote URL, commit status, uncommitted changes
- **Performance**: Git analysis < 50ms
- **Fallback**: Gracefully handle non-Git directories

**FR-CI-003**: System MUST detect infrastructure tools
- **Tools**: Docker (`docker-compose.yml`), Kubernetes (`*.yaml` with `kind:`), Terraform (`*.tf`), Railway CLI, etc.
- **Performance**: Tool detection < 200ms (parallel scanning)
- **Extensible**: Plugin system for custom tool detection

**FR-CI-004**: System MUST classify user intent from natural language
- **Intents**: Deploy, Search, Cleanup, Debug, Setup, Monitor, Test, Build, Migrate, Rollback
- **Method**: Embedding-based classification (sentence-transformers)
- **Accuracy**: 80%+ intent classification accuracy
- **Fallback**: Generic command generation if intent unclear

**FR-CI-005**: System MUST augment LLM prompts with project context
- **Format**: Structured context in system message
- **Content**: Project type, tools, Git state, recent commands
- **Size Limit**: Context < 500 tokens (to preserve speed)

**FR-CI-006**: System MUST analyze shell history for patterns
- **Source**: `~/.bash_history`, `~/.zsh_history`, `~/.history`
- **Analysis**: Extract frequently used commands, tools, directories
- **Privacy**: All analysis local, opt-out available
- **Performance**: History analysis < 300ms

### 2.2 Safety ML Engine

**FR-SM-001**: System MUST predict command risk using ML model
- **Model**: TensorFlow Lite (< 5MB, < 50ms inference)
- **Input Features**: Command tokens, AST structure, context, similarity to known dangerous patterns
- **Output**: Risk score (0-10), confidence (0-1), risk factors
- **Accuracy**: 90%+ precision on dangerous commands, < 5% false positives

**FR-SM-002**: System MUST estimate command impact
- **Metrics**: Files affected (count), data loss risk (0-1), reversibility (boolean), blast radius (local/system/network)
- **Method**: Heuristic analysis + file simulation
- **Performance**: Impact estimation < 200ms

**FR-SM-003**: System MUST provide sandbox execution environment
- **Implementation**: BTRFS snapshots (Linux), APFS snapshots (macOS), or FUSE overlay filesystem
- **Capabilities**: Full filesystem isolation, preview changes, rollback
- **Performance**: Snapshot creation < 500ms
- **Limitations**: Document what cannot be sandboxed (network, kernel)

**FR-SM-004**: System MUST log all command executions for audit
- **Format**: Structured JSON logs
- **Fields**: Timestamp, user, command, context, risk score, outcome
- **Storage**: Local by default, cloud sync optional (encrypted)
- **Retention**: Configurable (default 90 days)

**FR-SM-005**: System MUST support policy-as-code (Enterprise feature)
- **Format**: YAML-based policy files
- **Rules**: Pattern matching, context conditions, actions (block/warn/allow)
- **Distribution**: Git-based team policies
- **Validation**: Policy syntax checker

### 2.3 Learning Engine

**FR-LE-001**: System MUST store command interaction history
- **Database**: SQLite (local), encrypted at rest
- **Schema**: Prompt, generated command, user edits, context, timestamp, execution result
- **Indexing**: Embedding-based similarity search (HNSW index)
- **Privacy**: Opt-in telemetry, local-first by default

**FR-LE-002**: System MUST learn from user command modifications
- **Detection**: Diff between generated and executed command
- **Analysis**: Extract improvement patterns
- **Application**: Improve future generations for similar prompts
- **Metrics**: Track improvement over time (accuracy increase)

**FR-LE-003**: System MUST explain generated commands
- **Parsing**: Shell AST parser (bash/zsh/fish syntax)
- **Explanation**: Natural language description per command segment
- **Depth**: Basic (one-line) or detailed (step-by-step)
- **Examples**: Show input/output examples for each part

**FR-LE-004**: System MUST provide interactive tutorials
- **Topics**: Core commands (find, grep, awk, sed, etc.)
- **Format**: Step-by-step lessons with hands-on exercises
- **Progress**: Track completion, spaced repetition
- **Gamification**: Achievements, badges, streaks

**FR-LE-005**: System MUST suggest command improvements
- **Triggers**: Inefficient patterns detected, better alternatives available
- **Categories**: Performance (faster tools), safety (safer options), portability (POSIX compliance)
- **Presentation**: Non-intrusive suggestions after command execution

### 2.4 Community Marketplace

**FR-CM-001**: System MUST allow searching community commands
- **Index**: Centralized registry (Phase 1), P2P/IPFS (Phase 2)
- **Search**: Semantic search using embeddings
- **Filters**: Platform, tags, success rate, author reputation
- **Performance**: Search results < 500ms

**FR-CM-002**: System MUST allow submitting commands to community
- **Validation**: Automatic safety scan before publishing
- **Metadata**: Tags, platform compatibility, description
- **Review**: Community voting + maintainer curation
- **Moderation**: Report spam/malicious commands

**FR-CM-003**: System MUST track command success rates
- **Telemetry**: Opt-in execution outcome reporting
- **Metrics**: Success %, execution count, platforms tested
- **Privacy**: Anonymized aggregated data only
- **Display**: Show success rates in search results

**FR-CM-004**: System MUST support team playbooks
- **Format**: YAML workflow definitions
- **Features**: Multi-step execution, variables, conditionals, rollback
- **Sharing**: Public (marketplace) or private (team only)
- **Execution**: Interactive mode with prerequisite checks

**FR-CM-005**: System MUST implement reputation system
- **Factors**: Commands submitted, upvotes received, success rate, tenure
- **Levels**: New contributor → Trusted → Expert → Maintainer
- **Benefits**: Higher search ranking, moderation privileges
- **Anti-gaming**: Rate limits, verification requirements

---

## 3. Technical Implementation

### 3.1 Intelligence Engine

#### 3.1.1 Context Graph Builder

```rust
// src/intelligence/context_graph.rs

use serde::{Deserialize, Serialize};
use std::path::Path;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextGraph {
    pub project: ProjectContext,
    pub git: GitContext,
    pub infrastructure: InfrastructureContext,
    pub history: HistoryContext,
    pub environment: EnvironmentContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectContext {
    pub project_type: ProjectType,
    pub root_dir: PathBuf,
    pub dependencies: Vec<Dependency>,
    pub scripts: Vec<ScriptInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectType {
    Rust { has_workspace: bool },
    Node { package_manager: PackageManager },
    Python { dependency_tool: PythonTool },
    Go { has_modules: bool },
    Java { build_tool: JavaBuildTool },
    Ruby { has_bundler: bool },
    Unknown,
}

impl ContextGraph {
    /// Build context graph from current working directory
    pub async fn build(cwd: &Path) -> Result<Self> {
        let (project, git, infra, history, env) = tokio::try_join!(
            ProjectContext::detect(cwd),
            GitContext::analyze(cwd),
            InfrastructureContext::scan(cwd),
            HistoryContext::load(),
            EnvironmentContext::detect()
        )?;

        Ok(Self {
            project,
            git,
            infrastructure: infra,
            history,
            environment: env,
        })
    }

    /// Convert context to LLM-friendly string for prompt augmentation
    pub fn to_llm_context(&self) -> String {
        let mut context = String::new();

        // Project info
        context.push_str(&format!(
            "Project: {:?} at {}\n",
            self.project.project_type,
            self.project.root_dir.display()
        ));

        // Git info
        if let Some(branch) = &self.git.current_branch {
            context.push_str(&format!("Git branch: {}\n", branch));
        }

        // Infrastructure tools
        if !self.infrastructure.tools.is_empty() {
            context.push_str("Available tools: ");
            context.push_str(
                &self.infrastructure.tools.iter()
                    .map(|t| t.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            context.push_str("\n");
        }

        // Recent command patterns
        if !self.history.frequent_commands.is_empty() {
            context.push_str("Common commands: ");
            context.push_str(
                &self.history.frequent_commands.iter()
                    .take(3)
                    .map(|c| c.command.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            context.push_str("\n");
        }

        context
    }
}
```

#### 3.1.2 Intent Classifier

```rust
// src/intelligence/intent_classifier.rs

use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Intent {
    Deploy,      // Deploy application to environment
    Search,      // Find files, text, or resources
    Cleanup,     // Remove temporary files, cleanup
    Debug,       // Investigate errors, logs
    Setup,       // Initialize project, install deps
    Monitor,     // Check status, logs, metrics
    Test,        // Run tests, validation
    Build,       // Compile, bundle, package
    Migrate,     // Database migrations, data transforms
    Rollback,    // Undo changes, restore state
    Generic,     // Catch-all for unknown intents
}

pub struct IntentClassifier {
    // In production: use sentence-transformers embeddings
    // For MVP: simple keyword matching
    keywords: HashMap<Intent, Vec<&'static str>>,
}

impl IntentClassifier {
    pub fn new() -> Self {
        let mut keywords = HashMap::new();

        keywords.insert(Intent::Deploy, vec![
            "deploy", "ship", "release", "publish", "push to production"
        ]);

        keywords.insert(Intent::Search, vec![
            "find", "search", "look for", "locate", "where is"
        ]);

        keywords.insert(Intent::Cleanup, vec![
            "clean", "remove", "delete", "clear", "purge"
        ]);

        // ... more intent keywords

        Self { keywords }
    }

    pub fn classify(&self, prompt: &str, context: &ContextGraph) -> Intent {
        let prompt_lower = prompt.to_lowercase();

        // Score each intent based on keyword matching
        let mut scores: HashMap<Intent, f32> = HashMap::new();

        for (intent, keywords) in &self.keywords {
            let score: f32 = keywords.iter()
                .map(|kw| if prompt_lower.contains(kw) { 1.0 } else { 0.0 })
                .sum();
            scores.insert(*intent, score);
        }

        // Boost scores based on context
        self.apply_context_boost(&mut scores, context);

        // Return intent with highest score
        scores.iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(intent, _)| *intent)
            .unwrap_or(Intent::Generic)
    }

    fn apply_context_boost(&self, scores: &mut HashMap<Intent, f32>, context: &ContextGraph) {
        // Example: If Docker detected, boost Deploy intent
        if context.infrastructure.has_tool("docker") {
            *scores.entry(Intent::Deploy).or_insert(0.0) += 0.5;
        }

        // If on a feature branch, boost Deploy
        if let Some(branch) = &context.git.current_branch {
            if branch.starts_with("feature/") {
                *scores.entry(Intent::Deploy).or_insert(0.0) += 0.3;
            }
        }

        // ... more context-based boosting
    }
}
```

### 3.2 Safety ML Engine

#### 3.2.1 ML Risk Predictor

```rust
// src/safety/ml_predictor.rs

use anyhow::Result;

pub struct SafetyMLPredictor {
    model: TfLiteModel,
    feature_extractor: FeatureExtractor,
}

#[derive(Debug, Clone)]
pub struct RiskPrediction {
    pub risk_score: f32,        // 0.0-10.0
    pub confidence: f32,        // 0.0-1.0
    pub risk_factors: Vec<RiskFactor>,
    pub impact: ImpactEstimate,
    pub mitigations: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RiskFactor {
    pub name: String,
    pub severity: f32,          // 0.0-1.0
    pub explanation: String,
}

#[derive(Debug, Clone)]
pub struct ImpactEstimate {
    pub files_affected: Option<usize>,
    pub data_loss_risk: f32,    // 0.0-1.0
    pub is_reversible: bool,
    pub blast_radius: BlastRadius,
}

#[derive(Debug, Clone, Copy)]
pub enum BlastRadius {
    Local,      // Current directory
    Project,    // Project root
    User,       // Home directory
    System,     // System paths
    Network,    // Network resources
}

impl SafetyMLPredictor {
    pub async fn predict(&self, command: &str, context: &ContextGraph) -> Result<RiskPrediction> {
        // Extract features from command
        let features = self.feature_extractor.extract(command, context)?;

        // Run ML model
        let (risk_score, confidence) = self.model.predict(&features)?;

        // Identify specific risk factors
        let risk_factors = self.identify_risk_factors(command, context)?;

        // Estimate impact
        let impact = self.estimate_impact(command, context).await?;

        // Generate mitigation suggestions
        let mitigations = self.suggest_mitigations(command, &risk_factors);

        Ok(RiskPrediction {
            risk_score,
            confidence,
            risk_factors,
            impact,
            mitigations,
        })
    }

    fn identify_risk_factors(&self, command: &str, context: &ContextGraph) -> Result<Vec<RiskFactor>> {
        let mut factors = Vec::new();

        // Check for destructive operations
        if command.contains("rm -rf") {
            factors.push(RiskFactor {
                name: "Recursive forced deletion".to_string(),
                severity: 0.9,
                explanation: "Command will delete files recursively without confirmation".to_string(),
            });
        }

        // Check for privileged operations
        if command.starts_with("sudo") {
            factors.push(RiskFactor {
                name: "Elevated privileges".to_string(),
                severity: 0.7,
                explanation: "Command runs with administrator privileges".to_string(),
            });
        }

        // Check for system path modifications
        if command.contains("/usr") || command.contains("/bin") || command.contains("/etc") {
            factors.push(RiskFactor {
                name: "System path modification".to_string(),
                severity: 0.95,
                explanation: "Command modifies critical system directories".to_string(),
            });
        }

        // ... more risk pattern detection

        Ok(factors)
    }

    async fn estimate_impact(&self, command: &str, context: &ContextGraph) -> Result<ImpactEstimate> {
        // Parse command to understand what it does
        let ast = parse_shell_command(command)?;

        // Estimate files affected (for file operations)
        let files_affected = if let Some(pattern) = extract_file_pattern(&ast) {
            Some(count_matching_files(&pattern, context).await?)
        } else {
            None
        };

        // Assess data loss risk
        let data_loss_risk = if is_destructive_operation(&ast) {
            if is_system_path_affected(&ast) { 1.0 }
            else if is_user_data_affected(&ast) { 0.7 }
            else { 0.3 }
        } else {
            0.0
        };

        // Check reversibility
        let is_reversible = !is_destructive_operation(&ast) || has_recycle_bin_support();

        // Determine blast radius
        let blast_radius = determine_blast_radius(&ast, context);

        Ok(ImpactEstimate {
            files_affected,
            data_loss_risk,
            is_reversible,
            blast_radius,
        })
    }
}
```

### 3.3 Learning Engine

#### 3.3.1 Pattern Database

```rust
// src/learning/pattern_db.rs

use sqlx::SqlitePool;
use anyhow::Result;

pub struct PatternDB {
    pool: SqlitePool,
    embeddings: EmbeddingIndex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandPattern {
    pub id: Uuid,
    pub user_prompt: String,
    pub generated_command: String,
    pub final_command: Option<String>,  // After user edits
    pub context: serde_json::Value,
    pub success: Option<bool>,
    pub timestamp: DateTime<Utc>,
}

impl PatternDB {
    pub async fn new(db_path: &Path) -> Result<Self> {
        let pool = SqlitePool::connect(&format!("sqlite://{}", db_path.display())).await?;

        // Create schema
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS command_patterns (
                id TEXT PRIMARY KEY,
                user_prompt TEXT NOT NULL,
                generated_command TEXT NOT NULL,
                final_command TEXT,
                context TEXT NOT NULL,
                success BOOLEAN,
                timestamp TEXT NOT NULL
            )
        "#).execute(&pool).await?;

        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_timestamp ON command_patterns(timestamp DESC)
        "#).execute(&pool).await?;

        // Initialize embedding index
        let embeddings = EmbeddingIndex::new()?;

        Ok(Self { pool, embeddings })
    }

    pub async fn record_interaction(&self, interaction: CommandPattern) -> Result<()> {
        // Store in database
        sqlx::query(r#"
            INSERT INTO command_patterns (id, user_prompt, generated_command, final_command, context, success, timestamp)
            VALUES (?, ?, ?, ?, ?, ?, ?)
        "#)
        .bind(interaction.id.to_string())
        .bind(&interaction.user_prompt)
        .bind(&interaction.generated_command)
        .bind(&interaction.final_command)
        .bind(serde_json::to_string(&interaction.context)?)
        .bind(interaction.success)
        .bind(interaction.timestamp.to_rfc3339())
        .execute(&self.pool)
        .await?;

        // Add to embedding index for similarity search
        let embedding = self.embeddings.encode(&interaction.user_prompt)?;
        self.embeddings.add(interaction.id, embedding)?;

        Ok(())
    }

    pub async fn find_similar(&self, prompt: &str, limit: usize) -> Result<Vec<CommandPattern>> {
        // Encode prompt
        let embedding = self.embeddings.encode(prompt)?;

        // Find similar in embedding space
        let similar_ids = self.embeddings.search(&embedding, limit)?;

        // Fetch from database
        let mut patterns = Vec::new();
        for id in similar_ids {
            let pattern = sqlx::query_as::<_, CommandPattern>(r#"
                SELECT * FROM command_patterns WHERE id = ?
            "#)
            .bind(id.to_string())
            .fetch_one(&self.pool)
            .await?;

            patterns.push(pattern);
        }

        Ok(patterns)
    }

    pub async fn learn_from_edit(&self, generated: &str, edited: &str) -> Result<()> {
        // Analyze the difference
        let diff = diff_commands(generated, edited);

        // Extract pattern (e.g., user always adds --color flag)
        if let Some(pattern) = extract_edit_pattern(&diff) {
            // Store learned pattern
            self.store_improvement_pattern(pattern).await?;
        }

        Ok(())
    }
}
```

---

## 4. Data Models

### 4.1 Core Domain Models

```rust
// src/models/domain.rs

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Represents a command generation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandRequest {
    pub id: Uuid,
    pub user_prompt: String,
    pub context: ContextGraph,
    pub preferences: UserPreferences,
    pub timestamp: DateTime<Utc>,
}

/// Represents a generated command with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedCommand {
    pub id: Uuid,
    pub request_id: Uuid,
    pub command: String,
    pub explanation: String,
    pub confidence: f32,
    pub risk_assessment: RiskPrediction,
    pub alternatives: Vec<AlternativeCommand>,
    pub community_commands: Vec<CommunityCommand>,
    pub timestamp: DateTime<Utc>,
}

/// Alternative command suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeCommand {
    pub command: String,
    pub reason: String,
    pub benefits: Vec<String>,
}

/// Command from community marketplace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityCommand {
    pub id: Uuid,
    pub command_template: String,
    pub description: String,
    pub author: Author,
    pub upvotes: u32,
    pub downvotes: u32,
    pub success_rate: f32,
    pub execution_count: u32,
    pub platforms: Vec<Platform>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
}

/// User preferences and settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub safety_level: SafetyLevel,
    pub auto_confirm: bool,
    pub explain_mode: ExplainMode,
    pub preferred_backend: BackendPreference,
    pub enable_community: bool,
    pub enable_learning: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SafetyLevel {
    Strict,     // Block high-risk commands
    Moderate,   // Warn on high-risk
    Permissive, // Allow all with confirmation
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ExplainMode {
    None,       // No explanations
    Brief,      // One-line summary
    Detailed,   // Step-by-step breakdown
    Interactive, // Tutorial-style
}
```

### 4.2 Cloud Sync Models

```rust
// src/models/sync.rs

/// Cloud-synced user data (Pro/Team tier)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncedUserData {
    pub user_id: Uuid,
    pub patterns: Vec<CommandPattern>,
    pub preferences: UserPreferences,
    pub playbooks: Vec<Playbook>,
    pub achievements: Vec<Achievement>,
    pub last_sync: DateTime<Utc>,
}

/// Team playbook definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playbook {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub steps: Vec<PlaybookStep>,
    pub prerequisites: Vec<Prerequisite>,
    pub team_id: Option<Uuid>,
    pub visibility: Visibility,
    pub author: Author,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookStep {
    pub name: String,
    pub command_template: String,
    pub variables: HashMap<String, VariableSpec>,
    pub conditional: Option<String>,  // Boolean expression
    pub timeout: Option<Duration>,
    pub rollback_command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Visibility {
    Private,    // Only author
    Team,       // Team members
    Public,     // Community marketplace
}
```

---

## 5. API Contracts

### 5.1 Intelligence Engine API

```rust
// src/intelligence/mod.rs

pub trait IntelligenceEngine {
    /// Build context graph from current environment
    async fn build_context(&self, cwd: &Path) -> Result<ContextGraph>;

    /// Classify user intent from prompt
    fn classify_intent(&self, prompt: &str, context: &ContextGraph) -> Intent;

    /// Augment LLM prompt with context
    fn augment_prompt(&self, prompt: &str, context: &ContextGraph) -> String;

    /// Detect available tools in project
    async fn detect_tools(&self, project_root: &Path) -> Result<Vec<Tool>>;
}
```

### 5.2 Safety Engine API

```rust
// src/safety/mod.rs

pub trait SafetyEngine {
    /// Predict risk of command execution
    async fn predict_risk(&self, command: &str, context: &ContextGraph) -> Result<RiskPrediction>;

    /// Execute command in sandbox
    async fn execute_sandboxed(&self, command: &str) -> Result<SandboxExecution>;

    /// Log command for audit trail
    async fn log_execution(&self, entry: AuditEntry) -> Result<()>;

    /// Check command against policy
    async fn check_policy(&self, command: &str, policy: &Policy) -> Result<PolicyDecision>;
}
```

### 5.3 Learning Engine API

```rust
// src/learning/mod.rs

pub trait LearningEngine {
    /// Record command interaction
    async fn record_interaction(&self, interaction: CommandPattern) -> Result<()>;

    /// Find similar past commands
    async fn find_similar(&self, prompt: &str, limit: usize) -> Result<Vec<CommandPattern>>;

    /// Generate command explanation
    fn explain_command(&self, command: &str) -> Result<Explanation>;

    /// Learn improvement from user edit
    async fn learn_from_edit(&self, generated: &str, edited: &str) -> Result<()>;
}
```

### 5.4 Community API

```rust
// src/community/mod.rs

pub trait CommunityMarketplace {
    /// Search community commands
    async fn search_commands(&self, query: &str, filters: SearchFilters) -> Result<Vec<CommunityCommand>>;

    /// Submit command to marketplace
    async fn submit_command(&self, command: CommunityCommand) -> Result<Uuid>;

    /// Vote on community command
    async fn vote(&self, command_id: Uuid, vote: Vote) -> Result<()>;

    /// Report command (spam/malicious)
    async fn report_command(&self, command_id: Uuid, reason: String) -> Result<()>;

    /// Execute playbook
    async fn execute_playbook(&self, playbook_id: Uuid, variables: HashMap<String, String>) -> Result<PlaybookExecution>;
}
```

---

## 6. Success Metrics

### 6.1 Product Metrics

| Metric | Definition | Target (M3) | Target (M6) | Target (M12) |
|--------|-----------|-------------|-------------|--------------|
| **MAU** | Monthly Active Users | 1,000 | 10,000 | 100,000 |
| **DAU/MAU** | Daily/Monthly active ratio | 0.2 | 0.25 | 0.3 |
| **Commands/User/Week** | Avg commands generated | 3 | 7 | 15 |
| **7-Day Retention** | Users active after 7 days | 40% | 50% | 60% |
| **30-Day Retention** | Users active after 30 days | 20% | 30% | 40% |
| **Edit Rate** | % commands edited by user | <30% | <20% | <15% |
| **Explanation Views** | % commands explained | 20% | 30% | 40% |
| **Community Usage** | % using community commands | 5% | 15% | 25% |
| **Sandbox Usage** | % using sandbox execution | 2% | 5% | 10% |

### 6.2 Business Metrics

| Metric | M3 | M6 | M9 | M12 |
|--------|-----|-----|-----|------|
| **Free Users** | 1,000 | 10,000 | 50,000 | 100,000 |
| **Pro Users** | 20 | 200 | 1,500 | 5,000 |
| **Team Seats** | 0 | 100 | 500 | 1,000 |
| **Enterprise Customers** | 0 | 1 | 3 | 10 |
| **MRR** | $180 | $6,100 | $42,000 | $116,500 |
| **ARR Run Rate** | $2.2K | $73K | $504K | $1.4M |
| **Free→Pro Conversion** | 2% | 3% | 4% | 5% |
| **Churn (Monthly)** | 10% | 5% | 3% | 2% |

### 6.3 Technical Metrics

| Metric | Target | Critical Threshold |
|--------|--------|-------------------|
| **Startup Time** | < 100ms | < 200ms |
| **Context Build Time** | < 300ms | < 500ms |
| **First Command Time** | < 2s | < 5s |
| **ML Inference Time** | < 50ms | < 100ms |
| **Sandbox Create Time** | < 500ms | < 1s |
| **Binary Size** | < 50MB | < 100MB |
| **Memory Usage** | < 100MB | < 200MB |
| **Crash Rate** | < 0.1% | < 1% |

---

## 7. Implementation Roadmap

### Phase 1: Foundation (M1-M3)

#### Month 1: Context Intelligence
**Deliverable**: Context-aware command generation

**Tasks**:
- [ ] T001: Implement `ProjectContext` detection (5 languages)
- [ ] T002: Implement `GitContext` analysis
- [ ] T003: Implement `InfrastructureContext` tool detection
- [ ] T004: Implement `HistoryContext` shell history parsing
- [ ] T005: Implement `ContextGraph` builder with parallel execution
- [ ] T006: Implement intent classification (keyword-based MVP)
- [ ] T007: Augment LLM prompts with context
- [ ] T008: Add context visualization in verbose mode
- [ ] T009: Write integration tests for context detection
- [ ] T010: Benchmark context build performance (< 300ms)

**Definition of Done**:
- `cmdai "deploy"` correctly interprets project type and generates appropriate deployment command
- Context detection works for Node.js, Python, Rust, Go, Docker projects
- Context build completes in < 300ms on typical project
- Unit test coverage > 80%

#### Month 2: Safety ML Engine
**Deliverable**: ML-powered risk prediction

**Tasks**:
- [ ] T011: Design feature extraction for commands
- [ ] T012: Create training dataset (10K commands, labeled dangerous/safe)
- [ ] T013: Train TFLite risk prediction model (target >90% accuracy)
- [ ] T014: Implement `SafetyMLPredictor` with TFLite integration
- [ ] T015: Implement impact estimation heuristics
- [ ] T016: Implement sandbox environment (btrfs snapshots on Linux)
- [ ] T017: Implement rollback mechanism
- [ ] T018: Add risk visualization in output
- [ ] T019: Write tests for safety engine
- [ ] T020: Benchmark ML inference (< 50ms)

**Definition of Done**:
- ML model achieves >90% precision on dangerous commands
- Risk prediction completes in < 50ms
- Sandbox can execute commands and rollback changes
- User sees clear risk assessment with specific factors

#### Month 3: Polish & Launch
**Deliverable**: Public beta launch

**Tasks**:
- [ ] T021: Improve CLI output (colors, formatting, progress bars)
- [ ] T022: Add interactive onboarding tutorial
- [ ] T023: Write comprehensive documentation
- [ ] T024: Create demo videos (30s, 2min, 10min versions)
- [ ] T025: Build landing page (cmdai.dev)
- [ ] T026: Set up analytics (PostHog or similar)
- [ ] T027: Prepare HN launch post
- [ ] T028: Product Hunt submission
- [ ] T029: Social media campaign (Twitter, Reddit)
- [ ] T030: Fix top 20 bugs from beta testing

**Definition of Done**:
- Launched on Hacker News (target >200 upvotes)
- 1,000 downloads in first month
- >50% 7-day retention
- NPS score >40

### Phase 2: Learning & Community (M4-M6)

#### Month 4: Learning Engine
**Deliverable**: System that learns from interactions

**Tasks**:
- [ ] T031: Implement `PatternDB` with SQLite
- [ ] T032: Implement embedding-based similarity search
- [ ] T033: Implement user edit tracking and learning
- [ ] T034: Implement command explainer (shell AST parser)
- [ ] T035: Create 5 interactive tutorials (find, grep, awk, sed, docker)
- [ ] T036: Implement achievement system
- [ ] T037: Add explanation UI in CLI
- [ ] T038: Privacy controls (opt-in/out)
- [ ] T039: Write tests for learning engine
- [ ] T040: Performance optimization (db queries < 100ms)

**Definition of Done**:
- System learns from user edits and improves suggestions
- Command explanations are clear and helpful (user survey >4/5)
- 5 tutorials available with completion tracking
- Pattern DB can handle 100K+ interactions

#### Month 5: Community Marketplace
**Deliverable**: Community command sharing

**Tasks**:
- [ ] T041: Design command registry schema
- [ ] T042: Implement REST API for marketplace (backend)
- [ ] T043: Implement search functionality (semantic search)
- [ ] T044: Implement command submission workflow
- [ ] T045: Implement voting system
- [ ] T046: Implement reputation engine
- [ ] T047: Build moderation tools
- [ ] T048: CLI integration for searching/submitting
- [ ] T049: Safety scanning for submitted commands
- [ ] T050: Launch marketplace with seed content (100 commands)

**Definition of Done**:
- Marketplace API deployed and operational
- 100+ curated commands available
- Users can search, vote, and submit commands from CLI
- Moderation queue handles reported commands

#### Month 6: Team Playbooks
**Deliverable**: Shareable workflows

**Tasks**:
- [ ] T051: Design playbook YAML format
- [ ] T052: Implement playbook parser and validator
- [ ] T053: Implement execution engine (step-by-step)
- [ ] T054: Implement variable templating
- [ ] T055: Implement prerequisite checking
- [ ] T056: Implement rollback on failure
- [ ] T057: Build playbook editor (CLI wizard)
- [ ] T058: Create 10 example playbooks (deploy, setup, etc.)
- [ ] T059: Write playbook execution tests
- [ ] T060: Documentation for playbook authoring

**Definition of Done**:
- Playbooks can be authored, shared, and executed
- 10 community playbooks available
- Execution handles errors gracefully with rollback
- Users can customize playbooks with variables

### Phase 3: Monetization (M7-M9)

#### Month 7: Pro Tier Launch
**Deliverable**: Paid tier with cloud sync

**Tasks**:
- [ ] T061: Implement Stripe integration
- [ ] T062: Build cloud sync backend (encrypted storage)
- [ ] T063: Implement device synchronization
- [ ] T064: Build analytics dashboard (web UI)
- [ ] T065: Implement priority support system
- [ ] T066: Create pricing page and checkout flow
- [ ] T067: Build user account management
- [ ] T068: Email drip campaign for conversions
- [ ] T069: A/B test pricing ($7 vs $9 vs $12)
- [ ] T070: Launch Pro tier publicly

**Definition of Done**:
- Payment flow working end-to-end
- Cloud sync operational across devices
- Analytics dashboard live for Pro users
- First 50 paying customers

#### Month 8: Team Tier Launch
**Deliverable**: Team collaboration features

**Tasks**:
- [ ] T071: Implement team management (invite, roles)
- [ ] T072: Build shared playbook storage
- [ ] T073: Implement team analytics dashboard
- [ ] T074: Build audit log system
- [ ] T075: Implement SSO (Google, GitHub, SAML)
- [ ] T076: Create team admin portal
- [ ] T077: Billing for team subscriptions
- [ ] T078: Team onboarding flow
- [ ] T079: Sales materials and case studies
- [ ] T080: Launch Team tier with 5 pilot customers

**Definition of Done**:
- Teams can share playbooks and view analytics
- SSO working for enterprise customers
- 5-10 paying team customers
- Positive feedback from pilot teams

#### Month 9: Enterprise Features
**Deliverable**: Enterprise-ready product

**Tasks**:
- [ ] T081: Implement policy-as-code engine
- [ ] T082: Build SIEM integration (Splunk, Datadog)
- [ ] T083: Implement SOC2 compliance exports
- [ ] T084: Create on-premise deployment docs
- [ ] T085: Build enterprise sales deck
- [ ] T086: Hire first sales person
- [ ] T087: Enterprise pilot program (3 companies)
- [ ] T088: Custom contract negotiation
- [ ] T089: Security audit and penetration testing
- [ ] T090: Enterprise customer success program

**Definition of Done**:
- Enterprise features operational
- 1-2 enterprise contracts signed
- Security audit passed
- Reference customers for case studies

### Phase 4: Dominance (M10-M12)

#### Month 10: Ecosystem Expansion
**Deliverable**: Integrations with developer tools

**Tasks**:
- [ ] T091: Build VS Code extension
- [ ] T092: Warp terminal integration
- [ ] T093: Starship prompt module
- [ ] T094: GitHub Actions integration
- [ ] T095: API documentation for third-party integrations
- [ ] T096: Developer portal for partners
- [ ] T097: Plugin system for custom backends
- [ ] T098: Conference talks (3-5 conferences)
- [ ] T099: Partner co-marketing campaigns
- [ ] T100: Developer community meetups

**Definition of Done**:
- 3+ integrations live (VS Code, Warp, Starship)
- API available for third-party developers
- 50K+ MAU across all platforms

#### Month 11: Advanced Features
**Deliverable**: Next-gen capabilities

**Tasks**:
- [ ] T101: Multi-step goal completion (GPT-4 orchestration)
- [ ] T102: Shell script generation from high-level goals
- [ ] T103: Command optimization engine
- [ ] T104: Custom model fine-tuning (enterprise)
- [ ] T105: Advanced analytics and insights
- [ ] T106: Predictive command suggestions
- [ ] T107: Natural language debugging
- [ ] T108: Automated error recovery
- [ ] T109: Performance optimization (sub-second commands)
- [ ] T110: Advanced UX improvements

**Definition of Done**:
- Advanced features available to Pro/Enterprise users
- User testimonials highlighting indispensability
- Daily usage becoming habitual (DAU/MAU >0.3)

#### Month 12: Series A Preparation
**Deliverable**: Investment-ready company

**Tasks**:
- [ ] T111: Build investor metrics dashboard
- [ ] T112: Write detailed case studies (5-10 customers)
- [ ] T113: Financial model (3-year projections)
- [ ] T114: Competitive analysis deep dive
- [ ] T115: Growth playbook documentation
- [ ] T116: Pitch deck creation
- [ ] T117: Investor outreach (warm intros)
- [ ] T118: Due diligence preparation
- [ ] T119: Team expansion plan
- [ ] T120: Series A raise ($5-10M)

**Definition of Done**:
- $1M+ ARR achieved
- 100K+ MAU
- Strong unit economics (LTV/CAC >3)
- Term sheet from tier-1 VC

---

## 8. Open Questions & Risks

### Open Questions

1. **ML Model Training**: Who labels the training data for risk prediction? Internal team or crowdsourced?
2. **Community Moderation**: How do we prevent malicious commands in marketplace while staying permissionless?
3. **Privacy vs Features**: How much telemetry is acceptable for improving ML models?
4. **Pricing**: Should we charge per-user or per-command-limit for Pro tier?
5. **Enterprise Sales**: Build sales team in-house or partner with external agency?

### Technical Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| ML model underperforms | Medium | High | Fall back to rule-based safety, invest in data quality |
| Sandbox performance issues | High | Medium | Offer dry-run mode as alternative, optimize filesystem layer |
| Community spam/abuse | High | Medium | Automated safety scans + reputation system + moderation |
| Cloud sync scaling costs | Medium | Medium | Efficient encoding, caching, usage limits for free tier |
| Competition copies features | High | Low | Focus on network effects (community data is moat) |

### Business Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Low adoption (no PMF) | Medium | Critical | Beta program, iterate fast, pivot if needed |
| Slow viral growth | High | High | Paid acquisition budget, partnerships, content marketing |
| Enterprise sales cycle too long | High | Medium | Start with SMB/mid-market, bottom-up adoption |
| Regulatory compliance issues | Low | High | Early legal review, compliance-first design |
| Open source competitors | Medium | Medium | Offer superior UX, enterprise features, community |

---

## 9. Next Steps

### Immediate Actions (This Week)

1. **Review & Approve**: Maintainer reviews this spec, approves direction
2. **Team Alignment**: Share with all contributors, gather feedback
3. **Rename Project**: Kill "cmdai" → Rebrand to "Shell Sensei" or better name
4. **Create Epic**: Break down M1 tasks into GitHub issues
5. **Kickoff Meeting**: Full team sync on V2 vision and Q1 goals

### Week 1-2 Actions

1. **Setup Infrastructure**:
   - Create `src/intelligence/` module structure
   - Add TFLite dependency to `Cargo.toml`
   - Setup ML training environment (Python + TensorFlow)
   - Create `PatternDB` schema

2. **Start Building**:
   - Implement `ProjectContext` detection (T001)
   - Implement `GitContext` analysis (T002)
   - Create training dataset for ML model (T012)

3. **Documentation**:
   - Write contributor guide for V2
   - Create architecture decision records (ADRs)
   - Document ML model requirements

### Month 1 Goal

**Ship Context Intelligence MVP**: Users can run `cmdai "deploy"` and get contextually perfect command based on their project type, Git state, and available tools.

**Success Criteria**:
- Context detection works for 5 languages
- Context build < 300ms
- Commands are noticeably better than V1
- Internal team dogfooding daily

---

## 10. Appendices

### Appendix A: Competitive Analysis

| Feature | cmdai V2 | Shell-GPT | Aider | Warp AI | Copilot CLI |
|---------|----------|-----------|-------|---------|-------------|
| Context Awareness | ✅ Full | ❌ None | ⚠️ Basic | ⚠️ Basic | ⚠️ Basic |
| ML Safety | ✅ Yes | ❌ No | ❌ No | ⚠️ Patterns | ⚠️ Patterns |
| Sandbox Execution | ✅ Yes | ❌ No | ❌ No | ❌ No | ❌ No |
| Learning Engine | ✅ Yes | ❌ No | ⚠️ Limited | ❌ No | ❌ No |
| Community Commands | ✅ Yes | ❌ No | ❌ No | ❌ No | ❌ No |
| Team Playbooks | ✅ Yes | ❌ No | ❌ No | ❌ No | ❌ No |
| Explanations | ✅ Interactive | ⚠️ Basic | ❌ No | ⚠️ Basic | ⚠️ Basic |
| Enterprise Features | ✅ Yes | ❌ No | ❌ No | ✅ Yes | ✅ Yes |
| Pricing | Free + Pro | Free | Free + Pro | Free + Pro | Free + Pro |
| Open Source | ✅ AGPL | ✅ MIT | ✅ Apache | ❌ Closed | ❌ Closed |

**Verdict**: cmdai V2 has 6+ unique differentiating features that create defensible moat.

### Appendix B: Technology Stack

**Core Application**:
- Rust (CLI, backends, core logic)
- SQLite (local data storage)
- TensorFlow Lite (ML inference)
- BTRFS/APFS (sandboxing)

**ML Training**:
- Python + TensorFlow (model training)
- Sentence-transformers (embeddings)
- Hugging Face (model hosting)

**Cloud Backend** (Optional, Pro/Team):
- Rust + Axum (REST API)
- PostgreSQL (user data)
- Redis (caching)
- S3 (encrypted blob storage)

**Web Dashboard** (Pro/Team):
- Next.js + TypeScript
- Tailwind CSS
- Recharts (analytics)

**Infrastructure**:
- Fly.io or Railway (hosting)
- Cloudflare (CDN)
- Stripe (payments)
- PostHog (analytics)

### Appendix C: Naming Options

**Current**: cmdai (rejected - too generic)

**Alternatives**:
1. **Shell Sensei** - Implies mastery and teaching
2. **CommandIQ** - Intelligence + commands
3. **Terminal Sage** - Wisdom and expertise
4. **Invoke** - Simple, powerful, available domain
5. **Cartographer** - Maps intent to commands
6. **Lodestar** - Guides you (navigation theme)
7. **Catalyst** - Accelerates your workflow

**Recommendation**: "Shell Sensei" - Best conveys the learning/teaching aspect while being memorable and searchable.

---

## Document History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2025-11-19 | Architecture Team | Initial V2 specification |

---

**END OF SPECIFICATION**
