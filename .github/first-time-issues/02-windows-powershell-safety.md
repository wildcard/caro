# 🛡️ Expand Dangerous Command Patterns for Windows PowerShell

**Labels**: `good-first-issue`, `first-time-contributor`, `safety`, `security`, `cross-platform`, `windows`
**Difficulty**: Easy-Medium ⭐⭐
**Skills**: PowerShell knowledge, regex basics, security awareness
**Perfect for**: Security-conscious developers, Windows power users, safety advocates

## The Mission

cmdai currently has comprehensive safety patterns for Unix/Linux/macOS commands, but **Windows PowerShell protection is limited**. Help us keep Windows users safe by expanding our dangerous command detection!

You'll be directly protecting real users from potentially destructive AI-generated PowerShell commands.

## What You'll Build

Add 10-15 new dangerous PowerShell command patterns to our safety validator, including:

### High-Priority Dangerous Patterns

1. **System Destruction**
   - `Remove-Item -Path C:\ -Recurse -Force`
   - `Remove-Item -Path $env:USERPROFILE -Recurse`
   - `Clear-Disk`
   - `Format-Volume`

2. **Privilege Escalation**
   - `Start-Process powershell -Verb RunAs`
   - `Set-ExecutionPolicy Unrestricted`
   - `Add-LocalGroupMember -Group "Administrators"`

3. **System Modification**
   - `Set-MpPreference -DisableRealtimeMonitoring $true` (Disable Windows Defender)
   - `Stop-Service -Name "WinDefend" -Force`
   - `Disable-WindowsOptionalFeature`

4. **Data Destruction**
   - `Remove-Item -Path "C:\Program Files" -Recurse`
   - `Clear-RecycleBin -Force`
   - `Remove-ItemProperty` on critical registry keys

## Implementation Guide

### Step 1: Study Existing Patterns

Look at `src/safety/patterns.rs` to see how Unix patterns are defined:

```rust
pub const DANGEROUS_PATTERNS: &[DangerousPattern] = &[
    DangerousPattern {
        pattern: r"rm\s+-rf\s+/",
        severity: Severity::Critical,
        description: "Attempts to recursively delete root directory",
    },
    // ... more patterns
];
```

### Step 2: Add PowerShell Patterns

In `src/safety/patterns.rs`, create a new section:

```rust
/// Dangerous PowerShell command patterns
pub const POWERSHELL_DANGEROUS_PATTERNS: &[DangerousPattern] = &[
    DangerousPattern {
        pattern: r"Remove-Item.*-Recurse.*-Force",
        severity: Severity::Critical,
        description: "Recursive forced deletion in PowerShell",
    },
    DangerousPattern {
        pattern: r"Set-ExecutionPolicy\s+Unrestricted",
        severity: Severity::High,
        description: "Disables PowerShell execution policy",
    },
    // Add your patterns here!
];
```

### Step 3: Update the Safety Validator

In `src/safety/mod.rs`, integrate PowerShell patterns:

```rust
pub fn validate_command(&self, command: &str) -> ValidationResult {
    // Existing Unix validation...

    // Add PowerShell validation
    for pattern in POWERSHELL_DANGEROUS_PATTERNS {
        if let Ok(regex) = Regex::new(pattern.pattern) {
            if regex.is_match(command) {
                return ValidationResult::Dangerous {
                    severity: pattern.severity,
                    reason: pattern.description.to_string(),
                };
            }
        }
    }

    ValidationResult::Safe
}
```

### Step 4: Add Tests

Create tests in `tests/safety_powershell_tests.rs`:

```rust
#[test]
fn test_detects_recursive_deletion() {
    let validator = SafetyValidator::new();
    let result = validator.validate_command(
        "Remove-Item -Path C:\\ -Recurse -Force"
    );
    assert!(matches!(result, ValidationResult::Dangerous { .. }));
}

#[test]
fn test_detects_execution_policy_change() {
    let validator = SafetyValidator::new();
    let result = validator.validate_command(
        "Set-ExecutionPolicy Unrestricted"
    );
    assert!(matches!(result, ValidationResult::Dangerous { .. }));
}
```

## Acceptance Criteria

- [ ] At least 10 new PowerShell dangerous patterns added
- [ ] Each pattern has appropriate severity level (Critical/High/Moderate)
- [ ] Each pattern has clear, helpful description
- [ ] Regex patterns are tested and validated
- [ ] Tests cover all new patterns
- [ ] Documentation explains why each pattern is dangerous
- [ ] No false positives on safe PowerShell commands
- [ ] Code passes `cargo fmt` and `cargo clippy`

## Research Resources

### Dangerous PowerShell Commands
- [PowerShell Security Best Practices](https://docs.microsoft.com/en-us/powershell/scripting/security/security-best-practices)
- [Common PowerShell Attacks](https://attack.mitre.org/techniques/T1059/001/)
- [PowerShell Execution Policy](https://docs.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_execution_policies)

### Testing Your Regex
- [Regex101](https://regex101.com/) - Test patterns before adding them
- [PowerShell Regex Guide](https://docs.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_regular_expressions)

## Why This Matters

1. **Real User Protection**: Windows users deserve the same safety as Unix users
2. **Cross-Platform Safety**: cmdai should be safe everywhere
3. **Security Education**: Learning dangerous patterns teaches security
4. **Impact**: Directly prevents potential data loss and system damage

## Example Patterns to Detect

```powershell
# Critical - System destruction
Remove-Item -Path C:\ -Recurse -Force

# High - Disable security
Set-MpPreference -DisableRealtimeMonitoring $true

# High - Privilege escalation
Add-LocalGroupMember -Group "Administrators" -Member "BadActor"

# Moderate - Execution policy change
Set-ExecutionPolicy Unrestricted -Scope CurrentUser
```

## Questions?

Not a Windows expert? **That's okay!** We'll help you:
- Identify dangerous patterns
- Write effective regex
- Test on Windows systems
- Understand PowerShell semantics

## Pro Tips

1. **Test on Real Windows**: Use Windows VM or WSL2 to test patterns
2. **Start Simple**: Begin with obvious patterns, refine later
3. **Check Documentation**: PowerShell has excellent docs
4. **Ask for Review**: We'll verify patterns before merging

**Ready to protect Windows users? Let's build the safest AI terminal tool ever! 🛡️**
