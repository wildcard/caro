//! WebSocket handler for real-time caro-server communication.
//!
//! Provides bidirectional command generation and execution over WebSocket,
//! enabling cabinet agents to stream results in real time.

use axum::{
    extract::{
        ws::{Message, WebSocket},
        Query, State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use std::sync::Arc;
use tracing::{info, warn};

use crate::models::ShellType;
use crate::safety::{SafetyConfig, SafetyValidator};

use super::types::*;
use super::AppState;

/// Query parameters for WebSocket connection (auth via query string).
#[derive(Debug, serde::Deserialize)]
pub struct WsQueryParams {
    pub token: Option<String>,
}

/// GET /api/v1/ws -- WebSocket upgrade handler.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Query(_params): Query<WsQueryParams>,
) -> impl IntoResponse {
    // Note: Token auth for WS is handled at the application level if needed.
    // The auth middleware already skips /api/v1/ws, so token validation
    // via query param can be added here for stricter setups.
    info!("WebSocket connection requested");
    ws.on_upgrade(move |socket| handle_ws(socket, state))
}

/// Handle a WebSocket connection.
async fn handle_ws(mut socket: WebSocket, state: Arc<AppState>) {
    info!("WebSocket client connected");

    let mut heartbeat_interval =
        tokio::time::interval(std::time::Duration::from_secs(30));

    loop {
        tokio::select! {
            // Heartbeat tick
            _ = heartbeat_interval.tick() => {
                let msg = serde_json::to_string(&WsMessage::Heartbeat).unwrap();
                if socket.send(Message::Text(msg.into())).await.is_err() {
                    break;
                }
            }
            // Incoming message
            maybe_msg = socket.recv() => {
                match maybe_msg {
                    Some(Ok(Message::Text(text))) => {
                        let response = handle_ws_message(&text, &state).await;
                        if let Some(resp_json) = response {
                            if socket.send(Message::Text(resp_json.into())).await.is_err() {
                                break;
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        info!("WebSocket client disconnected");
                        break;
                    }
                    Some(Err(e)) => {
                        warn!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {} // Ignore binary, ping, pong
                }
            }
        }
    }

    info!("WebSocket connection closed");
}

/// Process a single WebSocket text message and return a response.
async fn handle_ws_message(text: &str, state: &AppState) -> Option<String> {
    let msg: WsMessage = match serde_json::from_str(text) {
        Ok(m) => m,
        Err(e) => {
            let err = WsMessage::Error {
                message: format!("Invalid message: {}", e),
            };
            return Some(serde_json::to_string(&err).unwrap());
        }
    };

    match msg {
        WsMessage::CommandRequest { id, input, agent_id } => {
            info!(id = %id, agent_id = ?agent_id, "WS command request");

            let agent_loop = match &state.agent_loop {
                Some(al) => al,
                None => {
                    let resp = WsMessage::CommandResult {
                        id,
                        status: ApiStatus::Error,
                        command: None,
                        explanation: None,
                        risk_level: None,
                        warnings: vec![],
                        error: Some("No backend available".to_string()),
                    };
                    return Some(serde_json::to_string(&resp).unwrap());
                }
            };

            match agent_loop.generate_command(&input).await {
                Ok(generated) => {
                    // Safety validation
                    let validator = SafetyValidator::new(SafetyConfig::moderate()).ok()?;
                    let validation = validator
                        .validate_command(&generated.command, ShellType::Bash)
                        .await
                        .ok()?;

                    let resp = if validation.allowed {
                        WsMessage::CommandResult {
                            id,
                            status: ApiStatus::Ok,
                            command: Some(generated.command),
                            explanation: Some(generated.explanation),
                            risk_level: Some(validation.risk_level),
                            warnings: validation.warnings,
                            error: None,
                        }
                    } else {
                        WsMessage::CommandResult {
                            id,
                            status: ApiStatus::Blocked,
                            command: Some(generated.command),
                            explanation: None,
                            risk_level: Some(validation.risk_level),
                            warnings: validation.warnings,
                            error: Some(validation.explanation),
                        }
                    };
                    Some(serde_json::to_string(&resp).unwrap())
                }
                Err(e) => {
                    let resp = WsMessage::CommandResult {
                        id,
                        status: ApiStatus::Error,
                        command: None,
                        explanation: None,
                        risk_level: None,
                        warnings: vec![],
                        error: Some(format!("Generation failed: {}", e)),
                    };
                    Some(serde_json::to_string(&resp).unwrap())
                }
            }
        }

        WsMessage::ExecutionRequest {
            id,
            command,
            confirmed,
        } => {
            info!(id = %id, command = %command, "WS execution request");

            // Safety check
            let validator = SafetyValidator::new(SafetyConfig::moderate()).ok()?;
            let validation = validator
                .validate_command(&command, ShellType::Bash)
                .await
                .ok()?;

            if !confirmed
                && validation.risk_level != crate::models::RiskLevel::Safe
            {
                let resp = WsMessage::Error {
                    message: format!(
                        "Command has risk level '{:?}' and requires confirmed=true",
                        validation.risk_level
                    ),
                };
                return Some(serde_json::to_string(&resp).unwrap());
            }

            if !validation.allowed {
                let resp = WsMessage::Error {
                    message: format!("Command blocked: {}", validation.explanation),
                };
                return Some(serde_json::to_string(&resp).unwrap());
            }

            let executor =
                crate::execution::CommandExecutor::new(state.shell_type);
            match executor.execute(&command) {
                Ok(result) => {
                    let resp = WsMessage::ExecutionResult {
                        id,
                        exit_code: result.exit_code,
                        stdout: result.stdout,
                        stderr: result.stderr,
                        execution_time_ms: result.execution_time_ms,
                    };
                    Some(serde_json::to_string(&resp).unwrap())
                }
                Err(e) => {
                    let resp = WsMessage::Error {
                        message: format!("Execution failed: {}", e),
                    };
                    Some(serde_json::to_string(&resp).unwrap())
                }
            }
        }

        WsMessage::Heartbeat => {
            // Echo heartbeat back
            Some(serde_json::to_string(&WsMessage::Heartbeat).unwrap())
        }

        _ => {
            // Server-originated messages sent by client -- ignore
            None
        }
    }
}
