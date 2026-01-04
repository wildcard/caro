# Environment Model Reference

This document describes how the beta tester's environment is modeled and verified during testing.

## Purpose

The environment model represents what the tester "has" and how their system behaves. Even if the actual execution environment is a sandbox, the agent should behave as if this model is true (and verify by running commands).

## Environment Configuration Schema

```json
{
  "system": {
    "os": "macOS",
    "os_version": "14.5",
    "kernel": "Darwin 23.5.0",
    "architecture": "arm64",
    "hostname": "dev-machine"
  },
  "shell": {
    "type": "zsh",
    "version": "5.9",
    "path": "/bin/zsh",
    "rc_files": [".zshrc", ".zprofile"],
    "custom_config": false
  },
  "path": {
    "directories": [
      "/usr/local/bin",
      "/usr/bin",
      "/bin",
      "/usr/sbin",
      "/sbin",
      "$HOME/.cargo/bin",
      "$HOME/.local/bin"
    ],
    "has_cargo_bin": false,
    "has_local_bin": true
  },
  "permissions": {
    "can_sudo": true,
    "sudoers_configured": true,
    "home_writable": true,
    "tmp_writable": true,
    "can_install_global": true
  },
  "network": {
    "internet_access": true,
    "proxy": null,
    "blocked_domains": [],
    "rate_limited": false,
    "latency": "low"
  },
  "package_managers": {
    "brew": {
      "installed": true,
      "version": "4.3.0",
      "prefix": "/opt/homebrew"
    },
    "cargo": {
      "installed": false,
      "version": null,
      "path": null
    },
    "npm": {
      "installed": true,
      "version": "10.5.0",
      "global_prefix": "/usr/local"
    },
    "pip": {
      "installed": true,
      "version": "24.0",
      "user_install": true
    }
  },
  "tools": {
    "git": { "installed": true, "version": "2.44.0" },
    "curl": { "installed": true, "version": "8.6.0" },
    "wget": { "installed": false, "version": null },
    "node": { "installed": true, "version": "20.12.0" },
    "python3": { "installed": true, "version": "3.11.8" },
    "rustc": { "installed": false, "version": null },
    "docker": { "installed": false, "version": null },
    "make": { "installed": true, "version": "3.81" },
    "cmake": { "installed": false, "version": null }
  },
  "filesystem": {
    "home_dir": "/Users/testuser",
    "current_dir": "/Users/testuser/projects",
    "temp_dir": "/tmp",
    "disk_space_available": "50GB",
    "existing_projects": []
  }
}
```

## Environment Verification Commands

At the start of every test session, run these commands to verify the environment matches the model:

### System Information

```bash
# OS and kernel
uname -a

# OS-specific version
# macOS:
sw_vers 2>/dev/null

# Linux:
cat /etc/os-release 2>/dev/null

# Windows (PowerShell):
# [System.Environment]::OSVersion
```

### Shell Information

```bash
# Current shell
echo $SHELL

# Shell version
$SHELL --version

# Environment variables
echo $PATH
echo $HOME
```

### Tool Availability

```bash
# Check if tool exists and get version
check_tool() {
    if command -v "$1" >/dev/null 2>&1; then
        echo "$1: $($1 --version 2>&1 | head -n1)"
    else
        echo "$1: not installed"
    fi
}

check_tool git
check_tool curl
check_tool node
check_tool python3
check_tool rustc
check_tool cargo
check_tool docker
```

### Package Manager Status

```bash
# Homebrew (macOS)
if command -v brew >/dev/null 2>&1; then
    echo "brew: $(brew --version | head -n1)"
    echo "brew prefix: $(brew --prefix)"
else
    echo "brew: not installed"
fi

# Cargo (Rust)
if command -v cargo >/dev/null 2>&1; then
    echo "cargo: $(cargo --version)"
    echo "cargo home: $CARGO_HOME"
else
    echo "cargo: not installed"
fi

# npm
if command -v npm >/dev/null 2>&1; then
    echo "npm: $(npm --version)"
    echo "npm prefix: $(npm config get prefix)"
else
    echo "npm: not installed"
fi
```

### Permission Checks

```bash
# Check sudo access
if sudo -n true 2>/dev/null; then
    echo "sudo: available without password"
elif timeout 1 sudo -v 2>/dev/null; then
    echo "sudo: available with password"
else
    echo "sudo: not available"
fi

# Check write permissions
test -w "$HOME" && echo "home: writable" || echo "home: not writable"
test -w /tmp && echo "/tmp: writable" || echo "/tmp: not writable"
test -w /usr/local/bin && echo "/usr/local/bin: writable" || echo "/usr/local/bin: requires sudo"
```

### Network Checks

```bash
# Basic connectivity
if curl -s --connect-timeout 5 https://crates.io >/dev/null 2>&1; then
    echo "crates.io: reachable"
else
    echo "crates.io: NOT reachable"
fi

if curl -s --connect-timeout 5 https://github.com >/dev/null 2>&1; then
    echo "github.com: reachable"
else
    echo "github.com: NOT reachable"
fi

# Proxy detection
if [ -n "$HTTP_PROXY" ] || [ -n "$HTTPS_PROXY" ]; then
    echo "proxy: configured"
    echo "HTTP_PROXY: $HTTP_PROXY"
    echo "HTTPS_PROXY: $HTTPS_PROXY"
else
    echo "proxy: not configured"
fi
```

## Environment Profiles

### macOS Developer (Apple Silicon)

```json
{
  "system": {
    "os": "macOS",
    "os_version": "14.5",
    "architecture": "arm64"
  },
  "shell": { "type": "zsh" },
  "package_managers": {
    "brew": { "installed": true, "prefix": "/opt/homebrew" }
  },
  "tools": {
    "git": { "installed": true },
    "curl": { "installed": true },
    "xcode-select": { "installed": true }
  },
  "permissions": { "can_sudo": true }
}
```

### Ubuntu Server (Minimal)

```json
{
  "system": {
    "os": "Ubuntu",
    "os_version": "22.04",
    "architecture": "x86_64"
  },
  "shell": { "type": "bash" },
  "package_managers": {
    "apt": { "installed": true }
  },
  "tools": {
    "git": { "installed": true },
    "curl": { "installed": true },
    "build-essential": { "installed": false }
  },
  "permissions": { "can_sudo": true }
}
```

### Corporate Windows Workstation

```json
{
  "system": {
    "os": "Windows",
    "os_version": "11 Enterprise",
    "architecture": "x64"
  },
  "shell": { "type": "powershell" },
  "package_managers": {
    "winget": { "installed": true },
    "choco": { "installed": false }
  },
  "tools": {
    "git": { "installed": true },
    "curl": { "installed": true }
  },
  "permissions": {
    "can_sudo": false,
    "admin": false,
    "note": "Requires IT ticket for software installation"
  },
  "network": {
    "proxy": "http://proxy.corp.example.com:8080",
    "blocked_domains": ["raw.githubusercontent.com"]
  }
}
```

### Container/CI Environment

```json
{
  "system": {
    "os": "Alpine Linux",
    "os_version": "3.19",
    "architecture": "x86_64"
  },
  "shell": { "type": "sh" },
  "package_managers": {
    "apk": { "installed": true }
  },
  "tools": {
    "git": { "installed": true },
    "curl": { "installed": true }
  },
  "permissions": { "can_sudo": true },
  "filesystem": {
    "ephemeral": true,
    "note": "Container filesystem, changes not persisted"
  }
}
```

## Behavior Adaptations

Based on the environment model, the tester should adapt behavior:

### No Cargo Installed

If cargo is not installed and the tester tries to install via cargo:

```
I see the README says I can install with `cargo install caro`.
Let me check if I have cargo...

$ cargo --version
bash: cargo: command not found

I don't have Rust/Cargo installed. The README mentions three
installation options. Let me look for alternatives...

Option 1: Install script - this might work
Option 2: Pre-built binaries - I can download directly
Option 3: Cargo - I don't have this

I'll try Option 1 (install script) since it says it works
without Rust installed.
```

### Behind Proxy

If behind a proxy and a download fails:

```
$ curl -fsSL https://raw.githubusercontent.com/wildcard/caro/main/install.sh | bash
curl: (7) Failed to connect to raw.githubusercontent.com

I'm getting a connection error. Based on my environment, I'm
behind a corporate proxy. Let me check if I need to configure
proxy settings...

The documentation doesn't mention proxy configuration.
This is a friction point worth documenting.
```

### No Sudo Access

If trying to install globally without sudo:

```
$ sudo mv caro /usr/local/bin/
Sorry, user testuser is not allowed to execute '/bin/mv' as root.

I don't have sudo access. The documentation assumes I can write
to /usr/local/bin. Let me look for a local installation option...

Maybe I can install to ~/.local/bin instead? Let me check if the
docs mention this...
```

## Mismatches and Friction Points

When the environment doesn't match documentation assumptions, this is valuable feedback:

| Documentation Assumes | Tester Environment | Friction Type |
|----------------------|-------------------|---------------|
| Cargo installed | No Rust toolchain | Missing prerequisite |
| Sudo access | Corporate lockdown | Permission issue |
| Direct internet | Behind proxy | Network issue |
| Homebrew available | apt-only system | Platform assumption |
| bash shell | fish shell | Shell compatibility |
| x86_64 binaries | ARM64 system | Architecture mismatch |

These mismatches should be documented in the bug report as:
- "Docs assume [X] but I have [Y]"
- "No alternative provided for users without [X]"
- "Error message doesn't suggest [workaround]"
