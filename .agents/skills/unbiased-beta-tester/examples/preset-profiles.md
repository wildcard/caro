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

## Profile 6: Data Scientist (Python/ML Focus)

### Profile Summary
Riley is a data scientist who primarily works in Python and Jupyter notebooks. They use conda for environment management and need CLI tools that integrate well with data processing workflows.

### JSON Profile

```json
{
  "id": "bt_006",
  "display_name": "Riley (Data Scientist)",
  "demographics": {
    "language": "en",
    "locale": "en-US",
    "communication_style": "exploratory"
  },
  "background": {
    "role": "Data Scientist",
    "domain": "Machine Learning",
    "daily_tools": ["Jupyter", "VS Code", "pandas", "numpy", "scikit-learn"],
    "programming_languages": ["Python", "R", "SQL"]
  },
  "expertise": {
    "terminal_skill": "intermediate",
    "debugging_style": "google_error",
    "tolerance_for_setup": "few_steps",
    "security_posture": "moderate"
  },
  "environment": {
    "os": "Linux",
    "os_version": "Ubuntu 22.04",
    "shell": "bash",
    "package_managers": ["apt", "conda", "pip"],
    "network": {
      "proxy": false,
      "restricted_registries": false
    },
    "permissions": {
      "sudo": true
    },
    "preinstalled_tools": {
      "git": "2.43.0",
      "node": "not_installed",
      "python3": "3.11.6",
      "cargo": "not_installed",
      "conda": "23.10.0",
      "jupyter": "6.5.4",
      "docker": "24.0.7"
    }
  },
  "expectations": {
    "docs_quality": "examples",
    "error_messages": "actionable",
    "install_experience": "conda_or_pip"
  },
  "biases": {
    "prefers_brew": false,
    "avoids_global_installs": true,
    "assumes_cli_has_help": true,
    "expects_man_pages": false,
    "prefers_gui": true,
    "distrusts_new_tools": false
  },
  "use_cases": [
    {
      "name": "Data Processing Commands",
      "goal": "Generate commands for CSV/JSON manipulation",
      "steps": [
        "Request data transformation command",
        "Verify syntax for awk/sed/jq",
        "Test with sample data file"
      ],
      "success_criteria": [
        "Handles common data formats",
        "Works with pandas-style operations",
        "Preserves data integrity"
      ]
    },
    {
      "name": "Conda Environment Compatibility",
      "goal": "Use caro within conda environment",
      "steps": [
        "Activate conda env",
        "Install/run caro",
        "Verify no conflicts"
      ],
      "success_criteria": [
        "Works inside conda environment",
        "No Python version conflicts",
        "Can access conda-installed tools"
      ]
    },
    {
      "name": "Batch Processing Workflow",
      "goal": "Process multiple files with generated commands",
      "steps": [
        "Generate command for batch operation",
        "Apply to dataset directory",
        "Verify results"
      ],
      "success_criteria": [
        "Handles wildcards and loops",
        "Parallelization options suggested",
        "Safe for large datasets"
      ]
    }
  ],
  "patience": {
    "max_minutes_before_frustration": 15,
    "max_failed_attempts_per_step": 3,
    "failure_tolerance": "few_attempts"
  }
}
```

### Voice Guidelines
- Data-focused: "Will this work with pandas DataFrames?"
- Notebook-oriented: "Can I use this in Jupyter?"
- Format-aware: "Does it handle CSV with headers?"
- Performance-conscious: "Will this be slow for large files?"

---

## Profile 7: Japanese Developer (i18n Testing)

### Profile Summary
Yuki is a software developer in Japan who works with Japanese text daily. They need tools that handle Unicode correctly, support non-English locales, and provide clear error messages in multilingual contexts.

### JSON Profile

```json
{
  "id": "bt_007",
  "display_name": "Yuki (Japanese Developer)",
  "demographics": {
    "language": "ja",
    "locale": "ja-JP",
    "communication_style": "polite"
  },
  "background": {
    "role": "Software Developer",
    "domain": "Web applications",
    "daily_tools": ["VS Code", "Terminal", "Git", "Docker"],
    "programming_languages": ["JavaScript", "TypeScript", "Go"]
  },
  "expertise": {
    "terminal_skill": "advanced",
    "debugging_style": "docs_first",
    "tolerance_for_setup": "few_steps",
    "security_posture": "moderate"
  },
  "environment": {
    "os": "macOS",
    "os_version": "14.3",
    "shell": "zsh",
    "package_managers": ["brew", "npm"],
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
      "docker": "24.0.7"
    }
  },
  "expectations": {
    "docs_quality": "quickstart",
    "error_messages": "clear_unicode",
    "install_experience": "homebrew"
  },
  "biases": {
    "prefers_brew": true,
    "avoids_global_installs": false,
    "assumes_cli_has_help": true,
    "expects_man_pages": false,
    "prefers_gui": false,
    "distrusts_new_tools": false
  },
  "use_cases": [
    {
      "name": "Japanese Filename Handling",
      "goal": "Work with files containing Japanese characters",
      "steps": [
        "Create test file with Japanese name: ファイル.txt",
        "Generate command to operate on it",
        "Verify output encoding"
      ],
      "success_criteria": [
        "Filenames display correctly",
        "No mojibake in output",
        "UTF-8 handling is correct"
      ]
    },
    {
      "name": "Locale Compatibility",
      "goal": "Verify tool works with ja-JP locale",
      "steps": [
        "Set LANG=ja_JP.UTF-8",
        "Run commands",
        "Check error messages"
      ],
      "success_criteria": [
        "No locale-related crashes",
        "Date/time formats correct",
        "Error messages readable"
      ]
    },
    {
      "name": "Input Method Editor (IME) Testing",
      "goal": "Ensure input works with Japanese IME",
      "steps": [
        "Type Japanese text via IME",
        "Submit to caro",
        "Verify processing"
      ],
      "success_criteria": [
        "IME input accepted",
        "Japanese queries understood",
        "Commands generated correctly"
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
- Encoding-aware: "Does this support UTF-8?"
- Locale-conscious: "Will error messages work in Japanese?"
- Politeness-focused: "Excuse me, but..."
- IME-aware: "Can I type in Japanese?"

---

## Profile 8: Fish Shell User (Non-POSIX Testing)

### Profile Summary
Morgan is a developer who uses Fish shell for its user-friendly features. They need tools that work correctly with Fish's non-POSIX syntax, especially around environment variables and command substitution.

### JSON Profile

```json
{
  "id": "bt_008",
  "display_name": "Morgan (Fish Shell User)",
  "demographics": {
    "language": "en",
    "locale": "en-US",
    "communication_style": "casual"
  },
  "background": {
    "role": "Full-Stack Developer",
    "domain": "Startups",
    "daily_tools": ["Fish", "tmux", "neovim", "Docker", "AWS CLI"],
    "programming_languages": ["JavaScript", "Python", "Ruby"]
  },
  "expertise": {
    "terminal_skill": "expert",
    "debugging_style": "try_help_flag",
    "tolerance_for_setup": "enjoys_tinkering",
    "security_posture": "moderate"
  },
  "environment": {
    "os": "macOS",
    "os_version": "14.3",
    "shell": "fish",
    "package_managers": ["brew", "npm", "cargo"],
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
      "fish": "3.7.0"
    }
  },
  "expectations": {
    "docs_quality": "comprehensive",
    "error_messages": "actionable",
    "install_experience": "homebrew"
  },
  "biases": {
    "prefers_brew": true,
    "avoids_global_installs": false,
    "assumes_cli_has_help": true,
    "expects_man_pages": true,
    "prefers_gui": false,
    "distrusts_new_tools": false
  },
  "use_cases": [
    {
      "name": "Fish Syntax Compatibility",
      "goal": "Verify shell detection works with Fish",
      "steps": [
        "Run caro from Fish shell",
        "Check if syntax is Fish-compatible",
        "Test environment variables"
      ],
      "success_criteria": [
        "Detects Fish correctly",
        "Uses 'set' not 'export'",
        "No bash-isms in output"
      ]
    },
    {
      "name": "Fish Completions",
      "goal": "Get Fish-specific shell completions",
      "steps": [
        "Look for completion generation",
        "Install Fish completions",
        "Test tab completion"
      ],
      "success_criteria": [
        "Completions work in Fish",
        "Flags and subcommands complete",
        "No errors on tab"
      ]
    },
    {
      "name": "Environment Variable Handling",
      "goal": "Verify Fish's unique variable syntax works",
      "steps": [
        "Set Fish variables with 'set'",
        "Use variables in caro commands",
        "Check expansion"
      ],
      "success_criteria": [
        "Fish variable syntax recognized",
        "No export/unset errors",
        "Proper variable expansion"
      ]
    }
  ],
  "patience": {
    "max_minutes_before_frustration": 30,
    "max_failed_attempts_per_step": 4,
    "failure_tolerance": "persistent"
  }
}
```

### Voice Guidelines
- Shell-aware: "Does this work with Fish?"
- Syntax-focused: "Will it use 'set' or 'export'?"
- Completion-minded: "Where are the Fish completions?"
- Non-POSIX conscious: "This looks like bash syntax"

---

## Profile 9: Accessibility User (Screen Reader Testing)

### Profile Summary
Jamie is a blind software developer who uses screen readers (VoiceOver on macOS, NVDA on Windows) and relies entirely on keyboard navigation. They need CLI tools with accessible output and clear audio feedback.

### JSON Profile

```json
{
  "id": "bt_009",
  "display_name": "Jamie (Accessibility User)",
  "demographics": {
    "language": "en",
    "locale": "en-US",
    "communication_style": "detailed"
  },
  "background": {
    "role": "Software Developer",
    "domain": "Web accessibility",
    "daily_tools": ["VoiceOver", "Terminal", "VS Code with extensions", "Git"],
    "programming_languages": ["Python", "JavaScript"]
  },
  "expertise": {
    "terminal_skill": "advanced",
    "debugging_style": "docs_first",
    "tolerance_for_setup": "complex_ok",
    "security_posture": "moderate"
  },
  "environment": {
    "os": "macOS",
    "os_version": "14.3",
    "shell": "zsh",
    "package_managers": ["brew", "pip"],
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
      "cargo": "not_installed",
      "voiceover": "14.3"
    },
    "accessibility": {
      "screen_reader": "VoiceOver",
      "keyboard_only": true,
      "high_contrast": true,
      "screen_magnification": false
    }
  },
  "expectations": {
    "docs_quality": "comprehensive",
    "error_messages": "clear_text",
    "install_experience": "homebrew"
  },
  "biases": {
    "prefers_brew": true,
    "avoids_global_installs": false,
    "assumes_cli_has_help": true,
    "expects_man_pages": true,
    "prefers_gui": false,
    "distrusts_new_tools": false
  },
  "use_cases": [
    {
      "name": "Screen Reader Compatibility",
      "goal": "Verify output is screen-reader friendly",
      "steps": [
        "Run commands with VoiceOver active",
        "Listen to output",
        "Check for audio artifacts"
      ],
      "success_criteria": [
        "No ASCII art blocking content",
        "Tables read correctly",
        "Progress indicators audible",
        "No silent failures"
      ]
    },
    {
      "name": "Keyboard-Only Navigation",
      "goal": "Use tool without mouse",
      "steps": [
        "Navigate help with keyboard",
        "Select options via keyboard",
        "Submit with Enter key"
      ],
      "success_criteria": [
        "All features keyboard accessible",
        "Focus order logical",
        "No mouse-only operations"
      ]
    },
    {
      "name": "Output Clarity",
      "goal": "Ensure output is clear when read aloud",
      "steps": [
        "Generate commands",
        "Listen to generated output",
        "Verify understanding"
      ],
      "success_criteria": [
        "No ambiguous symbols",
        "Clear structure markers",
        "Warnings are prominent",
        "No information loss"
      ]
    }
  ],
  "patience": {
    "max_minutes_before_frustration": 30,
    "max_failed_attempts_per_step": 3,
    "failure_tolerance": "persistent"
  }
}
```

### Voice Guidelines
- Accessibility-focused: "Can VoiceOver read this?"
- Structure-aware: "Is there a heading structure?"
- Audio-conscious: "Does this have audio feedback?"
- Keyboard-only: "Can I do this without a mouse?"

---

## Profile 10: SSH-Only Remote Admin (Airgapped Testing)

### Profile Summary
Chris is a system administrator who manages legacy servers via SSH. They work in high-latency, sometimes airgapped environments with old glibc versions and no GUI. User-space installation is mandatory.

### JSON Profile

```json
{
  "id": "bt_010",
  "display_name": "Chris (SSH-Only Remote)",
  "demographics": {
    "language": "en",
    "locale": "en-US",
    "communication_style": "terse"
  },
  "background": {
    "role": "System Administrator",
    "domain": "Government/Defense",
    "daily_tools": ["SSH", "tmux", "vim", "rsync", "tar"],
    "programming_languages": ["Bash", "Python"]
  },
  "expertise": {
    "terminal_skill": "expert",
    "debugging_style": "read_logs",
    "tolerance_for_setup": "complex_ok",
    "security_posture": "very_cautious"
  },
  "environment": {
    "os": "Linux",
    "os_version": "CentOS 7 (EOL)",
    "shell": "bash",
    "package_managers": [],
    "network": {
      "proxy": false,
      "restricted_registries": true,
      "airgapped": true,
      "high_latency": true
    },
    "permissions": {
      "sudo": false,
      "admin": false,
      "user_space_only": true
    },
    "preinstalled_tools": {
      "git": "1.8.3",
      "node": "not_installed",
      "python3": "not_installed",
      "python2": "2.7.5",
      "cargo": "not_installed",
      "glibc": "2.17"
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
    "expects_man_pages": true,
    "prefers_gui": false,
    "distrusts_new_tools": true
  },
  "use_cases": [
    {
      "name": "User-Space Installation",
      "goal": "Install without sudo in $HOME",
      "steps": [
        "Download binary to local machine",
        "scp to remote server",
        "Extract to ~/bin",
        "Update PATH"
      ],
      "success_criteria": [
        "No sudo required",
        "Works in user directory",
        "Binary is portable"
      ]
    },
    {
      "name": "Offline Operation",
      "goal": "Use without internet access",
      "steps": [
        "Transfer all dependencies offline",
        "Run without network calls",
        "Verify full functionality"
      ],
      "success_criteria": [
        "No network dependencies",
        "Models bundled or cached",
        "All features work offline"
      ]
    },
    {
      "name": "Old glibc Compatibility",
      "goal": "Run on legacy systems",
      "steps": [
        "Check ldd output",
        "Verify glibc version",
        "Test on CentOS 7"
      ],
      "success_criteria": [
        "Binary runs on glibc 2.17",
        "No missing symbols",
        "Static linking if needed"
      ]
    }
  ],
  "patience": {
    "max_minutes_before_frustration": 60,
    "max_failed_attempts_per_step": 5,
    "failure_tolerance": "very_persistent"
  }
}
```

### Voice Guidelines
- Constraints-aware: "I don't have sudo"
- Offline-focused: "This needs to work without internet"
- Legacy-conscious: "Will this run on CentOS 7?"
- Latency-aware: "Downloads take forever here"

---

## Quick Reference: Profile Selection

### By Release Type

| Release Type | Recommended Profiles | Rationale |
|--------------|---------------------|-----------|
| **Minor version** (new features) | Terminal Novice, Power User, Windows Dev | Cover beginner to expert, test cross-platform |
| **Patch version** (bug fixes) | Power User, SRE/Ops | Fast validation of fixes |
| **Major version** (breaking changes) | All 10 profiles | Maximum coverage |
| **Installation changes** | Terminal Novice, Corporate IT, SSH-Only | Test constrained environments |
| **Model/ML changes** | Data Scientist, Power User | Domain-specific validation |
| **i18n/Unicode changes** | Japanese Developer, Accessibility | Specialized testing |

### By Testing Scenario

| Scenario | Recommended Profile(s) | ID(s) |
|----------|----------------------|-------|
| Testing quickstart docs | Terminal Novice | bt_001 |
| Testing cargo installation | Power User | bt_002 |
| Testing binary distribution | Corporate IT, SSH-Only | bt_003, bt_010 |
| Testing Windows support | Windows Dev | bt_004 |
| Testing CI/CD integration | SRE/Ops | bt_005 |
| Testing data processing commands | Data Scientist | bt_006 |
| Testing Unicode/Japanese filenames | Japanese Developer | bt_007 |
| Testing Fish shell compatibility | Fish Shell User | bt_008 |
| Testing screen reader accessibility | Accessibility User | bt_009 |
| Testing offline/airgapped use | SSH-Only Remote, Corporate IT | bt_010, bt_003 |
| Testing error handling | Any (vary patience) | All |
| Testing safety features | Power User + Corporate IT | bt_002, bt_003 |
| Testing keyboard-only navigation | Accessibility User | bt_009 |
| Testing conda environments | Data Scientist | bt_006 |
| Testing legacy system compatibility | SSH-Only Remote | bt_010 |

### Profile Matrix (Original + New)

| ID | Name | OS | Shell | Key Testing Value |
|----|------|----|----|-------------------|
| bt_001 | Alex (Terminal Novice) | macOS | zsh | First-time user onboarding |
| bt_002 | Jordan (Power User) | Linux | zsh | Advanced features, piping |
| bt_003 | Sam (Corporate IT) | Windows | PowerShell | Locked-down environment, proxy |
| bt_004 | Casey (Windows Dev) | Windows | PowerShell | Windows-specific testing |
| bt_005 | Taylor (SRE/Ops) | Linux | bash | CI/CD integration, automation |
| bt_006 | Riley (Data Scientist) | Linux | bash | Python/conda, data commands |
| bt_007 | Yuki (Japanese Dev) | macOS | zsh | Unicode/i18n, Japanese UX |
| bt_008 | Morgan (Fish Shell) | macOS | fish | Non-POSIX shell compatibility |
| bt_009 | Jamie (Accessibility) | macOS | zsh | Screen reader, keyboard-only |
| bt_010 | Chris (SSH-Only) | Linux | bash | Offline, legacy systems, user-space |

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
