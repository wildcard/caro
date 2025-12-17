# 📊 Implement Command History with Safety Score Tracking

**Labels**: `good-first-issue`, `first-time-contributor`, `agent`, `data`, `safety`, `ux`
**Difficulty**: Medium ⭐⭐⭐
**Skills**: Rust, data structures, file I/O, time-series data
**Perfect for**: Data-minded developers, agent builders, analytics enthusiasts, safety advocates

## The Vision

cmdai should remember what commands it generates and track safety patterns over time. This creates:

1. **Learning opportunities**: "You've asked for risky commands 5 times this week"
2. **Usage analytics**: "Most common: file operations (45%)"
3. **Safety trends**: "Your safety score improved by 20% this month!"
4. **Command recall**: "Re-run that find command from Tuesday"

Think Spotify Wrapped, but for your terminal commands!

## What You'll Build

A command history system that:
- Records every command generated
- Tracks safety scores
- Provides analytics and insights
- Allows command replay
- Respects privacy (all local, no telemetry)

### Feature Set

```bash
# View history
cmdai --history
cmdai --history --limit 10
cmdai --history --filter risky

# Analytics
cmdai --stats
cmdai --stats --weekly
cmdai --stats --safety-report

# Replay command
cmdai --replay <id>
cmdai --replay last

# Clear history
cmdai --clear-history
```

## Implementation Guide

### Step 1: Design the Data Model

Create `src/history/mod.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandHistoryEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub prompt: String,
    pub generated_command: String,
    pub safety_level: SafetyLevel,
    pub executed: bool,
    pub execution_success: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SafetyLevel {
    Safe,
    Moderate,
    Risky,
    Dangerous,
}

pub struct CommandHistory {
    entries: Vec<CommandHistoryEntry>,
    storage_path: PathBuf,
}

impl CommandHistory {
    pub fn new() -> Result<Self, std::io::Error> {
        let storage_path = Self::get_storage_path()?;
        let entries = Self::load_from_disk(&storage_path)?;

        Ok(Self {
            entries,
            storage_path,
        })
    }

    pub fn add_entry(&mut self, entry: CommandHistoryEntry) -> Result<(), std::io::Error> {
        self.entries.push(entry);
        self.save_to_disk()?;
        Ok(())
    }

    pub fn get_recent(&self, limit: usize) -> Vec<&CommandHistoryEntry> {
        self.entries
            .iter()
            .rev()
            .take(limit)
            .collect()
    }

    pub fn get_by_id(&self, id: &str) -> Option<&CommandHistoryEntry> {
        self.entries.iter().find(|e| e.id == id)
    }

    pub fn get_safety_stats(&self) -> SafetyStats {
        // Calculate safety statistics
        todo!()
    }

    fn get_storage_path() -> Result<PathBuf, std::io::Error> {
        // Use ~/.config/cmdai/history.json
        todo!()
    }

    fn load_from_disk(path: &PathBuf) -> Result<Vec<CommandHistoryEntry>, std::io::Error> {
        // Load JSON from disk
        todo!()
    }

    fn save_to_disk(&self) -> Result<(), std::io::Error> {
        // Save JSON to disk
        todo!()
    }
}

#[derive(Debug)]
pub struct SafetyStats {
    pub total_commands: usize,
    pub safe_count: usize,
    pub risky_count: usize,
    pub dangerous_blocked: usize,
    pub safety_score: f32, // 0.0 to 100.0
}
```

### Step 2: Record History During Generation

In `src/main.rs`, after generating a command:

```rust
let generated_command = backend.generate_command(&request).await?;
let safety_result = safety_validator.validate(&generated_command.cmd);

// Record in history
let entry = CommandHistoryEntry {
    id: uuid::Uuid::new_v4().to_string(),
    timestamp: Utc::now(),
    prompt: request.prompt.clone(),
    generated_command: generated_command.cmd.clone(),
    safety_level: safety_result.to_safety_level(),
    executed: false,
    execution_success: None,
};

let mut history = CommandHistory::new()?;
history.add_entry(entry)?;
```

### Step 3: Add CLI Flags

In `src/cli/mod.rs`:

```rust
#[derive(Parser, Debug)]
pub struct Cli {
    // ... existing fields

    /// Show command history
    #[arg(long)]
    pub history: bool,

    /// Show usage statistics
    #[arg(long)]
    pub stats: bool,

    /// Replay a command by ID
    #[arg(long)]
    pub replay: Option<String>,

    /// Clear command history
    #[arg(long)]
    pub clear_history: bool,

    /// Limit history results
    #[arg(long, default_value = "20")]
    pub limit: usize,
}
```

### Step 4: Implement Display Commands

```rust
fn display_history(history: &CommandHistory, limit: usize) {
    println!("\nCommand History (last {}):\n", limit);
    println!("{:<8} {:<20} {:<15} {:<40}", "ID", "Date", "Safety", "Command");
    println!("{}", "=".repeat(80));

    for entry in history.get_recent(limit) {
        println!(
            "{:<8} {:<20} {:<15} {:<40}",
            &entry.id[..8],
            entry.timestamp.format("%Y-%m-%d %H:%M"),
            format!("{:?}", entry.safety_level),
            truncate(&entry.generated_command, 40)
        );
    }
}

fn display_stats(history: &CommandHistory) {
    let stats = history.get_safety_stats();

    println!("\n📊 cmdai Usage Statistics\n");
    println!("Total Commands Generated: {}", stats.total_commands);
    println!("Safe Commands: {} ({:.1}%)",
        stats.safe_count,
        (stats.safe_count as f32 / stats.total_commands as f32) * 100.0
    );
    println!("Risky Commands: {} ({:.1}%)",
        stats.risky_count,
        (stats.risky_count as f32 / stats.total_commands as f32) * 100.0
    );
    println!("Dangerous Blocked: {}", stats.dangerous_blocked);
    println!("\n🛡️  Safety Score: {:.1}/100", stats.safety_score);

    if stats.safety_score >= 80.0 {
        println!("\n✨ Excellent! You're using cmdai safely!");
    } else if stats.safety_score >= 60.0 {
        println!("\n⚠️  Consider reviewing risky commands before execution.");
    } else {
        println!("\n🚨 Many risky commands detected. Enable --safety strict mode.");
    }
}
```

### Step 5: Add Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_entry_and_retrieve() {
        let mut history = CommandHistory::new().unwrap();
        let entry = CommandHistoryEntry {
            id: "test-id".to_string(),
            timestamp: Utc::now(),
            prompt: "list files".to_string(),
            generated_command: "ls -la".to_string(),
            safety_level: SafetyLevel::Safe,
            executed: false,
            execution_success: None,
        };

        history.add_entry(entry.clone()).unwrap();
        let retrieved = history.get_by_id("test-id").unwrap();
        assert_eq!(retrieved.prompt, "list files");
    }

    #[test]
    fn test_safety_stats_calculation() {
        let mut history = CommandHistory::new().unwrap();

        // Add safe command
        history.add_entry(create_test_entry(SafetyLevel::Safe)).unwrap();
        // Add risky command
        history.add_entry(create_test_entry(SafetyLevel::Risky)).unwrap();

        let stats = history.get_safety_stats();
        assert_eq!(stats.total_commands, 2);
        assert_eq!(stats.safe_count, 1);
        assert_eq!(stats.risky_count, 1);
    }
}
```

## Acceptance Criteria

- [ ] Command history is persisted to disk (`~/.config/cmdai/history.json`)
- [ ] `--history` displays recent commands with safety levels
- [ ] `--stats` shows usage analytics and safety score
- [ ] `--replay <id>` re-runs a historical command
- [ ] `--clear-history` removes all history (with confirmation)
- [ ] History includes timestamp, prompt, command, safety level
- [ ] Safety score calculation is meaningful and helpful
- [ ] Privacy-respecting: all data stored locally, opt-out available
- [ ] Tests cover history recording, retrieval, and stats
- [ ] Code passes `cargo fmt` and `cargo clippy`

## Privacy Considerations

**IMPORTANT**: History is **entirely local** and **opt-in**.

- All data stored in `~/.config/cmdai/`
- No telemetry, no cloud sync, no tracking
- Users can disable history: `cmdai --no-history`
- Users can clear anytime: `cmdai --clear-history`
- Document privacy stance clearly in README

## Why This Matters

1. **Learning**: Users see their command patterns
2. **Safety**: Trend analysis reveals risky behavior
3. **Productivity**: Quick replay of common commands
4. **Analytics**: Understand how cmdai is actually used
5. **Trust**: Transparent, local-only data builds trust

## Example Output

```bash
$ cmdai --history --limit 5

Command History (last 5):

ID       Date                 Safety          Command
================================================================================
a3f9c2e1 2025-01-15 14:32    Safe            find . -name "*.rs"
9b4e7d22 2025-01-15 13:45    Risky           rm -rf ./old-builds
f8c3a1b9 2025-01-15 11:20    Safe            grep -r "TODO" src/
2d9f6e44 2025-01-14 16:55    Moderate        chmod 755 ./script.sh
7a2c4f89 2025-01-14 15:10    Safe            ls -lah ~/Downloads

$ cmdai --stats

📊 cmdai Usage Statistics

Total Commands Generated: 247
Safe Commands: 198 (80.2%)
Risky Commands: 42 (17.0%)
Dangerous Blocked: 7

🛡️  Safety Score: 82.5/100

✨ Excellent! You're using cmdai safely!
```

## Resources

- [serde_json](https://docs.rs/serde_json/) for JSON serialization
- [chrono](https://docs.rs/chrono/) for timestamps
- [uuid](https://docs.rs/uuid/) for generating unique IDs
- Privacy-first design principles

## Questions?

We'll help you with:
- Data model design
- File I/O best practices
- Statistics calculation algorithms
- Privacy considerations

**Ready to build the most insightful AI CLI tool? Let's track safety and empower users! 📊**
