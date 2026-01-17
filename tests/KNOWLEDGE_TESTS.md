# Knowledge System Integration Tests

Integration tests for Caro's knowledge system and vector database backends.

## Test Suites

### Knowledge Integration Tests (`knowledge_integration.rs`)

Tests the default **LanceDB backend** without requiring external services. Uses temporary directories for isolated testing.

**Run all tests:**
```bash
cargo test --features knowledge --test knowledge_integration
```

**Run specific test:**
```bash
cargo test --features knowledge test_lancedb_persistence
```

**Test Coverage:**
- ✅ LanceDB backend health checks
- ✅ Recording successful command executions
- ✅ Recording command corrections from agentic loops
- ✅ Semantic search with similarity scoring
- ✅ Database persistence across backend instances
- ✅ Context and metadata preservation
- ✅ Database clear/reset functionality
- ✅ Concurrent operations handling

**Tests (9 total):**
1. `test_lancedb_health` - Verify backend health check
2. `test_lancedb_record_success` - Record successful commands
3. `test_lancedb_record_correction` - Record command corrections
4. `test_lancedb_find_similar` - Semantic search functionality
5. `test_lancedb_clear` - Database clearing
6. `test_lancedb_multiple_operations` - Bulk operations
7. `test_lancedb_context_metadata` - Context preservation
8. `test_lancedb_persistence` - Data persistence verification

---

### ChromaDB Integration Tests (`chromadb_integration.rs`)

Tests the **ChromaDB server-based backend**.

**✅ WORKING** - ChromaDB 0.5.18 with chromadb-rs v2.3.0

#### Setup

1. **Start ChromaDB server:**
```bash
cd tests
docker-compose up -d
```

2. **Wait for startup:**
```bash
# Check server health
curl http://localhost:8000/api/v1/heartbeat
```

3. **Run tests:**
```bash
# Tests must run serially (--test-threads=1) to avoid collection interference
cargo test --features chromadb --test chromadb_integration -- --ignored --nocapture --test-threads=1
```

4. **Stop server:**
```bash
cd tests
docker-compose down

# To also remove data volumes
docker-compose down -v
```

**Test Coverage:**
- ✅ ChromaDB backend connection and health
- ✅ Recording successes and corrections
- ✅ Semantic search across entries
- ✅ Collection management (clear, stats)
- ✅ Multiple concurrent operations
- ✅ Context metadata preservation

**Tests (7 total, all passing when run serially):**
1. `test_chromadb_connection` - Verify server connectivity
2. `test_chromadb_record_success` - Record successful commands
3. `test_chromadb_record_correction` - Record corrections
4. `test_chromadb_find_similar` - Semantic search
5. `test_chromadb_clear` - Clear collection
6. `test_chromadb_multiple_operations` - Bulk operations
7. `test_chromadb_context_metadata` - Context preservation

#### Important Notes

**Version Compatibility** (Resolved #519)
- ChromaDB **0.5.18** is compatible with chromadb-rs **v2.3.0**
- ChromaDB 0.4.x versions don't support the `/api/v2` endpoints required by chromadb-rs v2.3.0
- No authentication configuration needed for local testing with 0.5.18

**Serial Test Execution Required**
- All tests use the same collection name (`caro_commands`)
- Tests must run with `--test-threads=1` to avoid interference
- Parallel test execution causes collection deletion conflicts
- Future improvement: Use unique collection names per test

---

## CI Integration

### Knowledge Tests (LanceDB)

Run in CI without external dependencies:

```yaml
- name: Run knowledge integration tests
  run: cargo test --features knowledge --test knowledge_integration
```

### ChromaDB Tests

Add to CI with ChromaDB service:

```yaml
services:
  chromadb:
    image: chromadb/chroma:0.5.18  # Must be 0.5.x for /api/v2 support
    ports:
      - 8000:8000
    env:
      IS_PERSISTENT: "TRUE"
      ALLOW_RESET: "true"

steps:
  - name: Run ChromaDB integration tests
    run: |
      # Tests must run serially to avoid collection interference
      cargo test --features chromadb --test chromadb_integration -- --ignored --test-threads=1
    env:
      CHROMADB_URL: http://localhost:8000
```

---

## Troubleshooting

### ChromaDB Connection Refused

**Problem:** Tests fail with connection errors.

**Solution:**
```bash
# Check if server is running
docker ps | grep chromadb

# Check server logs
docker logs caro-chromadb-test

# Verify port is accessible
curl http://localhost:8000/api/v1/heartbeat
```

### ChromaDB Port Conflict

**Problem:** Port 8000 already in use.

**Solution:** Modify `docker-compose.yml`:
```yaml
ports:
  - "8001:8000"  # Use different host port
```

Then set environment variable:
```bash
CHROMADB_URL=http://localhost:8001 cargo test --features chromadb ...
```

### LanceDB Permission Errors

**Problem:** Tests fail with "Permission denied" creating temp directory.

**Solution:** Check that `/tmp` is writable or set `TMPDIR`:
```bash
TMPDIR=/path/to/writable/dir cargo test --features knowledge ...
```

---

## Related Issues

- [#519](https://github.com/wildcard/caro/issues/519) - ChromaDB authentication blocking tests
- [#520](https://github.com/wildcard/caro/issues/520) - Separate test suites for knowledge and chromadb
