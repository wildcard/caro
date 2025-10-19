# cmdai Community Command Catalog

Welcome to the cmdai Community Command Catalog! This is a collaborative repository of shell commands, patterns, and workflows that enhances cmdai's command generation accuracy through collective knowledge.

## 🌟 Overview

The Community Catalog is a curated collection of:
- **Shell Commands**: Rated, categorized commands with examples
- **Usage Patterns**: Common command combinations and workflows  
- **Best Practices**: Community-validated command usage
- **Safety Guidelines**: Comprehensive safety classifications

## 📁 Structure

```
catalog/
├── commands/                    # Command definitions by category
│   ├── file-operations/        # File and directory operations
│   ├── system-admin/           # System administration tasks
│   ├── development/            # Development workflows
│   ├── network/                # Network operations
│   └── text-processing/        # Text manipulation and analysis
├── patterns/                   # Command pattern templates
├── metadata/                   # Catalog metadata and statistics
└── generated/                  # Auto-generated files for cmdai
```

## 🚀 Quick Start

### For Contributors

1. **Fork this repository**
2. **Add your command** in the appropriate category directory
3. **Follow the command format** (see [Command Format](#command-format))
4. **Submit a pull request** with clear description
5. **Participate in review** process

### For Users

The catalog is automatically integrated with cmdai. When you use cmdai:

```bash
cmdai "list files with sizes"
🤖 Analyzing community patterns...
📚 Found 3 similar commands in catalog
⚡ Generated: ls -lh
💡 Alternative: du -sh * (popular in community)
🔒 Safety: Safe operation
```

## 📝 Command Format

Each command is defined in a YAML file following this structure:

```yaml
# Basic information
command: "ls -la"
category: "file-operations"
subcategory: "listing"
description: "List all files with detailed information including hidden files"
tldr: "Shows files, permissions, sizes, modification dates"

# Quality metrics
rating: 4.8                     # Community rating (1.0-5.0)
usage_count: 15420             # Times used (from analytics)
safety_level: "safe"           # safe|caution|dangerous|critical

# Command alternatives
alternatives:
  - command: "ls -l"
    description: "List files without hidden files"
  - command: "exa -la" 
    description: "Modern alternative with colors"
    requirements: ["exa"]

# Usage examples
examples:
  - input: "list all files"
    context: "general"
  - input: "show files with permissions"
    context: "admin"

# Metadata
tags: ["listing", "permissions", "hidden", "basic"]
shell_compatibility:
  bash: true
  zsh: true
  fish: true
os_compatibility: ["linux", "macos", "unix"]
requirements: []                # External tool requirements

# Community data
contributors: ["username1", "username2"]
created_date: "2025-10-19"
last_updated: "2025-10-19"
review_status: "approved"
```

## 📂 Categories

### File Operations (`file-operations/`)
- **listing/**: Directory and file listing commands
- **manipulation/**: Copy, move, delete operations
- **search/**: Find files and content search
- **permissions/**: File permission management

### System Administration (`system-admin/`)
- **processes/**: Process management and monitoring
- **services/**: Service control and status
- **monitoring/**: System monitoring and diagnostics
- **users/**: User and group management

### Development (`development/`)
- **git/**: Git version control operations
- **build-tools/**: Compilation and build systems
- **testing/**: Testing frameworks and tools
- **packaging/**: Package management and distribution

### Network (`network/`)
- **connectivity/**: Network connectivity testing
- **transfer/**: File transfer and synchronization
- **monitoring/**: Network monitoring and diagnostics
- **security/**: Network security tools

### Text Processing (`text-processing/`)
- **search/**: Text search and pattern matching
- **manipulation/**: Text transformation and editing
- **analysis/**: Text analysis and statistics

## ✅ Quality Standards

### Command Requirements
1. **Accuracy**: Command must work as described
2. **Safety**: Appropriate safety classification
3. **Portability**: Works on standard Unix/Linux systems
4. **Clarity**: Clear, unambiguous description
5. **Completeness**: Relevant alternatives and examples

### Review Process
1. **Automated Validation** (30 seconds)
   - YAML format validation
   - Schema compliance check
   - Basic safety verification
2. **Community Review** (1-3 days)
   - Technical accuracy verification
   - Quality assessment
   - Alternative suggestions
3. **Maintainer Approval** (1-2 days)
   - Final quality check
   - Integration approval

## 🛡️ Safety Classification

Commands are classified by safety level:

- **Safe**: No system impact, read-only operations
- **Caution**: Modifies files/settings, requires user awareness
- **Dangerous**: Can cause data loss or system problems
- **Critical**: System-wide impact, requires expert knowledge

## 🤝 Contributing

### Contribution Guidelines

1. **One command per file**: Each YAML file should contain one command
2. **Descriptive filenames**: Use `command-description.yml` format
3. **Complete information**: Fill all required fields
4. **Test your commands**: Verify commands work on target systems
5. **Follow safety guidelines**: Accurate safety classification

### Naming Conventions

```
category/subcategory/command-brief-description.yml

Examples:
file-operations/listing/ls-detailed.yml
development/git/git-commit-message.yml
system-admin/processes/ps-detailed.yml
```

### Code of Conduct

- **Be respectful**: Treat all contributors with respect
- **Quality first**: Prioritize accuracy and usefulness
- **Safety conscious**: Always consider command safety implications
- **Collaborative**: Help others improve their contributions

## 🎯 Community Goals

### Short-term (3 months)
- [ ] 100+ high-quality commands across all categories
- [ ] 20+ active contributors
- [ ] Comprehensive coverage of basic operations
- [ ] Automated quality validation

### Medium-term (6 months)  
- [ ] 500+ commands with community ratings
- [ ] Advanced workflow patterns
- [ ] Multi-language support
- [ ] Integration with popular tools

### Long-term (12 months)
- [ ] 1000+ community-validated commands
- [ ] AI-powered command suggestions
- [ ] Enterprise and team features
- [ ] Cross-platform compatibility

## 📊 Statistics

*Updated automatically via GitHub Actions*

- **Total Commands**: 0 (catalog just initialized!)
- **Categories Covered**: 5
- **Contributors**: 0 (be the first!)
- **Average Rating**: N/A
- **Most Popular Category**: N/A

## 🔗 Resources

- **[Contributing Guide](CONTRIBUTING.md)**: Detailed contribution instructions
- **[Command Format Specification](../docs/command-format.md)**: Complete format documentation
- **[Safety Guidelines](../docs/safety-guidelines.md)**: Command safety standards
- **[API Documentation](../docs/api-reference.md)**: Catalog API reference

## 💬 Community

- **GitHub Discussions**: Share ideas and ask questions
- **Issues**: Report problems or suggest improvements
- **Pull Requests**: Contribute commands and improvements

## 📄 License

This catalog is released under the MIT License. See [LICENSE](LICENSE) for details.

---

**Ready to contribute?** Start by reading our [Contributing Guide](CONTRIBUTING.md) and adding your first command!

**Questions?** Open a [GitHub Discussion](https://github.com/cmdai/catalog/discussions) and we'll help you get started.