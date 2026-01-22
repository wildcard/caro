# Spec 008: Bubblewrap Sandbox Execution

## Overview

This specification defines the implementation of a sandbox execution layer using bubblewrap (bwrap) for command isolation. The sandbox prevents dangerous commands from affecting the host system, providing defense-in-depth protection beyond pattern-based validation.

## Documents

| Document | Description |
|----------|-------------|
| [spec.md](./spec.md) | Full product requirements document (PRD) |
| [plan.md](./plan.md) | Implementation plan and task breakdown |
| [ADR-004](../../docs/adr/ADR-004-bubblewrap-sandbox-execution.md) | Architecture decision record |

## Quick Links

- **Status**: Proposed
- **Priority**: High (security feature)
- **Target**: Community edition
- **Platforms**: Linux (bubblewrap), macOS (sandbox-exec)

## Key Features

1. **Filesystem isolation** - Commands can only access whitelisted paths
2. **Network isolation** - Block network access by default
3. **Process isolation** - Sandbox processes can't see host processes
4. **Configurable profiles** - Strict, Moderate, Permissive presets
5. **CLI integration** - `--sandbox`, `--no-sandbox`, `--sandbox-profile` flags

## Getting Started

### Prerequisites (Linux)

```bash
# Install bubblewrap
sudo apt install bubblewrap  # Debian/Ubuntu
sudo dnf install bubblewrap  # Fedora
sudo pacman -S bubblewrap    # Arch

# Verify user namespaces are enabled
cat /proc/sys/kernel/unprivileged_userns_clone  # Should return 1
```

### Usage

```bash
# Default: sandbox enabled with moderate profile
caro "find large files"

# Explicit control
caro "delete temp files" --sandbox-profile strict
caro "curl api endpoint" --no-sandbox  # Disable for network access
```

## Timeline

- **Phase 1**: Linux bubblewrap implementation
- **Phase 2**: macOS sandbox-exec implementation
- **Phase 3**: Production hardening, enable by default
- **Phase 4**: Windows investigation (future)

## Related

- [Security Policy](../../SECURITY.md)
- [Execution Module](../../src/execution/)
- [Safety Patterns](../../src/safety/)
