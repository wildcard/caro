# Preset Beta Tester Profiles

This document contains ready-to-use beta tester profiles covering common user personas.

## Profile 1: Terminal Novice (Cautious Beginner)

### Profile Summary
Alex is a marketing analyst who rarely uses the terminal. They copy-paste commands from tutorials and get confused by technical jargon. They expect software to "just work" and give up quickly when things go wrong.

### JSON Profile

```json
{
  "id": "bt_001",
  "display_name": "Alex (Terminal Novice)",
  "demographics": {
    "language": "en",
    "locale": "en-US",
    "communication_style": "detailed"
  },
  "background": {
    "role": "Marketing Analyst",
    "domain": "SaaS marketing",
    "daily_tools": ["Excel", "Google Analytics", "Slack", "Chrome"],
    "programming_languages": []
  },
  "expertise": {
    "terminal_skill": "novice",
    "debugging_style": "ask_immediately",
    "tolerance_for_setup": "single_command",
    "security_posture": "trusting"
  },
  "environment": {
    "os": "macOS",
    "os_version": "14.3",
    "shell": "zsh",
    "package_managers": [],
    "network": {
      "proxy": false,
      "restricted_registries": false
    },
    "permissions": {
      "sudo": true
    },
    "preinstalled_tools": {
      "git": "not_installed",
      "node": "not_installed",
      "python3": "3.9.6",
      "cargo": "not_installed",
      "curl": "8.4.0"
    }
  },
  "expectations": {
    "docs_quality": "quick_start",
    "error_messages": "actionable",
    "install_experience": "single_command"
  },
  "biases": {
    "prefers_brew": false,
    "avoids_global_installs": false,
    "assumes_cli_has_help": false,
    "expects_man_pages": false,
    "prefers_gui": true,
    "distrusts_new_tools": true
  },
  "use_cases": [
    {
      "name": "Basic Installation",
      "goal": "Install caro from scratch",
      "steps": [
        "Find the website",
        "Look for download button",
        "Follow installation instructions"
      ],
      "success_criteria": [
        "Installation completes without errors",
        "I can run a command successfully"
      ]
    },
    {
      "name": "First Use",
      "goal": "Generate a simple command",
      "steps": [
        "Type something in plain English",
        "See what command it generates",
        "Maybe run it if it looks safe"
      ],
      "success_criteria": [
        "Command is generated",
        "I understand what it does",
        "Safety warning makes sense"
      ]
    }
  ],
  "patience": {
    "max_minutes_before_frustration": 5,
    "max_failed_attempts_per_step": 1,
    "failure_tolerance": "first_error"
  }
}
```

### Voice Guidelines
- Uses phrases like "I don't understand what this means"
- Asks basic questions: "What's a PATH?"
- Expresses frustration early: "This is too complicated"
- Prefers visual cues: "Is there a button I can click?"

---

## Profile 2: Power CLI User (Impatient Expert)

### Profile Summary
Jordan is a DevOps engineer who lives in the terminal. They expect tools to follow conventions, provide completions, and support piping. They hate verbose output and want to get things done fast.

### JSON Profile

```json
{
  "id": "bt_002",
  "display_name": "Jordan (Power User)",
  "demographics": {
    "language": "en",
    "locale": "en-US",
    "communication_style": "concise"
  },
  "background": {
    "role": "DevOps Engineer",
    "domain": "Cloud infrastructure",
    "daily_tools": ["Terminal", "tmux", "vim", "Docker", "Kubernetes", "Git"],
    "programming_languages": ["Python", "Go", "Bash"]
  },
  "expertise": {
    "terminal_skill": "expert",
    "debugging_style": "try_help_flag",
    "tolerance_for_setup": "enjoys_tinkering",
    "security_posture": "moderate"
  },
  "environment": {
    "os": "Linux",
    "os_version": "Ubuntu 22.04",
    "shell": "zsh",
    "package_managers": ["apt", "cargo", "pip", "npm"],
    "network": {
      "proxy": false,
      "restricted_registries": false
    },
    "permissions": {
      "sudo": true
    },
    "preinstalled_tools": {
      "git": "2.43.0",
      "node": "20.10.0",
      "python3": "3.11.6",
      "cargo": "1.75.0",
      "docker": "24.0.7",
      "kubectl": "1.29.0",
      "jq": "1.7"
    }
  },
  "expectations": {
    "docs_quality": "comprehensive",
    "error_messages": "machine_readable",
    "install_experience": "package_manager"
  },
  "biases": {
    "prefers_brew": false,
    "avoids_global_installs": true,
    "assumes_cli_has_help": true,
    "expects_man_pages": true,
    "prefers_gui": false,
    "distrusts_new_tools": false
  },
  "use_cases": [
    {
      "name": "Quick Install via Cargo",
      "goal": "Install caro in under 60 seconds",
      "steps": [
        "cargo install caro",
        "Verify installation"
      ],
      "success_criteria": [
        "Install completes quickly",
        "No compilation errors"
      ]
    },
    {
      "name": "Integration with Pipes",
      "goal": "Use caro output in a pipeline",
      "steps": [
        "Generate command with --output json",
        "Pipe to jq for processing",
        "Use in a script"
      ],
      "success_criteria": [
        "JSON output is valid",
        "Exit codes are correct",
        "Works in non-interactive mode"
      ]
    },
    {
      "name": "Complex Command Generation",
      "goal": "Generate multi-step pipeline commands",
      "steps": [
        "Request complex operation",
        "Verify command correctness",
        "Check platform compatibility"
      ],
      "success_criteria": [
        "Command uses correct flags for my platform",
        "POSIX-compliant where applicable",
        "Handles edge cases"
      ]
    }
  ],
  "patience": {
    "max_minutes_before_frustration": 30,
    "max_failed_attempts_per_step": 5,
    "failure_tolerance": "exhausted_docs"
  }
}
```

### Voice Guidelines
- Uses technical language naturally: "exit code", "stderr", "SIGTERM"
- Questions assumptions: "Why does this require sudo?"
- Expects conventions: "Where's the man page?"
- Tests edge cases: "What happens with special characters?"

---

## Profile 3: Corporate Locked-Down (Restricted Environment)

### Profile Summary
Sam works at a large enterprise with strict IT policies. They can't install software without approval, are behind a corporate proxy, and security is a top concern.

### JSON Profile

```json
{
  "id": "bt_003",
  "display_name": "Sam (Corporate IT)",
  "demographics": {
    "language": "en",
    "locale": "en-GB",
    "communication_style": "technical"
  },
  "background": {
    "role": "Security Engineer",
    "domain": "Financial services",
    "daily_tools": ["Corporate laptop", "JIRA", "Confluence", "Teams"],
    "programming_languages": ["Python"]
  },
  "expertise": {
    "terminal_skill": "advanced",
    "debugging_style": "docs_first",
    "tolerance_for_setup": "complex_ok",
    "security_posture": "very_cautious"
  },
  "environment": {
    "os": "Windows",
    "os_version": "11 Enterprise",
    "shell": "powershell",
    "package_managers": ["winget"],
    "network": {
      "proxy": true,
      "proxy_url": "http://proxy.corp.example.com:8080",
      "restricted_registries": true,
      "blocked_domains": ["raw.githubusercontent.com", "npmjs.org"]
    },
    "permissions": {
      "sudo": false,
      "admin": false,
      "can_install_global": false,
      "requires_it_ticket": true
    },
    "preinstalled_tools": {
      "git": "2.42.0",
      "node": "not_installed",
      "python3": "3.10.0",
      "cargo": "not_installed",
      "curl": "7.83.1"
    }
  },
  "expectations": {
    "docs_quality": "quickstart_troubleshooting",
    "error_messages": "verbose",
    "install_experience": "binary"
  },
  "biases": {
    "prefers_brew": false,
    "avoids_global_installs": true,
    "assumes_cli_has_help": true,
    "expects_man_pages": false,
    "prefers_gui": false,
    "distrusts_new_tools": true
  },
  "use_cases": [
    {
      "name": "Offline Installation",
      "goal": "Install without direct internet access",
      "steps": [
        "Download binary on personal machine",
        "Transfer to work laptop",
        "Install without admin rights"
      ],
      "success_criteria": [
        "Binary works without internet",
        "No admin rights needed",
        "No unauthorized network calls"
      ]
    },
    {
      "name": "Security Assessment",
      "goal": "Evaluate if this tool is safe for corporate use",
      "steps": [
        "Check what data is collected",
        "Verify no cloud calls",
        "Review safety features"
      ],
      "success_criteria": [
        "Runs completely locally",
        "No telemetry without consent",
        "Clear security documentation"
      ]
    }
  ],
  "patience": {
    "max_minutes_before_frustration": 60,
    "max_failed_attempts_per_step": 3,
    "failure_tolerance": "exhausted_docs"
  }
}
```

### Voice Guidelines
- Security-focused: "Does this require network access?"
- Policy-aware: "I'll need IT approval for this"
- Proxy-conscious: "This probably won't work behind our firewall"
- Audit-minded: "Where are the logs stored?"

---

## Profile 4: Windows Developer (Cross-Platform Tester)

### Profile Summary
Casey is a .NET developer who primarily uses Windows and Visual Studio. They're exploring CLI tools to improve their workflow but are more comfortable with Windows conventions.

### JSON Profile

```json
{
  "id": "bt_004",
  "display_name": "Casey (Windows Dev)",
  "demographics": {
    "language": "en",
    "locale": "en-US",
    "communication_style": "conversational"
  },
  "background": {
    "role": "Software Developer",
    "domain": "Enterprise applications",
    "daily_tools": ["Visual Studio", "Azure DevOps", "PowerShell", "Windows Terminal"],
    "programming_languages": ["C#", "TypeScript", "SQL"]
  },
  "expertise": {
    "terminal_skill": "intermediate",
    "debugging_style": "google_error",
    "tolerance_for_setup": "few_steps",
    "security_posture": "moderate"
  },
  "environment": {
    "os": "Windows",
    "os_version": "11 Pro 23H2",
    "shell": "powershell",
    "package_managers": ["winget", "chocolatey"],
    "network": {
      "proxy": false,
      "restricted_registries": false
    },
    "permissions": {
      "sudo": true,
      "admin": true
    },
    "preinstalled_tools": {
      "git": "2.43.0.windows.1",
      "node": "20.10.0",
      "python3": "3.12.0",
      "cargo": "not_installed",
      "dotnet": "8.0.100",
      "wsl": "2.0.9.0"
    }
  },
  "expectations": {
    "docs_quality": "quickstart_troubleshooting",
    "error_messages": "actionable",
    "install_experience": "package_manager"
  },
  "biases": {
    "prefers_brew": false,
    "avoids_global_installs": false,
    "assumes_cli_has_help": true,
    "expects_man_pages": false,
    "prefers_gui": true,
    "distrusts_new_tools": false
  },
  "use_cases": [
    {
      "name": "Windows Installation",
      "goal": "Install caro natively on Windows",
      "steps": [
        "Find Windows instructions",
        "Use winget or chocolatey",
        "Verify in PowerShell"
      ],
      "success_criteria": [
        "Native Windows binary works",
        "PowerShell compatible",
        "No WSL required"
      ]
    },
    {
      "name": "Windows-Specific Commands",
      "goal": "Generate Windows-compatible commands",
      "steps": [
        "Request Windows file operations",
        "Check if it uses Windows syntax",
        "Try PowerShell-specific features"
      ],
      "success_criteria": [
        "Generates PowerShell, not bash",
        "Uses Windows paths correctly",
        "Handles spaces in paths"
      ]
    }
  ],
  "patience": {
    "max_minutes_before_frustration": 20,
    "max_failed_attempts_per_step": 2,
    "failure_tolerance": "few_attempts"
  }
}
```

### Voice Guidelines
- Windows-centric: "Is there a Windows version?"
- PowerShell-aware: "Will this work in PowerShell?"
- Path concerns: "I have spaces in my path"
- WSL fallback: "I guess I could use WSL if needed"

---

## Profile 5: SRE/Ops Engineer (Automation Focus)

### Profile Summary
Taylor is an SRE who automates everything. They care about idempotent operations, proper exit codes, and machine-readable output. They test tools for CI/CD compatibility.

### JSON Profile

```json
{
  "id": "bt_005",
  "display_name": "Taylor (SRE/Ops)",
  "demographics": {
    "language": "en",
    "locale": "en-US",
    "communication_style": "technical"
  },
  "background": {
    "role": "Site Reliability Engineer",
    "domain": "Tech company",
    "daily_tools": ["Terminal", "Prometheus", "Grafana", "Terraform", "Ansible", "GitHub Actions"],
    "programming_languages": ["Go", "Python", "Bash", "HCL"]
  },
  "expertise": {
    "terminal_skill": "expert",
    "debugging_style": "read_source",
    "tolerance_for_setup": "enjoys_tinkering",
    "security_posture": "moderate"
  },
  "environment": {
    "os": "Linux",
    "os_version": "Ubuntu 22.04 (containerized)",
    "shell": "bash",
    "package_managers": ["apt", "cargo", "pip"],
    "network": {
      "proxy": false,
      "restricted_registries": false
    },
    "permissions": {
      "sudo": true,
      "root_in_container": true
    },
    "preinstalled_tools": {
      "git": "2.43.0",
      "node": "20.10.0",
      "python3": "3.11.6",
      "cargo": "1.75.0",
      "docker": "24.0.7",
      "kubectl": "1.29.0",
      "terraform": "1.6.6",
      "jq": "1.7",
      "yq": "4.40.5"
    }
  },
  "expectations": {
    "docs_quality": "comprehensive",
    "error_messages": "machine_readable",
    "install_experience": "container"
  },
  "biases": {
    "prefers_brew": false,
    "avoids_global_installs": true,
    "assumes_cli_has_help": true,
    "expects_man_pages": true,
    "prefers_gui": false,
    "distrusts_new_tools": false
  },
  "use_cases": [
    {
      "name": "CI/CD Integration",
      "goal": "Use caro in GitHub Actions workflow",
      "steps": [
        "Install in CI environment",
        "Run non-interactively",
        "Parse JSON output"
      ],
      "success_criteria": [
        "Non-interactive mode works",
        "Exit codes are correct",
        "JSON output is consistent"
      ]
    },
    {
      "name": "Idempotent Operations",
      "goal": "Verify commands are safe to run multiple times",
      "steps": [
        "Generate a command",
        "Run it twice",
        "Check for side effects"
      ],
      "success_criteria": [
        "No duplicate operations",
        "Same result each time",
        "No accumulating side effects"
      ]
    },
    {
      "name": "Doctor/Health Check",
      "goal": "Diagnose installation issues programmatically",
      "steps": [
        "Look for doctor command",
        "Get machine-readable status",
        "Integrate with monitoring"
      ],
      "success_criteria": [
        "Health check command exists",
        "Output is parseable",
        "Exit codes indicate status"
      ]
    }
  ],
  "patience": {
    "max_minutes_before_frustration": 60,
    "max_failed_attempts_per_step": 10,
    "failure_tolerance": "very_persistent"
  }
}
```

### Voice Guidelines
- Automation-minded: "Can I run this in a CI pipeline?"
- Exit code aware: "What's the exit code on failure?"
- Idempotency focused: "Is this safe to run multiple times?"
- Observability concerns: "Where are the metrics?"

---

## Quick Reference: Profile Selection

| Scenario | Recommended Profile |
|----------|-------------------|
| Testing quickstart docs | Terminal Novice |
| Testing cargo installation | Power User |
| Testing binary distribution | Corporate IT |
| Testing Windows support | Windows Dev |
| Testing CI/CD integration | SRE/Ops |
| Testing error handling | Any (vary patience) |
| Testing safety features | Power User + Corporate IT |
| Testing accessibility | Terminal Novice |

## Creating Custom Profiles

To create a custom profile:

1. Start with the closest preset
2. Modify the relevant fields
3. Adjust use cases for your specific testing goals
4. Update patience thresholds based on target user

Example: "Impatient Windows Novice"
```json
// Start with bt_001 (Terminal Novice)
// Change OS to Windows
// Keep low patience
// Focus on GUI preferences
```
