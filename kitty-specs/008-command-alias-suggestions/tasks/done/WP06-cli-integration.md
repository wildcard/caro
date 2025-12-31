# WP06: CLI Integration

**Work Package**: WP06
**Status**: planned
**Priority**: high
**Estimated Effort**: 2-3 days
**Depends On**: WP02, WP03

## Objective

Integrate the tips system into the main Caro CLI, adding new commands and post-execution hooks for seamless tip display.

## Tasks

### T6.1: Tips CLI Commands
- [ ] Add `caro tips` subcommand
- [ ] Implement `caro tips` - show random tip
- [ ] Implement `caro tips "command"` - tips for command
- [ ] Implement `caro tips search "query"` - search tips
- [ ] Implement `caro tips history` - recently shown tips

### T6.2: Aliases CLI Commands
- [ ] Add `caro aliases` subcommand
- [ ] Implement `caro aliases list` - list all aliases
- [ ] Implement `caro aliases search "pattern"` - search
- [ ] Implement `caro aliases suggest` - suggest for recent

### T6.3: KB CLI Commands
- [ ] Add `caro kb` subcommand
- [ ] Implement `caro kb update` - fetch latest KB
- [ ] Implement `caro kb status` - show version/stats
- [ ] Implement `caro kb export` - export local aliases

### T6.4: Post-Execution Hook
- [ ] Add tip suggestion after command generation
- [ ] Respect tip frequency configuration
- [ ] Non-blocking display (don't slow down execution)
- [ ] Handle user dismissal gracefully

### T6.5: Configuration Commands
- [ ] Add `caro config tips.enabled true/false`
- [ ] Add `caro config tips.frequency <value>`
- [ ] Add `caro config tips.categories.<cat> true/false`
- [ ] Document configuration options

### T6.6: Help and Documentation
- [ ] Add help text for all new commands
- [ ] Update man page (if exists)
- [ ] Add examples to --help output
- [ ] Update README with tips section

## Acceptance Criteria

- [ ] All new CLI commands working
- [ ] Tips shown after command execution (when enabled)
- [ ] Configuration options functional
- [ ] Help text complete and accurate
- [ ] No performance regression in main flow

## Technical Notes

**Clap Command Structure**:
```rust
#[derive(Parser)]
enum Commands {
    // Existing commands...

    /// Show tips and productivity suggestions
    Tips(TipsCommand),

    /// Manage shell aliases
    Aliases(AliasesCommand),

    /// Manage knowledge base
    Kb(KbCommand),
}

#[derive(Args)]
struct TipsCommand {
    #[command(subcommand)]
    action: Option<TipsAction>,

    /// Show tips for specific command
    #[arg(value_name = "COMMAND")]
    command: Option<String>,
}

#[derive(Subcommand)]
enum TipsAction {
    /// Search for tips
    Search { query: String },

    /// Show tip history
    History,
}
```

**Post-Execution Integration**:
```rust
// In main.rs or command execution
pub async fn run_with_tips(cmd: &str, config: &Config) -> Result<()> {
    // Execute the main command
    let result = execute_command(cmd).await?;

    // Show tip (non-blocking, after result)
    if config.tips.enabled {
        let tips_engine = TipsEngine::new(config)?;
        if let Some(tip) = tips_engine.suggest(cmd).await? {
            // Display tip in a visually distinct way
            tips_engine.display(&tip)?;
        }
    }

    Ok(result)
}
```

**Output Format**:
```
$ caro "show git status"

Generated: git status

> Tip: `gst` is aliased to `git status` (saves 7 chars)
> Run `caro aliases list` to see all your aliases
```

## Dependencies

- WP02 (Tips Engine)
- WP03 (Knowledge Base)
- `clap` for CLI parsing

## Files to Modify/Create

```
src/
├── main.rs              # Add tips integration
├── commands/
│   ├── mod.rs           # Register new commands
│   ├── tips.rs          # Tips command
│   ├── aliases.rs       # Aliases command
│   └── kb.rs            # KB command
└── config/
    └── tips.rs          # Tips configuration
```

## Performance Requirements

| Operation | Target |
|-----------|--------|
| Tip suggestion | < 50ms |
| KB status | < 100ms |
| Aliases list | < 200ms |
| No added latency to main flow | Async, non-blocking |
