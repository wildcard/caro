# cmdai Cloud & Enterprise Architecture

> **Technical blueprint for scaling from CLI tool to cloud platform**

This document outlines the technical architecture for cmdai's cloud and enterprise features, designed for community contributors and engineering teams.

---

## Table of Contents

1. [Current Architecture (MVP/V1.0)](#current-architecture-mvpv10)
2. [Cloud Architecture (V2.0)](#cloud-architecture-v20)
3. [Enterprise Architecture (V2.5)](#enterprise-architecture-v25)
4. [Platform Architecture (V3.0)](#platform-architecture-v30)
5. [Infrastructure & Deployment](#infrastructure--deployment)
6. [Security & Compliance](#security--compliance)
7. [Data Architecture](#data-architecture)
8. [API Design](#api-design)

---

## Current Architecture (MVP/V1.0)

### System Overview

```
┌─────────────────────────────────────────────────────────────┐
│                      cmdai CLI Binary                        │
│  ┌────────────┐  ┌─────────────┐  ┌──────────────────────┐ │
│  │    CLI     │→ │   Safety    │→ │    Backends          │ │
│  │  (clap)    │  │  Validator  │  │ ┌─────────────────┐  │ │
│  └────────────┘  └─────────────┘  │ │  Embedded (MLX) │  │ │
│                                    │ │  Embedded (CPU) │  │ │
│                                    │ │  Ollama         │  │ │
│                                    │ │  vLLM           │  │ │
│                                    │ └─────────────────┘  │ │
│                                    └──────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
         │                    │                    │
         ▼                    ▼                    ▼
   Local Config        Local Cache          Local Execution
  (~/.config/cmdai)  (~/.cache/cmdai)     (user's shell)
```

### Module Structure

```
src/
├── main.rs                 # CLI entry point
├── lib.rs                  # Library exports
├── cli/                    # Command-line interface
│   └── mod.rs
├── backends/               # Inference backends
│   ├── mod.rs             # Backend trait
│   ├── embedded/          # Local inference
│   │   ├── mlx.rs        # Apple Silicon
│   │   └── cpu.rs        # Cross-platform
│   └── remote/           # Remote inference
│       ├── ollama.rs
│       └── vllm.rs
├── safety/                # Command validation
│   └── mod.rs
├── cache/                 # Model caching
│   ├── mod.rs
│   └── manifest.rs
├── config/                # Configuration
│   └── mod.rs
├── execution/             # Command execution
│   └── mod.rs
├── logging/               # Structured logging
│   └── mod.rs
└── models/                # Data types
    └── mod.rs
```

### Data Flow (Current)

```
User Input → CLI Parser → Safety Check → Backend Selection →
→ LLM Inference → Response Parse → Safety Revalidation →
→ User Confirmation → Command Execution
```

---

## Cloud Architecture (V2.0)

### System Overview

```
┌──────────────────────────────────────────────────────────────────┐
│                         cmdai Ecosystem                          │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────────┐                 ┌─────────────────────┐   │
│  │   CLI Client    │                 │   Cloud Backend     │   │
│  │  (Open Source)  │◄──── HTTPS ────►│     (SaaS)          │   │
│  └─────────────────┘                 └─────────────────────┘   │
│         │                                      │                │
│         │ (Local execution)                    │                │
│         │                                      ▼                │
│         │                             ┌──────────────────┐     │
│         │                             │   PostgreSQL     │     │
│         │                             │  (User data,     │     │
│         │                             │   commands,      │     │
│         │                             │   history)       │     │
│         │                             └──────────────────┘     │
│         │                                      │                │
│         │                                      ▼                │
│         │                             ┌──────────────────┐     │
│         └────────────────────────────►│   Redis Cache    │     │
│                                       │  (Sessions,       │     │
│                                       │   rate limits)    │     │
│                                       └──────────────────┘     │
└──────────────────────────────────────────────────────────────────┘
```

### Cloud Backend Architecture

```rust
// New module structure for cloud features
src/
├── cloud/                     # Cloud integration
│   ├── mod.rs                # Cloud client
│   ├── api_client.rs         # HTTP API client
│   ├── auth.rs               # Authentication (JWT)
│   ├── sync.rs               # Command history sync
│   └── teams.rs              # Team management
│
├── server/                    # Cloud backend (new crate)
│   ├── main.rs               # Server entry point
│   ├── api/                  # REST API
│   │   ├── auth.rs          # Login, API keys
│   │   ├── commands.rs      # Command generation
│   │   ├── teams.rs         # Team management
│   │   └── history.rs       # Command history
│   ├── services/            # Business logic
│   │   ├── inference.rs     # LLM orchestration
│   │   ├── safety.rs        # Cloud-side validation
│   │   └── analytics.rs     # Usage tracking
│   ├── db/                  # Database layer
│   │   ├── models.rs        # Diesel ORM models
│   │   ├── schema.rs        # DB schema
│   │   └── migrations/      # SQL migrations
│   └── middleware/          # HTTP middleware
│       ├── auth.rs          # JWT verification
│       ├── rate_limit.rs    # Rate limiting
│       └── logging.rs       # Request logging
```

### API Endpoints (V2.0)

```
Authentication:
  POST   /api/v1/auth/login              # Email/password login
  POST   /api/v1/auth/signup             # Create account
  POST   /api/v1/auth/logout             # Invalidate session
  GET    /api/v1/auth/me                 # Current user info
  POST   /api/v1/auth/api-keys           # Generate API key
  DELETE /api/v1/auth/api-keys/:id       # Revoke API key

Commands:
  POST   /api/v1/commands/generate       # Generate command
  GET    /api/v1/commands/history        # Command history
  DELETE /api/v1/commands/history/:id    # Delete from history
  POST   /api/v1/commands/execute        # Execute command (future)

Teams:
  POST   /api/v1/teams                   # Create team
  GET    /api/v1/teams/:id               # Team details
  POST   /api/v1/teams/:id/members       # Invite member
  DELETE /api/v1/teams/:id/members/:uid  # Remove member
  GET    /api/v1/teams/:id/patterns      # Shared patterns
  POST   /api/v1/teams/:id/patterns      # Share pattern

Analytics:
  GET    /api/v1/analytics/usage         # Usage stats
  GET    /api/v1/analytics/patterns      # Popular patterns
```

### Database Schema (PostgreSQL)

```sql
-- Users and authentication
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    full_name VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    key_hash VARCHAR(255) NOT NULL,
    name VARCHAR(255),
    last_used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ
);

-- Teams and membership
CREATE TABLE teams (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    owner_id UUID REFERENCES users(id) ON DELETE CASCADE,
    plan_tier VARCHAR(50) NOT NULL DEFAULT 'free',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE team_members (
    team_id UUID REFERENCES teams(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    role VARCHAR(50) NOT NULL DEFAULT 'member',
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (team_id, user_id)
);

-- Commands and history
CREATE TABLE commands (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    team_id UUID REFERENCES teams(id) ON DELETE SET NULL,
    prompt TEXT NOT NULL,
    generated_command TEXT NOT NULL,
    backend VARCHAR(50),
    risk_level VARCHAR(50),
    executed BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_commands_user_created ON commands(user_id, created_at DESC);
CREATE INDEX idx_commands_team_created ON commands(team_id, created_at DESC);

-- Shared patterns
CREATE TABLE shared_patterns (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id UUID REFERENCES teams(id) ON DELETE CASCADE,
    created_by UUID REFERENCES users(id) ON DELETE SET NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    prompt TEXT NOT NULL,
    command_template TEXT NOT NULL,
    usage_count INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Usage analytics
CREATE TABLE usage_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    event_type VARCHAR(50) NOT NULL,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_usage_events_user_created ON usage_events(user_id, created_at DESC);
CREATE INDEX idx_usage_events_type ON usage_events(event_type);
```

### Tech Stack (Cloud Backend)

**Backend Framework**: Axum (Rust async web framework)
- Fast, type-safe, low-level control
- Native Rust (no need to learn new language)
- Excellent async support with Tokio

**Database**: PostgreSQL (via Supabase or RDS)
- JSONB for flexible schema
- Full-text search
- Reliable, battle-tested

**ORM**: Diesel or SQLx
- Type-safe queries
- Migration management
- Async support (SQLx)

**Authentication**: jsonwebtoken crate
- JWT tokens for stateless auth
- API keys for CLI authentication

**Caching**: Redis
- Rate limiting (by user/API key)
- Session storage
- Hot data caching

**Hosting**: Fly.io or Railway
- Easy Rust deployment
- Global edge network
- Auto-scaling
- Cost-effective for early stage

---

## Enterprise Architecture (V2.5)

### Additional Components

```
┌─────────────────────────────────────────────────────────────────┐
│                    Enterprise Features                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────────┐        ┌────────────────────────────┐   │
│  │  Audit Log Store │        │   SSO Integration          │   │
│  │  (Immutable)     │        │  (Okta, Azure AD, Google)  │   │
│  └──────────────────┘        └────────────────────────────┘   │
│           │                              │                     │
│           ▼                              ▼                     │
│  ┌──────────────────┐        ┌────────────────────────────┐   │
│  │  SIEM Export     │        │   RBAC Policy Engine       │   │
│  │  (Splunk, etc)   │        │   (OPA or custom)          │   │
│  └──────────────────┘        └────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Enterprise Module Structure

```rust
src/
├── enterprise/                # Enterprise features
│   ├── mod.rs
│   ├── audit_log.rs          # Immutable logging
│   ├── compliance.rs         # SOC 2, HIPAA controls
│   ├── export.rs             # SIEM integration
│   ├── rbac.rs               # Role-based access
│   ├── policies.rs           # Policy engine
│   └── sso.rs                # SAML, OIDC
│
├── self_hosted/              # Self-hosted deployment
│   ├── docker/               # Docker configs
│   ├── kubernetes/           # K8s manifests
│   └── scripts/              # Setup scripts
```

### Audit Log Schema

```sql
-- Immutable audit logs (append-only)
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    user_id UUID,
    event_type VARCHAR(100) NOT NULL,
    resource_type VARCHAR(100),
    resource_id UUID,
    action VARCHAR(100) NOT NULL,
    status VARCHAR(50) NOT NULL,
    metadata JSONB NOT NULL,
    ip_address INET,
    user_agent TEXT,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- Tamper detection
    previous_hash VARCHAR(64),
    event_hash VARCHAR(64) NOT NULL
);

-- Prevent updates and deletes
CREATE RULE audit_logs_no_update AS ON UPDATE TO audit_logs DO INSTEAD NOTHING;
CREATE RULE audit_logs_no_delete AS ON DELETE TO audit_logs DO INSTEAD NOTHING;

CREATE INDEX idx_audit_logs_org_timestamp ON audit_logs(organization_id, timestamp DESC);
CREATE INDEX idx_audit_logs_user_timestamp ON audit_logs(user_id, timestamp DESC);
CREATE INDEX idx_audit_logs_event_type ON audit_logs(event_type);
```

### RBAC System

```rust
// Role definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Role {
    Admin,       // Full access
    Engineer,    // Execute all safe commands
    Junior,      // Execute only approved patterns
    ReadOnly,    // View history, no execution
}

// Permission model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub action: Action,
    pub resource: Resource,
    pub conditions: Vec<Condition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    ExecuteCommand,
    ApproveCommand,
    ViewHistory,
    ManageTeam,
    ConfigurePolicy,
}

// Policy example
pub struct Policy {
    pub name: String,
    pub rules: Vec<Rule>,
}

pub struct Rule {
    pub if_condition: Condition,
    pub then_action: PolicyAction,
}

// Example policy: "Block all rm commands for Junior role"
let policy = Policy {
    name: "Restrict Dangerous Commands for Juniors".to_string(),
    rules: vec![
        Rule {
            if_condition: Condition::And(vec![
                Condition::UserHasRole(Role::Junior),
                Condition::CommandMatches(regex!("rm -rf")),
            ]),
            then_action: PolicyAction::Deny {
                reason: "Junior engineers cannot execute rm -rf commands".to_string(),
            },
        },
    ],
};
```

### SSO Integration (SAML 2.0)

```rust
use saml2::*;

pub struct SsoConfig {
    pub provider: SsoProvider,
    pub entity_id: String,
    pub sso_url: String,
    pub certificate: String,
}

pub enum SsoProvider {
    Okta,
    AzureAD,
    Google,
    Generic,
}

impl SsoService {
    pub async fn initiate_login(&self, relay_state: &str) -> Result<String> {
        // Generate SAML AuthnRequest
        let authn_request = self.build_authn_request(relay_state)?;

        // Redirect URL
        Ok(format!(
            "{}?SAMLRequest={}",
            self.config.sso_url,
            base64::encode(&authn_request)
        ))
    }

    pub async fn handle_callback(&self, saml_response: &str) -> Result<User> {
        // Verify SAML response
        let response = self.verify_response(saml_response)?;

        // Extract user attributes
        let email = response.get_attribute("email")?;
        let name = response.get_attribute("name")?;

        // Create or update user
        self.db.upsert_user(email, name).await
    }
}
```

---

## Platform Architecture (V3.0)

### Workflow Engine

```
┌─────────────────────────────────────────────────────────────┐
│                      Workflow Engine                        │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────┐     ┌───────────────┐     ┌──────────┐  │
│  │   Triggers   │────►│  DAG Executor │────►│  Actions │  │
│  │              │     │               │     │          │  │
│  │ - Webhook    │     │ - Parallel    │     │ - Cmd    │  │
│  │ - Schedule   │     │ - Sequential  │     │ - HTTP   │  │
│  │ - Manual     │     │ - Conditional │     │ - Script │  │
│  └──────────────┘     └───────────────┘     └──────────┘  │
│                              │                             │
│                              ▼                             │
│                       ┌─────────────┐                      │
│                       │  Execution  │                      │
│                       │   History   │                      │
│                       └─────────────┘                      │
└─────────────────────────────────────────────────────────────┘
```

### Workflow DSL (YAML)

```yaml
name: deploy-nextjs-to-aws
description: Deploy Next.js application to AWS ECS
version: 1.0.0
author: community

triggers:
  - type: webhook
    path: /deploy
  - type: schedule
    cron: "0 2 * * *"  # Daily at 2 AM

parameters:
  - name: app_name
    type: string
    required: true
  - name: environment
    type: string
    enum: [staging, production]
    default: staging

steps:
  - name: build_docker_image
    action: command
    command: "docker build -t ${app_name}:latest ."
    on_failure: rollback

  - name: push_to_ecr
    action: command
    command: "docker push ${ECR_REGISTRY}/${app_name}:latest"
    requires: [build_docker_image]

  - name: update_ecs_task
    action: aws
    service: ecs
    operation: update-service
    parameters:
      cluster: ${environment}
      service: ${app_name}
      force-new-deployment: true
    requires: [push_to_ecr]

  - name: run_migrations
    action: command
    command: "npm run migrate"
    requires: [update_ecs_task]

  - name: health_check
    action: http
    url: "https://${app_name}-${environment}.cmdai.dev/health"
    expected_status: 200
    max_retries: 10
    retry_delay: 30
    requires: [run_migrations]

  - name: notify_slack
    action: integration
    integration: slack
    channel: "#deployments"
    message: "Deployed ${app_name} to ${environment}"
    on_failure: notify_failure
```

### Integration SDK

```rust
// Integration trait
#[async_trait]
pub trait Integration: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;

    async fn execute(&self, params: &IntegrationParams) -> Result<IntegrationResult>;
    async fn validate_config(&self, config: &serde_json::Value) -> Result<()>;
}

// Example: GitHub integration
pub struct GitHubIntegration {
    client: Octocrab,
}

#[async_trait]
impl Integration for GitHubIntegration {
    fn name(&self) -> &str {
        "github"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    async fn execute(&self, params: &IntegrationParams) -> Result<IntegrationResult> {
        match params.action.as_str() {
            "create_issue" => {
                let title = params.get_string("title")?;
                let body = params.get_string("body")?;
                let repo = params.get_string("repo")?;

                let issue = self.client
                    .issues(params.owner, repo)
                    .create(title)
                    .body(body)
                    .send()
                    .await?;

                Ok(IntegrationResult::success(json!({
                    "issue_number": issue.number,
                    "url": issue.html_url,
                })))
            },
            _ => Err(anyhow!("Unknown action: {}", params.action)),
        }
    }

    async fn validate_config(&self, config: &serde_json::Value) -> Result<()> {
        // Validate GitHub token, permissions
        let token = config.get("token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing GitHub token"))?;

        // Test authentication
        self.client.current().user().await?;

        Ok(())
    }
}
```

---

## Infrastructure & Deployment

### Production Stack

```
┌─────────────────────────────────────────────────────────────────┐
│                        Production Infrastructure                │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────┐      ┌─────────────┐      ┌──────────────┐  │
│  │   Cloudflare │─────►│   Fly.io    │─────►│  Supabase    │  │
│  │     (CDN)    │      │  (Backend)  │      │ (PostgreSQL) │  │
│  └──────────────┘      └─────────────┘      └──────────────┘  │
│                               │                      │          │
│                               ▼                      ▼          │
│                        ┌─────────────┐      ┌──────────────┐  │
│                        │   Redis     │      │   S3/R2      │  │
│                        │  (Upstash)  │      │ (File Store) │  │
│                        └─────────────┘      └──────────────┘  │
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │              Observability (DataDog / Grafana)           │  │
│  │  - Metrics: Response times, error rates, usage          │  │
│  │  - Logs: Structured JSON logs                           │  │
│  │  - Traces: Distributed tracing                          │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### Self-Hosted Deployment

**Docker Compose** (small teams):
```yaml
version: '3.8'

services:
  cmdai-backend:
    image: cmdai/backend:latest
    ports:
      - "8000:8000"
    environment:
      DATABASE_URL: postgres://user:pass@postgres:5432/cmdai
      REDIS_URL: redis://redis:6379
      JWT_SECRET: ${JWT_SECRET}
    depends_on:
      - postgres
      - redis

  postgres:
    image: postgres:15-alpine
    volumes:
      - pgdata:/var/lib/postgresql/data
    environment:
      POSTGRES_DB: cmdai
      POSTGRES_USER: user
      POSTGRES_PASSWORD: ${DB_PASSWORD}

  redis:
    image: redis:7-alpine
    volumes:
      - redisdata:/data

  web-ui:
    image: cmdai/web:latest
    ports:
      - "3000:3000"
    environment:
      API_URL: http://cmdai-backend:8000

volumes:
  pgdata:
  redisdata:
```

**Kubernetes** (enterprise):
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: cmdai-backend
spec:
  replicas: 3
  selector:
    matchLabels:
      app: cmdai-backend
  template:
    metadata:
      labels:
        app: cmdai-backend
    spec:
      containers:
      - name: backend
        image: cmdai/backend:latest
        ports:
        - containerPort: 8000
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: cmdai-secrets
              key: database-url
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8000
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8000
          initialDelaySeconds: 5
          periodSeconds: 5
```

---

## Security & Compliance

### Security Layers

1. **Network Security**
   - TLS 1.3 for all connections
   - Cloudflare DDoS protection
   - Rate limiting per user/IP

2. **Authentication & Authorization**
   - JWT tokens with short expiry (15 min)
   - Refresh tokens (7 days)
   - API keys with scoped permissions
   - SSO for enterprise

3. **Data Security**
   - Encryption at rest (AES-256)
   - Encryption in transit (TLS 1.3)
   - PII redaction in logs
   - Secrets management (HashiCorp Vault or AWS Secrets Manager)

4. **Application Security**
   - Input validation (all user input)
   - SQL injection prevention (parameterized queries)
   - XSS prevention (sanitized output)
   - CSRF protection (tokens)

### Compliance (SOC 2 Type II)

**Control Objectives**:
- **CC6.1**: Audit logs for all sensitive operations
- **CC6.2**: Encryption of data at rest and in transit
- **CC6.3**: Access control (RBAC, MFA)
- **CC7.2**: System monitoring and alerting
- **CC9.2**: Risk assessment and mitigation

**Implementation**:
```rust
// Audit logging middleware
pub async fn audit_middleware(
    State(db): State<Database>,
    user: AuthenticatedUser,
    req: Request,
    next: Next,
) -> Response {
    let start = Instant::now();

    // Execute request
    let response = next.run(req).await;

    // Log audit event
    let audit_event = AuditEvent {
        user_id: user.id,
        action: req.method().to_string(),
        resource: req.uri().path().to_string(),
        status: response.status().as_u16(),
        duration_ms: start.elapsed().as_millis() as i64,
        ip_address: extract_ip(&req),
        user_agent: extract_user_agent(&req),
    };

    db.log_audit_event(audit_event).await;

    response
}
```

---

## Data Architecture

### Data Retention Policies

**Cloud Tier**:
- Free: 7 days command history
- Pro: 30 days command history
- Team: 90 days command history

**Enterprise Tier**:
- Cloud: 1 year command history
- Self-hosted: Configurable (up to 7 years for financial services)

### Data Anonymization (for ML Training)

```rust
pub struct AnonymizedCommand {
    pub prompt_hash: String,           // SHA256(prompt)
    pub command_template: String,      // Anonymized command
    pub risk_level: RiskLevel,
    pub execution_success: bool,
    pub backend: String,
    pub timestamp: DateTime<Utc>,
}

impl AnonymizedCommand {
    pub fn from_command(cmd: &Command) -> Self {
        Self {
            prompt_hash: sha256(&cmd.prompt),
            command_template: Self::anonymize_command(&cmd.generated_command),
            risk_level: cmd.risk_level,
            execution_success: cmd.executed,
            backend: cmd.backend.clone(),
            timestamp: cmd.created_at,
        }
    }

    fn anonymize_command(cmd: &str) -> String {
        // Replace file paths with placeholders
        let re_paths = Regex::new(r"/[\w/]+").unwrap();
        let cmd = re_paths.replace_all(cmd, "<PATH>");

        // Replace IP addresses
        let re_ips = Regex::new(r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}").unwrap();
        let cmd = re_ips.replace_all(&cmd, "<IP>");

        // Replace emails
        let re_emails = Regex::new(r"[\w\.-]+@[\w\.-]+").unwrap();
        let cmd = re_emails.replace_all(&cmd, "<EMAIL>");

        cmd.to_string()
    }
}
```

---

## API Design

### REST API Standards

**Versioning**: `/api/v1/`, `/api/v2/`

**Request/Response Format**: JSON

**Authentication**: Bearer token in `Authorization` header

**Error Responses**:
```json
{
  "error": {
    "code": "INVALID_API_KEY",
    "message": "The provided API key is invalid or expired",
    "details": {
      "api_key_id": "key_abc123"
    }
  }
}
```

**Rate Limiting**:
- Free tier: 50 requests/hour
- Pro tier: 1000 requests/hour
- Enterprise: Unlimited (with fair use)

**Response Headers**:
```
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 987
X-RateLimit-Reset: 1638360000
```

### GraphQL API (Future)

For complex queries (e.g., fetching team + members + history + analytics):

```graphql
query GetTeamDashboard($teamId: ID!) {
  team(id: $teamId) {
    id
    name
    members {
      id
      email
      role
    }
    commandHistory(limit: 50) {
      id
      prompt
      command
      executedAt
      executedBy {
        email
      }
    }
    analytics {
      totalCommands
      successRate
      topPatterns {
        prompt
        count
      }
    }
  }
}
```

---

## Migration Path

### MVP → Cloud (V1.0 → V2.0)

1. **Backend stays compatible**: Cloud is opt-in
2. **CLI changes**: Add `cmdai auth login` command
3. **Config changes**: Add `cloud.enabled = true` to config
4. **Data migration**: No data to migrate (fresh start)

### Cloud → Enterprise (V2.0 → V2.5)

1. **Feature flags**: Enable enterprise features for specific orgs
2. **Database migrations**: Add audit logs, RBAC tables
3. **SSO setup**: Configure SSO per organization
4. **Gradual rollout**: Pilot with 5 customers before GA

---

## Open Questions (Community Input Needed)

1. **Workflow Engine**: DAG-based (Airflow-style) or imperative (GitHub Actions-style)?
2. **Integration Marketplace**: Built-in SDK or webhook-based (Zapier-style)?
3. **Self-Hosted UI**: Full React dashboard or admin CLI?
4. **Model Fine-Tuning**: In-house or partner with Hugging Face/Replicate?

---

## Document Maintenance

**Owner**: Engineering team (community input welcome)
**Update cadence**: Per major release
**Last updated**: 2025-11-19

---

## Next Steps

1. Review this architecture with the community
2. Prototype cloud backend (Axum + Postgres + Redis)
3. Define API contracts for V2.0
4. Create GitHub issues for each component

---

**Questions?**
- Open a [Discussion](https://github.com/wildcard/cmdai/discussions)
- Comment on architecture issues (coming soon)

---

*Let's build this together.*
