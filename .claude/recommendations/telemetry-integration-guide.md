# Telemetry Integration Guide

**Status**: Infrastructure Complete, Event Emission Pending
**Date**: 2026-01-08

---

## Overview

The telemetry infrastructure is **fully implemented and functional**:
- ✅ Event types defined
- ✅ SQLite storage operational
- ✅ Privacy validation active
- ✅ CLI commands working
- ✅ Main.rs initialization complete
- ✅ SessionStart events emitting

**Remaining**: Wire event emissions into agent, safety, and backend components.

---

## Architecture Decision: Global Telemetry Collector

To avoid passing `Arc<TelemetryCollector>` through every component signature, we should use one of these patterns:

### Option 1: Thread-Local Global (Recommended)
```rust
// In src/telemetry/mod.rs
use std::cell::RefCell;

thread_local! {
    static TELEMETRY_COLLECTOR: RefCell<Option<Arc<TelemetryCollector>>> = RefCell::new(None);
}

pub fn set_global_collector(collector: Arc<TelemetryCollector>) {
    TELEMETRY_COLLECTOR.with(|c| {
        *c.borrow_mut() = Some(collector);
    });
}

pub fn emit_event(event_type: EventType) {
    TELEMETRY_COLLECTOR.with(|c| {
        if let Some(ref collector) = *c.borrow() {
            collector.emit(event_type);
        }
    });
}
```

**Usage in main.rs**:
```rust
// After creating telemetry_collector
caro::telemetry::set_global_collector(telemetry_collector.clone());
```

**Usage in components**:
```rust
// No signature changes needed!
caro::telemetry::emit_event(EventType::CommandGeneration { ... });
```

### Option 2: Pass Through Component Constructors
```rust
// Modify struct signatures
pub struct AgentLoop {
    backend: Arc<dyn CommandGenerator>,
    static_matcher: Option<StaticMatcher>,
    context: ExecutionContext,
    telemetry: Option<Arc<TelemetryCollector>>, // Add this
    // ...
}

impl AgentLoop {
    pub fn new(
        backend: Arc<dyn CommandGenerator>,
        context: ExecutionContext,
        telemetry: Option<Arc<TelemetryCollector>>, // Add this
    ) -> Self {
        // ...
    }
}
```

**Pros**: Explicit dependencies, testable
**Cons**: Changes many signatures, more refactoring

---

## Integration Points

### 1. CommandGeneration Events

**Location**: `src/agent/mod.rs:63-118` (AgentLoop::generate_command)

**What to track**:
- Backend used (static vs embedded)
- Duration (already tracking with `start`)
- Success/failure
- Error category on failure

**Emit points**:

```rust
// SUCCESS - Static matcher (line 74-80)
match matcher.generate_command(&request).await {
    Ok(command) => {
        info!("Static matcher found match in {:?}: {}", start.elapsed(), command.command);

        // EMIT EVENT HERE
        telemetry::emit_event(EventType::CommandGeneration {
            backend: "static".to_string(),
            duration_ms: start.elapsed().as_millis() as u64,
            success: true,
            error_category: None,
        });

        return Ok(command);
    }
}

// SUCCESS - LLM backend (line 116-117)
info!("Command generation complete in {:?}", start.elapsed());

// EMIT EVENT HERE (before Ok(refined))
telemetry::emit_event(EventType::CommandGeneration {
    backend: "embedded".to_string(),
    duration_ms: start.elapsed().as_millis() as u64,
    success: true,
    error_category: None,
});

Ok(refined)

// FAILURE - Any Err() return
// Add .map_err() wrapper to emit on error
```

**Error categories**:
- "timeout" - exceeded time limit
- "parse_error" - JSON parsing failed
- "backend_error" - LLM generation failed
- "validation_error" - command validation failed

---

### 2. SafetyValidation Events

**Location**: `src/safety/mod.rs`

Need to find where safety validation happens and which methods return validation results.

**What to track**:
- Risk level detected (low/medium/high/critical)
- Action taken (allowed/warned/blocked)
- Pattern category matched (optional)

**Emit points**:
```rust
// After validation decision
telemetry::emit_event(EventType::SafetyValidation {
    risk_level: format!("{:?}", risk_level),
    action_taken: if blocked { "blocked" } else if warned { "warned" } else { "allowed" },
    pattern_category: Some("destructive_rm".to_string()),
});
```

**Pattern categories** (from safety patterns):
- "destructive_rm" - rm -rf patterns
- "pipe_to_shell" - curl | sh patterns
- "privilege_escalation" - sudo/su abuse
- "data_exfiltration" - sending data externally
- "recursive_operations" - recursive delete/modify

---

### 3. BackendError Events

**Locations**:
- `src/backends/embedded/embedded_backend.rs`
- `src/backends/static_matcher.rs`
- Any other backend implementations

**What to track**:
- Which backend failed
- Error category
- Whether recoverable

**Emit points**:
```rust
// In backend generate_command() error paths
match model.generate(...).await {
    Err(e) => {
        telemetry::emit_event(EventType::BackendError {
            backend: "embedded".to_string(),
            error_category: categorize_error(&e),
            recoverable: is_recoverable(&e),
        });
        return Err(e);
    }
}
```

**Error categories**:
- "model_load_failed" - could not load ML model
- "inference_failed" - model inference error
- "timeout" - operation timed out
- "parse_error" - output parsing failed
- "oom" - out of memory
- "not_found" - model/resource not found

**Recoverable vs Fatal**:
- Recoverable: timeout, parse error, inference failed (can retry or fallback)
- Fatal: model not found, OOM, model load failed

---

## Implementation Steps

### Step 1: Add Global Telemetry API (30 min)

**File**: `src/telemetry/mod.rs`

```rust
use std::sync::OnceLock;
use std::sync::Arc;

static GLOBAL_COLLECTOR: OnceLock<Arc<TelemetryCollector>> = OnceLock::new();

/// Set the global telemetry collector (call once at startup)
pub fn set_global_collector(collector: Arc<TelemetryCollector>) {
    let _ = GLOBAL_COLLECTOR.set(collector);
}

/// Emit a telemetry event using the global collector
pub fn emit_event(event_type: EventType) {
    if let Some(collector) = GLOBAL_COLLECTOR.get() {
        collector.emit(event_type);
    }
}
```

**File**: `src/main.rs` (line ~560)

```rust
let telemetry_collector = std::sync::Arc::new(
    caro::TelemetryCollector::new(telemetry_storage.clone(), user_config.telemetry.enabled)
);

// Set as global collector
caro::telemetry::set_global_collector(telemetry_collector.clone());
```

### Step 2: Emit CommandGeneration Events (45 min)

**File**: `src/agent/mod.rs`

Add telemetry emissions at 3 points:
1. Static matcher success (line ~77)
2. LLM generation success (line ~116)
3. Any error paths (wrap with .map_err())

### Step 3: Emit SafetyValidation Events (45 min)

**File**: `src/safety/mod.rs`

1. Find validation decision points
2. Add telemetry emission after each decision
3. Include risk level, action, and pattern category

### Step 4: Emit BackendError Events (30 min)

**Files**:
- `src/backends/embedded/embedded_backend.rs`
- `src/backends/static_matcher.rs`

Add telemetry emissions in error paths with proper categorization.

---

## Testing Strategy

### Unit Tests

Each integration point should have a test:

```rust
#[tokio::test]
async fn test_command_generation_emits_telemetry() {
    // Setup telemetry collector
    let storage = TelemetryStorage::new(":memory:").unwrap();
    let collector = Arc::new(TelemetryCollector::new(Arc::new(storage), true));
    caro::telemetry::set_global_collector(collector);

    // Execute command generation
    let agent = AgentLoop::new(...);
    let _ = agent.generate_command("list files").await;

    // Verify event was emitted
    let events = storage.get_pending_events(10).await.unwrap();
    assert!(events.iter().any(|e| matches!(e.event_type, EventType::CommandGeneration { .. })));
}
```

### Integration Test

End-to-end test from main.rs:

```bash
# Enable telemetry
caro config set telemetry.enabled true

# Generate a command (should emit CommandGeneration)
caro "list files"

# Check blocked command (should emit SafetyValidation)
caro "rm -rf /"

# View telemetry
caro telemetry show
```

---

## Rollout Plan

### Phase A: Global API (30 min) - **Do First**
- Add global collector API
- Update main.rs to set global collector
- Test with existing SessionStart events

### Phase B: Agent Integration (45 min)
- Add CommandGeneration emissions to agent
- Test success and failure paths
- Verify event data accuracy

### Phase C: Safety Integration (45 min)
- Map safety validation points
- Add SafetyValidation emissions
- Test with known patterns

### Phase D: Backend Integration (30 min)
- Add BackendError emissions
- Test error categorization
- Verify recoverability flags

### Phase E: End-to-End Testing (30 min)
- Run full CLI workflows
- Check telemetry with `caro telemetry show`
- Export and inspect JSON
- Verify privacy validation

**Total**: ~3 hours (with buffer)

---

## Privacy Checklist

Before marking complete, verify:

- [ ] No commands appear in telemetry events
- [ ] No file paths in event data
- [ ] No environment variables leaked
- [ ] Session IDs are anonymous
- [ ] Error messages sanitized (no user data)
- [ ] Test with `caro telemetry show` output

---

## Success Criteria

✅ **Phase 6 Complete When**:
1. Global telemetry API implemented
2. CommandGeneration events emit on success/failure
3. SafetyValidation events emit with correct risk levels
4. BackendError events emit with categorization
5. `caro telemetry show` displays all event types
6. Privacy validation passes (no PII in events)
7. All tests pass

---

## Alternative: Minimal Viable Integration

If time is constrained, implement just the **global API + CommandGeneration** events:

**Priority 1** (1.5h):
- Global telemetry API
- CommandGeneration events (success only)
- Basic testing

**Priority 2** (1h):
- CommandGeneration error events
- SafetyValidation events

**Priority 3** (30m):
- BackendError events

This allows telemetry to go live with the most valuable data (command generation metrics) while deferring safety and error telemetry to a future iteration.

---

## Next Steps

1. Implement global telemetry API (Step 1)
2. Test with existing SessionStart events
3. Add CommandGeneration emissions (Step 2)
4. Test end-to-end workflow
5. Document completion in telemetry-summary.md
