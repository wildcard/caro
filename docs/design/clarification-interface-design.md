# Clarification Interface Design

**Technical Design Document**

## Overview

This document describes the reasoning-based prompt improvement interface for caro. When caro cannot confidently generate a command, it should use structured reasoning to understand WHY the query is ambiguous and help the user refine their request.

## Current Behavior (Problem)

```
$ caro how to reload powershell profile?
Command:
  echo 'Please clarify your request'

Execute this command?:
  Yes - execute
> No - skip
  Edit - modify in shell
```

**Issues:**
1. No explanation of what's unclear
2. No guidance on how to improve the query
3. Unhelpful fallback command
4. Dead-end user experience

## Proposed Behavior (Solution)

### Scenario 1: Platform-Detected Query

When the query explicitly mentions a platform:

```
$ caro how to reload powershell profile?

┌─ Command Generated ────────────────────────────────────────────────────┐
│                                                                         │
│  Detected: PowerShell environment                                      │
│                                                                         │
│  Command: . $PROFILE                                                    │
│                                                                         │
│  Reloads your PowerShell profile configuration in the current session. │
│                                                                         │
│  Alternatives for other shells:                                         │
│    bash  → source ~/.bashrc                                            │
│    zsh   → source ~/.zshrc                                             │
│    fish  → source ~/.config/fish/config.fish                           │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘

Execute? [Y]es / [n]o / [e]dit:
```

### Scenario 2: Ambiguous Query with Clarification

When the query is ambiguous and needs clarification:

```
$ caro clean up my disk

┌─ Clarification Needed ─────────────────────────────────────────────────┐
│                                                                         │
│  I can help you clean up disk space, but I need more context:          │
│                                                                         │
│  ┌───────────────────────────────────────────────────────────────────┐ │
│  │ 1. What type of cleanup?                                          │ │
│  │    [a] Find large files (for review)                              │ │
│  │    [b] Delete temp/cache files                                    │ │
│  │    [c] Find duplicate files                                       │ │
│  │    [d] Show disk usage breakdown                                  │ │
│  └───────────────────────────────────────────────────────────────────┘ │
│                                                                         │
│  ┌───────────────────────────────────────────────────────────────────┐ │
│  │ 2. Where should I look?                                           │ │
│  │    [a] Current directory (.)                                      │ │
│  │    [b] Home directory (~)                                         │ │
│  │    [c] Entire system (may require sudo)                           │ │
│  └───────────────────────────────────────────────────────────────────┘ │
│                                                                         │
│  Enter choices (e.g., "1a 2b") or [s]kip:                              │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘

> 1a 2b

Command: find ~ -type f -size +100M -exec ls -lh {} \;
Finds large files (>100MB) in your home directory.

Execute? [Y]es / [n]o / [e]dit:
```

### Scenario 3: Outside Expertise

When the request is outside caro's domain:

```
$ caro write a python script to parse JSON

┌─ Outside My Expertise ─────────────────────────────────────────────────┐
│                                                                         │
│  This request involves writing code, which is beyond my scope.         │
│                                                                         │
│  I can help with related shell tasks:                                   │
│    [1] Parse JSON with jq                                              │
│    [2] Pretty-print a JSON file                                        │
│    [3] Extract a field from JSON                                       │
│                                                                         │
│  For Python scripting, try:                                             │
│    • GitHub Copilot                                                     │
│    • ChatGPT                                                            │
│    • Your IDE's AI assistant                                            │
│                                                                         │
│  Would you like help with one of the alternatives? [1/2/3/n]:          │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### Scenario 4: Reasoning Mode (Verbose)

When the user wants to understand caro's thinking:

```
$ caro --verbose "deploy my app"

┌─ Reasoning ────────────────────────────────────────────────────────────┐
│                                                                         │
│  Analysis:                                                              │
│    Understood: User wants to deploy an application                     │
│    Confidence: 0.25 (low)                                              │
│                                                                         │
│  Ambiguity factors:                                                     │
│    • Deployment target unknown (local, cloud, container?)              │
│    • Application type unspecified (web, CLI, service?)                 │
│    • Deployment method unclear (docker, k8s, rsync?)                   │
│                                                                         │
│  Ambiguity type: missing_context                                        │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘

┌─ Clarification Needed ─────────────────────────────────────────────────┐
│                                                                         │
│  [Questions would appear here...]                                       │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

## Data Structures

### AmbiguityAnalysis

```rust
pub struct AmbiguityAnalysis {
    /// Overall confidence score (0.0 - 1.0)
    pub confidence: f64,

    /// Type of ambiguity detected
    pub ambiguity_type: AmbiguityType,

    /// What we understood from the query
    pub understood: String,

    /// What remains unclear
    pub unclear_aspects: Vec<String>,

    /// Platform hints detected in query
    pub platform_hints: Vec<PlatformHint>,

    /// Generated clarification questions
    pub questions: Vec<ClarificationQuestion>,

    /// Partial answers if we can infer some
    pub partial_answers: HashMap<String, String>,
}

pub enum AmbiguityType {
    /// Query could apply to multiple shells/platforms
    PlatformAmbiguous,
    /// Unclear target files/directories/scope
    ScopeAmbiguous,
    /// Multiple possible operations
    ActionAmbiguous,
    /// Need additional parameters
    MissingContext,
    /// Outside caro's expertise
    DomainUnknown,
    /// Potentially destructive operation
    SafetyConcern,
    /// High confidence, no clarification needed
    Clear,
}
```

### ClarificationQuestion

```rust
pub struct ClarificationQuestion {
    /// Unique identifier for this question
    pub id: String,

    /// Human-readable question text
    pub question: String,

    /// Available options
    pub options: Vec<QuestionOption>,

    /// Allow freeform text input
    pub allow_freeform: bool,

    /// Detected hint from the query (if any)
    pub detected_hint: Option<String>,
}

pub struct QuestionOption {
    /// Single character key (a, b, c, d)
    pub key: char,

    /// Display label
    pub label: String,

    /// Description of what this option does
    pub description: Option<String>,

    /// What this option maps to in the enhanced query
    pub maps_to: String,
}
```

### ClarificationResponse

```rust
pub struct ClarificationResponse {
    /// Questions that were answered
    pub answers: Vec<Answer>,

    /// Enhanced query after applying answers
    pub enhanced_query: String,

    /// Platform override if detected
    pub platform_override: Option<ShellType>,
}

pub struct Answer {
    /// Question ID
    pub question_id: String,

    /// Selected option (or freeform text)
    pub selection: AnswerSelection,
}

pub enum AnswerSelection {
    /// User selected an option
    Option(char),
    /// User provided freeform text
    Freeform(String),
    /// User skipped this question
    Skipped,
}
```

## UI Components

### Terminal Renderer

```rust
pub struct ClarificationRenderer {
    /// Terminal width for box drawing
    width: usize,

    /// Color support level
    colors: ColorSupport,
}

impl ClarificationRenderer {
    /// Render a clarification question box
    pub fn render_questions(&self, analysis: &AmbiguityAnalysis) -> String {
        let mut output = String::new();

        // Draw header
        output.push_str(&self.draw_header("Clarification Needed"));

        // Draw explanation
        output.push_str(&self.wrap_text(&analysis.explanation()));
        output.push('\n');

        // Draw each question
        for (i, question) in analysis.questions.iter().enumerate() {
            output.push_str(&self.draw_question(i + 1, question));
        }

        // Draw input prompt
        output.push_str(&self.draw_input_prompt());

        output
    }

    /// Render a detected platform note
    pub fn render_platform_detected(&self, platform: &str, command: &str) -> String {
        // ... render platform-detected box
    }

    /// Render an outside-expertise message
    pub fn render_outside_expertise(&self, alternatives: &[Alternative]) -> String {
        // ... render alternatives box
    }
}
```

### Input Handler

```rust
pub struct ClarificationInputHandler {
    /// Questions being answered
    questions: Vec<ClarificationQuestion>,
}

impl ClarificationInputHandler {
    /// Parse user input like "1a 2b" or "1a,2b" or "a b"
    pub fn parse_input(&self, input: &str) -> Result<Vec<Answer>, InputError> {
        // Handle formats:
        // "1a 2b" - numbered question + option
        // "a b" - just options (in order)
        // "1a, 2b" - with comma separation
        // "s" or "skip" - skip clarification

        let input = input.trim().to_lowercase();

        if input == "s" || input == "skip" {
            return Ok(vec![]);  // Skip clarification
        }

        // Parse input tokens
        let tokens: Vec<&str> = input.split(|c: char| c.is_whitespace() || c == ',')
            .filter(|s| !s.is_empty())
            .collect();

        let mut answers = Vec::new();

        for token in tokens {
            let answer = self.parse_token(token)?;
            answers.push(answer);
        }

        Ok(answers)
    }

    fn parse_token(&self, token: &str) -> Result<Answer, InputError> {
        // Parse "1a" format (question number + option)
        if let Some(caps) = Regex::new(r"^(\d)([a-d])$").unwrap().captures(token) {
            let q_num: usize = caps[1].parse().unwrap();
            let option = caps[2].chars().next().unwrap();

            if q_num < 1 || q_num > self.questions.len() {
                return Err(InputError::InvalidQuestionNumber(q_num));
            }

            let question = &self.questions[q_num - 1];
            if !question.options.iter().any(|o| o.key == option) {
                return Err(InputError::InvalidOption(option));
            }

            return Ok(Answer {
                question_id: question.id.clone(),
                selection: AnswerSelection::Option(option),
            });
        }

        // Parse single letter (option only, assume order)
        if token.len() == 1 {
            let option = token.chars().next().unwrap();
            // ... handle single option
        }

        Err(InputError::InvalidFormat(token.to_string()))
    }
}
```

## Integration Points

### Agent Loop Integration

The clarification system integrates into the existing agent loop:

```rust
// In src/agent/mod.rs

pub async fn generate_command(&self, request: CommandRequest) -> Result<GeneratedCommand, GeneratorError> {
    // 1. Try static matcher first
    if let Ok(cmd) = self.static_matcher.generate_command(&request).await {
        return Ok(cmd);
    }

    // 2. Try LLM generation
    let result = self.backend.generate_command(&request).await?;

    // 3. Check confidence and potentially clarify
    if result.confidence_score < self.config.clarification_threshold {
        // Analyze ambiguity
        let analysis = self.ambiguity_analyzer.analyze(&request.input).await?;

        // If clarification questions available, enter clarification flow
        if !analysis.questions.is_empty() {
            return self.handle_clarification(request, analysis).await;
        }
    }

    Ok(result)
}

async fn handle_clarification(
    &self,
    original_request: CommandRequest,
    analysis: AmbiguityAnalysis,
) -> Result<GeneratedCommand, GeneratorError> {
    // Render clarification UI
    let renderer = ClarificationRenderer::new();
    let ui = renderer.render_questions(&analysis);

    // Get user input
    println!("{}", ui);
    let input = self.read_user_input()?;

    // Parse answers
    let handler = ClarificationInputHandler::new(&analysis.questions);
    let answers = handler.parse_input(&input)?;

    // If user skipped, return best guess
    if answers.is_empty() {
        return self.generate_best_guess(&original_request).await;
    }

    // Enhance query with answers
    let enhancer = QueryEnhancer::new();
    let enhanced = enhancer.enhance(&original_request.input, &answers);

    // Retry generation with enhanced query
    let enhanced_request = CommandRequest::new(&enhanced.enhanced_query, original_request.shell);
    self.backend.generate_command(&enhanced_request).await
}
```

### LLM Prompt Integration

The LLM prompt includes instructions for reasoning output:

```
You are a shell command assistant. Analyze the user's query and respond with structured JSON.

OUTPUT FORMAT:
{
  "understood": true|false,
  "confidence": 0.0-1.0,
  "reasoning": {
    "what_understood": "Brief summary of interpreted intent",
    "unclear_aspects": ["List of ambiguous aspects"],
    "ambiguity_type": "platform|scope|action|context|domain|safety|clear"
  },
  "command": "the_command_if_understood",
  "clarification_questions": [
    {
      "id": "unique_id",
      "question": "Human-readable question",
      "options": [
        {"key": "a", "label": "Option A", "maps_to": "command_variant_a"},
        {"key": "b", "label": "Option B", "maps_to": "command_variant_b"}
      ]
    }
  ],
  "platform_detected": "powershell|bash|zsh|fish|posix|unknown"
}

RULES:
1. If query explicitly mentions a platform (e.g., "powershell"), set high confidence
2. Generate clarification questions ONLY for truly ambiguous cases
3. For known single-answer queries, just provide the command
4. Always explain reasoning in the "reasoning" field

User Query: {{user_input}}
```

## Question Templates

Pre-defined templates for common ambiguity types:

### Platform Questions

```yaml
# templates/questions/platform.yaml
id: shell_type
question: "Which shell are you using?"
options:
  - key: a
    label: PowerShell
    description: Windows PowerShell or PowerShell Core
  - key: b
    label: bash
    description: Bourne Again Shell (Linux/macOS)
  - key: c
    label: zsh
    description: Z Shell (macOS default)
  - key: d
    label: fish
    description: Friendly Interactive Shell
allow_freeform: false
```

### Scope Questions

```yaml
# templates/questions/scope.yaml
id: target_location
question: "Where should I look?"
options:
  - key: a
    label: Current directory
    description: The directory you're in (.)
  - key: b
    label: Home directory
    description: Your home folder (~)
  - key: c
    label: Specific path
    description: Enter a custom path
allow_freeform: true
```

### Action Questions

```yaml
# templates/questions/action_cleanup.yaml
id: cleanup_type
question: "What kind of cleanup?"
options:
  - key: a
    label: Find large files
    description: List files over 100MB for review
  - key: b
    label: Delete temp files
    description: Remove temporary and cache files
  - key: c
    label: Find duplicates
    description: Identify duplicate files
  - key: d
    label: Usage breakdown
    description: Show disk usage by folder
allow_freeform: false
```

## Configuration

Users can configure clarification behavior:

```toml
# ~/.config/caro/config.toml

[clarification]
# Enable/disable clarification prompts
enabled = true

# Confidence threshold for triggering clarification (0.0-1.0)
threshold = 0.7

# Maximum clarification rounds before giving up
max_rounds = 2

# Show reasoning output by default
verbose = false

[preferences]
# Default shell for ambiguous queries
default_shell = "bash"  # or "powershell", "zsh", "fish"
```

## Error Handling

```rust
pub enum ClarificationError {
    /// User skipped clarification
    Skipped,

    /// User provided invalid input
    InvalidInput(InputError),

    /// Maximum clarification rounds exceeded
    MaxRoundsExceeded,

    /// Query still ambiguous after clarification
    StillAmbiguous,

    /// Outside caro's domain
    OutsideDomain(String),
}

impl ClarificationError {
    /// Generate a helpful error message
    pub fn user_message(&self) -> String {
        match self {
            Self::Skipped => "Clarification skipped. Using best guess.".to_string(),
            Self::InvalidInput(e) => format!("Invalid input: {}. Try again.", e),
            Self::MaxRoundsExceeded => "Couldn't understand after 2 rounds. Try rephrasing.".to_string(),
            Self::StillAmbiguous => "Query still unclear. Please be more specific.".to_string(),
            Self::OutsideDomain(domain) => format!("{} is outside my expertise.", domain),
        }
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
    fn test_parse_input_numbered() {
        let questions = vec![
            ClarificationQuestion { id: "q1".to_string(), /* ... */ },
            ClarificationQuestion { id: "q2".to_string(), /* ... */ },
        ];

        let handler = ClarificationInputHandler::new(&questions);

        let answers = handler.parse_input("1a 2b").unwrap();
        assert_eq!(answers.len(), 2);
        assert_eq!(answers[0].question_id, "q1");
        assert_eq!(answers[1].question_id, "q2");
    }

    #[test]
    fn test_parse_input_skip() {
        let handler = ClarificationInputHandler::new(&[]);

        let answers = handler.parse_input("skip").unwrap();
        assert!(answers.is_empty());
    }

    #[test]
    fn test_platform_detection() {
        let analyzer = AmbiguityAnalyzer::new();

        let analysis = analyzer.analyze("reload powershell profile").await.unwrap();

        assert!(analysis.platform_hints.contains(&PlatformHint::PowerShell));
        assert!(analysis.confidence > 0.8);
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_clarification_flow_end_to_end() {
    let agent = Agent::new_with_mock_backend();

    // Simulate ambiguous query
    let request = CommandRequest::new("clean up disk", ShellType::Bash);

    // Should trigger clarification
    let result = agent.analyze_for_clarification(&request).await;
    assert!(result.is_clarification_needed());

    // Provide answers
    let answers = vec![
        Answer { question_id: "cleanup_type".to_string(), selection: AnswerSelection::Option('a') },
        Answer { question_id: "target_location".to_string(), selection: AnswerSelection::Option('b') },
    ];

    // Enhanced query should generate command
    let enhanced = agent.enhance_query(&request, &answers).await;
    let result = agent.generate_command(&enhanced).await;

    assert!(result.is_ok());
    assert!(result.unwrap().command.contains("find ~"));
}
```

## Migration Path

1. **Phase 1**: Add static patterns for PowerShell and shell profiles (done in this PR)
2. **Phase 2**: Implement basic clarification UI without LLM reasoning
3. **Phase 3**: Add LLM-based reasoning output and dynamic questions
4. **Phase 4**: Add configuration and verbose mode
5. **Phase 5**: Implement query enhancement and retry logic

---

*This design document was created in January 2026 for caro v1.3.0.*
