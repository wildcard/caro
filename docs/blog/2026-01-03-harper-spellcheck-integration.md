# Harper Spell Check Integration

**Date**: January 3, 2026
**Type**: Feature Release
**Author**: Caro Team

---

## Caro Now Understands You Better

We've integrated [harper-core](https://crates.io/crates/harper-core), the privacy-first grammar and spell checking engine by Automattic, to improve how Caro understands your natural language input.

### The Problem

Small LLMs can struggle with typos. When you quickly type `shwo me teh files`, a smaller model might get confused about your intent. Larger models handle this better, but we want Caro to work well even with compact, local models.

### The Solution

Before sending your prompt to the LLM, Caro now automatically corrects common spelling mistakes:

```
✨ I understood what you meant: teh → the, shwo → show

Command:
  ls -la
```

This simple optimization:
- **Improves LLM comprehension** - cleaner input leads to better commands
- **Works offline** - harper-core runs locally, no API calls
- **Stays fast** - minimal overhead, doesn't slow down inference
- **Respects your privacy** - no data leaves your machine

### User Control

Spell checking is on by default, but you're always in control:

```bash
# Disable spell checking for a single command
caro --no-spellcheck "list teh files"
```

### Technical Details

Harper uses a curated FST (Finite State Transducer) dictionary with American English dialect support. The library:

- Detects spelling issues via the `PlainEnglish` parser
- Provides suggestions via the `LintGroup` linter
- Applies corrections automatically when confidence is high

We also maintain an ignore list for common shell/CLI terms (`sudo`, `chmod`, `grep`, etc.) to avoid false positives on technical vocabulary.

### Caro's Philosophy

Like the real Caro this tool is named after, she doesn't care about perfect grammar. She focuses on understanding your *intent*. Type comfortably, make typos - Caro's got your back.

---

**Links**:
- [Harper on GitHub](https://github.com/Automattic/harper)
- [harper-core on crates.io](https://crates.io/crates/harper-core)
- [Harper Website](https://writewithharper.com/)

---

*This is part of our ongoing effort to make Caro more accessible and forgiving for all users.*
