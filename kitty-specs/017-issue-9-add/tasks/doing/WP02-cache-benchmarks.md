---
work_package_id: "WP02"
subtasks: ["T006", "T007", "T008", "T009", "T010", "T011", "T012", "T013"]
title: "Cache Benchmarks"
phase: "Phase 1 - Benchmark Implementation"
lane: "doing"
assignee: ""
agent: "claude"
shell_pid: "1505"
review_status: ""
reviewed_by: ""
history:
  - timestamp: "2026-01-08T00:00:00Z"
    lane: "planned"
    agent: "system"
    shell_pid: ""
    action: "Prompt generated via /spec-kitty.tasks"
---

# Work Package Prompt: WP02 – Cache Benchmarks

## Objectives & Success Criteria

**Goal**: Implement FR1.1 cache operation benchmarks with statistical analysis.

**Success Criteria**:
- ✅ `cargo bench --bench cache` runs successfully
- ✅ HTML reports in `target/criterion/cache/`
- ✅ Four benchmarks: get_model (hit), add_model, remove_model, lru_eviction
- ✅ Performance ranges documented in code comments

## Context

**Spec**: FR1.1 Cache Operations - benchmark get, add, remove, LRU eviction  
**Plan**: `benches/cache.rs` with modular organization  
**Expected Performance**: cache hit 10-100ns, eviction <1μs

## Subtasks

### T006 – Create benches/cache.rs
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use caro::cache::Cache; // Adjust import path

fn bench_cache_get_model_hit(c: &mut Criterion) {
    // Implementation here
}

criterion_group!(benches, bench_cache_get_model_hit);
criterion_main!(benches);
```

### T007-T010 – Implement benchmarks
- **T007**: `cache_get_model_hit` - pre-warm cache, measure lookup
- **T008**: `cache_add_model` - measure insertion + eviction trigger  
- **T009**: `cache_remove_model` - measure deletion
- **T010**: `cache_lru_eviction_full` - fill cache, measure eviction

**Key Pattern**:
```rust
c.bench_function("cache/get_model", |b| {
    let cache = setup_cache(); // T011 fixture
    b.iter(|| black_box(cache.get("model-id")));
});
```

### T011 – Add fixtures
Create test models and pre-populated cache instances.

### T012 – Run locally
```bash
cargo bench --bench cache
open target/criterion/index.html
```

### T013 – Document ranges
Add comments with expected performance from research.md.

## Definition of Done
- [ ] All subtasks completed
- [ ] `cargo bench --bench cache` passes
- [ ] HTML report generated
- [ ] Performance within expected ranges

## Activity Log
- 2026-01-08T00:00:00Z – system – lane=planned – Prompt created
- 2026-01-08T13:32:42Z – claude – shell_pid=1505 – lane=doing – Started WP02 implementation
