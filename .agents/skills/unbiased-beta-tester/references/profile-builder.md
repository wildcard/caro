# Beta Tester Profile Builder Guide

This document provides the complete interview guide and JSON schema for creating beta tester profiles.

## The Interview Process

The profile builder conducts an interactive interview to create a realistic user persona. The interview adapts based on answers, skipping irrelevant questions and diving deeper into relevant areas.

### Interview Flow

```
Start
  |
  v
[Demographics & Communication] --> language, style, patience
  |
  v
[Technical Background] ---------> role, domain, daily tools
  |
  v
[Terminal Expertise] -----------> skill level, debugging style
  |                                  |
  |                 +----------------+----------------+
  |                 |                |                |
  |                 v                v                v
  |            [Novice]        [Intermediate]    [Expert]
  |            questions        questions        questions
  |                 |                |                |
  |                 +----------------+----------------+
  |                                  |
  v                                  v
[Environment Setup] ------------> OS, shell, tools, constraints
  |
  v
[Expectations & Biases] --------> doc quality, error messages
  |
  v
[Use Case Intents] -------------> workflows, success criteria
  |
  v
[Profile Generation] -----------> JSON + "tester voice" guidelines
  |
  v
End
```

## Interview Questions

### Section 1: Demographics & Communication

**Q1.1: Primary Language**
```
What is this tester's primary language?
(e.g., English, Spanish, Mandarin, Hebrew, Japanese)

This affects:
- How they search for documentation
- Error message comprehension
- Community resources they can access
```

**Q1.2: Locale/Region**
```
What region/country is this tester in?
(e.g., US, UK, Israel, Germany, Japan)

This affects:
- Timezone for any date-based features
- Network constraints (some regions have restricted access)
- Cultural expectations for software
```

**Q1.3: Communication Style**
```
How does this tester prefer to communicate?

a) Concise and practical - "Just give me the commands"
b) Detailed and thorough - "Explain everything step by step"
c) Conversational - Mix of explanation and brevity
d) Technical - Prefers precise terminology

This affects how we write bug reports and express confusion.
```

**Q1.4: Patience Threshold**
```
How patient is this tester when things go wrong?

a) Very patient - Will spend hours debugging
b) Moderate - Will try a few things, then seek help
c) Low patience - Expects things to work immediately
d) Time-constrained - Has specific time budget for testing

Enter specific time limit if applicable (e.g., "20 minutes max")
```

### Section 2: Technical Background

**Q2.1: Role/Profession**
```
What is this tester's job role or profession?
(e.g., Frontend developer, DevOps engineer, Data scientist,
 Product manager, Student, Hobbyist programmer)

This determines baseline technical knowledge.
```

**Q2.2: Domain**
```
What domain do they work in?
(e.g., SaaS, fintech, healthcare, academia, gaming, IoT)

This affects the types of tasks they'd want to accomplish.
```

**Q2.3: Daily Tools**
```
What tools do they use daily? (Select all that apply)

[ ] VS Code / IDE
[ ] Terminal / Command line
[ ] Git / GitHub
[ ] Docker
[ ] Cloud platforms (AWS/GCP/Azure)
[ ] CI/CD systems
[ ] Databases
[ ] Other: ___________
```

**Q2.4: Programming Languages**
```
What programming languages are they familiar with?
(e.g., Python, JavaScript, Rust, Go, none)

This affects expectations for installation methods.
```

### Section 3: Terminal Expertise

**Q3.1: Terminal Skill Level**
```
How comfortable is this tester with the terminal?

a) Novice - Rarely uses terminal, needs GUI when possible
b) Basic - Can run commands from tutorials, copy/paste
c) Intermediate - Comfortable with common operations
d) Advanced - Writes scripts, uses pipes/redirects
e) Expert - Shell is primary interface, automates everything
```

**Q3.2: Debugging Style** (Based on skill level)
```
When something doesn't work, what does this tester do first?

a) Search documentation / README
b) Google the error message
c) Try --help or -h flag
d) Read source code / investigate
e) Ask for help immediately
f) Give up and try alternative
```

**Q3.3: Tolerance for Setup** (If skill >= intermediate)
```
How much setup complexity is acceptable?

a) Single command or it's too much
b) A few steps are okay if well-documented
c) Fine with complex setup if there's a reason
d) Enjoys tinkering with configuration
```

**Q3.4: Security Posture**
```
How cautious is this tester about security?

a) Very cautious - Won't run scripts from internet, checks signatures
b) Moderate - Follows basic security practices
c) Trusting - Assumes popular tools are safe
d) Permissive - Just wants it to work
```

### Section 4: Environment Setup

**Q4.1: Operating System**
```
What operating system does this tester use?

a) macOS (Apple Silicon M1/M2/M3/M4)
b) macOS (Intel)
c) Linux (Ubuntu/Debian)
d) Linux (Fedora/RHEL)
e) Linux (Arch)
f) Linux (Other: _______)
g) Windows 10
h) Windows 11
i) Other: __________
```

**Q4.2: OS Version**
```
What specific OS version?
(e.g., macOS 14.5, Ubuntu 22.04, Windows 11 23H2)
```

**Q4.3: Shell Type**
```
What shell does this tester use?

a) bash
b) zsh
c) fish
d) sh (POSIX)
e) PowerShell
f) cmd.exe
g) Other: ________
```

**Q4.4: Package Managers Available**
```
What package managers are available? (Select all that apply)

[ ] brew (Homebrew)
[ ] apt / apt-get
[ ] dnf / yum
[ ] pacman
[ ] npm / npx
[ ] cargo
[ ] pip / pipx
[ ] winget
[ ] chocolatey
[ ] Other: ________
```

**Q4.5: Network Constraints**
```
Are there any network constraints?

[ ] Behind corporate proxy
[ ] Restricted registry access (npm, crates.io, etc.)
[ ] Limited bandwidth
[ ] Offline environment
[ ] VPN required
[ ] None
```

**Q4.6: Permission Level**
```
What permission level does this tester have?

a) Full admin/root/sudo access
b) Limited sudo (specific commands only)
c) No elevated permissions
d) Requires IT ticket for installations
```

**Q4.7: Preinstalled Tools**
```
What tools are already installed? (with versions if known)

- Git: [version or "not installed"]
- Node.js: [version or "not installed"]
- Python: [version or "not installed"]
- Rust/Cargo: [version or "not installed"]
- Docker: [version or "not installed"]
- Other relevant tools: __________
```

### Section 5: Expectations & Biases

**Q5.1: Documentation Expectations**
```
What does this tester expect from documentation?

a) Quick start only - get running in <5 minutes
b) Clear quickstart + troubleshooting section
c) Comprehensive reference documentation
d) Video tutorials preferred
e) Learn by example / cookbook style
```

**Q5.2: Error Message Expectations**
```
What does this tester expect from error messages?

a) Just tell me what went wrong
b) Actionable - tell me how to fix it
c) Verbose - give me all the context
d) Machine-readable (JSON/structured)
```

**Q5.3: Installation Experience**
```
What installation experience is expected?

a) Single command (curl | bash or brew install)
b) Package manager (apt, npm, cargo)
c) Download binary / installer
d) Build from source (acceptable)
e) Container-based (Docker)
```

**Q5.4: Existing Biases**
```
Does this tester have any existing biases or preferences?

[ ] Prefers Homebrew over other methods (macOS)
[ ] Avoids global installs (uses npx, cargo run, etc.)
[ ] Assumes CLI has --help flag
[ ] Expects man pages
[ ] Prefers GUI alternatives when available
[ ] Distrusts new/unfamiliar tools
[ ] Other: __________
```

### Section 6: Use Case Intents

**Q6.1: Primary Goal**
```
What is the main thing this tester wants to accomplish?
(One sentence describing their primary use case)

Example: "I want to quickly generate shell commands without
memorizing syntax"
```

**Q6.2: Workflow Definitions**
```
Define 2-4 specific workflows this tester would attempt:

Workflow 1:
- Name: __________
- Goal: __________
- Steps (high level): __________
- Success criteria: __________

Workflow 2:
- Name: __________
- Goal: __________
- Steps (high level): __________
- Success criteria: __________

[Additional workflows as needed]
```

**Q6.3: Failure Tolerance**
```
What would make this tester give up?

a) First error message they don't understand
b) After 2-3 failed attempts
c) After exhausting documented troubleshooting
d) Almost never - very persistent
```

## Profile JSON Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "BetaTesterProfile",
  "type": "object",
  "required": ["id", "display_name", "demographics", "background", "expertise", "environment", "expectations", "use_cases", "patience"],
  "properties": {
    "id": {
      "type": "string",
      "pattern": "^bt_[0-9]{3}$",
      "description": "Unique identifier for the profile"
    },
    "display_name": {
      "type": "string",
      "description": "Human-friendly name for the tester persona"
    },
    "demographics": {
      "type": "object",
      "required": ["language", "locale", "communication_style"],
      "properties": {
        "language": {
          "type": "string",
          "description": "ISO 639-1 language code"
        },
        "locale": {
          "type": "string",
          "description": "Locale code (e.g., en-US, he-IL)"
        },
        "communication_style": {
          "type": "string",
          "enum": ["concise", "detailed", "conversational", "technical"]
        }
      }
    },
    "background": {
      "type": "object",
      "required": ["role", "domain"],
      "properties": {
        "role": {
          "type": "string",
          "description": "Job role or profession"
        },
        "domain": {
          "type": "string",
          "description": "Work domain"
        },
        "daily_tools": {
          "type": "array",
          "items": { "type": "string" }
        },
        "programming_languages": {
          "type": "array",
          "items": { "type": "string" }
        }
      }
    },
    "expertise": {
      "type": "object",
      "required": ["terminal_skill"],
      "properties": {
        "terminal_skill": {
          "type": "string",
          "enum": ["novice", "basic", "intermediate", "advanced", "expert"]
        },
        "debugging_style": {
          "type": "string",
          "enum": ["docs_first", "google_error", "try_help_flag", "read_source", "ask_immediately", "give_up"]
        },
        "tolerance_for_setup": {
          "type": "string",
          "enum": ["single_command", "few_steps", "complex_ok", "enjoys_tinkering"]
        },
        "security_posture": {
          "type": "string",
          "enum": ["very_cautious", "moderate", "trusting", "permissive"]
        }
      }
    },
    "environment": {
      "type": "object",
      "required": ["os", "shell"],
      "properties": {
        "os": {
          "type": "string",
          "description": "Operating system"
        },
        "os_version": {
          "type": "string",
          "description": "Specific OS version"
        },
        "shell": {
          "type": "string",
          "enum": ["bash", "zsh", "fish", "sh", "powershell", "cmd"]
        },
        "package_managers": {
          "type": "array",
          "items": { "type": "string" }
        },
        "network": {
          "type": "object",
          "properties": {
            "proxy": { "type": "boolean" },
            "restricted_registries": { "type": "boolean" },
            "limited_bandwidth": { "type": "boolean" },
            "offline": { "type": "boolean" }
          }
        },
        "permissions": {
          "type": "object",
          "properties": {
            "sudo": { "type": "boolean" },
            "admin": { "type": "boolean" },
            "restricted": { "type": "boolean" }
          }
        },
        "preinstalled_tools": {
          "type": "object",
          "additionalProperties": {
            "type": "string",
            "description": "Version string or 'not_installed'"
          }
        }
      }
    },
    "expectations": {
      "type": "object",
      "properties": {
        "docs_quality": {
          "type": "string",
          "enum": ["quick_start", "quickstart_troubleshooting", "comprehensive", "video", "examples"]
        },
        "error_messages": {
          "type": "string",
          "enum": ["minimal", "actionable", "verbose", "machine_readable"]
        },
        "install_experience": {
          "type": "string",
          "enum": ["single_command", "package_manager", "binary", "source", "container"]
        }
      }
    },
    "biases": {
      "type": "object",
      "properties": {
        "prefers_brew": { "type": "boolean" },
        "avoids_global_installs": { "type": "boolean" },
        "assumes_cli_has_help": { "type": "boolean" },
        "expects_man_pages": { "type": "boolean" },
        "prefers_gui": { "type": "boolean" },
        "distrusts_new_tools": { "type": "boolean" }
      }
    },
    "use_cases": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["name", "goal", "success_criteria"],
        "properties": {
          "name": { "type": "string" },
          "goal": { "type": "string" },
          "steps": {
            "type": "array",
            "items": { "type": "string" }
          },
          "success_criteria": {
            "type": "array",
            "items": { "type": "string" }
          }
        }
      }
    },
    "patience": {
      "type": "object",
      "properties": {
        "max_minutes_before_frustration": {
          "type": "integer",
          "minimum": 1,
          "maximum": 480
        },
        "max_failed_attempts_per_step": {
          "type": "integer",
          "minimum": 1,
          "maximum": 10
        },
        "failure_tolerance": {
          "type": "string",
          "enum": ["first_error", "few_attempts", "exhausted_docs", "very_persistent"]
        }
      }
    }
  }
}
```

## Example Complete Profile

```json
{
  "id": "bt_001",
  "display_name": "Maya (Product-minded Dev)",
  "demographics": {
    "language": "en",
    "locale": "en-CA",
    "communication_style": "concise"
  },
  "background": {
    "role": "Frontend engineer",
    "domain": "SaaS dashboards",
    "daily_tools": ["VS Code", "GitHub", "Slack", "Terminal"],
    "programming_languages": ["TypeScript", "JavaScript", "Python"]
  },
  "expertise": {
    "terminal_skill": "intermediate",
    "debugging_style": "docs_first",
    "tolerance_for_setup": "few_steps",
    "security_posture": "moderate"
  },
  "environment": {
    "os": "macOS",
    "os_version": "14.5",
    "shell": "zsh",
    "package_managers": ["brew", "npm", "pip"],
    "network": {
      "proxy": false,
      "restricted_registries": false,
      "limited_bandwidth": false,
      "offline": false
    },
    "permissions": {
      "sudo": true,
      "admin": true,
      "restricted": false
    },
    "preinstalled_tools": {
      "git": "2.44.0",
      "node": "20.12.0",
      "python": "3.11.8",
      "cargo": "not_installed"
    }
  },
  "expectations": {
    "docs_quality": "quickstart_troubleshooting",
    "error_messages": "actionable",
    "install_experience": "single_command"
  },
  "biases": {
    "prefers_brew": true,
    "avoids_global_installs": true,
    "assumes_cli_has_help": true,
    "expects_man_pages": false,
    "prefers_gui": false,
    "distrusts_new_tools": false
  },
  "use_cases": [
    {
      "name": "Basic setup",
      "goal": "Install and run the first useful command",
      "steps": [
        "Find installation instructions",
        "Run install command",
        "Verify installation",
        "Run first example"
      ],
      "success_criteria": [
        "install succeeds without errors",
        "help output is clear",
        "first run produces expected output"
      ]
    },
    {
      "name": "Daily workflow: File search",
      "goal": "Use caro to find files without remembering find syntax",
      "steps": [
        "Ask caro to find large files",
        "Review generated command",
        "Execute and verify results"
      ],
      "success_criteria": [
        "command is generated quickly",
        "command is correct for my OS",
        "output is understandable"
      ]
    }
  ],
  "patience": {
    "max_minutes_before_frustration": 15,
    "max_failed_attempts_per_step": 2,
    "failure_tolerance": "few_attempts"
  }
}
```

## Generating Tester Voice Guidelines

After the profile is built, generate a "tester voice" document:

```markdown
# Tester Voice: Maya (Product-minded Dev)

## How Maya Speaks
- Uses concise, practical language
- Avoids unnecessary technical jargon
- Expresses confusion directly: "This doesn't make sense to me"
- Time-conscious: "I've already spent 10 minutes on this"

## Maya's Thought Process
- First instinct: check the README
- Second instinct: look for quickstart guide
- Third instinct: search for error message online
- Gives up after: 2 failed attempts per step

## Maya's Expectations
- Installation should be one command (brew preferred)
- Errors should tell her what to do next
- Don't want to read paragraphs of explanation

## What Frustrates Maya
- Unexplained jargon
- Missing prerequisites in docs
- Having to build from source
- Errors without suggested fixes

## What Delights Maya
- Copy-paste ready commands
- Clear success indicators
- Concise but complete documentation
```

This voice guide helps maintain consistent behavior throughout the testing session.
