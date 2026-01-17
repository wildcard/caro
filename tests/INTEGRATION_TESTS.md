# Integration Test Suites

This document describes the integration test organization for feature-specific backends and services.

## Test Suite Organization

Integration tests are organized by feature flags to allow independent testing:

| Test Suite | Feature Flag | Purpose |
|------------|--------------|---------|
| `knowledge_integration.rs` | `knowledge` | Tests for LanceDB-based knowledge index |
| `chromadb_integration.rs` | `chromadb` | Tests for ChromaDB vector backend (future) |

## Running Tests

### Knowledge Index Tests

The knowledge integration tests use LanceDB and FastEmbed, which require model downloads on first run.

```bash
# Run all knowledge tests (requires knowledge feature)
cargo test --test knowledge_integration --features knowledge -- --ignored

# Run specific knowledge test
cargo test --test knowledge_integration --features knowledge test_record_and_search -- --ignored

# Run without ignored tests (will skip model-dependent tests)
cargo test --test knowledge_integration --features knowledge
```

**Note**: Tests are marked `#[ignore]` because they:
- Download embedding models (~100MB) on first run
- Require network access for model download
- Take longer to execute than unit tests

### ChromaDB Tests (Future)

When ChromaDB integration is implemented (#504, #519), tests will be organized similarly:

```bash
# Run ChromaDB tests (requires chromadb feature + server)
cargo test --test chromadb_integration --features chromadb -- --ignored

# Start ChromaDB server for testing
docker compose -f tests/docker-compose.yml up chromadb
```

## CI/CD Integration

Different test suites can run in separate CI jobs:

```yaml
# Example GitHub Actions workflow
jobs:
  knowledge-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run knowledge tests
        run: cargo test --test knowledge_integration --features knowledge -- --ignored

  chromadb-tests:
    runs-on: ubuntu-latest
    services:
      chromadb:
        image: chromadb/chromadb:latest
        ports:
          - 8000:8000
    steps:
      - uses: actions/checkout@v4
      - name: Run ChromaDB tests
        run: cargo test --test chromadb_integration --features chromadb -- --ignored
```

## Test Organization Principles

1. **Feature Flag Alignment**: Each test suite matches a feature flag
2. **Independent Execution**: Test suites can run independently without each other
3. **Service Isolation**: Tests requiring external services (databases, servers) are clearly marked
4. **Model Dependencies**: Tests requiring model downloads are marked `#[ignore]`

## Adding New Integration Tests

When adding a new feature with integration tests:

1. Create a new test file: `tests/<feature>_integration.rs`
2. Add feature flag guard: `#[cfg(feature = "feature_name")]`
3. Mark expensive tests with `#[ignore = "reason"]`
4. Document in this file

### Example Template

```rust
//! Feature Integration Tests
//!
//! Tests for <feature description>.
//! These tests require <dependencies> and are marked as `#[ignore]` by default.

#[cfg(feature = "feature_name")]
mod feature_tests {
    use caro::feature::Module;

    #[tokio::test]
    #[ignore = "requires external service"]
    async fn test_feature() {
        // Test logic
    }
}
```

## Related Documentation

- [Test Suite README](./README.md) - Setup script tests
- [QA Process Guide](../docs/QA_PROCESS.md) - QA workflow
- [Issue #520](https://github.com/wildcard/caro/issues/520) - Test suite separation
