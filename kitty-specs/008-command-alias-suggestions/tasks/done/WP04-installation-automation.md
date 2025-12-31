# WP04: Installation Automation

**Work Package**: WP04
**Status**: planned
**Priority**: medium
**Estimated Effort**: 3-4 days
**Depends On**: WP01

## Objective

Build the automation system for installing shell plugins and modifying configuration files safely with backup/rollback capabilities.

## Tasks

### T4.1: Installation Flow Types
- [ ] Create `src/tips/automation/mod.rs`
- [ ] Define `InstallationPlan` struct
- [ ] Define `InstallStep` enum (Run, Backup, ConfigAdd, etc.)
- [ ] Define `Prerequisite` enum
- [ ] Define `VerificationStep` enum
- [ ] Define `RollbackPlan` struct

### T4.2: Plugin Suggester Module
- [ ] Create `src/tips/suggestions/plugin_suggester.rs`
- [ ] Detect when user could benefit from plugin
- [ ] Match command patterns to plugin recommendations
- [ ] Generate plugin suggestion with benefits
- [ ] Integrate with TipsEngine

### T4.3: Config Editor Module
- [ ] Create `src/tips/automation/config_editor.rs`
- [ ] Read config files safely
- [ ] Create timestamped backups before modification
- [ ] Implement atomic writes (write to temp, rename)
- [ ] Add/replace lines matching patterns
- [ ] Preserve file permissions and ownership

### T4.4: Installer Module
- [ ] Create `src/tips/automation/installer.rs`
- [ ] Execute installation plans step by step
- [ ] Prompt for user confirmation at key points
- [ ] Handle step failures gracefully
- [ ] Execute rollback on failure

### T4.5: Shell Reload Module
- [ ] Create `src/tips/automation/shell_reload.rs`
- [ ] Source config files for current shell
- [ ] Handle different shells (zsh, bash, fish)
- [ ] Verify changes took effect
- [ ] Notify user if manual reload needed

### T4.6: Built-in Installation Plans
- [ ] Create Oh My Zsh installation plan
- [ ] Create Prezto installation plan
- [ ] Create Fish Fisher installation plan
- [ ] Create common plugin enable plans
- [ ] Store plans as embedded data or config

### T4.7: CLI Commands
- [ ] Add `caro install ohmyzsh` command
- [ ] Add `caro install plugin <name>` command
- [ ] Add dry-run mode (`--dry-run`)
- [ ] Add verbose mode (`--verbose`)

### T4.8: Tests
- [ ] Unit test installation step execution
- [ ] Test config backup/restore
- [ ] Test atomic file writes
- [ ] Integration test with mock filesystem
- [ ] Test rollback scenarios

## Acceptance Criteria

- [ ] Oh My Zsh can be installed via `caro install ohmyzsh`
- [ ] Config files backed up before modification
- [ ] Rollback works if installation fails
- [ ] User prompted for confirmation
- [ ] Shell reload works (or provides instructions)
- [ ] Dry-run shows what would happen

## Technical Notes

**Installation Plan Example**:
```rust
fn ohmyzsh_plan() -> InstallationPlan {
    InstallationPlan {
        name: "Oh My Zsh".into(),
        description: "Framework for managing Zsh configuration".into(),
        prerequisites: vec![
            Prerequisite::ShellType(ShellType::Zsh),
            Prerequisite::CommandExists("curl".into()),
            Prerequisite::NotInstalled("~/.oh-my-zsh".into()),
        ],
        steps: vec![
            InstallStep::Confirmation {
                message: "Install Oh My Zsh? This will modify ~/.zshrc".into(),
            },
            InstallStep::Backup {
                path: "~/.zshrc".into(),
                label: "zshrc-backup".into(),
            },
            InstallStep::Run {
                command: r#"sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)" "" --unattended"#.into(),
                description: "Download and install Oh My Zsh".into(),
            },
        ],
        verification: vec![
            VerificationStep::PathExists("~/.oh-my-zsh".into()),
            VerificationStep::ConfigContains {
                path: "~/.zshrc".into(),
                pattern: "oh-my-zsh.sh".into(),
            },
        ],
        rollback: Some(RollbackPlan {
            restore_backups: vec!["zshrc-backup".into()],
            remove_paths: vec!["~/.oh-my-zsh".into()],
        }),
    }
}
```

**Config Editor Safety**:
```rust
pub fn modify_config(path: &Path, modification: ConfigModification) -> Result<()> {
    // 1. Create backup
    let backup_path = create_timestamped_backup(path)?;

    // 2. Read current content
    let content = fs::read_to_string(path)?;

    // 3. Apply modification
    let new_content = apply_modification(&content, &modification)?;

    // 4. Write to temp file
    let temp_path = path.with_extension("tmp");
    fs::write(&temp_path, &new_content)?;

    // 5. Atomic rename
    fs::rename(&temp_path, path)?;

    Ok(())
}
```

## Dependencies

- WP01 (Shell Intelligence) for shell detection
- `tempfile` crate for temp files

## Files to Create

```
src/tips/automation/
├── mod.rs
├── config_editor.rs
├── installer.rs
├── shell_reload.rs
└── plans/
    ├── mod.rs
    ├── ohmyzsh.rs
    ├── prezto.rs
    └── plugins.rs

src/tips/suggestions/
└── plugin_suggester.rs
```

## Security Considerations

1. **Explicit Consent**: Always prompt before modifications
2. **Backup First**: Never modify without backup
3. **Atomic Writes**: Use temp file + rename pattern
4. **Privilege Check**: Verify we have write permission
5. **Verification**: Confirm changes took effect
6. **Rollback Ready**: Keep backups until verified
