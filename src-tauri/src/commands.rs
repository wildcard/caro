use crate::database::Database;
use crate::models::*;
use chrono::Utc;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

type DbState<'a> = State<'a, crate::AppState>;

/// Get current configuration
#[tauri::command]
pub async fn get_config() -> Result<serde_json::Value, String> {
    let config_manager = cmdai::config::ConfigManager::new()
        .map_err(|e| format!("Failed to create config manager: {}", e))?;

    let config = config_manager
        .load()
        .map_err(|e| format!("Failed to load config: {}", e))?;

    serde_json::to_value(&config).map_err(|e| format!("Failed to serialize config: {}", e))
}

/// Update configuration
#[tauri::command]
pub async fn update_config(config_json: serde_json::Value) -> Result<(), String> {
    let config_manager = cmdai::config::ConfigManager::new()
        .map_err(|e| format!("Failed to create config manager: {}", e))?;

    let config: cmdai::models::UserConfiguration = serde_json::from_value(config_json)
        .map_err(|e| format!("Failed to deserialize config: {}", e))?;

    config_manager
        .save(&config)
        .map_err(|e| format!("Failed to save config: {}", e))?;

    Ok(())
}

/// Get execution history with filters
#[tauri::command]
pub async fn get_execution_history(
    state: DbState<'_>,
    filter: HistoryFilter,
) -> Result<Vec<ExecutionRecord>, String> {
    let db = state.db.lock().await;
    db.get_execution_history(&filter)
        .map_err(|e| format!("Failed to get execution history: {}", e))
}

/// Add new execution to history
#[tauri::command]
pub async fn add_execution(
    state: DbState<'_>,
    record: ExecutionRecord,
) -> Result<i64, String> {
    let db = state.db.lock().await;
    db.add_execution(&record)
        .map_err(|e| format!("Failed to add execution: {}", e))
}

/// Get single execution by ID
#[tauri::command]
pub async fn get_execution_by_id(
    state: DbState<'_>,
    id: i64,
) -> Result<Option<ExecutionRecord>, String> {
    let db = state.db.lock().await;
    db.get_execution_by_id(id)
        .map_err(|e| format!("Failed to get execution: {}", e))
}

/// Delete execution record
#[tauri::command]
pub async fn delete_execution(state: DbState<'_>, id: i64) -> Result<(), String> {
    let db = state.db.lock().await;
    db.delete_execution(id)
        .map_err(|e| format!("Failed to delete execution: {}", e))
}

/// Rate an execution
#[tauri::command]
pub async fn rate_execution(
    state: DbState<'_>,
    execution_id: i64,
    rating: i32,
    feedback: Option<String>,
) -> Result<i64, String> {
    if !(1..=5).contains(&rating) {
        return Err("Rating must be between 1 and 5".to_string());
    }

    let db = state.db.lock().await;
    let rating_record = ExecutionRating {
        id: None,
        execution_id,
        rating,
        feedback,
        created_at: Utc::now(),
    };

    db.add_rating(&rating_record)
        .map_err(|e| format!("Failed to add rating: {}", e))
}

/// Get ratings for an execution
#[tauri::command]
pub async fn get_execution_ratings(
    state: DbState<'_>,
    execution_id: i64,
) -> Result<Vec<ExecutionRating>, String> {
    let db = state.db.lock().await;
    db.get_ratings(execution_id)
        .map_err(|e| format!("Failed to get ratings: {}", e))
}

/// Vote on an execution
#[tauri::command]
pub async fn vote_execution(
    state: DbState<'_>,
    execution_id: i64,
    vote_type: String,
) -> Result<i64, String> {
    let vote_type = VoteType::from_str(&vote_type)
        .map_err(|e| format!("Invalid vote type: {}", e))?;

    let db = state.db.lock().await;
    let vote_record = ExecutionVote {
        id: None,
        execution_id,
        vote_type,
        created_at: Utc::now(),
    };

    db.add_vote(&vote_record)
        .map_err(|e| format!("Failed to add vote: {}", e))
}

/// Get votes for an execution
#[tauri::command]
pub async fn get_execution_votes(
    state: DbState<'_>,
    execution_id: i64,
) -> Result<Vec<ExecutionVote>, String> {
    let db = state.db.lock().await;
    db.get_votes(execution_id)
        .map_err(|e| format!("Failed to get votes: {}", e))
}

/// Generate command from natural language
#[tauri::command]
pub async fn generate_command(
    state: DbState<'_>,
    request: CommandGenerationRequest,
) -> Result<CommandGenerationResponse, String> {
    use cmdai::cli::{CliApp, IntoCliArgs};
    use std::time::Instant;

    // Create CLI app
    let app = CliApp::new()
        .await
        .map_err(|e| format!("Failed to create CLI app: {}", e))?;

    // Create args adapter
    struct ArgsAdapter {
        prompt: String,
        shell: Option<String>,
        safety: Option<String>,
    }

    impl IntoCliArgs for ArgsAdapter {
        fn prompt(&self) -> Option<String> {
            Some(self.prompt.clone())
        }
        fn shell(&self) -> Option<String> {
            self.shell.clone()
        }
        fn safety(&self) -> Option<String> {
            self.safety.clone()
        }
        fn output(&self) -> Option<String> {
            None
        }
        fn confirm(&self) -> bool {
            false
        }
        fn verbose(&self) -> bool {
            false
        }
        fn config_file(&self) -> Option<String> {
            None
        }
    }

    let args = ArgsAdapter {
        prompt: request.prompt.clone(),
        shell: request.shell,
        safety: request.safety_level,
    };

    let start = Instant::now();
    let result = app
        .run_with_args(args)
        .await
        .map_err(|e| format!("Command generation failed: {}", e))?;
    let generation_time = start.elapsed();

    // Save to history if not dry run
    if !request.dry_run {
        let record = ExecutionRecord {
            id: None,
            timestamp: Utc::now(),
            prompt: request.prompt,
            generated_command: result.generated_command.clone(),
            explanation: Some(result.explanation.clone()),
            shell_type: result.shell_used.to_string(),
            risk_level: "Safe".to_string(), // TODO: Extract from result
            executed: result.executed,
            blocked_reason: result.blocked_reason.clone(),
            generation_time_ms: generation_time.as_millis() as i64,
            execution_time_ms: 0,
            warnings: result.warnings.clone(),
            alternatives: result.alternatives.clone(),
            backend_used: "embedded".to_string(), // TODO: Extract from result
        };

        let db = state.db.lock().await;
        let _ = db.add_execution(&record);
    }

    Ok(CommandGenerationResponse {
        generated_command: result.generated_command,
        explanation: result.explanation,
        risk_level: "Safe".to_string(), // TODO: Extract from validation
        warnings: result.warnings,
        alternatives: result.alternatives,
        requires_confirmation: result.requires_confirmation,
        blocked_reason: result.blocked_reason,
        generation_time_ms: generation_time.as_millis() as u64,
        backend_used: "embedded".to_string(),
    })
}

/// Get analytics data
#[tauri::command]
pub async fn get_analytics(state: DbState<'_>) -> Result<Analytics, String> {
    let db = state.db.lock().await;
    db.get_analytics()
        .map_err(|e| format!("Failed to get analytics: {}", e))
}

/// Export history to JSON
#[tauri::command]
pub async fn export_history(
    state: DbState<'_>,
    format: String,
) -> Result<String, String> {
    let db = state.db.lock().await;
    let filter = HistoryFilter {
        shell_type: None,
        risk_level: None,
        executed: None,
        blocked: None,
        search_query: None,
        limit: None,
        offset: None,
    };

    let history = db
        .get_execution_history(&filter)
        .map_err(|e| format!("Failed to get history: {}", e))?;

    match format.as_str() {
        "json" => serde_json::to_string_pretty(&history)
            .map_err(|e| format!("Failed to serialize to JSON: {}", e)),
        "csv" => {
            let mut csv = String::from("timestamp,prompt,command,shell,risk_level,executed\n");
            for record in history {
                csv.push_str(&format!(
                    "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",{}\n",
                    record.timestamp,
                    record.prompt.replace('"', "\"\""),
                    record.generated_command.replace('"', "\"\""),
                    record.shell_type,
                    record.risk_level,
                    record.executed
                ));
            }
            Ok(csv)
        }
        _ => Err(format!("Unsupported format: {}", format)),
    }
}
