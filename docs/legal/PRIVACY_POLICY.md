# Caro Privacy Policy

**Last Updated:** January 2026

## The Short Version

**Caro runs entirely on your machine.** Your prompts and commands never leave your device. We collect minimal, anonymized telemetry to improve the product—and you control whether to participate.

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

## What We Don't Collect

- **No prompts** - Your natural language queries stay on your machine
- **No commands** - Generated shell commands are never transmitted
- **No file contents** - We never access or transmit your files
- **No accounts** - No registration, no user database
- **No selling data** - We will never sell or share your data with advertisers

---

## Telemetry

Caro includes optional, anonymized telemetry to help us understand how the tool is used and where to focus improvements.

### What Telemetry Collects

| Data | Example | Purpose |
|------|---------|---------|
| Feature usage | "safety validation triggered" | Know which features matter |
| Error events | "backend timeout" | Fix bugs faster |
| Performance metrics | "inference time: 1.2s" | Optimize speed |
| Platform info | "macOS 14, arm64, zsh" | Prioritize platform support |

### What Telemetry Never Collects

- Your prompts or natural language input
- Generated commands
- File paths, contents, or directory structures
- Personal identifiers (name, email, IP address)
- Keystrokes or clipboard contents

### Your Control

| Release Stage | Default | How to Change |
|---------------|---------|---------------|
| **Beta** | Enabled (opt-out) | `caro config set telemetry.enabled false` |
| **GA (Stable)** | Disabled (opt-in) | `caro config set telemetry.enabled true` |

We use [PostHog](https://posthog.com) for telemetry. PostHog is privacy-focused and GDPR-compliant. You can review our telemetry implementation in the source code—it's fully auditable under AGPL-3.0.

### Why Opt-Out in Beta, Opt-In at GA?

During beta, telemetry helps us rapidly identify issues and improve the product. Once stable, we respect that most users prefer privacy by default. We believe this balance serves both early adopters who want to help improve Caro and users who prioritize minimal data collection

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

Our website at [caro.sh](https://caro.sh) uses [PostHog](https://posthog.com) for analytics to understand how visitors use the site.

### What Website Analytics Collects

- Pages visited and time spent
- Referral source (how you found us)
- Browser and device type
- Country (derived from IP, IP not stored)
- Button clicks and form interactions

### What Website Analytics Doesn't Collect

- Personal identifiers
- Cross-site tracking
- Data sold to advertisers

### Cookies

PostHog sets cookies to distinguish unique visitors. See our [Cookie Notice](./COOKIE_NOTICE.md) for details.

### Server Logs

Standard web server logs record:

- IP address (retained max 30 days)
- Browser user-agent
- Pages requested
- Timestamp

These are used solely for security monitoring.

---

## Binary Distribution

Pre-built binaries are distributed via GitHub Releases. Downloading binaries creates standard download logs on GitHub's servers, subject to [GitHub's Privacy Policy](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement).

---

## Your Rights

- **Opt-out of telemetry**: `caro config set telemetry.enabled false`
- **Full Transparency**: This software is open source under AGPL-3.0. You can audit exactly what telemetry sends.
- **Complete Control**: Your prompts and commands never leave your machine.
- **No Vendor Lock-in**: Delete the binary and config file; you're done.
- **GDPR/CCPA**: PostHog is GDPR and CCPA compliant. For data requests, contact us via GitHub.

---

## Children's Privacy

Caro does not collect personal data or require accounts. Telemetry is anonymized and contains no personal identifiers. There is no age restriction on using this software.

---

## Changes to This Policy

We will update this document when our privacy practices change. Changes will be committed to the repository with dated entries. Significant changes will be noted in release notes.

---

## Contact

For privacy questions:
- **GitHub Issues**: [github.com/wildcard/caro/issues](https://github.com/wildcard/caro/issues)
- **Discussions**: [github.com/wildcard/caro/discussions](https://github.com/wildcard/caro/discussions)

---

## Summary

| Question | Answer |
|----------|--------|
| Do you collect my prompts? | No, never |
| Do you collect my commands? | No, never |
| Is there telemetry? | Yes, anonymized usage stats (opt-out in beta, opt-in at GA) |
| Can I disable telemetry? | Yes: `caro config set telemetry.enabled false` |
| Do you sell my data? | No, never |
| Can I use this offline? | Yes, fully functional offline |
| Is this auditable? | Yes (AGPL-3.0 source available) |
| What about the website? | PostHog analytics, see Cookie Notice |

---

*This privacy policy is provided under [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/), inspired by [Automattic's Legalmattic](https://github.com/Automattic/legalmattic).*
