# Add Azure Foundry Backend Support

## Inspiration & Community Credit

Inspired by **Umi** from the **Vancouver Dev community** - thank you for championing this feature request and highlighting the need for enterprise AI infrastructure integration! 🙏

## Overview

Add official support for Azure Foundry as an additional inference backend, enabling IT organizations already using Azure's AI infrastructure to seamlessly integrate `cmdai` with their existing Azure Foundry deployments.

## Product Requirements

### User Story

**As an** enterprise developer or IT administrator using Azure Foundry,
**I want to** configure `cmdai` to use my organization's Azure Foundry models,
**So that** I can generate safe shell commands using my company's approved AI infrastructure without managing separate model deployments.

### Key Features

#### 1. Azure Foundry Backend Implementation

- New backend type: `AzureFoundryBackend` implementing the `CommandGenerator` trait
- Support for Azure Foundry's REST API endpoints
- Model selection from available Azure Foundry catalog
- Standard Azure authentication integration (API keys, managed identities)
- Health check and availability detection
- Fallback to embedded backend on connection failure

#### 2. Interactive Setup Flow

When users choose Azure Foundry during initial configuration, provide an interactive onboarding experience:

```bash
$ cmdai --setup-backend azure-foundry

Welcome to Azure Foundry Setup for cmdai!
==========================================

This wizard will help you connect to your Azure Foundry deployment.

? Enter your Azure Foundry endpoint URL: https://my-foundry.eastus.inference.ml.azure.com
? Choose authentication method:
  > API Key
    Azure Managed Identity
    Azure CLI credentials

? Enter your API key: ****************************************
✓ Testing connection... Success!

? Select a default model:
  > gpt-4o (Available)
    gpt-4-turbo (Available)
    claude-3-5-sonnet (Available)
    llama-3-70b-instruct (Available)

? Set safety level:
    Strict (Block high-risk commands)
  > Moderate (Confirm high-risk commands)
    Permissive (Allow with confirmation)

✓ Configuration saved to ~/.config/cmdai/config.toml
✓ You're all set! Try: cmdai "list all files"
```

#### 3. Configuration Schema

Extend the existing `UserConfiguration` structure in `config.toml`:

```toml
[backend]
type = "azure-foundry"
endpoint = "https://my-foundry.eastus.inference.ml.azure.com"
api_key = "your-api-key"  # Or use environment variable
model = "gpt-4o"
timeout_seconds = 30
max_retries = 3

[backend.azure]
# Optional: Azure-specific settings
subscription_id = "..."
resource_group = "..."
deployment_name = "..."
api_version = "2024-05-01-preview"
use_managed_identity = false
```

#### 4. Environment Variable Support

```bash
export CMDAI_AZURE_FOUNDRY_ENDPOINT="https://..."
export CMDAI_AZURE_FOUNDRY_API_KEY="..."
export CMDAI_AZURE_FOUNDRY_MODEL="gpt-4o"
```

#### 5. Model Discovery & Selection

- API call to list available models from Azure Foundry
- Display model capabilities (context window, pricing tier)
- Allow runtime model override: `cmdai --model claude-3-5-sonnet "deploy kubernetes pod"`
- Cache model list locally to reduce API calls

## Technical Implementation

### Architecture Pattern

Follow the existing remote backend pattern (similar to `vllm.rs` and `ollama.rs`):

```
src/backends/remote/
├── vllm.rs              # Existing
├── ollama.rs            # Existing
└── azure_foundry.rs     # New - Azure Foundry backend
```

### API Integration

**Azure Foundry REST API Structure:**

```
Endpoint: https://<endpoint-name>.<region>.inference.ml.azure.com/v1/chat/completions
Method: POST
Headers:
  - Authorization: Bearer <api-key>
  - Content-Type: application/json
  - api-version: 2024-05-01-preview

Request Body:
{
  "messages": [
    {"role": "system", "content": "<cmdai-system-prompt>"},
    {"role": "user", "content": "list all python files"}
  ],
  "model": "gpt-4o",
  "temperature": 0.1,
  "max_tokens": 100,
  "response_format": {"type": "json_object"}
}

Response:
{
  "id": "chatcmpl-...",
  "object": "chat.completion",
  "created": 1234567890,
  "model": "gpt-4o",
  "choices": [{
    "message": {
      "role": "assistant",
      "content": "{\"cmd\": \"find . -name '*.py'\"}"
    },
    "finish_reason": "stop"
  }],
  "usage": {...}
}
```

### Code Structure

```rust
// src/backends/remote/azure_foundry.rs

pub struct AzureFoundryBackend {
    client: Client,
    endpoint: Url,
    api_key: String,
    model: String,
    api_version: String,
    timeout: Duration,
    max_retries: u32,
}

impl AzureFoundryBackend {
    pub fn new(config: AzureFoundryConfig) -> Result<Self, GeneratorError>
    pub fn with_api_key(mut self, api_key: String) -> Self
    pub fn with_model(mut self, model: String) -> Self
    pub async fn list_available_models(&self) -> Result<Vec<ModelInfo>, GeneratorError>

    async fn check_health(&self) -> Result<bool, GeneratorError>
    async fn parse_azure_response(&self, response: Response) -> Result<GeneratedCommand, GeneratorError>
}

#[async_trait]
impl CommandGenerator for AzureFoundryBackend {
    async fn generate_command(&self, request: &CommandRequest) -> Result<GeneratedCommand, GeneratorError>
    async fn is_available(&self) -> bool
    fn backend_info(&self) -> BackendInfo
    async fn shutdown(&self) -> Result<(), GeneratorError>
}
```

### Authentication Methods

**Phase 1: API Key (MVP)**
- Bearer token authentication in Authorization header
- Support for environment variables
- Secure storage recommendations

**Phase 2: Advanced Auth (Future)**
- Azure Managed Identity support (`azure_identity` crate)
- Azure CLI credential chain
- Service principal authentication

### Error Handling

Handle Azure-specific errors:
- `401 Unauthorized`: Invalid API key
- `403 Forbidden`: Insufficient permissions
- `404 Not Found`: Invalid endpoint or model
- `429 Too Many Requests`: Rate limiting (with retry logic)
- `500+ Server Errors`: Azure service issues (fallback to embedded)

### Testing Strategy

1. **Unit Tests:**
   - Configuration parsing and validation
   - Request/response serialization
   - Error handling scenarios
   - Mock Azure API responses

2. **Integration Tests:**
   - Live Azure Foundry connection (gated behind feature flag)
   - Model discovery and selection
   - End-to-end command generation
   - Fallback behavior validation

3. **Manual Testing Checklist:**
   - [ ] Setup wizard completes successfully
   - [ ] Configuration persists correctly
   - [ ] Command generation works with multiple models
   - [ ] Error messages are helpful and actionable
   - [ ] Fallback to embedded backend works on failure
   - [ ] Environment variables override config file

## Acceptance Criteria

- [ ] `AzureFoundryBackend` implements `CommandGenerator` trait
- [ ] Interactive setup flow (`cmdai --setup-backend azure-foundry`)
- [ ] Configuration schema includes Azure Foundry settings
- [ ] Environment variable support for all Azure settings
- [ ] Model discovery API integration
- [ ] Health check endpoint implementation
- [ ] Comprehensive error handling with Azure-specific messages
- [ ] Fallback to embedded backend on connection failure
- [ ] Unit test coverage ≥ 80%
- [ ] Integration tests with mock Azure API
- [ ] Documentation updates:
  - [ ] README.md with Azure Foundry setup instructions
  - [ ] CLAUDE.md with backend architecture details
  - [ ] Configuration examples in docs/
- [ ] CLI help text includes Azure Foundry options

## User Documentation Requirements

### README Section

```markdown
### Azure Foundry Backend

cmdai supports Azure Foundry for enterprise deployments:

**Interactive Setup:**
```bash
cmdai --setup-backend azure-foundry
```

**Manual Configuration:**

Edit `~/.config/cmdai/config.toml`:
```toml
[backend]
type = "azure-foundry"
endpoint = "https://your-deployment.region.inference.ml.azure.com"
api_key = "your-api-key"
model = "gpt-4o"
```

**Environment Variables:**
```bash
export CMDAI_AZURE_FOUNDRY_ENDPOINT="..."
export CMDAI_AZURE_FOUNDRY_API_KEY="..."
```

**List Available Models:**
```bash
cmdai --list-models
```
```

### Troubleshooting Guide

Common Azure Foundry issues and solutions:
- Authentication failures
- Model not found errors
- Connection timeouts
- Rate limiting

## Implementation Phases

### Phase 1: Core Backend (Week 1-2)
- Implement `AzureFoundryBackend` struct and trait
- Basic API integration (chat completions)
- Configuration schema and parsing
- Health check implementation

### Phase 2: Interactive Setup (Week 2-3)
- Setup wizard implementation using `dialoguer`
- Model discovery and listing
- Configuration validation and testing
- Error handling and user feedback

### Phase 3: Advanced Features (Week 3-4)
- Model metadata caching
- Retry logic and rate limiting
- Azure Managed Identity support (optional)
- Performance optimization

### Phase 4: Documentation & Testing (Week 4)
- Comprehensive documentation
- Integration test suite
- User guide with examples
- Community feedback integration

## Related Issues

- #XX - Backend extensibility improvements
- #XX - Configuration management enhancements
- #XX - Interactive setup wizard design

## Questions for Discussion

1. **Default Model:** Should we recommend specific Azure Foundry models for cmdai?
2. **Cost Optimization:** How should we handle token usage and cost awareness?
3. **Regional Endpoints:** Should we support multiple regional endpoints in one config?
4. **Caching:** Should we cache model responses for repeated requests?

## Community Involvement

We welcome contributions from the community! Areas where help is appreciated:
- Testing with real Azure Foundry deployments
- Documentation improvements
- Edge case identification
- Performance benchmarking

---

**Special Thanks:** This feature request is directly inspired by **Umi** from the **Vancouver Dev community**, whose advocacy for enterprise AI tooling integration helps make cmdai more accessible to organizations worldwide.

**Maintainer Notes:**
- [ ] Assign to milestone: v0.X.0
- [ ] Add labels: `enhancement`, `backend`, `enterprise`, `community-request`
- [ ] Create feature branch: `feature/azure-foundry-backend`
- [ ] Link to project board
