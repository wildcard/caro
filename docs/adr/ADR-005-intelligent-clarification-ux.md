# ADR-005: Intelligent Clarification UX

| **Status**     | Proposed                            |
|----------------|-------------------------------------|
| **Date**       | January 2026                        |
| **Authors**    | Caro Maintainers                    |
| **Supersedes** | N/A                                 |
| **Relates To** | ADR-004 (Pre-Processing Pipeline), Spec 006 (Intelligent Prompt Generation) |

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Context and Problem Statement](#context-and-problem-statement)
3. [Decision Drivers](#decision-drivers)
4. [Proposed Solution](#proposed-solution)
5. [Architecture](#architecture)
6. [Implementation Details](#implementation-details)
7. [User Experience Design](#user-experience-design)
8. [Consequences](#consequences)
9. [Alternatives Considered](#alternatives-considered)

---

## Executive Summary

This ADR defines the architecture for an **intelligent clarification system** that transforms the current unhelpful "Please clarify your request" response into a collaborative, reasoning-based query improvement experience. Instead of dead-ending users with vague error messages, caro will use structured reasoning to understand ambiguity, generate targeted questions, and help users refine their queries to get accurate commands.

**Core Principle**: When caro doesn't understand, it should say **why** and offer **specific help** to improve the query.

---

## Context and Problem Statement

### The Problem

When a user submits a query that caro cannot confidently translate to a shell command, the current behavior is:

```
PS C:\Users\User> caro how to reload powershell profile?
Command:
  echo 'Please clarify your request'

Execute this command?:
  Yes - execute
> No - skip
  Edit - modify in shell
```

This is a poor user experience because:

1. **Unhelpful**: "Please clarify" doesn't tell the user *what* to clarify or *why* the request was unclear
2. **Dead-end**: No guidance on how to rephrase the query
3. **Wastes time**: User must guess what went wrong and retry blindly
4. **Platform blindness**: Doesn't recognize this is a PowerShell-specific query (vs POSIX)
5. **No learning**: System doesn't improve from these interactions

### Real Example Analysis

The query "how to reload powershell profile?" fails because:
- **Platform mismatch**: Caro is POSIX-focused, PowerShell is a different paradigm
- **Intent unclear to model**: "reload profile" could mean several things
- **No static pattern**: PowerShell commands aren't in the pattern library
- **LLM fallback**: Model defaults to clarification when uncertain

The **correct answer** for PowerShell is: `. $PROFILE`

But caro can't reach this answer because:
1. No PowerShell-specific knowledge in current prompts
2. No clarification mechanism to ask "Did you mean PowerShell?" vs "bash/zsh profile?"
3. No reasoning chain to identify the ambiguity type

---

## Decision Drivers

### Primary Drivers

1. **User Success**: Every interaction should move toward a successful command
2. **Transparency**: Users should understand why caro is uncertain
3. **Actionability**: Clarification requests must be specific and answerable
4. **Learning**: System should improve from clarification interactions
5. **Efficiency**: Minimize round-trips to successful command

### Secondary Drivers

- Cross-platform awareness (POSIX vs PowerShell vs CMD)
- Graceful degradation when clarification isn't possible
- Consistent UX across all backends
- Integration with existing safety validation

---

## Proposed Solution

### Core Concept: Reasoning-Based Clarification

Instead of a binary "understood/not understood" model, implement a **reasoning chain** that:

1. **Analyzes** why the query is ambiguous
2. **Categorizes** the ambiguity type
3. **Generates** targeted clarification questions
4. **Enhances** the query with answers
5. **Retries** generation with improved context

### Ambiguity Categories

| Category | Description | Example Query | Clarification Strategy |
|----------|-------------|---------------|----------------------|
| **Platform Ambiguous** | Could apply to multiple shells/OS | "reload profile" | Ask which shell (PowerShell/bash/zsh) |
| **Scope Ambiguous** | Unclear target files/directories | "delete old files" | Ask location, age threshold, confirmation |
| **Action Ambiguous** | Multiple possible operations | "clean up disk" | Ask specific cleanup type |
| **Missing Context** | Needs additional parameters | "connect to server" | Ask server address, protocol |
| **Domain Unknown** | Outside caro's knowledge | "deploy to kubernetes" | Explain limitations, suggest alternatives |
| **Safety Concern** | Potentially destructive | "remove everything" | Confirm intent, explain risks |

### Reasoning Output Format

```json
{
  "understood": false,
  "confidence": 0.3,
  "reasoning": {
    "what_i_understood": "User wants to reload some kind of profile configuration",
    "what_is_unclear": [
      "Which shell environment (PowerShell, bash, zsh, fish)?",
      "Current profile or all profiles?",
      "Reload in current session or restart shell?"
    ],
    "ambiguity_type": "platform_ambiguous",
    "knowledge_gap": "PowerShell-specific commands not in training data"
  },
  "clarification_questions": [
    {
      "id": "shell_type",
      "question": "Which shell are you using?",
      "options": ["PowerShell", "bash", "zsh", "fish"],
      "detected_hint": "Query mentions 'powershell' - likely PowerShell"
    }
  ],
  "partial_answer": {
    "if_powershell": ". $PROFILE",
    "if_bash": "source ~/.bashrc",
    "if_zsh": "source ~/.zshrc"
  }
}
```

---

## Architecture

### System Flow

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                          INTELLIGENT CLARIFICATION FLOW                        │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                                │
│  User Query: "how to reload powershell profile?"                              │
│       │                                                                        │
│       ▼                                                                        │
│  ┌─────────────────────────────────────────────────────────────────────────┐  │
│  │                    STAGE 1: INITIAL ANALYSIS                            │  │
│  │                                                                          │  │
│  │  ┌──────────────────┐  ┌──────────────────┐  ┌───────────────────────┐  │  │
│  │  │ Static Matcher   │  │ Platform Detector│  │ Keyword Extractor     │  │  │
│  │  │ Result: NO_MATCH │  │ Hint: PowerShell │  │ ["reload", "profile"] │  │  │
│  │  └──────────────────┘  └──────────────────┘  └───────────────────────┘  │  │
│  │                                                                          │  │
│  └─────────────────────────────────────────────────────────────────────────┘  │
│       │                                                                        │
│       ▼                                                                        │
│  ┌─────────────────────────────────────────────────────────────────────────┐  │
│  │                    STAGE 2: LLM REASONING                               │  │
│  │                                                                          │  │
│  │  System Prompt: "Analyze this query. If unclear, explain WHY and        │  │
│  │                  generate specific clarification questions."             │  │
│  │                                                                          │  │
│  │  Reasoning Output:                                                       │  │
│  │  • Platform hint detected: "powershell" in query                        │  │
│  │  • Intent: reload shell configuration                                    │  │
│  │  • Confidence: 0.85 (high due to explicit platform mention)             │  │
│  │  • Suggested command: `. $PROFILE` (PowerShell)                         │  │
│  │                                                                          │  │
│  └─────────────────────────────────────────────────────────────────────────┘  │
│       │                                                                        │
│       ▼                                                                        │
│  ┌─────────────────────────────────────────────────────────────────────────┐  │
│  │                    STAGE 3: CONFIDENCE GATE                             │  │
│  │                                                                          │  │
│  │  If confidence >= 0.7:                                                   │  │
│  │    → Generate command with platform note                                 │  │
│  │                                                                          │  │
│  │  If confidence < 0.7:                                                    │  │
│  │    → Enter clarification flow                                            │  │
│  │                                                                          │  │
│  └─────────────────────────────────────────────────────────────────────────┘  │
│       │                                                                        │
│       ▼ (In this case: HIGH confidence due to explicit "powershell")          │
│  ┌─────────────────────────────────────────────────────────────────────────┐  │
│  │                    OUTPUT: COMMAND WITH CONTEXT                          │  │
│  │                                                                          │  │
│  │  Command: . $PROFILE                                                     │  │
│  │  Shell: PowerShell                                                       │  │
│  │  Explanation: Reloads PowerShell profile in current session              │  │
│  │  Note: Detected "powershell" in query, using PowerShell syntax           │  │
│  │                                                                          │  │
│  └─────────────────────────────────────────────────────────────────────────┘  │
│                                                                                │
└──────────────────────────────────────────────────────────────────────────────┘
```

### Clarification Flow (Low Confidence Path)

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                          CLARIFICATION FLOW (Confidence < 0.7)                │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                                │
│  User Query: "clean up my disk"                                               │
│       │                                                                        │
│       ▼                                                                        │
│  ┌─────────────────────────────────────────────────────────────────────────┐  │
│  │  Reasoning: Ambiguous - multiple interpretations possible                │  │
│  │  • Could mean: delete temp files, find large files, empty trash, etc.   │  │
│  │  • Confidence: 0.35                                                      │  │
│  └─────────────────────────────────────────────────────────────────────────┘  │
│       │                                                                        │
│       ▼                                                                        │
│  ┌─────────────────────────────────────────────────────────────────────────┐  │
│  │                    CLARIFICATION UI                                      │  │
│  │                                                                          │  │
│  │  I need a bit more context to help you clean up disk space.             │  │
│  │                                                                          │  │
│  │  1. What type of cleanup?                                               │  │
│  │     [a] Find large files (to review and delete manually)                │  │
│  │     [b] Delete temp/cache files                                         │  │
│  │     [c] Find and remove duplicate files                                 │  │
│  │     [d] Show disk usage breakdown                                       │  │
│  │                                                                          │  │
│  │  2. Which location?                                                      │  │
│  │     [a] Current directory                                               │  │
│  │     [b] Home directory (~)                                              │  │
│  │     [c] System-wide (may require sudo)                                  │  │
│  │                                                                          │  │
│  │  Your choice (e.g., "1a 2b"): _                                         │  │
│  │                                                                          │  │
│  └─────────────────────────────────────────────────────────────────────────┘  │
│       │                                                                        │
│       ▼ (User enters: "1a 2b")                                                │
│  ┌─────────────────────────────────────────────────────────────────────────┐  │
│  │  Enhanced Query: "Find large files in home directory for review"        │  │
│  │                                                                          │  │
│  │  Generated Command: find ~ -type f -size +100M -exec ls -lh {} \;       │  │
│  │  Confidence: 0.95                                                        │  │
│  └─────────────────────────────────────────────────────────────────────────┘  │
│                                                                                │
└──────────────────────────────────────────────────────────────────────────────┘
```

### Component Design

#### 1. Ambiguity Analyzer

```rust
pub struct AmbiguityAnalyzer {
    platform_detector: PlatformDetector,
    keyword_extractor: KeywordExtractor,
    domain_classifier: DomainClassifier,
}

impl AmbiguityAnalyzer {
    /// Analyze a query and determine ambiguity type and questions
    pub async fn analyze(&self, query: &str) -> AmbiguityAnalysis {
        let platform_hints = self.platform_detector.detect_hints(query);
        let keywords = self.keyword_extractor.extract(query);
        let domain = self.domain_classifier.classify(query);

        AmbiguityAnalysis {
            platform_hints,
            keywords,
            domain,
            ambiguity_type: self.determine_ambiguity_type(&platform_hints, &keywords, &domain),
            suggested_questions: self.generate_questions(&platform_hints, &keywords, &domain),
        }
    }
}

pub struct AmbiguityAnalysis {
    pub platform_hints: Vec<PlatformHint>,
    pub keywords: Vec<String>,
    pub domain: CommandDomain,
    pub ambiguity_type: AmbiguityType,
    pub suggested_questions: Vec<ClarificationQuestion>,
}

pub enum AmbiguityType {
    PlatformAmbiguous,      // Which shell/OS?
    ScopeAmbiguous,         // Which files/directories?
    ActionAmbiguous,        // What operation exactly?
    MissingContext,         // Need more parameters
    DomainUnknown,          // Outside caro's expertise
    SafetyConcern,          // Potentially destructive
    HighConfidence,         // No clarification needed
}
```

#### 2. Clarification Question Generator

```rust
pub struct ClarificationQuestion {
    pub id: String,
    pub question: String,
    pub options: Vec<QuestionOption>,
    pub allow_freeform: bool,
    pub detected_hint: Option<String>,
}

pub struct QuestionOption {
    pub key: char,           // 'a', 'b', 'c', etc.
    pub label: String,       // "PowerShell"
    pub description: String, // "Windows command shell"
    pub maps_to: String,     // ". $PROFILE"
}

impl ClarificationQuestionGenerator {
    pub fn generate_for_platform_ambiguity(&self, hints: &[PlatformHint]) -> Vec<ClarificationQuestion> {
        vec![ClarificationQuestion {
            id: "shell_type".to_string(),
            question: "Which shell are you using?".to_string(),
            options: vec![
                QuestionOption {
                    key: 'a',
                    label: "PowerShell".to_string(),
                    description: "Windows PowerShell or PowerShell Core".to_string(),
                    maps_to: ". $PROFILE".to_string(),
                },
                QuestionOption {
                    key: 'b',
                    label: "bash".to_string(),
                    description: "Bourne Again Shell (Linux/macOS)".to_string(),
                    maps_to: "source ~/.bashrc".to_string(),
                },
                QuestionOption {
                    key: 'c',
                    label: "zsh".to_string(),
                    description: "Z Shell (macOS default)".to_string(),
                    maps_to: "source ~/.zshrc".to_string(),
                },
            ],
            allow_freeform: false,
            detected_hint: hints.first().map(|h| format!("Detected '{}' in your query", h.keyword)),
        }]
    }
}
```

#### 3. Query Enhancer

```rust
pub struct QueryEnhancer {
    template_engine: TemplateEngine,
}

impl QueryEnhancer {
    /// Enhance the original query with clarification answers
    pub fn enhance(&self, original: &str, answers: &[Answer]) -> EnhancedQuery {
        let context = answers.iter()
            .map(|a| format!("{}: {}", a.question_id, a.selected_option.label))
            .collect::<Vec<_>>()
            .join(", ");

        EnhancedQuery {
            original: original.to_string(),
            enhanced: format!("{} [Context: {}]", original, context),
            platform_override: answers.iter()
                .find(|a| a.question_id == "shell_type")
                .map(|a| a.selected_option.maps_to.clone()),
        }
    }
}
```

---

## User Experience Design

### Terminal UI for Clarification

```
$ caro clean up disk space

┌─ Clarification Needed ─────────────────────────────────────────────────┐
│                                                                         │
│  I can help you clean up disk space, but I need to know more:          │
│                                                                         │
│  ❓ What type of cleanup?                                               │
│     [1] Find large files (for manual review)                           │
│     [2] Delete temporary/cache files                                    │
│     [3] Find duplicate files                                            │
│     [4] Show disk usage breakdown                                       │
│                                                                         │
│  ❓ Which location?                                                     │
│     [a] Current directory (.)                                          │
│     [b] Home directory (~)                                              │
│     [c] System-wide (requires sudo)                                     │
│                                                                         │
│  💡 Tip: Enter choices like "1b" or press Enter for default [1a]       │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘

Your choice: 1b

✓ Generated command for: "Find large files in home directory"

Command: find ~ -type f -size +100M -exec ls -lh {} \;
```

### PowerShell-Specific Example

```
$ caro how to reload powershell profile

┌─ Quick Answer ─────────────────────────────────────────────────────────┐
│                                                                         │
│  💡 Detected "powershell" in your query                                │
│                                                                         │
│  Command: . $PROFILE                                                    │
│                                                                         │
│  This reloads your PowerShell profile in the current session.          │
│                                                                         │
│  📋 Alternative for other shells:                                       │
│     • bash:  source ~/.bashrc                                          │
│     • zsh:   source ~/.zshrc                                           │
│     • fish:  source ~/.config/fish/config.fish                         │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘

Execute this command? [Y/n/e]:
```

### Error Recovery Flow

```
$ caro deploy to kubernetes

┌─ Outside My Expertise ─────────────────────────────────────────────────┐
│                                                                         │
│  ⚠️  This request involves Kubernetes deployment, which requires        │
│     complex orchestration beyond single shell commands.                 │
│                                                                         │
│  I can help with related tasks:                                         │
│     [1] Check if kubectl is installed and configured                   │
│     [2] List current Kubernetes contexts                                │
│     [3] Show pods in current namespace                                  │
│     [4] Get deployment status                                           │
│                                                                         │
│  For full deployment workflows, consider:                               │
│     • kubectl apply -f <manifest.yaml>                                 │
│     • Helm charts                                                       │
│     • CI/CD pipelines                                                   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘

Would you like help with one of these alternatives? [1/2/3/4/n]:
```

---

## Implementation Details

### LLM Prompt for Reasoning

```
You are a shell command assistant. When given a user query, analyze it and respond with structured reasoning.

OUTPUT FORMAT (JSON):
{
  "understood": true|false,
  "confidence": 0.0-1.0,
  "reasoning": {
    "what_i_understood": "Brief summary of interpreted intent",
    "what_is_unclear": ["List of ambiguous aspects"],
    "ambiguity_type": "platform|scope|action|context|domain|safety|none",
    "knowledge_gap": "Specific area where more context would help"
  },
  "command": "the_command_if_understood",
  "clarification_questions": [
    {
      "id": "unique_id",
      "question": "Human-readable question",
      "options": [{"key": "a", "label": "Option A", "maps_to": "command_a"}]
    }
  ],
  "platform_detected": "powershell|bash|zsh|fish|posix|unknown"
}

RULES:
1. If query explicitly mentions a platform (e.g., "powershell"), set high confidence
2. Generate clarification questions ONLY for truly ambiguous cases
3. For known single-answer queries, just provide the command
4. Always explain your reasoning in the "reasoning" field

User Query: {{user_input}}
Platform Context: {{platform_context}}
```

### Integration with Static Matcher

The clarification system integrates as a fallback:

```
User Query
    │
    ▼
┌──────────────────┐
│ Static Matcher   │ ─── Match? ──► Return command
└──────────────────┘
    │ No match
    ▼
┌──────────────────┐
│ LLM Generation   │ ─── High confidence? ──► Return command
└──────────────────┘
    │ Low confidence
    ▼
┌──────────────────┐
│ Clarification    │ ─── Questions answered ──► Enhanced query ──► LLM retry
└──────────────────┘
    │ User declines
    ▼
"I couldn't understand your request. Try rephrasing or use 'caro --help'."
```

---

## Consequences

### Positive

1. **Better UX**: Users get specific guidance instead of dead-ends
2. **Higher success rate**: Clarification increases command accuracy
3. **Platform awareness**: System explicitly handles cross-platform queries
4. **Transparency**: Reasoning shows users why caro is uncertain
5. **Learning opportunity**: Clarification data improves future training

### Negative

1. **Increased latency**: Clarification adds interactive round-trips
2. **Complexity**: More code paths to maintain and test
3. **LLM dependency**: Reasoning quality depends on model capability
4. **Edge cases**: Some queries may loop in clarification

### Mitigation Strategies

| Risk | Mitigation |
|------|------------|
| Latency | Cache common clarification patterns; stream questions while processing |
| Complexity | Well-defined state machine; comprehensive tests |
| LLM quality | Fallback to heuristic-based questions; A/B test prompts |
| Infinite loops | Max 2 clarification rounds; escalate to "I can't help" |

---

## Alternatives Considered

### Alternative 1: Just Improve Static Patterns

**Description**: Add more patterns to static matcher instead of clarification
**Pros**: Faster, deterministic, no LLM dependency
**Cons**: Can't cover all variations; doesn't help truly ambiguous queries
**Decision**: Partial - add PowerShell patterns, but clarification still needed

### Alternative 2: Always Ask for Platform First

**Description**: Start every session by asking user's shell
**Pros**: Simple, avoids per-query detection
**Cons**: Annoying UX for POSIX users (majority); state management needed
**Decision**: Rejected - detect from query or default to POSIX

### Alternative 3: Provide Multiple Commands

**Description**: When ambiguous, show all possible commands
**Pros**: No interaction needed
**Cons**: Overwhelming; user may pick wrong one; no learning
**Decision**: Partial - show alternatives in low-ambiguity cases

---

## Success Metrics

| Metric | Current | Target |
|--------|---------|--------|
| "Please clarify" responses | ~15% | <5% |
| Command generation success rate | 60% | 85% |
| User retry rate after failure | 40% | 15% |
| Average interactions to success | 2.5 | 1.3 |
| Cross-platform query success | 20% | 75% |

---

## References

- [Spec 006: Intelligent Prompt Generation](../../specs/006-intelligent-prompt-generation/spec.md)
- [ADR-004: Pre-Processing Pipeline](./ADR-004-pre-processing-pipeline.md)
- [PowerShell Profile Documentation](https://learn.microsoft.com/powershell/module/microsoft.powershell.core/about/about_profiles)

---

*This ADR was authored in January 2026 and defines the intelligent clarification UX for caro.*
