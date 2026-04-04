//! Integration tests for caro-server HTTP API.
//!
//! Tests the server routes using axum's test utilities without starting
//! a real TCP listener.

#[cfg(feature = "server")]
mod server_tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use caro::models::{ServerConfig, ShellType};
    use caro::server::types::*;
    use caro::server::{self, AppState};
    use http_body_util::BodyExt;
    use std::sync::Arc;
    use std::time::Instant;
    use tower::ServiceExt;

    /// Create test app state with no backend (static matcher only).
    fn test_state() -> Arc<AppState> {
        Arc::new(AppState {
            agent_loop: None,
            shell_type: ShellType::Bash,
            start_time: Instant::now(),
        })
    }

    /// Create a test router without auth.
    fn test_router() -> axum::Router {
        let config = ServerConfig::default();
        server::build_router(test_state(), &config)
    }

    /// Create a test router with auth.
    fn test_router_with_auth(token: &str) -> axum::Router {
        let config = ServerConfig {
            auth_token: Some(token.to_string()),
            ..ServerConfig::default()
        };
        server::build_router(test_state(), &config)
    }

    /// Helper: extract JSON body from response.
    async fn body_json<T: serde::de::DeserializeOwned>(
        response: axum::response::Response,
    ) -> T {
        let body = response.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&body).unwrap()
    }

    // ─── Health endpoint ───

    #[tokio::test]
    async fn test_health_returns_ok() {
        let app = test_router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let health: ApiHealthResponse = body_json(response).await;
        assert_eq!(health.status, "ok");
        assert_eq!(health.version, env!("CARGO_PKG_VERSION"));
        assert!(health.backends.static_matcher);
    }

    #[tokio::test]
    async fn test_health_accessible_without_auth() {
        let app = test_router_with_auth("secret-token");

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    // ─── Generate endpoint ───

    #[tokio::test]
    async fn test_generate_empty_input_returns_400() {
        let app = test_router();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/generate")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"input": ""}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let resp: ApiCommandResponse = body_json(response).await;
        assert_eq!(resp.status, ApiStatus::Error);
        assert!(resp.error.unwrap().contains("empty"));
    }

    #[tokio::test]
    async fn test_generate_no_backend_returns_503() {
        let app = test_router();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/generate")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"input": "list files"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Without a backend, should return 503
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    // ─── Execute endpoint ───

    #[tokio::test]
    async fn test_execute_safe_command() {
        let app = test_router();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/execute")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"command": "echo hello", "confirmed": true}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let resp: ApiExecuteResponse = body_json(response).await;
        assert_eq!(resp.status, ApiStatus::Ok);
        assert_eq!(resp.exit_code, Some(0));
        assert!(resp.stdout.unwrap().contains("hello"));
    }

    #[tokio::test]
    async fn test_execute_unconfirmed_risky_command_rejected() {
        let app = test_router();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/execute")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"command": "rm -rf /tmp/test", "confirmed": false}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should be blocked -- rm -rf without confirmation
        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        let resp: ApiExecuteResponse = body_json(response).await;
        assert_eq!(resp.status, ApiStatus::Blocked);
    }

    #[tokio::test]
    async fn test_execute_dangerous_command_blocked() {
        let app = test_router();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/execute")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"command": "rm -rf /", "confirmed": true}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Critical commands blocked even with confirmation
        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        let resp: ApiExecuteResponse = body_json(response).await;
        assert_eq!(resp.status, ApiStatus::Blocked);
    }

    // ─── Authentication ───

    #[tokio::test]
    async fn test_auth_missing_token_rejected() {
        let app = test_router_with_auth("secret-token");

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/generate")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"input": "list files"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_auth_wrong_token_rejected() {
        let app = test_router_with_auth("secret-token");

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/generate")
                    .header("content-type", "application/json")
                    .header("authorization", "Bearer wrong-token")
                    .body(Body::from(r#"{"input": "list files"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_auth_correct_token_accepted() {
        let app = test_router_with_auth("secret-token");

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/execute")
                    .header("content-type", "application/json")
                    .header("authorization", "Bearer secret-token")
                    .body(Body::from(
                        r#"{"command": "echo auth-test", "confirmed": true}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let resp: ApiExecuteResponse = body_json(response).await;
        assert_eq!(resp.status, ApiStatus::Ok);
        assert!(resp.stdout.unwrap().contains("auth-test"));
    }

    // ─── 404 ───

    #[tokio::test]
    async fn test_unknown_route_returns_404() {
        let app = test_router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/nonexistent")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
