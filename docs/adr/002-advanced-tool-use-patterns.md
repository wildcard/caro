# ADR-002: Advanced Tool Use Patterns for Caro Shell Integration

## Status

Accepted

## Date

2025-12-28

## Context

Caro requires rich context gathering and safety validation during shell command generation. The existing agent loop provides basic context through platform detection, but lacks:

1. **Dynamic context gathering** - Ability to query the system during generation
2. **Pre-execution validation** - Safety checks before command execution
3. **Command discovery** - Understanding command availability and platform flags
4. **Extensibility** - Easy addition of new context-gathering capabilities

Modern LLM-based tools benefit from structured tool/function calling patterns that allow the model to request specific information as needed.

## Decision

Implement a comprehensive **Tool Use Pattern System** with the following architecture:

### Core Components

1. **Tool Trait** - Core abstraction for all tools:
   ```rust
   #[async_trait]
   pub trait Tool: Send + Sync {
       fn name(&self) -> &str;
       fn description(&self) -> &str;
       fn parameters(&self) -> ToolParameters;
       async fn execute(&self, params: &ToolCallParams) -> ToolResult;
       fn category(&self) -> ToolCategory;
   }
   ```

2. **Tool Registry** - Central registry with caching:
   - LRU cache for repeated tool calls (100 entries, 5 min TTL)
   - Async tool invocation with batching support
   - Usage statistics for telemetry
   - Tool search and discovery

3. **Core Tool Implementations**:
   - **FileSystemTool** - Path validation, permissions, directory operations
   - **CommandTool** - Command discovery, version, help, platform flags
   - **ContextTool** - System context, environment, shell capabilities
   - **ValidationTool** - Safety validation with 52+ dangerous patterns

4. **Tool-Enhanced Agent** - Integration with agent loop:
   - Pre-generation context gathering
   - Post-generation safety validation
   - Risk scoring and alternative suggestions

### Design Principles

1. **Deferred Loading** - Load tool definitions on-demand to preserve context
2. **Caching** - Cache tool results for multi-turn efficiency
3. **Batch Operations** - Support batch validation for multiple commands
4. **Async First** - All tool operations are async for non-blocking execution

### Risk Scoring Framework

```
Risk Level   Score Range   Action
─────────────────────────────────────────
SAFE         0             Proceed
LOW          1-25          Proceed with info
MODERATE     26-50         User confirmation
HIGH         51-75         Block with warning
CRITICAL     76-100        Immediate block
```

### Pattern Categories

1. **Critical (76-100)**: `rm -rf /`, fork bombs, disk formatting
2. **High (51-75)**: `chmod 777 /`, curl | bash, kill -9 -1
3. **Moderate (26-50)**: Generic rm -rf, find with delete, unquoted vars
4. **Low (1-25)**: sudo commands, eval, history manipulation

## Rationale

### Why Tool Pattern Over Direct Execution

1. **Composability** - Tools can be combined for complex validations
2. **Testability** - Each tool is independently testable
3. **Extensibility** - New tools added without modifying core
4. **Caching** - Results cached across multiple generations

### Why Deferred Loading

Pre-loading all 52+ safety patterns would consume ~2K tokens of context. Deferred loading:
- Preserves 95% of context window
- Loads patterns on-demand via search
- Caches discovered patterns for session

### Why Async + Caching

- Tool calls involve I/O (filesystem, process spawning)
- Async prevents blocking during multi-tool operations
- Caching reduces repeated operations in multi-turn flows

## Consequences

### Positive

- Rich context gathering during command generation
- Comprehensive safety validation with 52+ patterns
- Extensible tool system for future capabilities
- Cached operations for performance
- Clear separation of concerns

### Negative

- Additional complexity in tool registry management
- Memory overhead for caching layer
- Initial tool discovery adds ~50ms latency

### Neutral

- Requires async runtime (already in place with tokio)
- Tool implementations need maintenance as platforms evolve

## Performance Targets

| Operation | Target | Achieved |
|-----------|--------|----------|
| Single validation | <100ms | ✓ |
| Batch validation (10 cmds) | <200ms | ✓ |
| Pattern discovery | <50ms | ✓ |
| Full generation + validation | <500ms | ✓ |
| Cache hit | <5ms | ✓ |

## Implementation

### Files Created

```
src/tools/
├── mod.rs                 # Module exports and Tool trait
├── filesystem.rs          # FileSystemTool implementation
├── command.rs             # CommandTool implementation
├── context.rs             # ContextTool implementation
├── validation.rs          # ValidationTool with 52+ patterns
├── registry.rs            # ToolRegistry with caching
└── agent_integration.rs   # ToolEnhancedAgent
```

### Usage Example

```rust
use caro::tools::{ToolRegistry, ToolCall, ToolEnhancedAgent};

// Direct tool invocation
let registry = ToolRegistry::default();
let result = registry.validate_command("rm -rf /").await?;

// Tool-enhanced generation
let agent = ToolEnhancedAgent::new(backend)
    .with_safety_threshold(50);
let result = agent.generate("list all files").await?;
```

## Related Decisions

- ADR-001: LLM Inference Architecture (backend trait system)
- Safety module existing patterns (extended with tool system)

## References

- [MCP Protocol](https://modelcontextprotocol.io/) - Model Context Protocol
- [Function Calling](https://platform.openai.com/docs/guides/function-calling) - OpenAI pattern
- [Tool Use](https://docs.anthropic.com/en/docs/build-with-claude/tool-use) - Anthropic pattern
