# cmdai Enhanced Architecture

*Incorporating insights from butterfish, atuin, and semantic-code-search*

## Overview

cmdai represents a unique synthesis of three cutting-edge CLI tools while pioneering safety-first command generation. This document outlines the enhanced architecture that combines the best aspects of:

- **Butterfish**: Contextual AI interaction and goal-oriented planning
- **Atuin**: Rich history management and advanced search
- **Semantic Code Search**: Semantic understanding and local embeddings

## Core Architecture

```rust
┌─────────────────────────────────────────────────────────────────┐
│                        cmdai Core System                        │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │   Contextual    │  │    Semantic     │  │   Safety-First  │ │
│  │   AI Engine     │  │   Understanding │  │   Generation    │ │
│  │  (Butterfish)   │  │ (Semantic Code) │  │   (cmdai)       │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │  Rich History   │  │   Streaming     │  │   Multi-Backend │ │
│  │   Management    │  │   Generation    │  │    Selection    │ │
│  │    (Atuin)      │  │    (cmdai)      │  │    (cmdai)      │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## Enhanced System Components

### 1. Contextual AI Engine (Butterfish-inspired)

```rust
/// Contextual AI interaction with goal-oriented planning
pub struct ContextualAIEngine {
    /// Current interaction mode
    mode: InteractionMode,
    /// Contextual prompt library
    prompt_library: PromptLibrary,
    /// Local context embeddings
    context_embeddings: LocalEmbeddingStore,
    /// Goal planning system
    goal_planner: MultiStepPlanner,
}

#[derive(Debug, Clone)]
pub enum InteractionMode {
    /// Direct command generation
    Standard,
    /// Multi-step goal completion
    Goal {
        objective: String,
        steps: Vec<PlannedStep>,
        current_step: usize,
    },
    /// Error analysis and debugging
    Debug {
        failed_command: String,
        error_output: String,
        context: ExecutionContext,
    },
    /// Exploratory code understanding
    Explore {
        project_context: ProjectContext,
        focus_area: String,
    },
}

#[derive(Debug, Clone)]
pub struct PlannedStep {
    description: String,
    estimated_command: Option<String>,
    safety_level: RiskLevel,
    dependencies: Vec<String>,
    validation_criteria: Vec<String>,
}
```

### 2. Semantic Understanding Engine (Semantic Code Search-inspired)

```rust
/// Semantic understanding with code and command comprehension
pub struct SemanticEngine {
    /// Pre-trained sentence transformer
    embedding_model: SentenceTransformer,
    /// Local embedding cache
    embedding_cache: EmbeddingCache,
    /// Project structure parser
    project_parser: ProjectStructureParser,
    /// Command similarity engine
    similarity_engine: CommandSimilarityEngine,
}

#[derive(Debug, Clone)]
pub struct ProjectContext {
    /// Project type detection
    project_type: ProjectType,
    /// File structure embeddings
    structure_embeddings: Vec<FileEmbedding>,
    /// Build system detection
    build_system: Option<BuildSystem>,
    /// Git context
    git_context: Option<GitContext>,
    /// Language detection
    languages: Vec<ProgrammingLanguage>,
}

#[derive(Debug, Clone)]
pub enum ProjectType {
    RustCargo,
    NodeJS,
    Python,
    Go,
    Docker,
    Generic,
}

/// Command with semantic understanding
#[derive(Debug, Clone)]
pub struct SemanticCommand {
    /// Original command text
    command: String,
    /// Semantic embedding
    embedding: Vec<f32>,
    /// Intent classification
    intent: CommandIntent,
    /// Semantic similarity to project context
    context_relevance: f32,
    /// Safety classification embedding
    safety_embedding: Vec<f32>,
}

#[derive(Debug, Clone)]
pub enum CommandIntent {
    FileManagement,
    GitOperations,
    BuildAndTest,
    SystemAdministration,
    Development,
    NetworkOperations,
    DataProcessing,
}
```

### 3. Rich History Management (Atuin-inspired)

```rust
/// Enhanced command history with rich context
pub struct EnhancedHistoryManager {
    /// SQLite database storage
    database: HistoryDatabase,
    /// Encryption for sensitive data
    encryption: HistoryEncryption,
    /// Search interface
    search_interface: AdvancedSearchInterface,
    /// Sync engine (optional)
    sync_engine: Option<EncryptedSyncEngine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RichHistoryEntry {
    /// Unique entry ID
    id: String,
    /// Original command
    command: String,
    /// AI-generated flag
    ai_generated: bool,
    /// Generation prompt (if AI-generated)
    generation_prompt: Option<String>,
    /// Execution context
    context: ExecutionContext,
    /// Execution result
    result: ExecutionResult,
    /// Safety assessment
    safety_assessment: ValidationResult,
    /// User feedback
    user_feedback: Option<UserFeedback>,
    /// Semantic embedding
    embedding: Option<Vec<f32>>,
    /// Tags and categories
    tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Exit code
    exit_code: Option<i32>,
    /// Execution duration
    duration: Duration,
    /// Output summary (truncated for privacy)
    output_summary: Option<String>,
    /// Success indicator
    success: bool,
    /// Error type if failed
    error_type: Option<ErrorType>,
}
```

### 4. Advanced Search Interface (Atuin-inspired)

```rust
/// Full-screen search interface with advanced filtering
pub struct AdvancedSearchInterface {
    /// Current search query
    query: SearchQuery,
    /// Search results
    results: Vec<SearchResult>,
    /// UI state
    ui_state: SearchUIState,
    /// Search engine
    search_engine: HybridSearchEngine,
}

#[derive(Debug, Clone)]
pub struct SearchQuery {
    /// Text query (supports natural language)
    text: String,
    /// Semantic similarity threshold
    similarity_threshold: f32,
    /// Filters
    filters: SearchFilters,
    /// Sort criteria
    sort: SortCriteria,
}

#[derive(Debug, Clone)]
pub struct SearchFilters {
    /// Time range
    time_range: Option<TimeRange>,
    /// Working directory
    directory: Option<PathBuf>,
    /// Command success status
    success_only: bool,
    /// AI-generated only
    ai_generated_only: bool,
    /// Safety level range
    safety_level: Option<RangeInclusive<RiskLevel>>,
    /// Project context
    project_type: Option<ProjectType>,
    /// Command intent
    intent: Option<CommandIntent>,
}

/// Hybrid search combining text, semantic, and contextual search
pub struct HybridSearchEngine {
    /// Full-text search (SQLite FTS)
    text_search: FullTextSearch,
    /// Semantic similarity search
    semantic_search: SemanticSimilaritySearch,
    /// Contextual relevance scoring
    context_scorer: ContextualRelevanceScorer,
}
```

### 5. Privacy-Preserving Learning System

```rust
/// Privacy-preserving learning and adaptation
pub struct PrivacyPreservingLearning {
    /// Local pattern learning
    local_patterns: LocalPatternLearner,
    /// Differential privacy engine
    privacy_engine: DifferentialPrivacyEngine,
    /// Federated learning (optional)
    federated_learning: Option<FederatedLearningClient>,
}

#[derive(Debug, Clone)]
pub struct LocalPatternLearner {
    /// User command patterns
    command_patterns: HashMap<String, CommandPattern>,
    /// Context patterns
    context_patterns: HashMap<String, ContextPattern>,
    /// Safety feedback patterns
    safety_patterns: HashMap<String, SafetyPattern>,
    /// Adaptation rate
    learning_rate: f32,
}

#[derive(Debug, Clone)]
pub struct CommandPattern {
    /// Pattern template
    template: String,
    /// Frequency of use
    frequency: u32,
    /// Success rate
    success_rate: f32,
    /// Context associations
    contexts: Vec<String>,
    /// User satisfaction score
    satisfaction: f32,
}
```

### 6. Multi-Modal Safety Validation (Enhanced)

```rust
/// Enhanced safety validation with multiple detection methods
pub struct MultiModalSafetyValidator {
    /// Traditional pattern matching
    pattern_validator: PatternSafetyValidator,
    /// ML-based semantic analysis
    semantic_validator: SemanticSafetyValidator,
    /// Behavioral detection
    behavioral_validator: BehavioralSafetyValidator,
    /// User feedback learning
    adaptive_validator: AdaptiveSafetyValidator,
    /// Project context awareness
    context_validator: ContextualSafetyValidator,
}

#[derive(Debug, Clone)]
pub struct ContextualSafetyValidator {
    /// Project-specific risk assessment
    project_risk_model: ProjectRiskModel,
    /// Directory-based risk scoring
    directory_risk_scorer: DirectoryRiskScorer,
    /// File operation risk analysis
    file_operation_analyzer: FileOperationAnalyzer,
    /// Network operation context
    network_context_analyzer: NetworkContextAnalyzer,
}
```

## Integration Patterns

### 1. Contextual Command Generation

```rust
impl ContextualAIEngine {
    pub async fn generate_contextual_command(
        &self,
        prompt: &str,
        context: &ProjectContext,
        history: &[RichHistoryEntry],
    ) -> Result<ContextualCommandResult, GenerationError> {
        // Analyze project context
        let project_embedding = self.analyze_project_context(context).await?;
        
        // Find relevant history patterns
        let relevant_history = self.find_relevant_history(prompt, history).await?;
        
        // Generate command with enhanced context
        let enhanced_prompt = self.enhance_prompt_with_context(
            prompt,
            &project_embedding,
            &relevant_history,
        ).await?;
        
        // Generate with safety validation
        let command = self.generate_with_safety(enhanced_prompt).await?;
        
        Ok(ContextualCommandResult {
            command,
            context_relevance: project_embedding.relevance_score,
            historical_similarity: relevant_history.similarity_score,
            safety_assessment: command.safety_assessment,
            alternatives: self.generate_alternatives(&command).await?,
        })
    }
}
```

### 2. Semantic Search Integration

```rust
impl SemanticEngine {
    pub async fn find_similar_commands(
        &self,
        query: &str,
        history: &[RichHistoryEntry],
        context: &ProjectContext,
    ) -> Result<Vec<SemanticMatch>, SearchError> {
        // Generate query embedding
        let query_embedding = self.embed_query(query).await?;
        
        // Search command embeddings
        let mut matches = Vec::new();
        for entry in history {
            if let Some(embedding) = &entry.embedding {
                let similarity = self.calculate_similarity(&query_embedding, embedding);
                
                // Apply context relevance scoring
                let context_relevance = self.score_context_relevance(
                    &entry.context,
                    context,
                ).await?;
                
                let combined_score = similarity * 0.7 + context_relevance * 0.3;
                
                if combined_score > 0.6 {
                    matches.push(SemanticMatch {
                        entry: entry.clone(),
                        similarity,
                        context_relevance,
                        combined_score,
                    });
                }
            }
        }
        
        // Sort by combined score
        matches.sort_by(|a, b| b.combined_score.partial_cmp(&a.combined_score).unwrap());
        
        Ok(matches)
    }
}
```

### 3. Goal-Oriented Planning

```rust
impl MultiStepPlanner {
    pub async fn plan_goal_achievement(
        &self,
        goal: &str,
        context: &ProjectContext,
    ) -> Result<GoalPlan, PlanningError> {
        // Analyze goal complexity
        let goal_analysis = self.analyze_goal_complexity(goal).await?;
        
        // Break down into steps
        let steps = self.decompose_goal_into_steps(goal, &goal_analysis).await?;
        
        // Validate each step for safety
        let validated_steps = self.validate_steps_safety(&steps).await?;
        
        // Estimate execution time and risk
        let execution_estimate = self.estimate_execution(&validated_steps).await?;
        
        Ok(GoalPlan {
            goal: goal.to_string(),
            steps: validated_steps,
            estimated_duration: execution_estimate.duration,
            risk_assessment: execution_estimate.risk,
            prerequisites: goal_analysis.prerequisites,
            success_criteria: goal_analysis.success_criteria,
        })
    }
}
```

## Performance Optimizations

### 1. Embedding Cache Management

```rust
pub struct EmbeddingCache {
    /// In-memory LRU cache
    memory_cache: LruCache<String, Vec<f32>>,
    /// Persistent disk cache
    disk_cache: SqliteEmbeddingStore,
    /// Cache statistics
    stats: CacheStats,
}

impl EmbeddingCache {
    pub async fn get_or_compute_embedding(
        &mut self,
        text: &str,
        model: &SentenceTransformer,
    ) -> Result<Vec<f32>, EmbeddingError> {
        // Check memory cache first
        if let Some(embedding) = self.memory_cache.get(text) {
            self.stats.memory_hits += 1;
            return Ok(embedding.clone());
        }
        
        // Check disk cache
        if let Some(embedding) = self.disk_cache.get(text).await? {
            self.memory_cache.put(text.to_string(), embedding.clone());
            self.stats.disk_hits += 1;
            return Ok(embedding);
        }
        
        // Compute new embedding
        let embedding = model.encode(text).await?;
        
        // Store in both caches
        self.memory_cache.put(text.to_string(), embedding.clone());
        self.disk_cache.store(text, &embedding).await?;
        
        self.stats.computed += 1;
        Ok(embedding)
    }
}
```

### 2. Incremental Learning

```rust
pub struct IncrementalLearner {
    /// User preference model
    preference_model: UserPreferenceModel,
    /// Command success predictor
    success_predictor: CommandSuccessPredictor,
    /// Safety feedback integrator
    safety_integrator: SafetyFeedbackIntegrator,
}

impl IncrementalLearner {
    pub async fn update_from_execution(
        &mut self,
        command: &str,
        result: &ExecutionResult,
        user_feedback: Option<UserFeedback>,
    ) -> Result<(), LearningError> {
        // Update success prediction model
        self.success_predictor.update(command, result.success).await?;
        
        // Update user preferences
        if let Some(feedback) = user_feedback {
            self.preference_model.update(command, feedback).await?;
        }
        
        // Update safety model if safety-related feedback
        if result.error_type == Some(ErrorType::SafetyViolation) {
            self.safety_integrator.update_from_violation(command, result).await?;
        }
        
        Ok(())
    }
}
```

## Future Extensions

### 1. Team Collaboration Features

```rust
pub struct TeamCollaboration {
    /// Encrypted command sharing
    shared_commands: EncryptedCommandStore,
    /// Team safety policies
    team_policies: TeamSafetyPolicies,
    /// Collaborative learning
    collaborative_learning: FederatedLearningEngine,
}
```

### 2. Editor Integration

```rust
pub struct EditorIntegration {
    /// LSP server for command suggestions
    lsp_server: CommandLSPServer,
    /// VSCode extension interface
    vscode_interface: VSCodeExtension,
    /// Vim plugin interface
    vim_interface: VimPlugin,
}
```

### 3. Advanced Agent Workflows

```rust
pub struct AgentWorkflow {
    /// Multi-tool orchestration
    tool_orchestrator: ToolOrchestrator,
    /// Workflow definition language
    workflow_dsl: WorkflowDSL,
    /// Execution monitoring
    execution_monitor: WorkflowMonitor,
}
```

## Conclusion

This enhanced architecture positions cmdai as a unique synthesis of the best CLI tools while pioneering safety-first command generation. The system combines:

- **Contextual Intelligence** (Butterfish inspiration)
- **Rich History** (Atuin inspiration)  
- **Semantic Understanding** (Semantic Code Search inspiration)
- **Safety-First Generation** (cmdai innovation)

The result is a tool that enhances productivity while preventing disasters, learning from users while preserving privacy, and understanding intent while maintaining control.