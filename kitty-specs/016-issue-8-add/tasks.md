# Work Packages: Property-Based Tests for LRU Cache

**Feature**: 016-issue-8-add
**Created**: 2026-01-08
**Total Work Packages**: 4

## Work Package Overview

| WP ID | Title | Subtasks | Priority | Status |
|-------|-------|----------|----------|--------|
| WP01 | Setup PropTest Infrastructure | 2 | High | Planned |
| WP02 | Eviction Order Properties | 3 | High | Planned |
| WP03 | Size & Access Pattern Properties | 3 | Medium | Planned |
| WP04 | Edge Cases & Documentation | 2 | Medium | Planned |

---

## WP01: Setup PropTest Infrastructure
**Priority**: High
**Goal**: Add proptest dependency and verify integration
**Dependencies**: None
**Parallelizable**: No (foundation for other WPs)

### Subtasks
- [ ] T001: Add proptest to Cargo.toml dev-dependencies
- [ ] T002: Create test module structure with smoke test

### Implementation Sketch
1. Edit `Cargo.toml`:
   ```toml
   [dev-dependencies]
   proptest = "1.4"
   ```
2. Add test module in `src/cache/mod.rs`:
   ```rust
   #[cfg(test)]
   mod property_tests {
       use super::*;
       use proptest::prelude::*;

       proptest! {
           #[test]
           fn smoke_test(x in 0..100i32) {
               assert!(x >= 0 && x < 100);
           }
       }
   }
   ```
3. Run `cargo test` to verify setup

### Success Criteria
- PropTest dependency resolves without conflicts
- Smoke test passes
- `cargo test prop_` runs successfully

### Risks
- Version conflicts with existing dependencies (Low probability)

---

## WP02: Eviction Order Properties
**Priority**: High
**Goal**: Verify LRU eviction semantics
**Dependencies**: WP01
**Parallelizable**: No (sequential after WP01)

### Subtasks
- [ ] T003: Implement `prop_lru_evicts_least_recent` test
- [ ] T004: Implement `prop_access_updates_position` test
- [ ] T005: Implement `prop_eviction_sequence_follows_history` test

### Implementation Sketch
```rust
proptest! {
    #[test]
    fn prop_lru_evicts_least_recent(
        max_size in 2usize..20,
        items in prop::collection::vec(any::<String>(), 10..50)
    ) {
        let mut cache = LruCache::new(max_size);
        let mut access_order = Vec::new();

        for item in items {
            cache.add(item.clone());
            access_order.push(item);

            // When full, verify least recent is evicted
            if cache.len() > max_size {
                let evicted = cache.last_evicted();
                assert_eq!(evicted, access_order.first());
                access_order.remove(0);
            }
        }
    }
}
```

### Success Criteria
- All three property tests pass with 100+ iterations
- Tests verify eviction order correctness
- No false positives or negatives

### Risks
- Cache API may not expose eviction history (need to mock or add test helpers)

---

## WP03: Size & Access Pattern Properties
**Priority**: Medium
**Goal**: Verify size constraints and access tracking
**Dependencies**: WP01
**Parallelizable**: Can run in parallel with WP02 (different test module)

### Subtasks
- [ ] T006: Implement `prop_cache_respects_size_limit` test
- [ ] T007: Implement `prop_eviction_before_overflow` test
- [ ] T008: Implement `prop_access_updates_timestamp` test

### Implementation Sketch
```rust
proptest! {
    #[test]
    fn prop_cache_respects_size_limit(
        max_size in 1usize..100,
        items in prop::collection::vec(any::<String>(), 0..200)
    ) {
        let mut cache = LruCache::new(max_size);
        for item in items {
            cache.add(item);
            assert!(cache.len() <= max_size,
                "Cache exceeded max_size: {} > {}", cache.len(), max_size);
        }
    }

    #[test]
    fn prop_access_updates_timestamp(
        initial_items in prop::collection::vec(any::<String>(), 5..20),
        access_idx in prop::collection::vec(0usize..20, 10..50)
    ) {
        let mut cache = LruCache::new(100);
        for item in &initial_items {
            cache.add(item.clone());
        }

        // Access items in random order
        for idx in access_idx {
            if idx < initial_items.len() {
                cache.get(&initial_items[idx]);
            }
        }

        // Fill cache to force eviction
        // Verify accessed items not evicted
    }
}
```

### Success Criteria
- Size constraint properties pass
- Access pattern properties verify timestamp updates
- Tests handle edge cases (empty cache, single item)

### Risks
- Need access to cache internals for timestamp verification

---

## WP04: Edge Cases & Documentation
**Priority**: Medium
**Goal**: Comprehensive coverage and maintainability
**Dependencies**: WP02, WP03
**Parallelizable**: No (final polish phase)

### Subtasks
- [ ] T009: Add edge case tests (single-item cache, empty operations)
- [ ] T010: Add documentation and examples

### Implementation Sketch
1. Edge case tests:
   ```rust
   proptest! {
       #[test]
       fn prop_single_item_cache(items in prop::collection::vec(any::<String>(), 1..50)) {
           let mut cache = LruCache::new(1);
           for item in items {
               cache.add(item);
               assert_eq!(cache.len(), 1);
           }
       }
   }
   ```

2. Documentation:
   ```rust
   /// Property-based tests for LRU cache eviction algorithm.
   ///
   /// These tests verify:
   /// - Eviction order follows LRU semantics
   /// - Cache size never exceeds max_size
   /// - Access operations update item recency
   ///
   /// Each property runs 100+ random test cases to explore edge cases.
   mod property_tests {
       // ...
   }
   ```

### Success Criteria
- Edge cases covered (empty cache, single item, duplicates)
- Doc comments explain each property
- Examples demonstrate usage
- All tests documented in module header

### Risks
- None

---

## Execution Order

### Phase 1: Foundation
1. WP01 (Setup) - **Must complete first**

### Phase 2: Core Properties
2. WP02 (Eviction Order) - **Sequential after WP01**
3. WP03 (Size & Access) - **Can run in parallel with WP02**

### Phase 3: Polish
4. WP04 (Edge Cases & Docs) - **After WP02 and WP03**

---

## Acceptance Checklist

Before marking feature complete:
- [ ] All work packages completed
- [ ] All property tests pass with 100+ iterations
- [ ] `cargo test` passes
- [ ] `cargo clippy` reports no warnings
- [ ] `cargo fmt --check` passes
- [ ] Test execution time < 30 seconds
- [ ] Documentation complete
- [ ] Code review approved

---

## Next Command

```bash
/spec-kitty.implement
```

This will start executing work packages in the planned order.
