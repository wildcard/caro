# Shell History Integration Design

## Overview

**Document Type**: Technical Design  
**Related PRD**: Command Generation Enhancement  
**Version**: 1.0  
**Date**: 2025-10-19  

### Purpose
This document defines the technical design for integrating shell history analysis into cmdai's command generation system, enabling context-aware suggestions based on user's actual command usage patterns, similar to tools like Atuin but focused on improving AI command generation.

## Design Philosophy

### Core Principles
1. **Privacy First**: All history processing happens locally, never uploaded
2. **Pattern Recognition**: Learn from user's actual command patterns
3. **Context Awareness**: Consider working directory, recent commands, and usage frequency
4. **Performance**: Minimal impact on command generation speed
5. **Shell Agnostic**: Support multiple shell types and history formats
6. **Opt-in**: User consent required for history analysis

### Integration Goals
- **Improve Accuracy**: Generate commands that match user's typical patterns
- **Learn Preferences**: Adapt to user's preferred command styles and variations
- **Context Enhancement**: Use historical context to improve suggestions
- **Pattern Detection**: Identify common workflows and multi-command sequences

## Shell History Analysis

### Supported Shell Types
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ShellType {
    Bash,        // ~/.bash_history
    Zsh,         // ~/.zsh_history
    Fish,        // ~/.local/share/fish/fish_history
    PowerShell,  // PowerShell history files
    Nushell,     // ~/.config/nushell/history.txt
}

pub struct HistoryLocation {
    pub shell: ShellType,
    pub default_path: PathBuf,
    pub format: HistoryFormat,
}
```

### History Parsing Implementation
```rust
// src/history/parser.rs
pub struct ShellHistoryParser {
    shell_detectors: Vec<Box<dyn ShellDetector>>,
    privacy_filter: PrivacyFilter,
}

impl ShellHistoryParser {
    pub fn detect_available_shells(&self) -> Vec<DetectedShell> {
        self.shell_detectors
            .iter()
            .filter_map(|detector| detector.detect())
            .collect()
    }
    
    pub fn parse_history(&self, shell: ShellType) -> Result<HistoryData, ParseError> {
        let parser = self.get_parser_for_shell(shell)?;
        let raw_data = parser.read_history_file()?;
        let filtered_data = self.privacy_filter.filter(raw_data)?;
        Ok(self.extract_patterns(filtered_data)?)
    }
}

pub struct HistoryData {
    pub entries: Vec<HistoryEntry>,
    pub command_frequency: HashMap<String, u32>,
    pub pattern_sequences: Vec<CommandSequence>,
    pub working_directories: HashMap<PathBuf, Vec<String>>,
    pub time_patterns: Vec<TimeBasedPattern>,
    pub total_commands: u32,
    pub date_range: (DateTime<Utc>, DateTime<Utc>),
}

pub struct HistoryEntry {
    pub command: String,
    pub timestamp: Option<DateTime<Utc>>,
    pub working_directory: Option<PathBuf>,
    pub exit_code: Option<i32>,
    pub session_id: Option<String>,
    pub duration: Option<Duration>,
}
```

### Format-Specific Parsers

#### Bash History Parser
```rust
pub struct BashHistoryParser;

impl HistoryParser for BashHistoryParser {
    fn parse_file(&self, path: &Path) -> Result<Vec<HistoryEntry>, ParseError> {
        let content = fs::read_to_string(path)?;
        let mut entries = Vec::new();
        
        for line in content.lines() {
            if let Some(entry) = self.parse_bash_line(line) {
                entries.push(entry);
            }
        }
        
        Ok(entries)
    }
    
    fn parse_bash_line(&self, line: &str) -> Option<HistoryEntry> {
        // Handle both simple and timestamped formats
        if line.starts_with('#') {
            // Timestamped format: #1634567890
            None // Timestamp line, wait for next line
        } else {
            Some(HistoryEntry {
                command: line.to_string(),
                timestamp: None, // Will be filled in if timestamp available
                working_directory: None,
                exit_code: None,
                session_id: None,
                duration: None,
            })
        }
    }
}
```

#### Zsh History Parser
```rust
pub struct ZshHistoryParser;

impl HistoryParser for ZshHistoryParser {
    fn parse_file(&self, path: &Path) -> Result<Vec<HistoryEntry>, ParseError> {
        let content = fs::read_to_string(path)?;
        let mut entries = Vec::new();
        
        for line in content.lines() {
            if let Some(entry) = self.parse_zsh_line(line) {
                entries.push(entry);
            }
        }
        
        Ok(entries)
    }
    
    fn parse_zsh_line(&self, line: &str) -> Option<HistoryEntry> {
        // Zsh extended format: : 1634567890:0;command
        if line.starts_with(": ") {
            let parts: Vec<&str> = line.splitn(4, ':').collect();
            if parts.len() >= 4 {
                let timestamp = parts[1].parse::<i64>().ok()?;
                let duration = parts[2].parse::<u64>().ok();
                let command = parts[3].to_string();
                
                return Some(HistoryEntry {
                    command,
                    timestamp: Some(DateTime::from_timestamp(timestamp, 0)?),
                    working_directory: None,
                    exit_code: None,
                    session_id: None,
                    duration: duration.map(Duration::from_secs),
                });
            }
        }
        
        // Fallback to simple format
        Some(HistoryEntry {
            command: line.to_string(),
            timestamp: None,
            working_directory: None,
            exit_code: None,
            session_id: None,
            duration: None,
        })
    }
}
```

#### Fish History Parser
```rust
pub struct FishHistoryParser;

impl HistoryParser for FishHistoryParser {
    fn parse_file(&self, path: &Path) -> Result<Vec<HistoryEntry>, ParseError> {
        let content = fs::read_to_string(path)?;
        let mut entries = Vec::new();
        let mut current_entry = None;
        
        for line in content.lines() {
            if line.starts_with("- cmd: ") {
                // New command entry
                if let Some(entry) = current_entry.take() {
                    entries.push(entry);
                }
                current_entry = Some(HistoryEntry {
                    command: line[7..].to_string(), // Remove "- cmd: " prefix
                    timestamp: None,
                    working_directory: None,
                    exit_code: None,
                    session_id: None,
                    duration: None,
                });
            } else if let Some(ref mut entry) = current_entry {
                // Parse metadata lines
                if line.starts_with("  when: ") {
                    entry.timestamp = DateTime::parse_from_rfc3339(&line[8..])
                        .ok()
                        .map(|dt| dt.with_timezone(&Utc));
                } else if line.starts_with("  paths:") {
                    // Parse working directory from paths
                }
            }
        }
        
        // Don't forget the last entry
        if let Some(entry) = current_entry {
            entries.push(entry);
        }
        
        Ok(entries)
    }
}
```

## Pattern Recognition System

### Command Pattern Analysis
```rust
// src/history/patterns.rs
pub struct PatternAnalyzer {
    frequency_analyzer: FrequencyAnalyzer,
    sequence_analyzer: SequenceAnalyzer,
    context_analyzer: ContextAnalyzer,
    preference_detector: PreferenceDetector,
}

impl PatternAnalyzer {
    pub fn analyze_history(&self, history: &HistoryData) -> PatternAnalysis {
        PatternAnalysis {
            command_preferences: self.analyze_command_preferences(history),
            common_sequences: self.analyze_command_sequences(history),
            context_patterns: self.analyze_context_patterns(history),
            usage_statistics: self.calculate_usage_statistics(history),
            workflow_patterns: self.detect_workflows(history),
        }
    }
}

pub struct PatternAnalysis {
    pub command_preferences: CommandPreferences,
    pub common_sequences: Vec<CommandSequence>,
    pub context_patterns: Vec<ContextPattern>,
    pub usage_statistics: UsageStatistics,
    pub workflow_patterns: Vec<WorkflowPattern>,
}

pub struct CommandPreferences {
    pub preferred_flags: HashMap<String, Vec<String>>, // ls -> ["-la", "-l"]
    pub command_aliases: HashMap<String, String>,      // ll -> ls -la
    pub shell_features: Vec<ShellFeature>,             // Preferred shell features
    pub style_preferences: StylePreferences,          // Verbose vs concise, etc.
}
```

### Frequency Analysis
```rust
pub struct FrequencyAnalyzer;

impl FrequencyAnalyzer {
    pub fn analyze_command_frequency(&self, history: &HistoryData) -> FrequencyAnalysis {
        let mut command_counts = HashMap::new();
        let mut flag_patterns = HashMap::new();
        let mut command_variations = HashMap::new();
        
        for entry in &history.entries {
            // Count base commands
            if let Some(base_cmd) = self.extract_base_command(&entry.command) {
                *command_counts.entry(base_cmd.clone()).or_insert(0) += 1;
                
                // Analyze flag usage patterns
                let flags = self.extract_flags(&entry.command);
                flag_patterns.entry(base_cmd)
                    .or_insert_with(HashMap::new)
                    .entry(flags)
                    .and_modify(|e| *e += 1)
                    .or_insert(1);
            }
        }
        
        FrequencyAnalysis {
            command_counts,
            flag_patterns,
            total_commands: history.entries.len(),
        }
    }
    
    fn extract_base_command(&self, command: &str) -> Option<String> {
        command.split_whitespace().next().map(|s| s.to_string())
    }
    
    fn extract_flags(&self, command: &str) -> Vec<String> {
        command.split_whitespace()
            .skip(1) // Skip the base command
            .filter(|arg| arg.starts_with('-'))
            .map(|s| s.to_string())
            .collect()
    }
}
```

### Sequence Pattern Detection
```rust
pub struct SequenceAnalyzer;

impl SequenceAnalyzer {
    pub fn find_command_sequences(&self, history: &HistoryData) -> Vec<CommandSequence> {
        let mut sequences = Vec::new();
        let window_size = 5; // Look for sequences up to 5 commands
        
        for window in history.entries.windows(window_size) {
            if let Some(sequence) = self.identify_meaningful_sequence(window) {
                sequences.push(sequence);
            }
        }
        
        // Group similar sequences and count frequency
        self.group_and_rank_sequences(sequences)
    }
    
    fn identify_meaningful_sequence(&self, window: &[HistoryEntry]) -> Option<CommandSequence> {
        // Look for patterns like:
        // git add . && git commit -m "message" && git push
        // cd project && npm install && npm start
        // find . -name "*.log" | xargs rm
        
        let commands: Vec<String> = window.iter()
            .map(|entry| entry.command.clone())
            .collect();
            
        // Check if this looks like a meaningful workflow
        if self.is_workflow_pattern(&commands) {
            Some(CommandSequence {
                commands,
                frequency: 1,
                context: self.extract_sequence_context(window),
                workflow_type: self.classify_workflow(&commands),
            })
        } else {
            None
        }
    }
}

pub struct CommandSequence {
    pub commands: Vec<String>,
    pub frequency: u32,
    pub context: SequenceContext,
    pub workflow_type: WorkflowType,
}

pub enum WorkflowType {
    GitWorkflow,
    BuildProcess,
    FileManagement,
    SystemAdmin,
    Development,
    Testing,
    Other(String),
}
```

### Context Analysis
```rust
pub struct ContextAnalyzer;

impl ContextAnalyzer {
    pub fn analyze_context_patterns(&self, history: &HistoryData) -> Vec<ContextPattern> {
        let mut patterns = Vec::new();
        
        // Group commands by working directory
        let mut dir_commands = HashMap::new();
        for entry in &history.entries {
            if let Some(ref dir) = entry.working_directory {
                dir_commands.entry(dir.clone())
                    .or_insert_with(Vec::new)
                    .push(entry.command.clone());
            }
        }
        
        // Analyze patterns within each directory context
        for (dir, commands) in dir_commands {
            if let Some(pattern) = self.extract_directory_pattern(&dir, &commands) {
                patterns.push(pattern);
            }
        }
        
        patterns
    }
    
    fn extract_directory_pattern(&self, dir: &Path, commands: &[String]) -> Option<ContextPattern> {
        let command_frequency = self.count_command_frequency(commands);
        let project_type = self.detect_project_type(dir);
        
        Some(ContextPattern {
            context_type: ContextType::Directory(dir.to_path_buf()),
            common_commands: command_frequency,
            project_type,
            usage_frequency: commands.len() as u32,
        })
    }
    
    fn detect_project_type(&self, dir: &Path) -> Option<ProjectType> {
        // Check for project indicators
        if dir.join("Cargo.toml").exists() {
            Some(ProjectType::Rust)
        } else if dir.join("package.json").exists() {
            Some(ProjectType::Node)
        } else if dir.join(".git").exists() {
            Some(ProjectType::Git)
        } else if dir.join("Makefile").exists() {
            Some(ProjectType::Make)
        } else {
            None
        }
    }
}

pub struct ContextPattern {
    pub context_type: ContextType,
    pub common_commands: HashMap<String, u32>,
    pub project_type: Option<ProjectType>,
    pub usage_frequency: u32,
}

pub enum ContextType {
    Directory(PathBuf),
    ProjectRoot(ProjectType),
    TimeOfDay(u8), // Hour of day
    DayOfWeek(u8), // Day of week
}

pub enum ProjectType {
    Rust,
    Node,
    Python,
    Git,
    Make,
    Docker,
    Other(String),
}
```

## Privacy and Security

### Privacy Protection
```rust
// src/history/privacy.rs
pub struct PrivacyFilter {
    sensitive_patterns: Vec<Regex>,
    personal_data_patterns: Vec<Regex>,
}

impl PrivacyFilter {
    pub fn new() -> Self {
        let sensitive_patterns = vec![
            Regex::new(r"password").unwrap(),
            Regex::new(r"token").unwrap(),
            Regex::new(r"secret").unwrap(),
            Regex::new(r"key").unwrap(),
            Regex::new(r"auth").unwrap(),
            Regex::new(r"ssh.*-i").unwrap(),
            Regex::new(r"curl.*-u").unwrap(),
            Regex::new(r"wget.*--user").unwrap(),
        ];
        
        let personal_data_patterns = vec![
            Regex::new(r"/home/[^/]+").unwrap(), // Home directories
            Regex::new(r"/Users/[^/]+").unwrap(), // macOS home directories
            Regex::new(r"@[a-zA-Z0-9.-]+").unwrap(), // Email addresses
            Regex::new(r"[0-9]{3}-[0-9]{3}-[0-9]{4}").unwrap(), // Phone numbers
        ];
        
        Self {
            sensitive_patterns,
            personal_data_patterns,
        }
    }
    
    pub fn filter_history(&self, history: HistoryData) -> HistoryData {
        let filtered_entries = history.entries
            .into_iter()
            .filter_map(|entry| self.filter_entry(entry))
            .collect();
            
        HistoryData {
            entries: filtered_entries,
            ..history
        }
    }
    
    fn filter_entry(&self, mut entry: HistoryEntry) -> Option<HistoryEntry> {
        // Remove commands with sensitive data
        if self.contains_sensitive_data(&entry.command) {
            return None;
        }
        
        // Anonymize personal data in working directory
        if let Some(ref mut dir) = entry.working_directory {
            *dir = self.anonymize_path(dir);
        }
        
        // Anonymize the command itself
        entry.command = self.anonymize_command(&entry.command);
        
        Some(entry)
    }
    
    fn contains_sensitive_data(&self, command: &str) -> bool {
        self.sensitive_patterns.iter()
            .any(|pattern| pattern.is_match(command))
    }
    
    fn anonymize_path(&self, path: &Path) -> PathBuf {
        // Replace home directories with generic placeholders
        let path_str = path.to_string_lossy();
        let anonymized = self.personal_data_patterns.iter()
            .fold(path_str.to_string(), |acc, pattern| {
                pattern.replace_all(&acc, "[USER]").to_string()
            });
        PathBuf::from(anonymized)
    }
    
    fn anonymize_command(&self, command: &str) -> String {
        // Remove personal data while preserving command structure
        self.personal_data_patterns.iter()
            .fold(command.to_string(), |acc, pattern| {
                pattern.replace_all(&acc, "[REDACTED]").to_string()
            })
    }
}
```

### User Consent and Control
```rust
pub struct HistoryConsent {
    pub fn request_history_analysis_consent() -> ConsentResult {
        println!("🔍 Shell History Analysis");
        println!("cmdai can analyze your shell history to provide better command suggestions");
        println!("that match your preferred patterns and workflows.");
        println!();
        println!("This analysis:");
        println!("  ✓ Happens entirely on your local machine");
        println!("  ✓ Never uploads your command history");
        println!("  ✓ Filters out sensitive commands automatically");
        println!("  ✓ Can be disabled at any time");
        println!();
        println!("Your history will be used to:");
        println!("  • Learn your preferred command flags and options");
        println!("  • Suggest commands that match your typical patterns");
        println!("  • Provide context-aware suggestions based on your workflows");
        println!();
        
        let choice = prompt_user_choice(&[
            "Yes, analyze my shell history locally",
            "No, use only default patterns",
            "Show more details about what data is used"
        ]);
        
        match choice {
            0 => ConsentResult::Granted,
            1 => ConsentResult::Denied,
            2 => {
                self.show_detailed_analysis_info();
                self.request_history_analysis_consent()
            }
            _ => ConsentResult::Denied,
        }
    }
}
```

## Integration with Command Generation

### Enhanced Generation Pipeline
```rust
// src/generation/enhanced.rs
pub struct HistoryEnhancedGenerator {
    base_generator: Box<dyn CommandGenerator>,
    pattern_analyzer: PatternAnalyzer,
    context_matcher: ContextMatcher,
    preference_applier: PreferenceApplier,
}

impl CommandGenerator for HistoryEnhancedGenerator {
    async fn generate_command(&self, request: &CommandRequest) -> Result<GeneratedCommand, GeneratorError> {
        // Get base generation from LLM
        let base_result = self.base_generator.generate_command(request).await?;
        
        // Load user's patterns if available
        let user_patterns = self.load_user_patterns().await?;
        
        // Enhance the command based on patterns
        let enhanced_command = self.apply_user_preferences(
            &base_result.command,
            &user_patterns,
            request
        );
        
        // Generate alternatives based on history
        let history_alternatives = self.generate_history_alternatives(
            &enhanced_command,
            &user_patterns
        );
        
        Ok(GeneratedCommand {
            command: enhanced_command,
            explanation: self.enhance_explanation(&base_result.explanation, &user_patterns),
            alternatives: [base_result.alternatives, history_alternatives].concat(),
            confidence_score: self.calculate_enhanced_confidence(&base_result, &user_patterns),
            ..base_result
        })
    }
}

impl HistoryEnhancedGenerator {
    fn apply_user_preferences(
        &self,
        command: &str,
        patterns: &PatternAnalysis,
        request: &CommandRequest
    ) -> String {
        let mut enhanced = command.to_string();
        
        // Apply flag preferences
        if let Some(base_cmd) = self.extract_base_command(command) {
            if let Some(preferred_flags) = patterns.command_preferences
                .preferred_flags.get(&base_cmd) {
                enhanced = self.apply_preferred_flags(&enhanced, preferred_flags);
            }
        }
        
        // Apply context-specific enhancements
        if let Some(context_enhancement) = self.get_context_enhancement(
            command, patterns, &request.context
        ) {
            enhanced = context_enhancement;
        }
        
        enhanced
    }
    
    fn generate_history_alternatives(
        &self,
        command: &str,
        patterns: &PatternAnalysis
    ) -> Vec<String> {
        let mut alternatives = Vec::new();
        
        // Add user's most common variations
        if let Some(base_cmd) = self.extract_base_command(command) {
            if let Some(variations) = patterns.command_preferences
                .preferred_flags.get(&base_cmd) {
                for variation in variations.iter().take(3) {
                    let alt_command = format!("{} {}", base_cmd, variation);
                    if alt_command != command {
                        alternatives.push(alt_command);
                    }
                }
            }
        }
        
        // Add sequence-based alternatives
        if let Some(sequence_alt) = self.find_sequence_alternative(command, patterns) {
            alternatives.push(sequence_alt);
        }
        
        alternatives
    }
}
```

### Context-Aware Matching
```rust
pub struct ContextMatcher;

impl ContextMatcher {
    pub fn match_context(&self, request: &CommandRequest, patterns: &PatternAnalysis) -> ContextMatch {
        let mut context_score = 0.0;
        let mut matching_patterns = Vec::new();
        
        // Match working directory context
        if let Some(ref working_dir) = request.context.as_ref()
            .and_then(|ctx| ctx.working_directory.as_ref()) {
            
            for pattern in &patterns.context_patterns {
                if let ContextType::Directory(ref pattern_dir) = pattern.context_type {
                    if self.directories_match(working_dir, pattern_dir) {
                        context_score += 0.3;
                        matching_patterns.push(pattern.clone());
                    }
                }
            }
        }
        
        // Match project type context
        if let Some(project_type) = self.detect_current_project_type(&request.context) {
            for pattern in &patterns.context_patterns {
                if pattern.project_type == Some(project_type) {
                    context_score += 0.4;
                    matching_patterns.push(pattern.clone());
                }
            }
        }
        
        ContextMatch {
            score: context_score,
            matching_patterns,
        }
    }
}
```

## Performance Optimization

### Caching Strategy
```rust
// src/history/cache.rs
pub struct HistoryCache {
    pattern_cache: LruCache<String, PatternAnalysis>,
    last_analysis: Option<DateTime<Utc>>,
    cache_duration: Duration,
}

impl HistoryCache {
    pub fn get_or_analyze(&mut self, shell_type: ShellType) -> Result<PatternAnalysis, CacheError> {
        let cache_key = format!("{:?}", shell_type);
        
        // Check if we have a recent analysis
        if let Some(cached) = self.pattern_cache.get(&cache_key) {
            if let Some(last_analysis) = self.last_analysis {
                if Utc::now().signed_duration_since(last_analysis) < self.cache_duration {
                    return Ok(cached.clone());
                }
            }
        }
        
        // Perform new analysis
        let parser = ShellHistoryParser::new();
        let history = parser.parse_history(shell_type)?;
        let analyzer = PatternAnalyzer::new();
        let patterns = analyzer.analyze_history(&history);
        
        // Cache the results
        self.pattern_cache.put(cache_key, patterns.clone());
        self.last_analysis = Some(Utc::now());
        
        Ok(patterns)
    }
}
```

### Incremental Updates
```rust
pub struct IncrementalAnalyzer {
    last_processed_timestamp: Option<DateTime<Utc>>,
    accumulated_patterns: PatternAnalysis,
}

impl IncrementalAnalyzer {
    pub fn update_patterns(&mut self, new_entries: &[HistoryEntry]) -> Result<(), AnalysisError> {
        // Process only new entries since last update
        let new_entries: Vec<_> = new_entries.iter()
            .filter(|entry| {
                if let Some(last_timestamp) = self.last_processed_timestamp {
                    entry.timestamp.map_or(true, |ts| ts > last_timestamp)
                } else {
                    true
                }
            })
            .collect();
        
        if !new_entries.is_empty() {
            let incremental_patterns = self.analyze_incremental(&new_entries)?;
            self.merge_patterns(incremental_patterns);
            
            // Update last processed timestamp
            if let Some(latest_entry) = new_entries.iter()
                .filter_map(|entry| entry.timestamp)
                .max() {
                self.last_processed_timestamp = Some(latest_entry);
            }
        }
        
        Ok(())
    }
}
```

## Configuration and Setup

### Configuration Interface
```rust
// src/config/history.rs
pub struct HistoryConfig {
    pub enabled: bool,
    pub shells_to_analyze: Vec<ShellType>,
    pub max_history_entries: u32,
    pub update_frequency: UpdateFrequency,
    pub privacy_level: PrivacyLevel,
    pub cache_duration_hours: u32,
}

pub enum UpdateFrequency {
    OnStartup,      // Analyze history on cmdai startup
    Daily,          // Check for updates daily
    Weekly,         // Check for updates weekly
    Manual,         // Only when explicitly requested
}

pub enum PrivacyLevel {
    Strict,         // Maximum filtering, minimal data retention
    Moderate,       // Balance between privacy and functionality
    Permissive,     // Minimal filtering for maximum accuracy
}
```

### Setup Wizard
```rust
pub struct HistorySetupWizard;

impl HistorySetupWizard {
    pub fn run_setup() -> Result<HistoryConfig, SetupError> {
        println!("🔧 Shell History Integration Setup");
        println!();
        
        // Detect available shells
        let detector = ShellDetector::new();
        let available_shells = detector.detect_installed_shells();
        
        println!("Detected shells:");
        for shell in &available_shells {
            println!("  ✓ {} ({})", shell.name, shell.history_file.display());
        }
        println!();
        
        // Ask which shells to analyze
        let selected_shells = self.select_shells_to_analyze(&available_shells)?;
        
        // Configure privacy level
        let privacy_level = self.select_privacy_level()?;
        
        // Configure update frequency
        let update_frequency = self.select_update_frequency()?;
        
        Ok(HistoryConfig {
            enabled: true,
            shells_to_analyze: selected_shells,
            max_history_entries: 10000,
            update_frequency,
            privacy_level,
            cache_duration_hours: 24,
        })
    }
}
```

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bash_history_parsing() {
        let sample_history = r#"
ls -la
cd /home/user/projects
git status
git add .
git commit -m "test commit"
"#;
        
        let parser = BashHistoryParser;
        let entries = parser.parse_content(sample_history).unwrap();
        
        assert_eq!(entries.len(), 5);
        assert_eq!(entries[0].command, "ls -la");
        assert_eq!(entries[4].command, r#"git commit -m "test commit""#);
    }
    
    #[test]
    fn test_privacy_filtering() {
        let filter = PrivacyFilter::new();
        let sensitive_commands = vec![
            "ssh user@server -i ~/.ssh/private_key",
            "curl -u user:password http://api.example.com",
            "export SECRET_TOKEN=abc123",
        ];
        
        for cmd in sensitive_commands {
            let entry = HistoryEntry {
                command: cmd.to_string(),
                timestamp: None,
                working_directory: None,
                exit_code: None,
                session_id: None,
                duration: None,
            };
            
            assert!(filter.filter_entry(entry).is_none());
        }
    }
    
    #[test]
    fn test_pattern_recognition() {
        let history = create_test_history();
        let analyzer = PatternAnalyzer::new();
        let patterns = analyzer.analyze_history(&history);
        
        // Verify common patterns are detected
        assert!(patterns.command_preferences.preferred_flags.contains_key("ls"));
        assert!(patterns.common_sequences.len() > 0);
    }
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_enhanced_generation() {
    let mut generator = HistoryEnhancedGenerator::new_for_testing();
    
    // Load test history patterns
    let test_patterns = create_test_patterns();
    generator.load_patterns(test_patterns);
    
    let request = CommandRequest {
        input: "list files".to_string(),
        context: Some(CommandContext {
            working_directory: Some("/home/user/projects".into()),
            shell: ShellType::Bash,
        }),
        ..Default::default()
    };
    
    let result = generator.generate_command(&request).await.unwrap();
    
    // Should prefer user's typical flags
    assert!(result.command.contains("-la")); // User typically uses -la
    assert!(result.alternatives.len() > 0);
}
```

## Rollout Plan

### Phase 1: Core Infrastructure (3 weeks)
- History parser implementation for major shells
- Privacy filtering system
- Basic pattern recognition
- Configuration interface

### Phase 2: Pattern Analysis (2 weeks)
- Advanced pattern detection algorithms
- Context analysis implementation
- Preference detection system
- Caching and performance optimization

### Phase 3: Generation Integration (2 weeks)
- Enhanced command generator implementation
- Context-aware suggestion system
- Alternative generation based on patterns
- Testing and validation

### Phase 4: User Experience (1 week)
- Setup wizard implementation
- Configuration interface
- Privacy controls
- Documentation and help system

This shell history integration provides cmdai with the context and user preference awareness needed to generate commands that truly match how users actually work with their shell, while maintaining strict privacy protection and user control.