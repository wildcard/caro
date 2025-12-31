use crate::models::*;
use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::path::Path;

pub struct Database {
    conn: Connection,
}

impl Database {
    /// Create a new database connection and initialize schema
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    /// Initialize database schema
    fn init_schema(&self) -> Result<()> {
        // Execution history table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS execution_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                prompt TEXT NOT NULL,
                generated_command TEXT NOT NULL,
                explanation TEXT,
                shell_type TEXT NOT NULL,
                risk_level TEXT NOT NULL,
                executed BOOLEAN NOT NULL,
                blocked_reason TEXT,
                generation_time_ms INTEGER NOT NULL,
                execution_time_ms INTEGER NOT NULL,
                warnings TEXT,
                alternatives TEXT,
                backend_used TEXT NOT NULL
            )",
            [],
        )?;

        // Ratings table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS execution_ratings (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                execution_id INTEGER NOT NULL,
                rating INTEGER NOT NULL CHECK(rating >= 1 AND rating <= 5),
                feedback TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY(execution_id) REFERENCES execution_history(id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Votes table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS execution_votes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                execution_id INTEGER NOT NULL,
                vote_type TEXT NOT NULL CHECK(vote_type IN ('up', 'down')),
                created_at TEXT NOT NULL,
                FOREIGN KEY(execution_id) REFERENCES execution_history(id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Create indices for better query performance
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_execution_timestamp ON execution_history(timestamp DESC)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_execution_shell ON execution_history(shell_type)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_ratings_execution ON execution_ratings(execution_id)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_votes_execution ON execution_votes(execution_id)",
            [],
        )?;

        Ok(())
    }

    /// Add execution to history
    pub fn add_execution(&self, record: &ExecutionRecord) -> Result<i64> {
        let warnings_json = serde_json::to_string(&record.warnings)?;
        let alternatives_json = serde_json::to_string(&record.alternatives)?;

        self.conn.execute(
            "INSERT INTO execution_history
             (timestamp, prompt, generated_command, explanation, shell_type, risk_level,
              executed, blocked_reason, generation_time_ms, execution_time_ms,
              warnings, alternatives, backend_used)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                record.timestamp.to_rfc3339(),
                record.prompt,
                record.generated_command,
                record.explanation,
                record.shell_type,
                record.risk_level,
                record.executed,
                record.blocked_reason,
                record.generation_time_ms,
                record.execution_time_ms,
                warnings_json,
                alternatives_json,
                record.backend_used,
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Get execution history with optional filters
    pub fn get_execution_history(&self, filter: &HistoryFilter) -> Result<Vec<ExecutionRecord>> {
        let mut query = String::from(
            "SELECT id, timestamp, prompt, generated_command, explanation, shell_type,
             risk_level, executed, blocked_reason, generation_time_ms, execution_time_ms,
             warnings, alternatives, backend_used
             FROM execution_history WHERE 1=1",
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(shell) = &filter.shell_type {
            query.push_str(" AND shell_type = ?");
            params.push(Box::new(shell.clone()));
        }

        if let Some(risk) = &filter.risk_level {
            query.push_str(" AND risk_level = ?");
            params.push(Box::new(risk.clone()));
        }

        if let Some(executed) = filter.executed {
            query.push_str(" AND executed = ?");
            params.push(Box::new(executed));
        }

        if let Some(blocked) = filter.blocked {
            if blocked {
                query.push_str(" AND blocked_reason IS NOT NULL");
            } else {
                query.push_str(" AND blocked_reason IS NULL");
            }
        }

        if let Some(search) = &filter.search_query {
            query.push_str(" AND (prompt LIKE ? OR generated_command LIKE ?)");
            let search_pattern = format!("%{}%", search);
            params.push(Box::new(search_pattern.clone()));
            params.push(Box::new(search_pattern));
        }

        query.push_str(" ORDER BY timestamp DESC");

        if let Some(limit) = filter.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = filter.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        let mut stmt = self.conn.prepare(&query)?;
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let records = stmt
            .query_map(param_refs.as_slice(), |row| {
                let warnings_str: String = row.get(11)?;
                let alternatives_str: String = row.get(12)?;

                let warnings: Vec<String> =
                    serde_json::from_str(&warnings_str).unwrap_or_default();
                let alternatives: Vec<String> =
                    serde_json::from_str(&alternatives_str).unwrap_or_default();

                Ok(ExecutionRecord {
                    id: Some(row.get(0)?),
                    timestamp: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(1)?)
                        .unwrap()
                        .with_timezone(&Utc),
                    prompt: row.get(2)?,
                    generated_command: row.get(3)?,
                    explanation: row.get(4)?,
                    shell_type: row.get(5)?,
                    risk_level: row.get(6)?,
                    executed: row.get(7)?,
                    blocked_reason: row.get(8)?,
                    generation_time_ms: row.get(9)?,
                    execution_time_ms: row.get(10)?,
                    warnings,
                    alternatives,
                    backend_used: row.get(13)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(records)
    }

    /// Get single execution by ID
    pub fn get_execution_by_id(&self, id: i64) -> Result<Option<ExecutionRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, prompt, generated_command, explanation, shell_type,
             risk_level, executed, blocked_reason, generation_time_ms, execution_time_ms,
             warnings, alternatives, backend_used
             FROM execution_history WHERE id = ?1",
        )?;

        let result = stmt.query_row([id], |row| {
            let warnings_str: String = row.get(11)?;
            let alternatives_str: String = row.get(12)?;

            let warnings: Vec<String> = serde_json::from_str(&warnings_str).unwrap_or_default();
            let alternatives: Vec<String> =
                serde_json::from_str(&alternatives_str).unwrap_or_default();

            Ok(ExecutionRecord {
                id: Some(row.get(0)?),
                timestamp: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(1)?)
                    .unwrap()
                    .with_timezone(&Utc),
                prompt: row.get(2)?,
                generated_command: row.get(3)?,
                explanation: row.get(4)?,
                shell_type: row.get(5)?,
                risk_level: row.get(6)?,
                executed: row.get(7)?,
                blocked_reason: row.get(8)?,
                generation_time_ms: row.get(9)?,
                execution_time_ms: row.get(10)?,
                warnings,
                alternatives,
                backend_used: row.get(13)?,
            })
        });

        match result {
            Ok(record) => Ok(Some(record)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Delete execution record
    pub fn delete_execution(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM execution_history WHERE id = ?1", [id])?;
        Ok(())
    }

    /// Add rating for execution
    pub fn add_rating(&self, rating: &ExecutionRating) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO execution_ratings (execution_id, rating, feedback, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                rating.execution_id,
                rating.rating,
                rating.feedback,
                rating.created_at.to_rfc3339(),
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Get ratings for an execution
    pub fn get_ratings(&self, execution_id: i64) -> Result<Vec<ExecutionRating>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, execution_id, rating, feedback, created_at
             FROM execution_ratings WHERE execution_id = ?1 ORDER BY created_at DESC",
        )?;

        let ratings = stmt
            .query_map([execution_id], |row| {
                Ok(ExecutionRating {
                    id: Some(row.get(0)?),
                    execution_id: row.get(1)?,
                    rating: row.get(2)?,
                    feedback: row.get(3)?,
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                        .unwrap()
                        .with_timezone(&Utc),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ratings)
    }

    /// Add vote for execution
    pub fn add_vote(&self, vote: &ExecutionVote) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO execution_votes (execution_id, vote_type, created_at)
             VALUES (?1, ?2, ?3)",
            params![
                vote.execution_id,
                vote.vote_type.as_str(),
                vote.created_at.to_rfc3339(),
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Get votes for an execution
    pub fn get_votes(&self, execution_id: i64) -> Result<Vec<ExecutionVote>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, execution_id, vote_type, created_at
             FROM execution_votes WHERE execution_id = ?1 ORDER BY created_at DESC",
        )?;

        let votes = stmt
            .query_map([execution_id], |row| {
                let vote_str: String = row.get(2)?;
                Ok(ExecutionVote {
                    id: Some(row.get(0)?),
                    execution_id: row.get(1)?,
                    vote_type: VoteType::from_str(&vote_str).unwrap(),
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                        .unwrap()
                        .with_timezone(&Utc),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(votes)
    }

    /// Get analytics data
    pub fn get_analytics(&self) -> Result<Analytics> {
        // Total executions
        let total_executions: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM execution_history", [], |row| {
                row.get(0)
            })?;

        // Successful executions
        let successful_executions: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM execution_history WHERE executed = 1",
            [],
            |row| row.get(0),
        )?;

        // Blocked executions
        let blocked_executions: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM execution_history WHERE blocked_reason IS NOT NULL",
            [],
            |row| row.get(0),
        )?;

        // Average generation time
        let average_generation_time_ms: f64 = self.conn.query_row(
            "SELECT AVG(generation_time_ms) FROM execution_history",
            [],
            |row| row.get(0),
        )?;

        // Most used shell
        let most_used_shell: String = self
            .conn
            .query_row(
                "SELECT shell_type FROM execution_history
             GROUP BY shell_type ORDER BY COUNT(*) DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "bash".to_string());

        // Risk level distribution
        let mut stmt = self.conn.prepare(
            "SELECT risk_level, COUNT(*) as count FROM execution_history
             GROUP BY risk_level",
        )?;
        let risk_level_distribution: HashMap<String, i64> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .filter_map(Result::ok)
            .collect();

        // Backend usage
        let mut stmt = self.conn.prepare(
            "SELECT backend_used, COUNT(*) as count FROM execution_history
             GROUP BY backend_used",
        )?;
        let backend_usage: HashMap<String, i64> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .filter_map(Result::ok)
            .collect();

        Ok(Analytics {
            total_executions,
            successful_executions,
            blocked_executions,
            average_generation_time_ms,
            most_used_shell,
            risk_level_distribution,
            backend_usage,
        })
    }
}
