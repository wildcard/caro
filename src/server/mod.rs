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
use crate::models::{ServerConfig, ShellType};
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

/// Build the axum router with all API routes and optional CORS/auth.
pub fn build_router(
    state: std::sync::Arc<AppState>,
    server_config: &ServerConfig,
) -> axum::Router {
    let router = build_base_routes(state.clone());

    // Apply auth if token configured
    let router = if let Some(ref token) = server_config.auth_token {
        apply_auth_layer(router, token.clone(), state)
    } else {
        router.with_state(state)
    };

    // Apply CORS if origins configured
    if !server_config.allowed_origins.is_empty() {
        apply_cors_layer(router, &server_config.allowed_origins)
    } else {
        router
    }
}

/// Internal: create the base route definitions.
fn build_base_routes(
    _state: std::sync::Arc<AppState>,
) -> axum::Router<std::sync::Arc<AppState>> {
    use axum::routing::{get, post};

    axum::Router::new()
        .route("/api/v1/health", get(routes::health))
        .route("/api/v1/generate", post(routes::generate))
        .route("/api/v1/execute", post(routes::execute))
}

/// Internal: apply bearer token auth middleware.
fn apply_auth_layer(
    router: axum::Router<std::sync::Arc<AppState>>,
    token: String,
    state: std::sync::Arc<AppState>,
) -> axum::Router {
    use axum::{
        extract::Request,
        http::StatusCode,
        middleware::{self, Next},
        response::Response,
    };

    let token = std::sync::Arc::new(token);

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

    router
        .layer(middleware::from_fn(auth_middleware))
        .with_state(state)
}

/// Internal: apply CORS layer with configured origins.
fn apply_cors_layer(router: axum::Router, origins: &[String]) -> axum::Router {
    use tower_http::cors::{AllowOrigin, CorsLayer};

    let origins: Vec<_> = origins
        .iter()
        .filter_map(|o| o.parse().ok())
        .collect();

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
        ]);

    router.layer(cors)
}
