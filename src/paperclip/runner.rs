//! Heartbeat runner for Paperclip agent mode.
//!
//! When Caro is triggered by Paperclip, the runner:
//! 1. Fetches assigned tasks from the Paperclip API
//! 2. Checks budget constraints
//! 3. For each task: converts the NL description → shell command
//! 4. Runs safety validation on the generated command
//! 5. Reports results (or errors) back to Paperclip

use super::client::{PaperclipClient, TaskResult};
use crate::backends::CommandGenerator;
use crate::models::{CommandRequest, SafetyLevel, ShellType};
use crate::safety::SafetyValidator;
use anyhow::{Context, Result};
use colored::Colorize;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Orchestrates a single heartbeat cycle for Paperclip agent mode.
pub struct PaperclipRunner {
    client: PaperclipClient,
    backend: Arc<dyn CommandGenerator>,
    safety: SafetyValidator,
    shell: ShellType,
    safety_level: SafetyLevel,
}

/// Summary of a heartbeat execution.
#[derive(Debug)]
pub struct HeartbeatSummary {
    pub tasks_received: usize,
    pub tasks_completed: usize,
    pub tasks_failed: usize,
    pub tasks_blocked_safety: usize,
}

impl PaperclipRunner {
    pub fn new(
        client: PaperclipClient,
        backend: Arc<dyn CommandGenerator>,
        safety: SafetyValidator,
        shell: ShellType,
        safety_level: SafetyLevel,
    ) -> Self {
        Self {
            client,
            backend,
            safety,
            shell,
            safety_level,
        }
    }

    /// Execute a single heartbeat cycle: fetch tasks, process them, report back.
    pub async fn run_heartbeat(&self) -> Result<HeartbeatSummary> {
        // 1. Check budget first
        let budget = self.client.check_budget().await?;
        if budget.exceeded {
            warn!(
                "Budget exceeded ({:.2}/{:.2} {}). Skipping heartbeat.",
                budget.remaining, budget.limit, budget.currency
            );
            return Ok(HeartbeatSummary {
                tasks_received: 0,
                tasks_completed: 0,
                tasks_failed: 0,
                tasks_blocked_safety: 0,
            });
        }
        debug!(
            "Budget OK: {:.2}/{:.2} {} remaining",
            budget.remaining, budget.limit, budget.currency
        );

        // 2. Fetch tasks
        let tasks = self
            .client
            .get_tasks()
            .await
            .context("failed to fetch tasks")?;

        let total = tasks.len();
        info!("Received {} task(s) from Paperclip", total);

        let mut completed = 0usize;
        let mut failed = 0usize;
        let mut blocked = 0usize;

        // 3. Process each task
        for task in &tasks {
            info!("Processing task {}: {}", task.id, task.description);

            // Report progress
            if let Err(e) = self.client.report_progress(&task.id).await {
                warn!("Failed to report progress for task {}: {}", task.id, e);
            }

            // Generate command from natural language description
            let request =
                CommandRequest::new(&task.description, self.shell.clone())
                    .with_safety(self.safety_level.clone());

            match self.backend.generate_command(&request).await {
                Ok(generated) => {
                    // Run safety validation
                    let validation = self
                        .safety
                        .validate_command(&generated.command, self.shell.clone())
                        .await;

                    match validation {
                        Ok(result) if result.allowed => {
                            let task_result = TaskResult {
                                command: generated.command.clone(),
                                safety_level: format!("{:?}", result.risk_level),
                                explanation: Some(generated.explanation.clone()),
                                warnings: result.warnings.clone(),
                            };

                            match self.client.report_completion(&task.id, task_result).await {
                                Ok(()) => {
                                    info!("Task {} completed: {}", task.id, generated.command);
                                    completed += 1;
                                }
                                Err(e) => {
                                    error!("Failed to report completion for {}: {}", task.id, e);
                                    failed += 1;
                                }
                            }
                        }
                        Ok(result) => {
                            // Command blocked by safety validation
                            let msg = format!(
                                "Command blocked by safety validation (risk: {:?}): {}",
                                result.risk_level,
                                result.warnings.join("; ")
                            );
                            warn!("{}", msg);
                            let _ = self.client.report_error(&task.id, &msg).await;
                            blocked += 1;
                        }
                        Err(e) => {
                            let msg = format!("Safety validation error: {}", e);
                            error!("{}", msg);
                            let _ = self.client.report_error(&task.id, &msg).await;
                            failed += 1;
                        }
                    }
                }
                Err(e) => {
                    let msg = format!("Command generation failed: {}", e);
                    error!("{}", msg);
                    let _ = self.client.report_error(&task.id, &msg).await;
                    failed += 1;
                }
            }
        }

        let summary = HeartbeatSummary {
            tasks_received: total,
            tasks_completed: completed,
            tasks_failed: failed,
            tasks_blocked_safety: blocked,
        };

        info!(
            "Heartbeat complete: {} received, {} completed, {} failed, {} blocked",
            summary.tasks_received,
            summary.tasks_completed,
            summary.tasks_failed,
            summary.tasks_blocked_safety
        );

        Ok(summary)
    }

    /// Print a human-readable status report.
    pub fn print_agent_info(&self) {
        let config = self.client.config();
        println!("{}", "Caro Paperclip Agent".bold());
        println!("  Agent ID:  {}", config.agent_id.cyan());
        println!("  API URL:   {}", config.api_url);
        println!("  Run ID:    {}", config.run_id);
        println!("  Backend:   {}", self.backend.backend_info().model_name);
        println!("  Shell:     {:?}", self.shell);
        println!("  Safety:    {:?}", self.safety_level);
    }
}

impl std::fmt::Display for HeartbeatSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Heartbeat: {} tasks ({} completed, {} failed, {} blocked by safety)",
            self.tasks_received,
            self.tasks_completed,
            self.tasks_failed,
            self.tasks_blocked_safety
        )
    }
}
