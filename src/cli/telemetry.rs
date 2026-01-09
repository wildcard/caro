//! Telemetry CLI commands

use anyhow::{Context, Result};
use clap::Subcommand;
use colored::*;
use std::path::PathBuf;

use crate::telemetry::{TelemetryConfig, TelemetryStorage};

/// Telemetry subcommands
#[derive(Debug, Clone, Subcommand)]
pub enum TelemetryCommands {
    /// Show queued telemetry events
    Show {
        /// Number of recent events to display
        #[arg(short, long, default_value = "20")]
        limit: usize,

        /// Show events from a specific session ID
        #[arg(short, long)]
        session: Option<String>,
    },

    /// Export telemetry data (for air-gapped environments)
    Export {
        /// Output file path (defaults to telemetry-YYYYMMDD.json)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Clear all queued events
    Clear {
        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },

    /// Show telemetry status and configuration
    Status,
}

/// Handle telemetry commands
pub async fn handle_telemetry(cmd: TelemetryCommands, storage_path: PathBuf) -> Result<()> {
    match cmd {
        TelemetryCommands::Show { limit, session } => {
            show_events(storage_path, limit, session).await
        }
        TelemetryCommands::Export { output } => export_events(storage_path, output).await,
        TelemetryCommands::Clear { force } => clear_events(storage_path, force).await,
        TelemetryCommands::Status => show_status(storage_path).await,
    }
}

/// Show queued telemetry events
async fn show_events(storage_path: PathBuf, limit: usize, session: Option<String>) -> Result<()> {
    let storage =
        TelemetryStorage::new(storage_path).context("Failed to open telemetry storage")?;

    let events = if let Some(session_id) = session {
        storage.get_session_events(&session_id).await?
    } else {
        storage.get_pending_events(limit).await?
    };

    if events.is_empty() {
        println!("{}", "No telemetry events queued.".bright_black());
        println!();
        println!("Events will appear here after using caro with telemetry enabled.");
        return Ok(());
    }

    println!();
    println!(
        "{}",
        format!("Showing {} most recent events:", events.len())
            .bright_white()
            .bold()
    );
    println!();

    for (i, event) in events.iter().enumerate() {
        let event_num = format!("[{}]", i + 1).bright_blue();
        let timestamp = event
            .timestamp
            .format("%Y-%m-%d %H:%M:%S")
            .to_string()
            .bright_black();
        let session = format!("Session: {}", event.session_id.as_str()).bright_black();

        println!("{} {} {}", event_num, timestamp, session);

        match &event.event_type {
            crate::telemetry::EventType::SessionStart {
                version,
                platform,
                shell_type,
                backend_available,
            } => {
                println!("  {} {}", "Type:".bright_white(), "SessionStart".green());
                println!("  {} v{}", "Version:".bright_white(), version);
                println!(
                    "  {} {} ({})",
                    "Platform:".bright_white(),
                    platform,
                    shell_type
                );
                println!(
                    "  {} {}",
                    "Backends:".bright_white(),
                    backend_available.join(", ")
                );
            }
            crate::telemetry::EventType::SessionEnd {
                duration_ms,
                commands_generated,
                commands_executed,
            } => {
                println!("  {} {}", "Type:".bright_white(), "SessionEnd".yellow());
                println!("  {} {}ms", "Duration:".bright_white(), duration_ms);
                println!(
                    "  {} {} generated, {} executed",
                    "Commands:".bright_white(),
                    commands_generated,
                    commands_executed
                );
            }
            crate::telemetry::EventType::CommandGeneration {
                backend,
                duration_ms,
                success,
                error_category,
            } => {
                let status = if *success {
                    "Success".green()
                } else {
                    "Failed".red()
                };
                println!(
                    "  {} {} ({})",
                    "Type:".bright_white(),
                    "CommandGeneration".cyan(),
                    status
                );
                println!("  {} {}", "Backend:".bright_white(), backend);
                println!("  {} {}ms", "Duration:".bright_white(), duration_ms);
                if let Some(error) = error_category {
                    println!("  {} {}", "Error:".bright_white(), error.red());
                }
            }
            crate::telemetry::EventType::SafetyValidation {
                risk_level,
                action_taken,
                pattern_category,
            } => {
                let action_color = match action_taken.as_str() {
                    "blocked" => action_taken.red(),
                    "warned" => action_taken.yellow(),
                    "allowed" => action_taken.green(),
                    _ => action_taken.white(),
                };
                println!(
                    "  {} {} ({})",
                    "Type:".bright_white(),
                    "SafetyValidation".magenta(),
                    action_color
                );
                println!("  {} {}", "Risk:".bright_white(), risk_level);
                if let Some(category) = pattern_category {
                    println!("  {} {}", "Category:".bright_white(), category);
                }
            }
            crate::telemetry::EventType::BackendError {
                backend,
                error_category,
                recoverable,
            } => {
                let recovery = if *recoverable {
                    "Recoverable".yellow()
                } else {
                    "Fatal".red()
                };
                println!(
                    "  {} {} ({})",
                    "Type:".bright_white(),
                    "BackendError".red(),
                    recovery
                );
                println!("  {} {}", "Backend:".bright_white(), backend);
                println!("  {} {}", "Category:".bright_white(), error_category);
            }
        }
        println!();
    }

    println!(
        "{}",
        format!("Total: {} events", events.len()).bright_black()
    );
    println!();

    Ok(())
}

/// Export telemetry data to JSON file
async fn export_events(storage_path: PathBuf, output: Option<PathBuf>) -> Result<()> {
    let storage =
        TelemetryStorage::new(storage_path).context("Failed to open telemetry storage")?;

    let output_path = output.unwrap_or_else(|| {
        let date = chrono::Utc::now().format("%Y%m%d").to_string();
        PathBuf::from(format!("telemetry-{}.json", date))
    });

    let json = storage.export_json().await?;

    std::fs::write(&output_path, json).context("Failed to write export file")?;

    println!();
    println!("{}", "✓ Telemetry data exported successfully".green());
    println!("  {} {}", "File:".bright_white(), output_path.display());
    println!();
    println!("You can now share this file for analysis or upload it manually");
    println!("to the telemetry portal (for air-gapped environments).");
    println!();

    Ok(())
}

/// Clear all queued events
async fn clear_events(storage_path: PathBuf, force: bool) -> Result<()> {
    let storage =
        TelemetryStorage::new(storage_path).context("Failed to open telemetry storage")?;

    let count = storage.count_events().await?;

    if count == 0 {
        println!();
        println!("{}", "No telemetry events to clear.".bright_black());
        println!();
        return Ok(());
    }

    if !force {
        use dialoguer::{theme::ColorfulTheme, Confirm};

        println!();
        let confirmed = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("Delete {} queued telemetry events?", count))
            .default(false)
            .interact()?;

        if !confirmed {
            println!("{}", "Cancelled.".yellow());
            println!();
            return Ok(());
        }
    }

    storage.clear_all().await?;

    println!();
    println!(
        "{}",
        format!("✓ Deleted {} telemetry events", count).green()
    );
    println!();

    Ok(())
}

/// Show telemetry status and configuration
async fn show_status(storage_path: PathBuf) -> Result<()> {
    // Load config
    let config_dir = dirs::config_dir()
        .context("Failed to determine config directory")?
        .join("caro");

    let config_path = config_dir.join("config.toml");

    // Try to load telemetry config, use defaults if not found
    let config = if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        let full_config: toml::Value = toml::from_str(&content)?;

        if let Some(telemetry) = full_config.get("telemetry") {
            toml::from_str::<TelemetryConfig>(&telemetry.to_string())?
        } else {
            TelemetryConfig::default()
        }
    } else {
        TelemetryConfig::default()
    };

    let storage =
        TelemetryStorage::new(storage_path.clone()).context("Failed to open telemetry storage")?;

    let event_count = storage.count_events().await?;

    println!();
    println!("{}", "═".repeat(60).bright_blue());
    println!("{}", "Telemetry Status".bright_white().bold());
    println!("{}", "═".repeat(60).bright_blue());
    println!();

    // Status
    let status = if config.enabled {
        "ENABLED (collecting data)".green()
    } else {
        "DISABLED (no data collected)".red()
    };
    println!("  {} {}", "Status:".bright_white(), status);

    // Level
    println!("  {} {}", "Level:".bright_white(), config.level);

    // Air-gapped mode
    let air_gapped = if config.air_gapped {
        "ON (manual export only)".yellow()
    } else {
        "OFF (auto upload)".green()
    };
    println!("  {} {}", "Air-gapped:".bright_white(), air_gapped);

    // Endpoint
    if !config.air_gapped {
        println!(
            "  {} {}",
            "Endpoint:".bright_white(),
            config.endpoint.bright_black()
        );
    }

    println!();
    println!("{}", "─".repeat(60).bright_black());
    println!();

    // Storage info
    println!(
        "  {} {}",
        "Storage:".bright_white(),
        storage_path.display().to_string().bright_black()
    );
    println!("  {} {}", "Queued events:".bright_white(), event_count);

    println!();
    println!("{}", "═".repeat(60).bright_blue());
    println!();

    // Help commands
    println!("{}", "Commands:".bright_white().bold());
    println!(
        "  {} Disable telemetry",
        "caro config set telemetry.enabled false".cyan()
    );
    println!("  {} View queued events", "caro telemetry show".cyan());
    println!(
        "  {} Export data (air-gapped)",
        "caro telemetry export".cyan()
    );
    println!("  {} Clear all events", "caro telemetry clear".cyan());
    println!();

    println!("{}", "Learn more: https://caro.sh/telemetry".bright_black());
    println!();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_show_empty_events() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let result = show_events(db_path, 10, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_export_creates_file() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let output_path = temp_dir.path().join("export.json");

        // Create storage
        let _storage = TelemetryStorage::new(db_path.clone()).unwrap();

        let result = export_events(db_path, Some(output_path.clone())).await;
        assert!(result.is_ok());
        assert!(output_path.exists());
    }

    #[tokio::test]
    async fn test_clear_events_force() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let result = clear_events(db_path, true).await;
        assert!(result.is_ok());
    }
}
