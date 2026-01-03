# Caro Privacy Policy

**Last Updated:** January 2026

## The Short Version

**Caro runs entirely on your machine. We don't collect, store, or transmit your data.** Period.

---

## What Data Caro Processes

Caro processes the following data **locally on your device only**:

| Data Type | Purpose | Stored? | Transmitted? |
|-----------|---------|---------|--------------|
| Your natural language prompts | Generate shell commands | No | No |
| System environment info (OS, shell, architecture) | Platform-appropriate commands | No | No |
| Generated commands | Display for your review | No | No |
| Configuration preferences | Customize behavior | Yes (local file only) | No |

### Configuration Storage

Your preferences are stored in `~/.config/caro/config.toml` on your local filesystem. This file never leaves your machine.

---

## What We Don't Do

- **No telemetry** - We don't phone home
- **No analytics** - We don't track usage patterns
- **No cloud services** - Everything runs locally
- **No accounts** - No registration, no user database
- **No cookies** - It's a terminal program
- **No third-party data sharing** - There's nothing to share

---

## Network Connections

### Default Configuration (Embedded Backend)

Caro's default embedded backend requires **zero network connections**. The AI model runs directly on your CPU or Apple Silicon GPU.

### Optional Remote Backends

If you configure Caro to use optional remote backends (Ollama or vLLM), connections are made only to servers **you explicitly configure** in your local config file:

```toml
[backend.ollama]
base_url = "http://localhost:11434"  # Your server

[backend.vllm]
base_url = "http://your-server:8000"  # Your server
```

These connections transmit your prompts to **your own infrastructure**. We have no visibility into or access to these communications.

---

## Model Downloads (Future Feature)

When model auto-download becomes available, Caro will connect to Hugging Face Hub to download model weights. This will:

- Use HTTPS connections to `huggingface.co`
- Cache models locally in `~/.cache/caro/`
- Not transmit any of your prompts or usage data

---

## Website (caro.sh)

If you visit our website at [caro.sh](https://caro.sh), standard web server logs may record:

- IP address
- Browser user-agent
- Pages requested
- Timestamp

These logs are used solely for security monitoring and are retained for a maximum of 30 days. We do not use tracking cookies or analytics services.

---

## Binary Distribution

Pre-built binaries are distributed via GitHub Releases. Downloading binaries creates standard download logs on GitHub's servers, subject to [GitHub's Privacy Policy](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement).

---

## Your Rights

Since we don't collect personal data, traditional data subject rights (access, deletion, portability) don't apply in the usual sense. However:

- **Full Transparency**: This software is open source under AGPL-3.0. You can audit exactly what it does.
- **Complete Control**: All data stays on your machine under your control.
- **No Vendor Lock-in**: Delete the binary and config file; you're done.

---

## Children's Privacy

Caro does not collect data from anyone, including children. There is no age restriction on using this software.

---

## Changes to This Policy

If we ever change our privacy practices, we will update this document and note the changes here. Given our architecture (local-only processing), we don't anticipate collecting user data in the future.

---

## Contact

For privacy questions:
- **GitHub Issues**: [github.com/wildcard/caro/issues](https://github.com/wildcard/caro/issues)
- **Discussions**: [github.com/wildcard/caro/discussions](https://github.com/wildcard/caro/discussions)

---

## Summary

| Question | Answer |
|----------|--------|
| Do you collect my prompts? | No |
| Do you track my usage? | No |
| Do you sell my data? | No (there's no data) |
| Can I use this offline? | Yes |
| Is this auditable? | Yes (AGPL-3.0 source available) |

---

*This privacy policy is provided under [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/), inspired by [Automattic's Legalmattic](https://github.com/Automattic/legalmattic).*
