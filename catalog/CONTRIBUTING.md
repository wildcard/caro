# Contributing to cmdai Community Catalog

Thank you for your interest in contributing to the cmdai Community Catalog! This guide will help you contribute high-quality commands that enhance cmdai's generation accuracy for everyone.

## 🎯 Contribution Overview

The Community Catalog improves cmdai by:
- Providing community-validated command examples
- Establishing best practices for common operations
- Creating a knowledge base of rated, tested commands
- Enabling context-aware command suggestions

## 🚀 Quick Start

### Prerequisites
- GitHub account
- Basic understanding of shell commands
- Familiarity with YAML format

### Steps to Contribute
1. **Fork the repository**
2. **Clone your fork locally**
3. **Create a new branch** for your contribution
4. **Add your command(s)** following our format
5. **Validate your submission** using our tools
6. **Submit a pull request**
7. **Participate in the review process**

## 📝 Command Submission Guide

### Step 1: Choose the Right Category

Select the most appropriate category for your command:

```
catalog/commands/
├── file-operations/     # ls, cp, mv, find, chmod
├── system-admin/        # ps, top, systemctl, df
├── development/         # git, npm, cargo, make
├── network/             # curl, wget, ping, ssh
└── text-processing/     # grep, sed, awk, sort
```

If unsure, check existing commands in each category for guidance.

### Step 2: Create Your Command File

Create a new YAML file in the appropriate subcategory:

```bash
# Example paths:
catalog/commands/file-operations/listing/ls-detailed.yml
catalog/commands/development/git/git-status-short.yml
catalog/commands/system-admin/processes/ps-user-processes.yml
```

### Step 3: Follow the Command Format

```yaml
# Required fields
command: "ls -la"
category: "file-operations"
subcategory: "listing"
description: "List all files with detailed information including hidden files"
tldr: "Shows files, permissions, sizes, modification dates"

# Quality metrics (will be updated automatically)
rating: 0.0
usage_count: 0
safety_level: "safe"  # safe|caution|dangerous|critical

# Command alternatives (optional but recommended)
alternatives:
  - command: "ls -l"
    description: "List files without hidden files"
    usage_count: 0
  - command: "exa -la"
    description: "Modern alternative with colors and icons"
    requirements: ["exa"]

# Usage examples (required)
examples:
  - input: "list all files"
    context: "general"
    confidence: 0.95
  - input: "show hidden files and permissions"
    context: "admin"
    confidence: 0.88

# Metadata
tags: ["listing", "permissions", "hidden", "basic"]
shell_compatibility:
  bash: true
  zsh: true
  fish: true
  powershell: false
  cmd: false
requirements: []  # List any external tools needed
os_compatibility: ["linux", "macos", "unix"]

# Community metadata (filled automatically)
contributors: ["your-username"]
created_date: "2025-10-19"
last_updated: "2025-10-19"
review_status: "pending"
```

### Step 4: Field Guidelines

#### Required Fields

**command**: The exact shell command
- Use standard flags and options
- Follow POSIX conventions when possible
- Keep commands reasonably concise

**category**: Primary category (see directory structure)

**subcategory**: Secondary categorization within the category

**description**: Clear, detailed explanation of what the command does
- Start with a verb (e.g., "List", "Copy", "Display")
- Be specific about the command's behavior
- Mention important side effects or requirements

**tldr**: Brief summary (1-2 lines max)
- Focus on the main purpose
- Use plain language
- Suitable for quick reference

**safety_level**: Accurate safety classification
- `safe`: Read-only operations, no system impact
- `caution`: Modifies files/settings, user should understand implications
- `dangerous`: Can cause data loss or system problems
- `critical`: System-wide impact, expert knowledge required

**examples**: Natural language inputs that should generate this command
- Include realistic user queries
- Cover different phrasings and contexts
- Add confidence scores (0.0-1.0) for how well the example matches

#### Optional but Recommended Fields

**alternatives**: Related commands that serve similar purposes
- Include common variations
- Mention modern alternatives (e.g., `exa` for `ls`)
- Note any special requirements

**tags**: Relevant keywords for searchability
- Use lowercase, descriptive terms
- Include command name, main function, important flags
- Think about how users might search for this

**requirements**: External tools or packages needed
- List non-standard tools required
- Help users understand dependencies

**shell_compatibility**: Which shells support this command
- Test on different shells when possible
- Note any shell-specific variations

**os_compatibility**: Operating systems where this works
- Focus on Unix-like systems primarily
- Note any OS-specific behavior

### Step 5: Quality Checklist

Before submitting, verify:

- [ ] **Command works**: Test the command on your system
- [ ] **Description is accurate**: Command does exactly what's described  
- [ ] **Safety classification is correct**: Matches the actual risk level
- [ ] **Examples are realistic**: Natural language inputs users would actually type
- [ ] **YAML is valid**: Use a YAML validator to check syntax
- [ ] **Follows naming convention**: File named appropriately
- [ ] **Category is correct**: Command is in the right directory
- [ ] **No duplicates**: Similar command doesn't already exist

## 🔧 Validation Tools

### Local Validation

Use our validation script before submitting:

```bash
# Clone the repository
git clone https://github.com/your-username/cmdai-catalog.git
cd cmdai-catalog

# Install validation tools
pip install -r scripts/requirements.txt

# Validate your command file
python scripts/validate_command.py catalog/commands/your-category/your-file.yml

# Check for duplicates
python scripts/check_duplicates.py
```

### Automated Validation

Our CI system automatically checks:
- YAML format validity
- Schema compliance
- Safety classification accuracy
- Category placement appropriateness
- Duplicate detection

## 📋 Review Process

### Community Review
1. **Automated checks** run immediately on PR submission
2. **Community members** review for:
   - Technical accuracy
   - Description quality
   - Safety classification
   - Alternative suggestions
3. **Discussion and feedback** to improve the submission
4. **Approval** when quality standards are met

### Maintainer Review
1. **Final quality check** by maintainers
2. **Integration approval** for catalog inclusion
3. **Merge and processing** by automated systems

### Review Timeline
- **Automated validation**: < 1 minute
- **Community review**: 1-3 days typically
- **Maintainer approval**: 1-2 days after community approval

## 🛡️ Safety Guidelines

### Safety Classification Details

**Safe Commands**:
```yaml
# Examples
command: "ls -la"       # Read-only file listing
command: "pwd"          # Show current directory
command: "date"         # Display current date/time
command: "whoami"       # Show current user
```

**Caution Commands**:
```yaml
# Examples
command: "cp file.txt backup.txt"  # Creates new files
command: "mkdir newdir"             # Creates directories
command: "chmod 644 file.txt"       # Changes permissions
command: "touch newfile.txt"        # Creates empty files
```

**Dangerous Commands**:
```yaml
# Examples
command: "rm *.tmp"                 # Deletes files with pattern
command: "sudo systemctl restart service"  # Affects system services
command: "dd if=/dev/zero of=file"  # Can overwrite data
command: "chown user:group file"    # Changes file ownership
```

**Critical Commands**:
```yaml
# Examples - These require expert review
command: "rm -rf /"                 # System destruction
command: "mkfs.ext4 /dev/sda"      # Formats disk
command: "iptables -F"             # Flushes firewall rules
command: "kill -9 -1"              # Kills all processes
```

### Safety Review Process
1. **Self-assessment**: Contributor classifies safety level
2. **Community verification**: Reviewers validate classification
3. **Maintainer approval**: Final safety verification
4. **Automatic flagging**: System flags potentially dangerous patterns

## 🎨 Style Guidelines

### Command Style
- **Prefer long flags** when they improve readability: `--verbose` over `-v`
- **Use standard options** that work across platforms
- **Avoid shell-specific features** unless noted in compatibility
- **Keep commands focused** on single, clear tasks

### Description Style
- **Start with action verbs**: "List", "Copy", "Display", "Remove"
- **Be specific**: "List files in long format" not "Show files"
- **Mention important behavior**: "Recursively copies directories"
- **Use present tense**: "Lists files" not "Will list files"

### Example Style
- **Use natural language**: How users actually speak
- **Include context**: "show files in project directory"
- **Vary phrasing**: Multiple ways to express the same intent
- **Be realistic**: Actual queries users would type

## 🏷️ Tagging Guidelines

### Good Tags
```yaml
tags: ["listing", "permissions", "hidden", "detailed", "files"]
```

### Tag Categories
- **Function**: What the command does (`listing`, `copying`, `searching`)
- **Target**: What it operates on (`files`, `processes`, `network`)
- **Attributes**: Important characteristics (`recursive`, `verbose`, `force`)
- **Context**: When it's used (`debugging`, `admin`, `development`)

### Avoid
- Overly generic tags (`command`, `shell`, `unix`)
- Duplicate information already in category
- Too many tags (keep to 3-7 relevant tags)

## 🔄 Advanced Contributions

### Multi-Command Workflows
For complex workflows, create pattern files in `catalog/patterns/`:

```yaml
# catalog/patterns/git-workflow-basic.yml
name: "Basic Git Workflow"
description: "Standard git add, commit, push sequence"
pattern_type: "workflow"
commands:
  - command: "git add ."
    description: "Stage all changes"
  - command: "git commit -m 'commit message'"
    description: "Commit with message"
  - command: "git push"
    description: "Push to remote repository"
context: "development"
frequency: "high"
```

### Shell-Specific Variations
When commands vary by shell:

```yaml
command: "ls -la"
shell_variations:
  fish: "ls -la"
  powershell: "Get-ChildItem -Force"
  cmd: "dir /a"
```

### Context-Aware Commands
For commands that change based on context:

```yaml
command: "npm start"
context_variations:
  - context: "node_project"
    command: "npm start"
    confidence: 0.95
  - context: "general"
    command: "npm start"
    confidence: 0.60
```

## 🚫 What Not to Include

### Avoid These Commands
- **Personal/organization-specific**: Commands with hardcoded paths or credentials
- **Extremely dangerous**: Commands that could cause irreversible system damage
- **Deprecated tools**: Commands for tools that are no longer maintained
- **Platform-specific hacks**: Non-portable solutions to specific problems

### Command Examples to Avoid
```yaml
# DON'T include commands like these:
command: "rm -rf /"                    # Extremely dangerous
command: "cd /home/myusername/project" # Personal paths
command: "mysql -u root -p secret"    # Hardcoded credentials
command: "sudo chmod 777 / -R"        # Security nightmare
```

## 🤝 Community Guidelines

### Code of Conduct
- **Be respectful**: Treat all contributors with kindness and respect
- **Be constructive**: Provide helpful, actionable feedback
- **Be patient**: Remember that people have different experience levels
- **Be collaborative**: Work together to improve submissions

### Review Etiquette
- **Focus on the command**: Review technical accuracy and quality
- **Explain your suggestions**: Help contributors understand improvements
- **Acknowledge good work**: Recognize well-crafted contributions
- **Stay professional**: Keep discussions focused and constructive

### Getting Help
- **GitHub Discussions**: Ask questions and share ideas
- **Issues**: Report bugs or suggest improvements
- **Discord/Chat**: Real-time community support (link in README)

## 📈 Recognition

### Contributor Recognition
- **Contributor list**: Listed in command metadata
- **Statistics tracking**: Contribution counts and impact metrics
- **Quality badges**: Recognition for high-quality submissions
- **Maintainer path**: Outstanding contributors may become maintainers

### Quality Metrics
Commands are automatically rated based on:
- Community usage and adoption
- User feedback and ratings
- Technical accuracy and completeness
- Safety and best practices adherence

## 🔮 Future Features

### Planned Enhancements
- **Machine learning integration**: AI-powered command suggestions
- **Usage analytics**: Anonymous usage pattern analysis
- **Advanced search**: Semantic search capabilities
- **Interactive testing**: Automated command verification

### Contributing to Tools
Beyond commands, you can contribute:
- **Validation scripts**: Improve quality checking
- **Documentation**: Enhance guides and references  
- **Automation**: GitHub Actions and CI improvements
- **Analysis tools**: Usage pattern analysis

## 📞 Support

### Getting Help
- **Documentation**: Check existing docs first
- **Search issues**: Look for similar questions
- **Ask in discussions**: Community-driven support
- **Contact maintainers**: For urgent issues only

### Reporting Issues
- **Bug reports**: Use the bug report template
- **Feature requests**: Use the feature request template
- **Security issues**: Email security@cmdai.dev directly

---

## 🎉 Ready to Contribute?

1. **Read this guide thoroughly**
2. **Look at existing examples** in the catalog
3. **Start with a simple command** to get familiar with the process
4. **Ask questions** if anything is unclear
5. **Submit your first contribution**!

Thank you for helping make cmdai better for everyone! 🚀