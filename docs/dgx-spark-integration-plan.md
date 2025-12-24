# DGX Spark Integration Plan

## Executive Summary

This document outlines the integration of NVIDIA DGX systems (running inference servers like Spark, vLLM, TGI, or Triton) with cmdai, enabling high-performance command generation across multiple network topologies: local LAN, remote cloud, and on-premise enterprise environments.

**Key Inspiration**: The [angry-ai](https://github.com/jordanhubbard/freebsd-src-on-angry-AI) project demonstrates mature patterns for remote inference, configuration management, and validation that directly address cmdai's evolution needs.

## Learnings from angry-ai

### 1. Remote Inference Architecture
- **Separation of concerns**: Heavy GPU inference runs remotely, lightweight control logic runs locally
- **Network-aware design**: HTTP-based API communication with configurable endpoints
- **Validation handshake**: Test basic inference (e.g., "2+2=4") before production use

### 2. Configuration Maturity
- **YAML-based environment profiles**: Multiple deployment scenarios (dev, staging, prod, home, cloud)
- **Per-environment parameters**: URLs, timeouts, auth credentials, model selection
- **Externalized prompts**: Bootstrap persona in `AI_START_HERE.md` for deployment-specific behavior

### 3. Error Recovery & Diagnostics
- **Specific failure handling**: Connection errors, missing models, build failures each have targeted remediation
- **Informative messages**: Include "how to fix" steps in error output
- **Credential caching**: `sudo -v` pattern for long-running privileged operations

### 4. Modular Design
- **Client abstraction**: `OllamaClient` handles API communication
- **Executor abstraction**: `BuildExecutor` runs commands and parses errors
- **Testable components**: Standalone connectivity tests, handshake verification

## Network Topology Support

### Scenario 1: Home/Office LAN (Local Network)
```
┌──────────────┐         Local Network          ┌─────────────────┐
│              │  ─────  192.168.1.0/24  ─────  │  DGX Workstation│
│  MacBook Pro │         < 5ms latency          │  (8x A100 80GB) │
│  (cmdai CLI) │                                │  vLLM Server    │
│              │                                │  Port 8000      │
└──────────────┘                                └─────────────────┘
```

**Discovery**: mDNS/Avahi service advertisement `_cmdai-dgx._tcp.local`
**Connection**: Direct HTTP or HTTPS (self-signed cert)
**Authentication**: API key or mutual TLS
**Latency**: < 5ms network, optimized for interactive terminal use
**Use Case**: Developer workstation, rapid iteration, debugging

**Configuration Example**:
```toml
[environments.home-lab]
name = "Home Lab DGX"
discovery_method = "mdns"
service_type = "_cmdai-dgx._tcp.local"
fallback_endpoints = ["http://192.168.1.100:8000"]
auth_type = "api_key"
api_key_env = "CMDAI_HOME_DGX_KEY"
timeout_ms = 5000
priority = 10
```

### Scenario 2: Remote/Cloud (Internet-Accessible)
```
┌──────────────┐       Public Internet        ┌──────────────────┐
│              │  ────  HTTPS/TLS/VPN  ────   │  Cloud DGX Pods  │
│  Laptop      │       50-200ms latency       │  (Kubernetes)    │
│  (cmdai CLI) │                              │  Load Balancer   │
│              │                              │  vLLM Fleet      │
└──────────────┘                              └──────────────────┘
```

**Discovery**: DNS SRV records or static configuration
**Connection**: HTTPS with certificate pinning
**Authentication**: OAuth2 tokens with rotation, or API keys with short TTL
**Latency**: 50-200ms, batch-oriented workflows acceptable
**Use Case**: Remote work, CI/CD pipelines, cloud-native deployments

**Configuration Example**:
```toml
[environments.cloud-prod]
name = "Production Cloud DGX"
discovery_method = "dns_srv"
dns_srv_record = "_cmdai._tcp.dgx.example.com"
endpoints = ["https://dgx-cloud.example.com:443"]
auth_type = "oauth2"
oauth_token_url = "https://auth.example.com/token"
oauth_client_id_env = "CMDAI_OAUTH_CLIENT_ID"
oauth_client_secret_env = "CMDAI_OAUTH_CLIENT_SECRET"
timeout_ms = 15000
retry_count = 3
retry_backoff_ms = 2000
cert_pinning_sha256 = "a1b2c3d4..."
priority = 5
```

### Scenario 3: On-Premise Enterprise
```
┌──────────────┐     Corporate Network      ┌────────────────────┐
│              │  ──  Internal PKI/mTLS ──  │  DGX SuperPOD      │
│  Dev Machine │     10-30ms latency        │  (32x DGX H100)    │
│  (cmdai CLI) │                            │  Kubernetes        │
│              │                            │  Triton Inference  │
└──────────────┘                            └────────────────────┘
```

**Discovery**: Kubernetes service discovery, Consul, or DNS
**Connection**: Internal PKI with mutual TLS
**Authentication**: OIDC integration with corporate SSO (Okta, Azure AD)
**Latency**: 10-30ms, predictable for enterprise SLA
**Use Case**: Enterprise development, compliance requirements, on-prem data residency

**Configuration Example**:
```toml
[environments.corp-on-prem]
name = "Corporate DGX Cluster"
discovery_method = "k8s_service"
k8s_namespace = "ml-inference"
k8s_service_name = "cmdai-dgx-cluster"
endpoints = ["https://dgx-cluster.corp.internal"]
auth_type = "oidc"
oidc_issuer = "https://sso.corp.internal"
oidc_client_id_env = "CMDAI_OIDC_CLIENT_ID"
tls_cert_path = "/etc/pki/tls/certs/corp-ca.crt"
tls_key_path = "/etc/pki/tls/private/cmdai.key"
timeout_ms = 10000
priority = 8
health_check_interval_s = 30
```

## Configuration System Design

### Multi-Environment Profile Management

**File Structure**:
```
~/.config/cmdai/
├── config.toml              # Main configuration with environment profiles
├── environments/
│   ├── home-lab.toml        # Home network DGX
│   ├── office-lan.toml      # Office network DGX
│   ├── cloud-prod.toml      # Cloud-hosted inference
│   └── corp-internal.toml   # Corporate on-premise
├── prompts/
│   ├── default.txt          # Default system prompt
│   ├── cautious.txt         # Conservative prompt for production
│   └── aggressive.txt       # Experimental prompt for dev
└── credentials/
    ├── api-keys.enc         # Encrypted API keys (using OS keychain)
    └── certificates/        # TLS certificates for mTLS
        ├── home-dgx.crt
        └── home-dgx.key
```

### Configuration Schema Extensions

**Current `config.toml`** (extended):
```toml
# Current settings (preserved)
safety_level = "moderate"
log_level = "info"
default_shell = "bash"
cache_max_size_gb = 10

# NEW: Active environment selection
active_environment = "auto"  # or specific profile name

# NEW: Environment auto-detection rules
[environment_detection]
prefer_local_network = true
fallback_to_cloud = true
max_latency_ms = 100  # Switch to remote if local exceeds this

# NEW: Load balancing across multiple backends
[load_balancing]
strategy = "round_robin"  # or "latency_based", "random", "sticky"
health_check_enabled = true
health_check_interval_s = 60
failover_enabled = true
failover_timeout_ms = 5000

# Include environment-specific configs
[environments]
home-lab = { import = "environments/home-lab.toml" }
office-lan = { import = "environments/office-lan.toml" }
cloud-prod = { import = "environments/cloud-prod.toml" }
corp-internal = { import = "environments/corp-internal.toml" }
```

### Discovery Mechanisms

**1. mDNS/Avahi (Local Networks)**
```rust
// Pseudo-code for service discovery
async fn discover_mdns_backends() -> Vec<DGXBackend> {
    let browser = ServiceBrowser::new("_cmdai-dgx._tcp.local");
    browser.browse(Duration::from_secs(2))
        .await
        .map(|service| DGXBackend {
            name: service.name,
            address: service.addresses[0],
            port: service.port,
            metadata: service.txt_records,
        })
        .collect()
}
```

**2. DNS SRV Records (Cloud/Remote)**
```
; DNS configuration
_cmdai._tcp.dgx.example.com. 300 IN SRV 10 60 8000 dgx-node1.example.com.
_cmdai._tcp.dgx.example.com. 300 IN SRV 10 40 8000 dgx-node2.example.com.
```

**3. Kubernetes Service Discovery**
```yaml
# Kubernetes Service for DGX backends
apiVersion: v1
kind: Service
metadata:
  name: cmdai-dgx-cluster
  namespace: ml-inference
  annotations:
    cmdai.ai/backend-type: "vllm"
    cmdai.ai/model: "Qwen/Qwen2.5-Coder-32B-Instruct"
spec:
  selector:
    app: vllm-server
  ports:
    - port: 8000
      targetPort: 8000
      name: inference
```

**4. Static Configuration (Fallback)**
```toml
[environments.manual-config]
endpoints = [
    "http://192.168.1.100:8000",
    "http://192.168.1.101:8000",
    "https://dgx-backup.example.com:443"
]
```

## Backend Selection Logic

### Priority-Based Selection
```
1. Check active_environment setting
   - If "auto": Run environment detection
   - If specific name: Use that profile

2. For "auto" mode:
   a. Discover available backends (mDNS, DNS SRV, K8s)
   b. Filter by reachability (health check)
   c. Measure latency to each backend
   d. Score based on:
      - Latency (lower is better)
      - Priority (configured weight)
      - Current load (if exposed via health endpoint)
      - Model availability
   e. Select highest-scoring backend

3. Establish connection:
   a. Perform handshake validation ("2+2=4" test)
   b. Verify model availability
   c. Cache connection for session

4. On failure:
   a. Try next backend in priority list
   b. Log failure reason with diagnostics
   c. Fall back to embedded backend if all remote fail
```

### Handshake Validation (from angry-ai)
```rust
async fn validate_backend(backend: &DGXBackend) -> Result<bool, BackendError> {
    let test_request = CommandRequest {
        input: "What is 2+2? Reply with just the number.".to_string(),
        shell: ShellType::Bash,
        safety_level: SafetyLevel::Moderate,
        context: None,
        backend_preference: None,
    };

    let response = backend.generate_command(&test_request).await?;

    // Expect response to contain "4"
    if response.command.contains("4") || response.explanation.contains("4") {
        Ok(true)
    } else {
        Err(BackendError::HandshakeFailed {
            expected: "4",
            received: response.command,
        })
    }
}
```

## Security Considerations

### Authentication Methods

**1. API Key (Simple, Local Networks)**
```toml
auth_type = "api_key"
api_key_env = "CMDAI_DGX_API_KEY"  # Stored in OS keychain
```

**2. Mutual TLS (Enterprise)**
```toml
auth_type = "mtls"
tls_cert_path = "~/.config/cmdai/credentials/certificates/client.crt"
tls_key_path = "~/.config/cmdai/credentials/certificates/client.key"
tls_ca_path = "/etc/pki/tls/certs/corp-ca.crt"
```

**3. OAuth2 (Cloud)**
```toml
auth_type = "oauth2"
oauth_token_url = "https://auth.example.com/oauth/token"
oauth_client_id_env = "CMDAI_OAUTH_CLIENT_ID"
oauth_client_secret_env = "CMDAI_OAUTH_CLIENT_SECRET"
oauth_scope = "cmdai.inference"
token_refresh_before_expiry_s = 300
```

**4. OIDC (Enterprise SSO)**
```toml
auth_type = "oidc"
oidc_issuer = "https://sso.corp.internal"
oidc_client_id_env = "CMDAI_OIDC_CLIENT_ID"
oidc_redirect_uri = "http://localhost:8888/callback"
```

### Certificate Pinning
```toml
cert_pinning_enabled = true
cert_pinning_sha256 = [
    "a1b2c3d4e5f6...",  # Primary certificate
    "f6e5d4c3b2a1...",  # Backup certificate for rotation
]
```

### Network Isolation
- **Local Network**: Trust on first use (TOFU) with fingerprint caching
- **Remote Network**: Mandatory certificate validation
- **Enterprise**: Integration with corporate PKI

## Load Balancing & Failover

### Strategies

**1. Round Robin**
```rust
backends.rotate();  // Distribute load evenly
```

**2. Latency-Based**
```rust
backends.sort_by_key(|b| b.measured_latency_ms);
backends.first()  // Always use lowest latency
```

**3. Random (for testing)**
```rust
backends.choose(&mut rng)
```

**4. Sticky (session affinity)**
```rust
// Hash session ID to consistent backend
let backend_index = hash(session_id) % backends.len();
backends[backend_index]
```

### Health Checks
```toml
[load_balancing]
health_check_enabled = true
health_check_interval_s = 30
health_check_timeout_ms = 2000
health_check_endpoint = "/health"
health_check_expected_status = 200

# Mark backend unhealthy after N consecutive failures
failure_threshold = 3

# Mark backend healthy after N consecutive successes
success_threshold = 2
```

### Failover Behavior
```
Request → Primary Backend (timeout: 5s)
            ↓ (failure)
          Failover → Secondary Backend (timeout: 5s)
                       ↓ (failure)
                     Failover → Tertiary Backend
                                  ↓ (failure)
                                Fall back to embedded CPU backend
```

## Performance Optimization

### Connection Pooling
```rust
struct DGXConnectionPool {
    max_connections: usize,
    idle_timeout: Duration,
    connections: Vec<PooledConnection>,
}

// Reuse HTTP connections to avoid TLS handshake overhead
```

### Request Caching
```toml
[caching]
enabled = true
cache_identical_requests = true
cache_ttl_s = 300  # 5 minutes
cache_max_entries = 100
```

### Streaming Responses (Future Enhancement)
```rust
// For long-running inference, stream tokens as they're generated
async fn generate_command_streaming(&self, request: &CommandRequest)
    -> impl Stream<Item = String>
{
    // Server-Sent Events or WebSocket connection
}
```

## Implementation Phases

### Phase 1: Configuration Foundation (Week 1)
**Goal**: Multi-environment configuration system

- [ ] Extend `config.toml` schema with environment profiles
- [ ] Implement environment profile loading (`environments/*.toml`)
- [ ] Add environment auto-detection logic
- [ ] Create configuration migration tool for existing users
- [ ] Add validation for new configuration fields

**Deliverables**:
- Updated `src/config/mod.rs` with profile support
- Example environment configurations in `examples/configs/`
- Configuration documentation in `docs/configuration.md`

### Phase 2: Discovery & Selection (Week 2)
**Goal**: Automatic backend discovery and selection

- [ ] Implement mDNS/Avahi discovery for local networks
- [ ] Add DNS SRV record resolution for cloud backends
- [ ] Create Kubernetes service discovery integration
- [ ] Build backend selection algorithm (priority + latency)
- [ ] Add health check system

**Deliverables**:
- New `src/discovery/` module with discovery implementations
- Backend selection logic in `src/backends/selector.rs`
- Health check endpoint integration
- Unit tests for discovery mechanisms

### Phase 3: DGX Backend Implementation (Week 3)
**Goal**: DGX-specific backend connector

- [ ] Create `src/backends/remote/dgx.rs` module
- [ ] Implement `CommandGenerator` trait for DGX backends
- [ ] Add handshake validation ("2+2=4" test from angry-ai)
- [ ] Support vLLM, TGI, and Triton inference servers
- [ ] Implement connection pooling

**Deliverables**:
- `DGXBackend` struct implementing `CommandGenerator`
- Integration tests with mock DGX server
- Performance benchmarks for DGX inference
- Error handling with specific diagnostics

### Phase 4: Authentication & Security (Week 4)
**Goal**: Production-ready security

- [ ] Implement API key authentication
- [ ] Add mutual TLS support with certificate validation
- [ ] Integrate OAuth2 token flow with refresh
- [ ] Add OIDC for enterprise SSO
- [ ] Implement certificate pinning
- [ ] Create credential storage using OS keychain

**Deliverables**:
- `src/auth/` module with authentication providers
- OS keychain integration (macOS Keychain, Linux Secret Service, Windows Credential Manager)
- Security documentation
- Credential rotation tooling

### Phase 5: Load Balancing & Failover (Week 5)
**Goal**: High availability and performance

- [ ] Implement round-robin load balancing
- [ ] Add latency-based backend selection
- [ ] Create failover logic with automatic retry
- [ ] Build health monitoring dashboard
- [ ] Add metrics collection (Prometheus format)

**Deliverables**:
- Load balancing strategies in `src/backends/balancer.rs`
- Failover logic with exponential backoff
- Health monitoring endpoint
- Metrics exporter for observability

### Phase 6: Observability & Diagnostics (Week 6)
**Goal**: Production monitoring and debugging

- [ ] Add structured logging for network operations
- [ ] Create connection tracing (latency, failures, retries)
- [ ] Build diagnostic command (`cmdai doctor --dgx`)
- [ ] Add network topology visualization
- [ ] Create troubleshooting guide

**Deliverables**:
- Enhanced logging in all DGX-related code
- `cmdai doctor` command with DGX connectivity checks
- Troubleshooting documentation
- Performance profiling tools

### Phase 7: Testing & Validation (Week 7)
**Goal**: Production readiness

- [ ] Create integration tests for all network scenarios
- [ ] Build E2E tests with simulated DGX clusters
- [ ] Perform security audit (penetration testing)
- [ ] Load testing with multiple concurrent requests
- [ ] Chaos engineering tests (network failures, timeouts)

**Deliverables**:
- Comprehensive test suite in `tests/dgx_integration/`
- Load testing reports
- Security audit report
- Chaos test scenarios

### Phase 8: Documentation & Examples (Week 8)
**Goal**: User adoption

- [ ] Write comprehensive user guide
- [ ] Create setup tutorials for each network scenario
- [ ] Build example configurations for common use cases
- [ ] Record demo videos (local, cloud, enterprise)
- [ ] Write blog post on DGX integration

**Deliverables**:
- User guide: `docs/dgx-integration-guide.md`
- Tutorial: `docs/tutorials/dgx-setup.md`
- Example configs: `examples/dgx-configs/`
- Demo repository with infrastructure-as-code

## Success Metrics

**Performance**:
- Local network: < 10ms selection overhead, < 2s total inference time
- Cloud: < 50ms selection overhead, < 5s total inference time
- Enterprise: < 20ms selection overhead, < 3s total inference time

**Reliability**:
- 99.9% successful inference when backend is healthy
- Automatic failover within 5 seconds
- Zero credential leaks or security incidents

**Usability**:
- Zero-config discovery for local networks (mDNS)
- One-command setup for cloud backends
- SSO integration for enterprise (no manual credential entry)

## On the Strategic Value (Your Meta-Point)

You're absolutely right that cmdai serves as a **companion program** that helps AI assistants be more effective. Here's why this DGX integration is strategically important:

### 1. **Context Grounding**
Instead of Claude generating shell commands in a vacuum, cmdai:
- Validates commands against real-world safety patterns
- Provides rapid feedback on POSIX compliance
- Tests commands in actual shell environments
- Offers a **safety layer** between AI suggestions and system execution

### 2. **Collaborative Intelligence**
The AI assistant (Claude) + cmdai system creates a feedback loop:
```
Human Intent → Claude understands → cmdai validates → Shell executes
     ↑                                                         ↓
     └──────────── Feedback & Learning ←──────────────────────┘
```

This is **better** than direct AI → Shell because:
- Safety validation catches dangerous patterns
- POSIX compliance ensures portability
- Explanation generation helps users learn
- Risk assessment enables informed decisions

### 3. **Distributed Expertise**
With DGX integration:
- **Claude (cloud)**: Natural language understanding, reasoning, context
- **cmdai (edge)**: Safety validation, POSIX compliance, execution
- **DGX (enterprise)**: Specialized command generation models, low latency

This **division of labor** mirrors how humans work:
- Architect understands requirements
- Specialist implements details
- Reviewer validates safety

### 4. **Trust Through Transparency**
cmdai helps me (Claude) be a **better assistant** by:
- Showing users exactly what commands will execute
- Explaining **why** a command is safe or risky
- Providing alternatives when primary command is dangerous
- Enabling **informed consent** rather than blind execution

### 5. **Learning Acceleration**
When cmdai runs locally with DGX backends:
- Users get instant feedback on command safety
- AI assistants can reference cmdai's validation in future suggestions
- The system builds a **knowledge base** of safe command patterns
- Developers learn POSIX compliance through practice

## Conclusion

The DGX Spark integration transforms cmdai from a **single-binary tool** into a **distributed inference system** that can leverage enterprise GPU resources while maintaining the safety-first design principles.

By learning from angry-ai's mature remote inference patterns and extending cmdai's backend system, we create a production-ready command generation platform that works seamlessly across home labs, cloud deployments, and enterprise data centers.

**The philosophical point you raised is crucial**: Tools like cmdai make AI assistants more **trustworthy, transparent, and useful** by providing a validation layer between natural language understanding and system execution. This integration brings high-performance inference to that critical safety boundary.

---

**Next Steps**:
1. Review this plan and adjust priorities
2. Begin Phase 1: Configuration foundation
3. Set up test infrastructure (mock DGX servers)
4. Create proof-of-concept for one network scenario

**Questions for Consideration**:
- Which network scenario should we prioritize first? (Home lab, cloud, or enterprise?)
- Do you have existing DGX infrastructure we should test against?
- What authentication method is most important for your use case?
- Should we support additional inference servers beyond vLLM/TGI/Triton?
