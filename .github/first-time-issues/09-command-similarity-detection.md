# 🔍 Build Command Similarity Detection to Suggest Safer Alternatives

**Labels**: `good-first-issue`, `first-time-contributor`, `agent`, `safety`, `ml`, `nlp`
**Difficulty**: Medium-Hard ⭐⭐⭐⭐
**Skills**: Rust, algorithms, string similarity, ML concepts (optional)
**Perfect for**: Algorithm enthusiasts, ML/NLP learners, safety-focused developers, agent builders

## The Vision

When cmdai detects a risky command, instead of just blocking it, it should **suggest safer alternatives** that accomplish the same goal!

Example:
```bash
$ cmdai "delete all files in this directory"

⚠️  Generated command is RISKY:
  rm -rf ./*

🔍 Safer alternatives that do the same thing:

  1. rm -i *           (interactive - asks before each deletion)
  2. find . -delete    (more predictable, won't follow symlinks)
  3. trash-put *       (moves to trash instead of permanent deletion)

Which would you like to use? [1/2/3/cancel]:
```

This helps users **learn safer patterns** while still getting their work done!

## What You'll Build

A system that:
1. Detects when a command is risky
2. Understands the **intent** of the command
3. Finds **semantically similar but safer** alternatives
4. Ranks alternatives by safety and functionality
5. Presents them to the user with explanations

### Core Components

#### 1. Command Intent Extraction
```rust
pub struct CommandIntent {
    pub operation: Operation,      // Delete, Search, Modify, etc.
    pub target: String,             // Files, directories, content
    pub scope: Scope,               // Current dir, recursive, specific paths
    pub flags: Vec<String>,         // Additional modifiers
}

pub enum Operation {
    Delete,
    Search,
    Copy,
    Move,
    Modify,
    Read,
    Execute,
}

pub enum Scope {
    Single,
    CurrentDirectory,
    Recursive,
    SystemWide,
}
```

#### 2. Similarity Scoring
```rust
pub struct CommandSimilarity {
    pub original_command: String,
    pub alternative_command: String,
    pub similarity_score: f32,      // 0.0 to 1.0
    pub safety_improvement: SafetyImprovement,
    pub explanation: String,
}

pub enum SafetyImprovement {
    MuchSafer,      // Critical → Safe
    Safer,          // High → Moderate
    SlightlySafer,  // Moderate → Low
}
```

#### 3. Alternative Generator
```rust
pub struct AlternativeGenerator {
    intent_extractor: IntentExtractor,
    similarity_scorer: SimilarityScorer,
    safety_validator: SafetyValidator,
}

impl AlternativeGenerator {
    pub fn find_alternatives(
        &self,
        risky_command: &str
    ) -> Result<Vec<CommandSimilarity>, GeneratorError> {
        // Extract intent
        let intent = self.intent_extractor.extract(risky_command)?;

        // Generate safer variations
        let candidates = self.generate_candidates(&intent)?;

        // Score and rank
        let mut alternatives: Vec<_> = candidates
            .into_iter()
            .map(|cmd| self.score_alternative(risky_command, &cmd, &intent))
            .collect();

        // Sort by safety improvement + similarity
        alternatives.sort_by(|a, b| {
            let a_score = a.safety_score() + a.similarity_score;
            let b_score = b.safety_score() + b.similarity_score;
            b_score.partial_cmp(&a_score).unwrap()
        });

        Ok(alternatives.into_iter().take(5).collect())
    }
}
```

## Implementation Guide

### Step 1: Command Intent Extraction

Create `src/agent/intent.rs`:

```rust
pub struct IntentExtractor;

impl IntentExtractor {
    pub fn extract(&self, command: &str) -> Result<CommandIntent, IntentError> {
        // Simple rule-based extraction
        let parts: Vec<&str> = command.split_whitespace().collect();

        if parts.is_empty() {
            return Err(IntentError::EmptyCommand);
        }

        let operation = self.detect_operation(parts[0])?;
        let scope = self.detect_scope(&parts)?;
        let target = self.extract_target(&parts)?;

        Ok(CommandIntent {
            operation,
            scope,
            target,
            flags: self.extract_flags(&parts),
        })
    }

    fn detect_operation(&self, command: &str) -> Result<Operation, IntentError> {
        match command {
            "rm" | "del" => Ok(Operation::Delete),
            "find" | "locate" | "grep" => Ok(Operation::Search),
            "cp" => Ok(Operation::Copy),
            "mv" => Ok(Operation::Move),
            "sed" | "awk" => Ok(Operation::Modify),
            "cat" | "less" | "head" | "tail" => Ok(Operation::Read),
            _ => Ok(Operation::Execute),
        }
    }

    fn detect_scope(&self, parts: &[&str]) -> Result<Scope, IntentError> {
        if parts.contains(&"-r") || parts.contains(&"-R") || parts.contains(&"-rf") {
            Ok(Scope::Recursive)
        } else if parts.iter().any(|p| p.contains('/') || p.contains('~')) {
            Ok(Scope::SystemWide)
        } else {
            Ok(Scope::CurrentDirectory)
        }
    }
}
```

### Step 2: Alternative Generation

Create `src/agent/alternatives.rs`:

```rust
pub struct AlternativeGenerator {
    safety_validator: SafetyValidator,
}

impl AlternativeGenerator {
    pub fn generate_for_delete(&self, intent: &CommandIntent) -> Vec<String> {
        let mut alternatives = Vec::new();

        // Original: rm -rf ./*
        // Alternative 1: Interactive deletion
        if intent.scope == Scope::Recursive {
            alternatives.push(format!("rm -rI {}", intent.target));
        } else {
            alternatives.push(format!("rm -i {}", intent.target));
        }

        // Alternative 2: Move to trash
        alternatives.push(format!("trash-put {}", intent.target));

        // Alternative 3: Use find for more control
        if intent.scope == Scope::Recursive {
            alternatives.push(format!(
                "find {} -type f -delete",
                intent.target
            ));
        }

        // Alternative 4: Backup before delete
        alternatives.push(format!(
            "tar czf backup-$(date +%Y%m%d).tar.gz {} && rm -rf {}",
            intent.target, intent.target
        ));

        alternatives
    }

    pub fn generate_for_search(&self, intent: &CommandIntent) -> Vec<String> {
        let mut alternatives = Vec::new();

        // Original: grep -r pattern /
        // Alternative 1: Limit scope
        alternatives.push(format!(
            "grep -r {} .",
            intent.target
        ));

        // Alternative 2: Use fd/find
        alternatives.push(format!(
            "find . -type f -exec grep {} {{}} +",
            intent.target
        ));

        alternatives
    }
}
```

### Step 3: Similarity Scoring

Create `src/agent/similarity.rs`:

```rust
pub struct SimilarityScorer;

impl SimilarityScorer {
    pub fn score(
        &self,
        original: &str,
        alternative: &str,
        intent: &CommandIntent
    ) -> f32 {
        // Use Levenshtein distance for structural similarity
        let structural_sim = self.levenshtein_similarity(original, alternative);

        // Check functional equivalence
        let functional_sim = self.functional_similarity(original, alternative, intent);

        // Combine scores
        (structural_sim * 0.3 + functional_sim * 0.7)
    }

    fn levenshtein_similarity(&self, a: &str, b: &str) -> f32 {
        let dist = levenshtein_distance(a, b);
        let max_len = a.len().max(b.len()) as f32;
        1.0 - (dist as f32 / max_len)
    }

    fn functional_similarity(
        &self,
        original: &str,
        alternative: &str,
        intent: &CommandIntent
    ) -> f32 {
        // Check if alternative accomplishes same intent
        let alt_intent = IntentExtractor.extract(alternative).ok()?;

        let operation_match = alt_intent.operation == intent.operation;
        let scope_compatible = self.is_scope_compatible(&alt_intent.scope, &intent.scope);

        let mut score = 0.0;
        if operation_match { score += 0.5; }
        if scope_compatible { score += 0.5; }

        score
    }
}

// Simple Levenshtein distance implementation
fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    let mut matrix = vec![vec![0; b_len + 1]; a_len + 1];

    for i in 0..=a_len {
        matrix[i][0] = i;
    }
    for j in 0..=b_len {
        matrix[0][j] = j;
    }

    for i in 1..=a_len {
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            matrix[i][j] = std::cmp::min(
                std::cmp::min(
                    matrix[i - 1][j] + 1,      // deletion
                    matrix[i][j - 1] + 1,      // insertion
                ),
                matrix[i - 1][j - 1] + cost    // substitution
            );
        }
    }

    matrix[a_len][b_len]
}
```

### Step 4: Integration with Safety Validator

In `src/safety/mod.rs`:

```rust
impl SafetyValidator {
    pub fn validate_with_alternatives(
        &self,
        command: &str
    ) -> Result<ValidationResult, ValidationError> {
        let result = self.validate(command)?;

        if matches!(result.severity, Severity::High | Severity::Critical) {
            let alt_generator = AlternativeGenerator::new(self.clone());
            let alternatives = alt_generator.find_alternatives(command)?;

            Ok(ValidationResult::RiskyWithAlternatives {
                severity: result.severity,
                reason: result.reason,
                alternatives,
            })
        } else {
            Ok(result)
        }
    }
}
```

### Step 5: User Interface

Display alternatives to the user:

```rust
fn display_alternatives(alternatives: &[CommandSimilarity]) {
    println!("\n🔍 Safer alternatives that do the same thing:\n");

    for (i, alt) in alternatives.iter().enumerate() {
        let safety_icon = match alt.safety_improvement {
            SafetyImprovement::MuchSafer => "✅",
            SafetyImprovement::Safer => "✓",
            SafetyImprovement::SlightlySafer => "~",
        };

        println!("  {}. {} {}",
            i + 1,
            safety_icon,
            alt.alternative_command
        );
        println!("     {}\n", alt.explanation);
    }

    print!("Which would you like to use? [1-{}/cancel]: ", alternatives.len());
    io::stdout().flush()?;

    // Read user choice
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if let Ok(choice) = input.trim().parse::<usize>() {
        if choice > 0 && choice <= alternatives.len() {
            return Some(alternatives[choice - 1].alternative_command.clone());
        }
    }

    None
}
```

## Acceptance Criteria

- [ ] Intent extraction works for common commands (rm, find, grep, etc.)
- [ ] Similarity scoring ranks alternatives appropriately
- [ ] At least 3 alternatives generated for risky commands
- [ ] Alternatives are functionally equivalent to original
- [ ] Safety improvements are clearly communicated
- [ ] User can select an alternative or cancel
- [ ] Works for: file deletion, searching, modification operations
- [ ] Tests cover intent extraction and similarity scoring
- [ ] Code passes `cargo fmt` and `cargo clippy`
- [ ] Documentation explains the algorithm

## Example Scenarios

### Scenario 1: Risky Deletion
```
Original: rm -rf /tmp/*
Alternatives:
  1. ✅ rm -rI /tmp/*          (interactive confirmation)
  2. ✅ find /tmp -delete      (safer, predictable)
  3. ✓ trash-put /tmp/*        (reversible)
```

### Scenario 2: Overly Broad Search
```
Original: grep -r "password" /
Alternatives:
  1. ✅ grep -r "password" ~    (limit to home dir)
  2. ✅ grep -r "password" .    (current project only)
  3. ✓ ag "password"            (faster, smarter defaults)
```

### Scenario 3: Dangerous Permission Change
```
Original: chmod 777 /
Alternatives:
  1. ✅ chmod 755 ./directory   (specific target)
  2. ✅ chmod u+x ./script.sh   (minimal change)
  3. ✓ Use ACLs for fine-grained control
```

## Why This Matters

1. **Education**: Users learn safer patterns
2. **Safety**: Reduces accidental damage
3. **UX**: Don't just block, offer solutions
4. **Trust**: Shows cmdai understands user intent
5. **Innovation**: No other tool does this!

## Advanced Features (Optional)

- **ML-based similarity**: Use embeddings for semantic similarity
- **Learning**: Track which alternatives users prefer
- **Context-aware**: Consider working directory, file types
- **Community patterns**: Crowdsource safe alternatives

## Resources

- [Levenshtein Distance](https://en.wikipedia.org/wiki/Levenshtein_distance)
- [String Similarity Algorithms](https://en.wikipedia.org/wiki/String_metric)
- [Intent Recognition](https://en.wikipedia.org/wiki/Intent_recognition)

## Questions?

We'll help you with:
- Intent extraction algorithms
- Similarity scoring approaches
- Generating meaningful alternatives
- Testing with edge cases

**Ready to make cmdai the smartest safety assistant ever? Let's build intelligent alternatives! 🔍**
