# WP01: Shell Intelligence Foundation

**Work Package**: WP01
**Status**: planned
**Priority**: high
**Estimated Effort**: 3-4 days

## Objective

Build the core shell intelligence module that detects shell type, parses configuration files, and extracts aliases and plugins.

## Tasks

### T1.1: Shell Detector Module
- [ ] Create `src/tips/shell/detector.rs`
- [ ] Implement `ShellType` enum (Zsh, Bash, Fish, Sh)
- [ ] Detect shell from `$SHELL` environment variable
- [ ] Map shell types to default config file paths
- [ ] Handle edge cases (login shell, custom paths)

### T1.2: Config Parser Module
- [ ] Create `src/tips/shell/config_parser.rs`
- [ ] Read shell config files (.zshrc, .bashrc, config.fish)
- [ ] Handle `source` and `.` includes (recursive parsing)
- [ ] Expand environment variables and `~` in paths
- [ ] Return structured config representation

### T1.3: Alias Parser Module
- [ ] Create `src/tips/shell/alias_parser.rs`
- [ ] Parse Bash/Zsh `alias name='value'` syntax
- [ ] Parse Fish `alias name 'value'` syntax
- [ ] Handle quoted strings with escapes
- [ ] Build alias lookup HashMap
- [ ] Track alias source (file, line number)

### T1.4: Plugin Detector Module
- [ ] Create `src/tips/shell/plugin_detector.rs`
- [ ] Detect Oh My Zsh (`$ZSH`, `~/.oh-my-zsh`)
- [ ] Detect Prezto (`~/.zprezto`)
- [ ] Detect Zinit/Zi (`~/.zinit`)
- [ ] Detect Fisher for Fish (`~/.config/fish/functions`)
- [ ] Extract enabled plugin list from config

### T1.5: Integration and Types
- [ ] Create `src/tips/shell/mod.rs` module exports
- [ ] Define `ShellEnvironment` aggregate struct
- [ ] Create builder pattern for lazy loading
- [ ] Add caching for parsed results

### T1.6: Unit Tests
- [ ] Test shell detection for each shell type
- [ ] Test alias parsing with various syntaxes
- [ ] Test plugin detection with mock configs
- [ ] Test config include resolution

## Acceptance Criteria

- [ ] Shell type correctly detected from environment
- [ ] Aliases extracted from .zshrc, .bashrc, config.fish
- [ ] Oh My Zsh, Prezto, Zinit, Fisher detected when present
- [ ] Plugin list extracted from plugin managers
- [ ] All unit tests passing
- [ ] No panics on malformed config files

## Technical Notes

**Alias Parsing Regex Examples**:
```rust
// Bash/Zsh: alias name='value' or alias name="value"
let bash_alias_re = Regex::new(r#"alias\s+(\w+)=['"](.+?)['"]"#)?;

// Fish: alias name 'value' or alias name "value"
let fish_alias_re = Regex::new(r#"alias\s+(\w+)\s+['"](.+?)['"]"#)?;
```

**Oh My Zsh Detection**:
```rust
fn detect_ohmyzsh() -> Option<OhMyZsh> {
    let zsh_dir = std::env::var("ZSH")
        .ok()
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|h| h.join(".oh-my-zsh")))?;

    if zsh_dir.join("oh-my-zsh.sh").exists() {
        Some(OhMyZsh { path: zsh_dir })
    } else {
        None
    }
}
```

## Dependencies

- `shellexpand` crate for path expansion
- `regex` crate for pattern matching
- `dirs` crate for home directory

## Files to Create

```
src/tips/
├── mod.rs
└── shell/
    ├── mod.rs
    ├── detector.rs
    ├── config_parser.rs
    ├── alias_parser.rs
    └── plugin_detector.rs
```
