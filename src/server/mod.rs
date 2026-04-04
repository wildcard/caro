//! HTTP API server for caro (cabinet integration).
//!
//! Exposes caro's command generation, safety validation, and execution
//! pipeline as a REST API for external consumers like cabinet agents.
//!
//! # Architecture
//!
//! The server wraps the existing `AgentLoop` and `SafetyValidator` in axum
//! route handlers. It is a separate binary target (`caro-server`), feature-gated
//! under the `server` feature flag.
//!
//! # Endpoints
//!
//! - `GET  /api/v1/health`   - Backend status and version
//! - `POST /api/v1/generate` - Generate command from natural language
//! - `POST /api/v1/execute`  - Execute a command with safety validation

pub mod routes;
pub mod types;

use crate::agent::AgentLoop;
use crate::models::ShellType;
use std::time::Instant;

/// Shared application state for all route handlers.
pub struct AppState {
    /// Agent loop for command generation (None if no backend available)
    pub agent_loop: Option<AgentLoop>,

    /// Default shell type for command execution
    pub shell_type: ShellType,

    /// Server start time (for uptime reporting)
    pub start_time: Instant,
}

/// Build the axum router with all API routes.
pub fn build_router(state: std::sync::Arc<AppState>) -> axum::Router {
    use axum::routing::{get, post};

    axum::Router::new()
        .route("/api/v1/health", get(routes::health))
        .route("/api/v1/generate", post(routes::generate))
        .route("/api/v1/execute", post(routes::execute))
        .with_state(state)
}

/// Build the router with bearer token authentication middleware.
pub fn build_router_with_auth(
    state: std::sync::Arc<AppState>,
    token: String,
) -> axum::Router {
    use axum::{
        extract::Request,
        http::StatusCode,
        middleware::{self, Next},
        response::Response,
        routing::{get, post},
    };

    let token = std::sync::Arc::new(token);

    // Auth middleware: check Authorization: Bearer <token>
    let auth_middleware = {
        let token = token.clone();
        move |req: Request, next: Next| {
            let token = token.clone();
            async move {
                // Health endpoint is always accessible
                if req.uri().path() == "/api/v1/health" {
                    return next.run(req).await;
                }

                let auth_header = req
                    .headers()
                    .get("authorization")
                    .and_then(|v| v.to_str().ok());

                match auth_header {
                    Some(header) if header.starts_with("Bearer ") => {
                        let provided = &header[7..];
                        if provided == token.as_str() {
                            next.run(req).await
                        } else {
                            Response::builder()
                                .status(StatusCode::UNAUTHORIZED)
                                .header("content-type", "application/json")
                                .body(axum::body::Body::from(
                                    r#"{"status":"error","error":"Invalid bearer token"}"#,
                                ))
                                .unwrap()
                        }
                    }
                    _ => Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .header("content-type", "application/json")
                        .body(axum::body::Body::from(
                            r#"{"status":"error","error":"Missing Authorization: Bearer <token> header"}"#,
                        ))
                        .unwrap(),
                }
            }
        }
    };

    axum::Router::new()
        .route("/api/v1/health", get(routes::health))
        .route("/api/v1/generate", post(routes::generate))
        .route("/api/v1/execute", post(routes::execute))
        .layer(middleware::from_fn(auth_middleware))
        .with_state(state)
}
