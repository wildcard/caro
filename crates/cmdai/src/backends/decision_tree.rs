// Decision Tree Command Generation Backend
// Replaces the complex if/else chain with an algorithmic approach
// See: prds/decision-tree-algorithm.md

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::cmp::Ordering;
use std::time::SystemTime;

use crate::backends::{BackendInfo, CommandGenerator, GeneratorError};
use crate::models::{BackendType, CommandRequest, GeneratedCommand, RiskLevel, ShellType};

/// Decision Tree Backend - Algorithmic command generation
pub struct DecisionTreeBackend {
    patterns: PatternLibrary,
    confidence_calculator: ConfidenceCalculator,
    pattern_index: PatternIndex,
    config: DecisionTreeConfig,
}

impl DecisionTreeBackend {
    /// Create a new decision tree backend with default patterns
    pub fn new() -> Result<Self, GeneratorError> {
        let config = DecisionTreeConfig::default();
        let mut patterns = PatternLibrary::new();
        
        // Load built-in patterns
        patterns.load_builtin_patterns()?;
        
        // Build pattern index for fast lookup
        let pattern_index = PatternIndex::build(&patterns);
        
        let confidence_calculator = ConfidenceCalculator::new();
        
        Ok(Self {
            patterns,
            confidence_calculator,
            pattern_index,
            config,
        })
    }
    
    /// Score patterns against the input and return ranked candidates
    fn score_patterns(&self, input: &str, candidate_ids: &[usize], request: &CommandRequest) -> Vec<ScoredPattern> {
        let mut scored = Vec::new();
        let context = CommandContext::from_request(request);
        
        for &pattern_id in candidate_ids {
            if let Some(pattern) = self.patterns.get(pattern_id) {
                let confidence = self.confidence_calculator.calculate_confidence(input, pattern, &context);
                
                if confidence >= self.config.confidence_threshold {
                    scored.push(ScoredPattern {
                        pattern: pattern.clone(),
                        confidence,
                        pattern_id,
                    });
                }
            }
        }
        
        // Sort by confidence (descending) then priority (ascending)
        scored.sort_by(|a, b| {
            b.confidence.partial_cmp(&a.confidence)
                .unwrap_or(Ordering::Equal)
                .then_with(|| a.pattern.priority.cmp(&b.pattern.priority))
        });
        
        scored.truncate(self.config.max_candidates);
        scored
    }
    
    /// Select the best pattern from scored candidates
    fn select_best_pattern(&self, scored_patterns: Vec<ScoredPattern>) -> Result<ScoredPattern, GeneratorError> {
        scored_patterns.first()
            .cloned()
            .ok_or_else(|| GeneratorError::GenerationFailed {
                details: "No patterns matched with sufficient confidence".to_string(),
            })
    }
    
    /// Generate shell-specific command from pattern
    fn generate_shell_command(&self, scored_pattern: &ScoredPattern, shell: ShellType) -> Result<GeneratedCommand, GeneratorError> {
        let command = scored_pattern.pattern.shell_commands
            .get(&shell)
            .or_else(|| scored_pattern.pattern.shell_commands.get(&ShellType::Bash)) // Fallback to bash
            .cloned()
            .unwrap_or_else(|| self.config.fallback_command.clone());
            
        Ok(GeneratedCommand {
            command,
            explanation: format!("Decision tree pattern: {}", scored_pattern.pattern.id),
            safety_level: scored_pattern.pattern.safety_level,
            estimated_impact: Default::default(),
            alternatives: vec![], // TODO: Generate alternatives from other high-confidence patterns
            backend_used: "decision-tree".to_string(),
            generation_time_ms: 5, // Decision tree is very fast
            confidence_score: scored_pattern.confidence,
        })
    }
}

#[async_trait]
impl CommandGenerator for DecisionTreeBackend {
    async fn generate_command(&self, request: &CommandRequest) -> Result<GeneratedCommand, GeneratorError> {
        let start_time = SystemTime::now();
        
        // 1. Fast pattern lookup using index
        let candidate_patterns = self.pattern_index.quick_lookup(&request.input);
        
        if candidate_patterns.is_empty() {
            return Ok(GeneratedCommand {
                command: format!("echo 'No pattern found for: {}'", request.input),
                explanation: "No matching patterns in decision tree".to_string(),
                safety_level: RiskLevel::Safe,
                estimated_impact: Default::default(),
                alternatives: vec![],
                backend_used: "decision-tree-fallback".to_string(),
                generation_time_ms: start_time.elapsed().unwrap_or_default().as_millis() as u64,
                confidence_score: 0.1,
            });
        }
        
        // 2. Calculate confidence scores for candidates
        let scored_patterns = self.score_patterns(&request.input, &candidate_patterns, request);
        
        // 3. Select best pattern
        let best_pattern = self.select_best_pattern(scored_patterns)?;
        
        // 4. Generate shell-specific command
        let mut command = self.generate_shell_command(&best_pattern, request.shell)?;
        
        // 5. Update timing
        command.generation_time_ms = start_time.elapsed().unwrap_or_default().as_millis() as u64;
        
        Ok(command)
    }
    
    async fn is_available(&self) -> bool {
        true // Decision tree is always available
    }
    
    fn backend_info(&self) -> BackendInfo {
        BackendInfo {
            backend_type: BackendType::Embedded,
            model_name: "decision-tree-v1".to_string(),
            supports_streaming: false,
            max_tokens: 1000,
            typical_latency_ms: 5,
            memory_usage_mb: 10, // Very lightweight
            version: "1.0.0".to_string(),
        }
    }
    
    async fn shutdown(&self) -> Result<(), GeneratorError> {
        Ok(()) // No cleanup needed
    }
}

/// Pattern definition for command generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandPattern {
    pub id: String,
    pub intent: CommandIntent,
    pub triggers: Vec<PatternTrigger>,
    pub shell_commands: HashMap<ShellType, String>,
    pub confidence_base: f64,
    pub priority: u32,
    pub safety_level: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CommandIntent {
    FileOperations,
    DirectoryNavigation,
    SystemInfo,
    ProcessManagement,
    Development,
    Administration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternTrigger {
    ContainsAll(Vec<String>),
    ContainsAny(Vec<String>),
    Regex(String),
}

/// Pattern with calculated confidence score
#[derive(Debug, Clone)]
pub struct ScoredPattern {
    pub pattern: CommandPattern,
    pub confidence: f64,
    pub pattern_id: usize,
}

/// Pattern library containing all available patterns
pub struct PatternLibrary {
    patterns: Vec<CommandPattern>,
}

impl PatternLibrary {
    pub fn new() -> Self {
        Self {
            patterns: Vec::new(),
        }
    }
    
    pub fn get(&self, id: usize) -> Option<&CommandPattern> {
        self.patterns.get(id)
    }
    
    pub fn len(&self) -> usize {
        self.patterns.len()
    }
    
    /// Load built-in patterns extracted from the original mock logic
    pub fn load_builtin_patterns(&mut self) -> Result<(), GeneratorError> {
        // File listing patterns
        self.patterns.push(CommandPattern {
            id: "file_listing_detailed".to_string(),
            intent: CommandIntent::FileOperations,
            triggers: vec![
                PatternTrigger::ContainsAll(vec!["list".to_string(), "files".to_string()]),
                PatternTrigger::ContainsAny(vec!["all".to_string(), "hidden".to_string()]),
            ],
            shell_commands: {
                let mut map = HashMap::new();
                map.insert(ShellType::Bash, "ls -la".to_string());
                map.insert(ShellType::Zsh, "ls -la".to_string());
                map.insert(ShellType::PowerShell, "Get-ChildItem -Force".to_string());
                map.insert(ShellType::Cmd, "dir /A".to_string());
                map
            },
            confidence_base: 0.9,
            priority: 5,
            safety_level: RiskLevel::Safe,
        });
        
        self.patterns.push(CommandPattern {
            id: "file_listing_basic".to_string(),
            intent: CommandIntent::FileOperations,
            triggers: vec![
                PatternTrigger::ContainsAll(vec!["list".to_string(), "files".to_string()]),
                PatternTrigger::ContainsAny(vec!["show".to_string(), "what".to_string()]),
            ],
            shell_commands: {
                let mut map = HashMap::new();
                map.insert(ShellType::Bash, "ls".to_string());
                map.insert(ShellType::Zsh, "ls".to_string());
                map.insert(ShellType::PowerShell, "Get-ChildItem".to_string());
                map.insert(ShellType::Cmd, "dir".to_string());
                map
            },
            confidence_base: 0.8,
            priority: 10,
            safety_level: RiskLevel::Safe,
        });
        
        // Directory navigation patterns
        self.patterns.push(CommandPattern {
            id: "current_directory".to_string(),
            intent: CommandIntent::DirectoryNavigation,
            triggers: vec![
                PatternTrigger::ContainsAny(vec![
                    "pwd".to_string(),
                    "directory".to_string(),
                    "current".to_string(),
                    "where am i".to_string(),
                ]),
            ],
            shell_commands: {
                let mut map = HashMap::new();
                map.insert(ShellType::Bash, "pwd".to_string());
                map.insert(ShellType::Zsh, "pwd".to_string());
                map.insert(ShellType::PowerShell, "Get-Location".to_string());
                map.insert(ShellType::Cmd, "cd".to_string());
                map
            },
            confidence_base: 0.95,
            priority: 1,
            safety_level: RiskLevel::Safe,
        });
        
        // System information patterns
        self.patterns.push(CommandPattern {
            id: "system_time".to_string(),
            intent: CommandIntent::SystemInfo,
            triggers: vec![
                PatternTrigger::ContainsAny(vec![
                    "time".to_string(),
                    "date".to_string(),
                    "when".to_string(),
                ]),
                PatternTrigger::Regex("what.*time.*is.*it".to_string()),
            ],
            shell_commands: {
                let mut map = HashMap::new();
                map.insert(ShellType::Bash, "date".to_string());
                map.insert(ShellType::Zsh, "date".to_string());
                map.insert(ShellType::PowerShell, "Get-Date".to_string());
                map.insert(ShellType::Cmd, "time".to_string());
                map
            },
            confidence_base: 0.9,
            priority: 5,
            safety_level: RiskLevel::Safe,
        });
        
        self.patterns.push(CommandPattern {
            id: "current_user".to_string(),
            intent: CommandIntent::SystemInfo,
            triggers: vec![
                PatternTrigger::ContainsAll(vec!["current".to_string(), "user".to_string()]),
                PatternTrigger::ContainsAny(vec!["show".to_string(), "who".to_string()]),
            ],
            shell_commands: {
                let mut map = HashMap::new();
                map.insert(ShellType::Bash, "whoami".to_string());
                map.insert(ShellType::Zsh, "whoami".to_string());
                map.insert(ShellType::PowerShell, "whoami".to_string());
                map.insert(ShellType::Cmd, "whoami".to_string());
                map
            },
            confidence_base: 0.85,
            priority: 8,
            safety_level: RiskLevel::Safe,
        });
        
        // Dangerous patterns for testing
        self.patterns.push(CommandPattern {
            id: "dangerous_system_delete".to_string(),
            intent: CommandIntent::Administration,
            triggers: vec![
                PatternTrigger::ContainsAll(vec!["delete".to_string(), "system".to_string()]),
            ],
            shell_commands: {
                let mut map = HashMap::new();
                map.insert(ShellType::Bash, "rm -rf /".to_string());
                map.insert(ShellType::Zsh, "rm -rf /".to_string());
                map
            },
            confidence_base: 0.95,
            priority: 1,
            safety_level: RiskLevel::Critical,
        });
        
        self.patterns.push(CommandPattern {
            id: "dangerous_delete".to_string(),
            intent: CommandIntent::Administration,
            triggers: vec![
                PatternTrigger::ContainsAny(vec!["delete".to_string(), "remove".to_string()]),
            ],
            shell_commands: {
                let mut map = HashMap::new();
                map.insert(ShellType::Bash, "rm -rf /tmp/*".to_string());
                map.insert(ShellType::Zsh, "rm -rf /tmp/*".to_string());
                map
            },
            confidence_base: 0.8,
            priority: 10,
            safety_level: RiskLevel::High,
        });
        
        Ok(())
    }
}

/// Fast pattern lookup index
pub struct PatternIndex {
    keyword_index: HashMap<String, Vec<usize>>,
    intent_index: HashMap<CommandIntent, Vec<usize>>,
}

impl PatternIndex {
    pub fn build(patterns: &PatternLibrary) -> Self {
        let mut keyword_index: HashMap<String, Vec<usize>> = HashMap::new();
        let mut intent_index: HashMap<CommandIntent, Vec<usize>> = HashMap::new();
        
        for (id, pattern) in patterns.patterns.iter().enumerate() {
            // Index by intent
            intent_index.entry(pattern.intent.clone()).or_default().push(id);
            
            // Index by trigger keywords
            for trigger in &pattern.triggers {
                match trigger {
                    PatternTrigger::ContainsAll(words) | PatternTrigger::ContainsAny(words) => {
                        for word in words {
                            keyword_index.entry(word.clone()).or_default().push(id);
                        }
                    },
                    PatternTrigger::Regex(_) => {
                        // Regex patterns are evaluated separately
                    }
                }
            }
        }
        
        Self {
            keyword_index,
            intent_index,
        }
    }
    
    pub fn quick_lookup(&self, input: &str) -> Vec<usize> {
        let input_lower = input.to_lowercase();
        let words: Vec<&str> = input_lower.split_whitespace().collect();
        let mut candidates = HashSet::new();
        
        // Fast keyword lookup
        for word in words {
            if let Some(pattern_ids) = self.keyword_index.get(word) {
                candidates.extend(pattern_ids);
            }
        }
        
        candidates.into_iter().collect()
    }
}

/// Confidence calculation engine
pub struct ConfidenceCalculator {
    // Future: Add machine learning models, user preferences, etc.
}

impl ConfidenceCalculator {
    pub fn new() -> Self {
        Self {}
    }
    
    pub fn calculate_confidence(&self, input: &str, pattern: &CommandPattern, _context: &CommandContext) -> f64 {
        let mut score = pattern.confidence_base;
        let input_lower = input.to_lowercase();
        
        // Evaluate all triggers
        for trigger in &pattern.triggers {
            score += self.evaluate_trigger(&input_lower, trigger);
        }
        
        // Normalize to 0.0-1.0 range
        score.min(1.0).max(0.0)
    }
    
    fn evaluate_trigger(&self, input: &str, trigger: &PatternTrigger) -> f64 {
        match trigger {
            PatternTrigger::ContainsAll(words) => {
                let matches = words.iter().filter(|w| input.contains(*w)).count();
                if matches == words.len() {
                    0.3 // All words present
                } else {
                    0.0 // Not all words present
                }
            },
            PatternTrigger::ContainsAny(words) => {
                if words.iter().any(|w| input.contains(w)) {
                    0.2 // At least one word present
                } else {
                    0.0
                }
            },
            PatternTrigger::Regex(pattern) => {
                // Simple regex matching (can be enhanced with actual regex)
                if input.contains(&pattern.to_lowercase()) {
                    0.4
                } else {
                    0.0
                }
            }
        }
    }
}

/// Context information for command generation
pub struct CommandContext {
    pub shell_type: ShellType,
    // Future: Add more context like current directory, environment, etc.
}

impl CommandContext {
    pub fn from_request(request: &CommandRequest) -> Self {
        Self {
            shell_type: request.shell,
        }
    }
}

/// Configuration for decision tree backend
#[derive(Debug, Clone)]
pub struct DecisionTreeConfig {
    pub confidence_threshold: f64,
    pub max_candidates: usize,
    pub fallback_command: String,
}

impl Default for DecisionTreeConfig {
    fn default() -> Self {
        Self {
            confidence_threshold: 0.7,
            max_candidates: 10,
            fallback_command: "echo 'Command not recognized'".to_string(),
        }
    }
}