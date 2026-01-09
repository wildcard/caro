# Implementation Plan: Property-Based Tests for LRU Cache

**Feature**: 016-issue-8-add
**Created**: 2026-01-08
**Status**: Planning

## Technical Context

### Existing Architecture
- Cache module at `src/cache/mod.rs` implements LRU eviction
- Current tests use example-based unit tests
- No property-based testing infrastructure exists

### Technology Stack
- **Testing**: PropTest for property-based testing
- **Language**: Rust with standard test framework
- **Integration**: Cargo test infrastructure

## Implementation Approach

### Phase 1: PropTest Setup
**Goal**: Add proptest dependency and basic infrastructure

**Tasks**:
1. Add `proptest = "1.4"` to `[dev-dependencies]` in `Cargo.toml`
2. Create test module structure in `src/cache/mod.rs` or `tests/cache_property_tests.rs`
3. Verify proptest integration with simple smoke test

**Validation**:
- `cargo test` includes proptest tests
- No dependency conflicts

### Phase 2: Eviction Order Properties
**Goal**: Verify LRU eviction order correctness

**Properties to Test**:
1. **Least Recently Used First**: When cache is full, next eviction removes item with oldest access time
2. **Access Updates Position**: Accessing an item moves it to "most recent" position
3. **Eviction Sequence**: Multiple evictions follow access history order

**Strategy Definition**:
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_lru_evicts_least_recent(
        operations in prop::collection::vec(cache_operation_strategy(), 10..100)
    ) {
        // Generate random cache operations
        // Verify eviction order matches LRU semantics
    }
}
```

**Implementation**:
- Define `cache_operation_strategy()` for random ops (add, get, evict)
- Build expected eviction sequence from operation history
- Compare actual evictions against expected LRU order

### Phase 3: Size Constraint Properties
**Goal**: Verify cache respects size limits

**Properties to Test**:
1. **Never Exceed Max Size**: `cache.len() <= cache.max_size()` always
2. **Eviction Before Overflow**: Adding item to full cache triggers eviction first
3. **Empty Cache Accepts Items**: Cache accepts items up to max_size when empty

**Strategy Definition**:
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
            assert!(cache.len() <= max_size);
        }
    }
}
```

### Phase 4: Access Pattern Properties
**Goal**: Verify access time tracking

**Properties to Test**:
1. **Access Updates Timestamp**: Getting item updates its access time
2. **Single Entry Per Item**: Multiple accesses maintain one entry
3. **Chronological Ordering**: Internal LRU list maintains time order

**Strategy Definition**:
```rust
proptest! {
    #[test]
    fn prop_access_updates_recency(
        initial_items in prop::collection::vec(any::<String>(), 5..20),
        access_sequence in prop::collection::vec(0usize..20, 10..50)
    ) {
        // Add initial items
        // Access items according to sequence
        // Verify most recently accessed not evicted
    }
}
```

### Phase 5: Edge Cases & Documentation
**Goal**: Comprehensive edge case coverage and documentation

**Edge Cases**:
- Single-item cache (max_size = 1)
- Empty cache operations
- Duplicate adds
- Access to non-existent items

**Documentation**:
- Add doc comments explaining each property
- Document strategy generation logic
- Add examples in test module header

## File Structure

```
src/cache/
├── mod.rs (existing)
└── #[cfg(test)] mod tests {
    mod property_tests {
        // PropTest implementations
    }
}
```

Alternative: Create separate file `tests/cache_property_tests.rs` for better organization.

## Testing Strategy

### Property Test Configuration
```rust
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    // 100 random test cases per property
}
```

### Test Execution
```bash
# Run all tests
cargo test

# Run only property tests
cargo test prop_

# Verbose output for debugging
PROPTEST_VERBOSE=1 cargo test prop_
```

### Performance Targets
- Total property test suite: < 30 seconds
- Individual property: < 5 seconds
- CI integration: No timeout issues

## Dependencies

### Required Changes
- `Cargo.toml`: Add proptest dev-dependency
- `src/cache/mod.rs` or `tests/cache_property_tests.rs`: New test module

### No Changes Required
- Cache implementation (unless bugs found)
- Public API
- Production dependencies

## Risks & Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| PropTest too slow for CI | Medium | Reduce case count in CI (50 vs 100 local) |
| Strategy too complex | Low | Start simple, iterate based on failures |
| Flaky tests from randomness | Medium | Set deterministic seed for CI |

## Acceptance Validation

**Automated Checks**:
- `cargo test` passes all tests
- `cargo clippy` reports no warnings
- `cargo fmt --check` passes

**Manual Review**:
- Test code is readable and well-documented
- Properties accurately represent LRU semantics
- Edge cases are covered

## Next Steps

After plan approval:
1. Run `/spec-kitty.tasks` to generate work packages
2. Execute `/spec-kitty.implement` to start implementation
3. Review with `/spec-kitty.review` after completion
4. Accept with `/spec-kitty.accept` when tests pass
