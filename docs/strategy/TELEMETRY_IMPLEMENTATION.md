# Telemetry Implementation Specification

> **Companion to:** TELEMETRY_STRATEGY.md
> **Purpose:** Concrete implementation using PostHog
> **Target:** v1.1.0 Beta Release

## Overview

This document specifies telemetry implementation using [PostHog](https://posthog.com) - a product analytics platform that handles infrastructure, dashboards, and privacy compliance.

**Why PostHog over custom infrastructure:**
- No backend to build or maintain
- Built-in dashboards, funnels, and cohorts
- Privacy-compliant (GDPR, SOC2)
- Feature flags for future A/B testing
- Generous free tier (1M events/month)

---

## 1. PostHog Configuration

### API Keys

```bash
# Production (US Cloud)
POSTHOG_API_KEY=phc_zQioEeKLXat4tLnI6sb0yFLRti8nff4ALAkNQAfRhME
POSTHOG_HOST=https://us.i.posthog.com
```

### Rust SDK Setup

Add to `Cargo.toml`:

```toml
[dependencies]
posthog-rs = "0.2"
```

---

## 2. Configuration Schema

### Addition to `src/config/mod.rs`

```rust
use serde::{Deserialize, Serialize};

/// Telemetry configuration section
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TelemetryConfig {
    /// Master switch for telemetry collection
    /// Default: true for beta, false for GA
    pub enabled: bool,

    /// Air-gapped mode: collect locally, export manually
    #[serde(default)]
    pub air_gapped: bool,

    /// Explicitly configured (set after first-run prompt)
    #[serde(default)]
    pub explicitly_configured: bool,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: cfg!(feature = "beta"), // true for beta builds
            air_gapped: false,
            explicitly_configured: false,
        }
    }
}
```

### Config File Example

```toml
# ~/.config/caro/config.toml

[telemetry]
enabled = true
air_gapped = false
```

### Environment Variable Overrides

| Variable | Type | Description |
|----------|------|-------------|
| `CARO_TELEMETRY_ENABLED` | bool | Override enabled state |
| `CARO_TELEMETRY_AIR_GAPPED` | bool | Override air-gapped mode |

---

## 3. PostHog Client Implementation

### File: `src/telemetry/mod.rs`

```rust
use once_cell::sync::OnceCell;
use posthog_rs::{Client, Event};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::{debug, warn};

static CLIENT: OnceCell<PostHogClient> = OnceCell::new();

const POSTHOG_API_KEY: &str = "phc_zQioEeKLXat4tLnI6sb0yFLRti8nff4ALAkNQAfRhME";

pub struct PostHogClient {
    client: Client,
    enabled: AtomicBool,
    distinct_id: String,
}

impl PostHogClient {
    pub fn new(enabled: bool) -> Self {
        let client = posthog_rs::client(POSTHOG_API_KEY);

        // Generate anonymous distinct_id (rotates daily for privacy)
        let distinct_id = generate_anonymous_id();

        Self {
            client,
            enabled: AtomicBool::new(enabled),
            distinct_id,
        }
    }

    /// Capture an event (non-blocking, never fails visibly)
    pub fn capture(&self, event_name: &str, properties: HashMap<String, serde_json::Value>) {
        if !self.enabled.load(Ordering::Relaxed) {
            return;
        }

        // Validate no sensitive data in properties
        if let Err(e) = validate_properties(&properties) {
            warn!("Telemetry blocked (sensitive data): {:?}", e);
            return;
        }

        let event = Event::new(event_name, &self.distinct_id)
            .insert_props(properties);

        // Fire and forget - don't block CLI
        if let Err(e) = self.client.capture(event) {
            debug!("PostHog capture failed: {:?}", e);
        }
    }

    /// Identify user with properties (for cohort analysis)
    pub fn identify(&self, properties: HashMap<String, serde_json::Value>) {
        if !self.enabled.load(Ordering::Relaxed) {
            return;
        }

        // PostHog identify call
        let _ = self.client.identify(&self.distinct_id, properties);
    }

    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    pub fn distinct_id(&self) -> &str {
        &self.distinct_id
    }
}

/// Initialize global PostHog client
pub fn init(enabled: bool) -> &'static PostHogClient {
    CLIENT.get_or_init(|| PostHogClient::new(enabled))
}

/// Get global client (panics if not initialized)
pub fn get() -> &'static PostHogClient {
    CLIENT.get().expect("Telemetry not initialized")
}

/// Try to get global client
pub fn try_get() -> Option<&'static PostHogClient> {
    CLIENT.get()
}

/// Generate anonymous ID that rotates daily
fn generate_anonymous_id() -> String {
    use sha2::{Sha256, Digest};

    // Get machine-specific identifier (stable across sessions)
    let machine_id = machine_uid::get()
        .unwrap_or_else(|_| "unknown".to_string());

    // Add date for daily rotation (privacy)
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    // Hash for anonymization
    let mut hasher = Sha256::new();
    hasher.update(format!("caro:{machine_id}:{today}"));
    let result = hasher.finalize();

    // Take first 16 chars of hex
    format!("{:x}", result)[..16].to_string()
}

/// Validate properties don't contain sensitive data
fn validate_properties(props: &HashMap<String, serde_json::Value>) -> Result<(), &'static str> {
    for (key, value) in props {
        if let serde_json::Value::String(s) = value {
            // Check for file paths
            if s.starts_with('/') || s.starts_with('~') || s.contains(":\\") {
                return Err("file path detected");
            }
            // Check for secrets
            if s.contains("password") || s.contains("secret") || s.contains("token") {
                return Err("secret detected");
            }
            // Check for commands
            if s.contains('|') || s.contains(';') || s.contains('`') {
                return Err("command detected");
            }
            // Length limit
            if s.len() > 200 {
                return Err("field too long");
            }
        }
    }
    Ok(())
}
```

---

## 4. Event Helpers

### File: `src/telemetry/events.rs`

```rust
use crate::telemetry;
use std::collections::HashMap;
use serde_json::json;

/// Capture session start
pub fn session_start(
    caro_version: &str,
    os: &str,
    arch: &str,
    shell: &str,
    backend: &str,
) {
    if let Some(client) = telemetry::try_get() {
        let mut props = HashMap::new();
        props.insert("caro_version".into(), json!(caro_version));
        props.insert("os".into(), json!(os));
        props.insert("arch".into(), json!(arch));
        props.insert("shell".into(), json!(shell));
        props.insert("backend".into(), json!(backend));
        props.insert("$lib".into(), json!("caro-cli"));

        client.capture("session_started", props);
    }
}

/// Capture session end
pub fn session_end(
    duration_ms: u64,
    commands_generated: u32,
    commands_executed: u32,
    errors: u32,
) {
    if let Some(client) = telemetry::try_get() {
        let mut props = HashMap::new();
        props.insert("duration_ms".into(), json!(duration_ms));
        props.insert("commands_generated".into(), json!(commands_generated));
        props.insert("commands_executed".into(), json!(commands_executed));
        props.insert("errors".into(), json!(errors));

        // Calculate success rate (north star metric)
        let success_rate = if commands_generated > 0 {
            (commands_executed as f64 / commands_generated as f64) * 100.0
        } else {
            0.0
        };
        props.insert("command_success_rate".into(), json!(success_rate));

        client.capture("session_ended", props);
    }
}

/// Capture command generation
pub fn command_generated(
    backend: &str,
    inference_time_ms: u64,
    risk_level: &str,
    patterns_matched: u32,
    model_name: Option<&str>,
) {
    if let Some(client) = telemetry::try_get() {
        let mut props = HashMap::new();
        props.insert("backend".into(), json!(backend));
        props.insert("inference_time_ms".into(), json!(inference_time_ms));
        props.insert("risk_level".into(), json!(risk_level));
        props.insert("patterns_matched".into(), json!(patterns_matched));

        if let Some(model) = model_name {
            props.insert("model_name".into(), json!(model));
        }

        client.capture("command_generated", props);
    }
}

/// Capture command execution
pub fn command_executed(
    execution_mode: &str,
    modified: bool,
    exit_category: &str,
    execution_time_ms: Option<u64>,
) {
    if let Some(client) = telemetry::try_get() {
        let mut props = HashMap::new();
        props.insert("execution_mode".into(), json!(execution_mode));
        props.insert("modified_before_execution".into(), json!(modified));
        props.insert("exit_category".into(), json!(exit_category));

        if let Some(time) = execution_time_ms {
            props.insert("execution_time_ms".into(), json!(time));
        }

        client.capture("command_executed", props);
    }
}

/// Capture safety validation trigger
pub fn safety_triggered(
    risk_level: &str,
    pattern_category: &str,
    action: &str,
    user_override: bool,
) {
    if let Some(client) = telemetry::try_get() {
        let mut props = HashMap::new();
        props.insert("risk_level".into(), json!(risk_level));
        props.insert("pattern_category".into(), json!(pattern_category));
        props.insert("action".into(), json!(action));
        props.insert("user_override".into(), json!(user_override));

        client.capture("safety_triggered", props);
    }
}

/// Capture error
pub fn error_occurred(
    category: &str,
    component: &str,
    recoverable: bool,
) {
    if let Some(client) = telemetry::try_get() {
        let mut props = HashMap::new();
        props.insert("error_category".into(), json!(category));
        props.insert("component".into(), json!(component));
        props.insert("recoverable".into(), json!(recoverable));

        client.capture("error_occurred", props);
    }
}
```

---

## 5. First-Run Consent Prompt

### File: `src/telemetry/first_run.rs`

```rust
use crate::config::Config;
use colored::Colorize;
use std::io::{self, Write};

/// Check if first run and show consent prompt
pub fn check_first_run(config: &mut Config) -> anyhow::Result<bool> {
    // Skip if already configured
    if config.telemetry.explicitly_configured {
        return Ok(config.telemetry.enabled);
    }

    // Non-interactive mode: default to disabled
    if !atty::is(atty::Stream::Stdin) {
        config.telemetry.enabled = false;
        config.telemetry.explicitly_configured = true;
        config.save()?;
        return Ok(false);
    }

    // Show prompt
    println!();
    println!("{}", "Welcome to Caro!".bold().green());
    println!();
    println!("Caro collects anonymous usage metrics to improve the product.");
    println!();
    println!("{}", "What we collect:".bold());
    println!("  {} Session timing and performance", "•".dimmed());
    println!("  {} Feature usage (which backends, safety levels)", "•".dimmed());
    println!("  {} Error categories (not details)", "•".dimmed());
    println!("  {} Platform info (OS, architecture)", "•".dimmed());
    println!();
    println!("{}", "What we NEVER collect:".bold());
    println!("  {} Your commands or inputs", "✗".red());
    println!("  {} File paths or directories", "✗".red());
    println!("  {} Any identifying information", "✗".red());
    println!();
    println!("Disable anytime: {}", "caro config set telemetry.enabled false".cyan());
    println!();

    print!("Enable telemetry to help improve Caro? [Y/n] ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let enabled = input.trim().is_empty() || input.trim().to_lowercase() == "y";

    config.telemetry.enabled = enabled;
    config.telemetry.explicitly_configured = true;
    config.save()?;

    if enabled {
        println!("{} Telemetry enabled. Thank you!", "✓".green());
    } else {
        println!("{} Telemetry disabled.", "✓".yellow());
    }
    println!();

    Ok(enabled)
}
```

---

## 6. CLI Integration

### In `src/main.rs`

```rust
use crate::telemetry::{self, events};
use std::time::Instant;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let session_start = Instant::now();
    let mut session_stats = SessionStats::default();

    // Load config
    let mut config = Config::load()?;

    // Check for --no-telemetry flag
    let telemetry_enabled = if args.no_telemetry {
        false
    } else {
        telemetry::first_run::check_first_run(&mut config)?
    };

    // Initialize PostHog
    if telemetry_enabled && !config.telemetry.air_gapped {
        telemetry::init(true);

        // Capture session start
        events::session_start(
            env!("CARGO_PKG_VERSION"),
            std::env::consts::OS,
            std::env::consts::ARCH,
            &detect_shell(),
            &config.backend.primary,
        );
    }

    // Run CLI
    let result = run_cli(args, &mut session_stats).await;

    // Capture session end
    if let Some(_) = telemetry::try_get() {
        events::session_end(
            session_start.elapsed().as_millis() as u64,
            session_stats.commands_generated,
            session_stats.commands_executed,
            session_stats.errors,
        );
    }

    result
}

#[derive(Default)]
struct SessionStats {
    commands_generated: u32,
    commands_executed: u32,
    errors: u32,
}
```

### In command generation

```rust
use crate::telemetry::events;

pub async fn generate_command(input: &str, config: &Config) -> anyhow::Result<GeneratedCommand> {
    let start = Instant::now();

    let result = backend.generate(input).await;
    let elapsed = start.elapsed();

    match &result {
        Ok(cmd) => {
            events::command_generated(
                backend.name(),
                elapsed.as_millis() as u64,
                &cmd.risk_level.to_string(),
                cmd.patterns_matched as u32,
                Some(backend.model_name()),
            );
        }
        Err(_) => {
            events::error_occurred("generation", "backend", true);
        }
    }

    result
}
```

---

## 7. Air-Gapped Mode

For users without network access, events are stored locally for manual export:

### File: `src/telemetry/offline.rs`

```rust
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;

const MAX_OFFLINE_EVENTS: usize = 1000;

/// Offline event queue for air-gapped environments
pub struct OfflineQueue {
    events: Vec<serde_json::Value>,
    path: PathBuf,
}

impl OfflineQueue {
    pub fn new() -> Self {
        let path = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("caro")
            .join("telemetry-offline.json");

        // Load existing events
        let events = if path.exists() {
            fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        Self { events, path }
    }

    pub fn add(&mut self, event: serde_json::Value) {
        if self.events.len() >= MAX_OFFLINE_EVENTS {
            self.events.remove(0); // Drop oldest
        }
        self.events.push(event);
        self.save();
    }

    pub fn export(&self, output: &PathBuf) -> anyhow::Result<usize> {
        let count = self.events.len();

        let file = File::create(output)?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, &self.events)?;
        writer.flush()?;

        Ok(count)
    }

    pub fn clear(&mut self) {
        self.events.clear();
        let _ = fs::remove_file(&self.path);
    }

    fn save(&self) {
        if let Some(parent) = self.path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(file) = File::create(&self.path) {
            let _ = serde_json::to_writer(file, &self.events);
        }
    }
}
```

### CLI Commands

```bash
# Export offline events
caro telemetry export -o telemetry-export.json

# Clear offline queue
caro telemetry clear

# Show status
caro telemetry status
```

---

## 8. PostHog Dashboard Setup

### Events to Create

| Event Name | Properties | Purpose |
|------------|------------|---------|
| `session_started` | version, os, arch, shell, backend | User journey start |
| `session_ended` | duration_ms, commands_*, success_rate | Session summary |
| `command_generated` | backend, inference_time_ms, risk_level | Core metric |
| `command_executed` | execution_mode, modified, exit_category | Success tracking |
| `safety_triggered` | risk_level, pattern_category, action | Safety calibration |
| `error_occurred` | category, component, recoverable | Reliability |

### Key Insights to Create

1. **Command Success Rate (North Star)**
   - Formula: `command_executed / command_generated`
   - Trend over time

2. **Inference Performance**
   - P50, P95 of `inference_time_ms`
   - By backend

3. **Safety Effectiveness**
   - Block rate = `safety_triggered` / `command_generated`
   - False positive rate = `user_override=true` / `safety_triggered`

4. **Platform Distribution**
   - Breakdown by OS, arch, shell

5. **Error Rate**
   - By category and component

### Funnels to Create

1. **Command Flow**
   - session_started → command_generated → command_executed

2. **Safety Flow**
   - command_generated → safety_triggered → (blocked | allowed)

---

## 9. Website Integration (Astro)

### Install PostHog

```bash
npm install posthog-js
```

### Create Component: `src/components/PostHog.astro`

```astro
---
// PostHog analytics component
---

<script is:inline>
  // Only run in browser
  if (typeof window !== 'undefined') {
    // Check if already initialized
    if (!window.posthog) {
      !function(t,e){var o,n,p,r;e.__SV||(window.posthog=e,e._i=[],e.init=function(i,s,a){function g(t,e){var o=e.split(".");2==o.length&&(t=t[o[0]],e=o[1]),t[e]=function(){t.push([e].concat(Array.prototype.slice.call(arguments,0)))}}(p=t.createElement("script")).type="text/javascript",p.async=!0,p.src=s.api_host+"/static/array.js",(r=t.getElementsByTagName("script")[0]).parentNode.insertBefore(p,r);var u=e;for(void 0!==a?u=e[a]=[]:a="posthog",u.people=u.people||[],u.toString=function(t){var e="posthog";return"posthog"!==a&&(e+="."+a),t||(e+=" (stub)"),e},u.people.toString=function(){return u.toString(1)+".people (stub)"},o="capture identify alias people.set people.set_once set_config register register_once unregister opt_out_capturing has_opted_out_capturing opt_in_capturing reset isFeatureEnabled onFeatureFlags getFeatureFlag getFeatureFlagPayload reloadFeatureFlags group updateEarlyAccessFeatureEnrollment getEarlyAccessFeatures getActiveMatchingSurveys getSurveys onSessionId".split(" "),n=0;n<o.length;n++)g(u,o[n]);e._i.push([i,s,a])},e.__SV=1)}(document,window.posthog||[]);

      posthog.init('phc_zQioEeKLXat4tLnI6sb0yFLRti8nff4ALAkNQAfRhME', {
        api_host: 'https://us.i.posthog.com',
        person_profiles: 'identified_only',
        capture_pageview: true,
        capture_pageleave: true,
      });
    }
  }
</script>
```

### Add to Layout: `src/layouts/BaseLayout.astro`

```astro
---
import PostHog from '../components/PostHog.astro';
---

<html>
  <head>
    <!-- ... other head content -->
  </head>
  <body>
    <slot />
    <PostHog />
  </body>
</html>
```

### Track Custom Events

```astro
<script>
  // Track download clicks
  document.querySelectorAll('[data-track-download]').forEach(el => {
    el.addEventListener('click', () => {
      posthog.capture('download_clicked', {
        platform: el.dataset.platform,
        version: el.dataset.version,
      });
    });
  });

  // Track CTA clicks
  document.querySelectorAll('[data-track-cta]').forEach(el => {
    el.addEventListener('click', () => {
      posthog.capture('cta_clicked', {
        cta_name: el.dataset.trackCta,
        location: el.dataset.location,
      });
    });
  });
</script>
```

---

## 10. Dependencies Summary

### Rust CLI (`Cargo.toml`)

```toml
[dependencies]
posthog-rs = "0.2"
machine-uid = "0.3"
sha2 = "0.10"
chrono = { version = "0.4", features = ["serde"] }
once_cell = "1.19"
atty = "0.2"
```

### Website (`package.json`)

```json
{
  "dependencies": {
    "posthog-js": "^1.96.0"
  }
}
```

---

## 11. Rollout Checklist

### Week 1: Core Implementation
- [ ] Add `posthog-rs` to Cargo.toml
- [ ] Implement `src/telemetry/mod.rs` with PostHog client
- [ ] Implement event helpers (`events.rs`)
- [ ] Implement first-run prompt (`first_run.rs`)
- [ ] Add `--no-telemetry` CLI flag
- [ ] Unit tests

### Week 2: Integration
- [ ] Wire up session start/end in `main.rs`
- [ ] Wire up command events in generation flow
- [ ] Wire up safety events in validation
- [ ] Wire up error events
- [ ] Integration tests

### Week 3: Website + Polish
- [ ] Add PostHog to Astro website
- [ ] Set up PostHog dashboards
- [ ] Create key insights and funnels
- [ ] Documentation updates
- [ ] Air-gapped mode testing

### Week 4: Beta Release
- [ ] Enable `beta` feature flag
- [ ] Update privacy policy
- [ ] Release notes with telemetry section
- [ ] Monitor initial data

---

## Sources

- [PostHog Rust SDK](https://posthog.com/docs/libraries/rust)
- [PostHog Astro Integration](https://posthog.com/docs/libraries/astro)
- [How to set up Rust analytics](https://posthog.com/tutorials/rust-analytics)
- [GitHub: posthog-rs](https://github.com/PostHog/posthog-rs)
