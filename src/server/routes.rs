//! HTTP route handlers for caro-server.
//!
//! Each handler wraps caro's existing pipeline (AgentLoop, SafetyValidator,
//! CommandExecutor) and exposes it over HTTP JSON.

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use tracing::{info, warn};

use crate::models::ShellType;
use crate::safety::{SafetyConfig, SafetyValidator};

use super::types::*;
use super::AppState;

/// GET /api/v1/health
pub async fn health(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let uptime = state.start_time.elapsed().as_secs();

    let resp = ApiHealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        backends: BackendStatus {
            static_matcher: true,
            embedded: state.agent_loop.is_some(),
            ollama: false,
            claude: false,
        },
        safety_patterns: 52,
        uptime_seconds: uptime,
    };

    Json(resp)
}

/// POST /api/v1/generate
pub async fn generate(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ApiCommandRequest>,
) -> impl IntoResponse {
    info!(
        request_id = %req.request_id,
        agent_id = ?req.agent_id,
        input = %req.input,
        "Generate command request"
    );

    // Validate input
    if req.input.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiCommandResponse {
                request_id: req.request_id,
                status: ApiStatus::Error,
                command: None,
                explanation: None,
                risk_level: None,
                estimated_impact: None,
                alternatives: None,
                backend_used: None,
                generation_time_ms: None,
                confidence_score: None,
                warnings: vec![],
                error: Some("Input cannot be empty".to_string()),
                reason: None,
            }),
        );
    }

    // Try agent loop if available
    let agent_loop = match &state.agent_loop {
        Some(al) => al,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ApiCommandResponse {
                    request_id: req.request_id,
                    status: ApiStatus::Error,
                    command: None,
                    explanation: None,
                    risk_level: None,
                    estimated_impact: None,
                    alternatives: None,
                    backend_used: None,
                    generation_time_ms: None,
                    confidence_score: None,
                    warnings: vec![],
                    error: Some("No backend available".to_string()),
                    reason: None,
                }),
            );
        }
    };

    // Generate command through the agent loop
    match agent_loop.generate_command(&req.input).await {
        Ok(generated) => {
            // Run safety validation
            let validator = match SafetyValidator::new(SafetyConfig::from_level(req.safety_level)) {
                Ok(v) => v,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiCommandResponse {
                            request_id: req.request_id,
                            status: ApiStatus::Error,
                            command: None,
                            explanation: None,
                            risk_level: None,
                            estimated_impact: None,
                            alternatives: None,
                            backend_used: None,
                            generation_time_ms: None,
                            confidence_score: None,
                            warnings: vec![],
                            error: Some(format!("Safety validator init failed: {}", e)),
                            reason: None,
                        }),
                    );
                }
            };

            let shell = req.shell;
            match validator.validate_command(&generated.command, shell).await {
                Ok(validation) => {
                    if validation.allowed {
                        (
                            StatusCode::OK,
                            Json(ApiCommandResponse {
                                request_id: req.request_id,
                                status: ApiStatus::Ok,
                                command: Some(generated.command),
                                explanation: Some(generated.explanation),
                                risk_level: Some(validation.risk_level),
                                estimated_impact: Some(generated.estimated_impact),
                                alternatives: Some(generated.alternatives),
                                backend_used: Some(generated.backend_used),
                                generation_time_ms: Some(generated.generation_time_ms),
                                confidence_score: Some(generated.confidence_score),
                                warnings: validation.warnings,
                                error: None,
                                reason: None,
                            }),
                        )
                    } else {
                        warn!(
                            request_id = %req.request_id,
                            command = %generated.command,
                            risk_level = ?validation.risk_level,
                            "Command blocked by safety validation"
                        );
                        (
                            StatusCode::OK,
                            Json(ApiCommandResponse {
                                request_id: req.request_id,
                                status: ApiStatus::Blocked,
                                command: Some(generated.command),
                                explanation: Some(validation.explanation.clone()),
                                risk_level: Some(validation.risk_level),
                                estimated_impact: None,
                                alternatives: None,
                                backend_used: None,
                                generation_time_ms: None,
                                confidence_score: None,
                                warnings: validation.warnings,
                                error: None,
                                reason: Some(format!(
                                    "Command blocked by safety validation: {}",
                                    validation.explanation
                                )),
                            }),
                        )
                    }
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiCommandResponse {
                        request_id: req.request_id,
                        status: ApiStatus::Error,
                        command: None,
                        explanation: None,
                        risk_level: None,
                        estimated_impact: None,
                        alternatives: None,
                        backend_used: None,
                        generation_time_ms: None,
                        confidence_score: None,
                        warnings: vec![],
                        error: Some(format!("Safety validation error: {}", e)),
                        reason: None,
                    }),
                ),
            }
        }
        Err(e) => {
            warn!(
                request_id = %req.request_id,
                error = %e,
                "Command generation failed"
            );
            (
                StatusCode::OK,
                Json(ApiCommandResponse {
                    request_id: req.request_id,
                    status: ApiStatus::Error,
                    command: None,
                    explanation: None,
                    risk_level: None,
                    estimated_impact: None,
                    alternatives: None,
                    backend_used: None,
                    generation_time_ms: None,
                    confidence_score: None,
                    warnings: vec![],
                    error: Some(format!("Generation failed: {}", e)),
                    reason: None,
                }),
            )
        }
    }
}

/// POST /api/v1/execute
pub async fn execute(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ApiExecuteRequest>,
) -> impl IntoResponse {
    info!(
        request_id = %req.request_id,
        command = %req.command,
        confirmed = %req.confirmed,
        "Execute command request"
    );

    // Re-validate the command for safety (defense in depth)
    let validator = match SafetyValidator::new(SafetyConfig::moderate()) {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiExecuteResponse {
                    request_id: req.request_id,
                    status: ApiStatus::Error,
                    exit_code: None,
                    stdout: None,
                    stderr: None,
                    execution_time_ms: None,
                    error: Some(format!("Safety validator init failed: {}", e)),
                    risk_level: None,
                    warnings: vec![],
                }),
            );
        }
    };

    match validator
        .validate_command(&req.command, ShellType::Bash)
        .await
    {
        Ok(validation) => {
            // Block if not confirmed and command has risk
            if !req.confirmed && validation.risk_level != crate::models::RiskLevel::Safe {
                return (
                    StatusCode::FORBIDDEN,
                    Json(ApiExecuteResponse {
                        request_id: req.request_id,
                        status: ApiStatus::Blocked,
                        exit_code: None,
                        stdout: None,
                        stderr: None,
                        execution_time_ms: None,
                        error: Some(format!(
                            "Command has risk level '{:?}' and requires confirmed=true",
                            validation.risk_level
                        )),
                        risk_level: Some(validation.risk_level),
                        warnings: validation.warnings,
                    }),
                );
            }

            // Block if safety says not allowed (even with confirmation)
            if !validation.allowed {
                return (
                    StatusCode::FORBIDDEN,
                    Json(ApiExecuteResponse {
                        request_id: req.request_id,
                        status: ApiStatus::Blocked,
                        exit_code: None,
                        stdout: None,
                        stderr: None,
                        execution_time_ms: None,
                        error: Some(format!(
                            "Command blocked by safety validation: {}",
                            validation.explanation
                        )),
                        risk_level: Some(validation.risk_level),
                        warnings: validation.warnings,
                    }),
                );
            }

            // Execute the command
            let executor = crate::execution::CommandExecutor::new(state.shell_type)
                .with_timeout(req.timeout_ms);

            match executor.execute(&req.command) {
                Ok(result) => (
                    StatusCode::OK,
                    Json(ApiExecuteResponse {
                        request_id: req.request_id,
                        status: ApiStatus::Ok,
                        exit_code: Some(result.exit_code),
                        stdout: Some(result.stdout),
                        stderr: Some(result.stderr),
                        execution_time_ms: Some(result.execution_time_ms),
                        error: None,
                        risk_level: Some(validation.risk_level),
                        warnings: vec![],
                    }),
                ),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiExecuteResponse {
                        request_id: req.request_id,
                        status: ApiStatus::Error,
                        exit_code: None,
                        stdout: None,
                        stderr: None,
                        execution_time_ms: None,
                        error: Some(format!("Execution failed: {}", e)),
                        risk_level: None,
                        warnings: vec![],
                    }),
                ),
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiExecuteResponse {
                request_id: req.request_id,
                status: ApiStatus::Error,
                exit_code: None,
                stdout: None,
                stderr: None,
                execution_time_ms: None,
                error: Some(format!("Safety validation error: {}", e)),
                risk_level: None,
                warnings: vec![],
            }),
        ),
    }
}
