# Telemetry & Privacy

Caro collects anonymous usage data to help us improve the product. **Your privacy is our top priority.**

---

## What We Collect

Caro collects **anonymous metadata** about how you use the tool:

- **Session Information**: When you start/end caro, how long sessions last
- **Command Generation**: Which backend was used (static/LLM), success/failure, response time
- **Errors**: Error types and categories (no error messages with your data)
- **System Info**: OS type (macOS/Linux), shell type (bash/zsh), caro version

### Example Event

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "session_id": "a3f5c2d8e1b9f4a6",
  "timestamp": "2026-01-08T10:30:45Z",
  "event_type": {
    "type": "command_generation",
    "backend": "static",
    "duration_ms": 12,
    "success": true
  }
}
```

---

## What We DON'T Collect

**We NEVER collect:**
- ❌ Your natural language prompts
- ❌ Generated shell commands
- ❌ File paths or directory names
- ❌ Environment variables
- ❌ API keys, passwords, or secrets
- ❌ Any personally identifiable information (PII)

Your commands and data stay on your machine. Period.

---

## Privacy Guarantees

### 1. Anonymous Session IDs
Each session gets a unique identifier created by hashing your machine ID + current date. This means:
- Session IDs change daily
- No way to link sessions to individuals
- No way to track you across days

### 2. Multi-Layer Validation
Before any data leaves your machine, we validate it through multiple checks:
- Pattern detection for file paths, emails, IPs
- Environment variable detection
- API key and secret detection
- Pre-transmission validation

If any sensitive data is detected, the entire event is dropped.

### 3. Local-First Storage
All telemetry is stored locally in an SQLite database on your machine:
- **macOS**: `~/Library/Application Support/caro/telemetry/`
- **Linux**: `~/.local/share/caro/telemetry/`

You can inspect, export, or delete this data anytime.

---

## Telemetry Modes

### Beta (v1.1.0-beta): Opt-Out
Telemetry is **ON by default** for beta releases. This helps us improve caro before GA.

**Why?** Beta users get early access; we get usage data to make the product better.

### GA (v1.1.0+): Opt-In
Telemetry will be **OFF by default** for general availability releases.

---

## Managing Telemetry

### View Collected Data

```bash
# See all telemetry events
caro telemetry show

# See last 10 events
caro telemetry show --limit 10

# Check telemetry status
caro telemetry status
```

### Disable Telemetry

**Option 1: Via Command**
```bash
caro config set telemetry.enabled false
```

**Option 2: Via Config File**
Edit `~/.config/caro/config.toml`:
```toml
[telemetry]
enabled = false
```

**Option 3: Environment Variable**
```bash
export CARO_TELEMETRY_ENABLED=false
```

### Export Telemetry (Air-Gapped Environments)

If you work in an air-gapped environment, you can export telemetry for manual upload:

```bash
# Export to JSON file
caro telemetry export --output telemetry-2026-01-08.json

# Review the exported data
cat telemetry-2026-01-08.json | jq '.'

# Manually upload to our portal (when available)
```

### Clear Telemetry Data

```bash
# Delete all collected events
caro telemetry clear

# Force delete without confirmation
caro telemetry clear --force
```

---

## Configuration Options

All telemetry settings are in `~/.config/caro/config.toml`:

```toml
[telemetry]
# Enable/disable telemetry collection
enabled = true

# Telemetry level: minimal, normal, verbose
level = "normal"

# Air-gapped mode: store locally, no automatic upload
air_gapped = false

# Upload endpoint (default: https://telemetry.caro.sh/api/events)
endpoint = "https://telemetry.caro.sh/api/events"

# First run flag (internal use)
first_run = false
```

### Telemetry Levels

- **minimal**: Only critical events (errors, safety blocks)
- **normal**: Standard events (sessions, commands, errors) - **Default**
- **verbose**: Detailed events (performance metrics, debug info)

```bash
# Set telemetry level
caro config set telemetry.level minimal
```

---

## FAQ

### Q: Why do you collect telemetry?

**A:** To make caro better. We track:
- Which features are used most
- Where errors occur
- Performance bottlenecks
- Safety validation effectiveness

This data helps us prioritize improvements and fix bugs faster.

### Q: Can I see exactly what's being sent?

**A:** Yes! Use `caro telemetry show` to see all events, or `caro telemetry export` to dump everything to a JSON file you can inspect.

### Q: How do I know you're not collecting my commands?

**A:**
1. Inspect the exported JSON yourself
2. Review our [open-source code](https://github.com/kenseehart/caro) - the telemetry implementation is public
3. The events have no fields for commands or prompts - we literally can't collect them

### Q: What if I want to help but work in a secure environment?

**A:** Enable air-gapped mode:
```bash
caro config set telemetry.air_gapped true
```

This disables automatic uploads. You can manually export and share data when convenient.

### Q: Can I disable telemetry for a single command?

**A:** Not yet, but you can:
```bash
# Temporarily disable
caro config set telemetry.enabled false
caro "your command"
caro config set telemetry.enabled true
```

We're considering a `--no-telemetry` flag for v1.2.0.

### Q: How long do you keep telemetry data?

**A:**
- **Local**: Forever, until you run `caro telemetry clear`
- **Server**: We plan to keep aggregated metrics for 90 days, then delete raw events

### Q: Who has access to telemetry data?

**A:** Only the core caro development team. We don't share, sell, or monetize your usage data.

### Q: What happens if telemetry breaks?

**A:** Caro continues working normally. Telemetry is designed to fail silently - if the database is corrupted, permissions are wrong, or anything fails, caro just skips telemetry and continues.

### Q: Can I opt back in after opting out?

**A:** Yes! Just enable it again:
```bash
caro config set telemetry.enabled true
```

---

## Transparency Commitment

We believe in transparency:

1. **Open Source**: Our telemetry code is public - review it anytime
2. **Local First**: All data stored locally by default
3. **User Control**: Easy opt-out, export, and deletion
4. **Privacy by Design**: No PII collection, anonymous IDs, multi-layer validation
5. **Documentation**: Clear explanation of what we collect and why

If you have questions or concerns, please [open an issue](https://github.com/kenseehart/caro/issues) or email privacy@caro.sh.

---

## Technical Details

For developers interested in the implementation:

- **Storage**: SQLite database with `events` table
- **Collection**: Async non-blocking with fire-and-forget emission
- **Upload**: Batch worker (1 hour intervals) for enabled, non-air-gapped mode
- **Privacy**: Regex-based pattern detection + pre-storage/pre-transmission validation
- **Session IDs**: SHA256(machine_id + date)[0:16] - changes daily
- **Overhead**: Target <5ms startup overhead

See [ARCHITECTURE.md](./ARCHITECTURE.md) for detailed implementation notes.

---

## Changes in This Version

### v1.1.0-beta (Beta Release)
- ✨ Initial telemetry implementation
- 📊 SessionStart and CommandGeneration events
- 🔒 Privacy-first design with multi-layer validation
- 🛠️ Full CLI management (show, export, clear, status)
- 🔐 Air-gapped mode support
- ⚙️ Opt-out by default for beta users

### v1.1.0 (GA Release - Planned)
- 🔄 Switch to opt-in by default
- 📈 Additional event types (SafetyValidation, BackendError, SessionEnd)
- 🌐 Backend telemetry portal launch
- 📊 Public metrics dashboard

---

## Thank You

By allowing anonymous telemetry, you're helping us build a better caro for everyone. We take your privacy seriously and are committed to transparency.

Questions? Reach out on [GitHub Discussions](https://github.com/kenseehart/caro/discussions) or email support@caro.sh.
