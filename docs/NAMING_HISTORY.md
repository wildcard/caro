# Naming History: From cmdai to caro

## Overview

**caro** (formerly known as **cmdai**) is a Rust CLI tool that converts natural language to safe shell commands using local LLMs. This document explains the naming evolution and the reasoning behind the change.

## Timeline

### December 2024: The cmdai Era
The project was initially developed and published under the name **cmdai** (short for "command AI"). The name reflected the tool's purpose: using AI to generate commands.

### December 2025: The Caro Transition
After initial development and testing, the project was renamed to **caro** thanks to the generosity of [@aeplay](https://github.com/aeplay), who graciously transferred the `caro` crate name to the project.

## Why "caro"?

The name **caro** offers several advantages over the original **cmdai**:

1. **Brevity**: Shorter and easier to type (4 characters vs 5)
2. **Memorability**: More distinctive and memorable as a brand name
3. **Pronounceability**: Natural pronunciation in multiple languages
4. **Brandability**: Better suited for a product name and domain (caro.sh)
5. **Community**: Reflects the open-source nature with a friendly, approachable name

### Etymology

**caro** can be interpreted multiple ways:
- Latin: "dear" or "beloved" (feminine form of *carus*)
- A friendly, approachable name that's easy to remember
- Short enough to type quickly in terminal commands

## Migration Guide

### For Users

If you previously installed **cmdai**, you can migrate to **caro** easily:

```bash
# Uninstall the old version (optional)
cargo uninstall cmdai

# Install the new version
cargo install caro

# Update any shell aliases (if you manually added them)
# Remove: alias caro='cmdai'
# No alias needed - the binary is now named 'caro' directly
```

### For Developers

The package name change affects:
- Crate name: `cmdai` → `caro`
- Binary name: `cmdai` → `caro`
- Repository URL: `github.com/wildcard/cmdai` → `github.com/wildcard/caro`
- Config directory: `~/.config/cmdai/` → `~/.config/caro/`
- Documentation URLs and references

### Configuration Migration

Your configuration will need to be migrated:

```bash
# If you have existing configuration
mv ~/.config/cmdai ~/.config/caro
```

The configuration file format remains the same - only the directory location has changed.

## Acknowledgments

Special thanks to **[@aeplay](https://github.com/aeplay)** for:
- Graciously transferring the `caro` crate name to this project
- Believing in the project's future and potential
- Supporting the open-source Rust community

This generosity enabled the project to have a better, more memorable name that will serve it well as it grows.

## References

- Project website: [caro.sh](https://caro.sh)
- GitHub repository: [github.com/wildcard/caro](https://github.com/wildcard/caro)
- Crates.io: [crates.io/crates/caro](https://crates.io/crates/caro)

## FAQ

### Will cmdai continue to exist?

No. The **cmdai** crate name will be deprecated in favor of **caro**. We recommend all users migrate to the new name.

### What about the cmdai repository?

The repository has been renamed from `wildcard/cmdai` to `wildcard/caro`. GitHub automatically redirects old URLs to the new location.

### Do I need to update my scripts?

If your scripts reference the `cmdai` binary, you'll need to update them to use `caro` instead. The command-line interface and all flags remain identical.

### Will my old configuration work?

Yes, but you'll need to move it from `~/.config/cmdai/` to `~/.config/caro/`. The configuration file format has not changed.

---

**Last Updated**: December 2025
