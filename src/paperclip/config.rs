//! Paperclip AI environment variable detection and configuration.
//!
//! When Caro is launched as a Paperclip agent, the platform injects environment
//! variables that identify the agent, provide API credentials, and track the
//! current execution run. This module detects those variables and builds a
//! typed configuration struct.

use serde::{Deserialize, Serialize};
use std::env;

/// Environment variable names injected by Paperclip.
const ENV_AGENT_ID: &str = "PAPERCLIP_AGENT_ID";
const ENV_API_KEY: &str = "PAPERCLIP_API_KEY";
const ENV_API_URL: &str = "PAPERCLIP_API_URL";
const ENV_RUN_ID: &str = "PAPERCLIP_RUN_ID";

/// Configuration derived from Paperclip-injected environment variables.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperclipConfig {
    /// Unique identifier for this agent within the Paperclip org.
    pub agent_id: String,
    /// API key for authenticating with the Paperclip control plane.
    pub api_key: String,
    /// Base URL of the Paperclip API (e.g. `http://localhost:3000`).
    pub api_url: String,
    /// Identifier for the current heartbeat / execution run.
    pub run_id: String,
}

impl PaperclipConfig {
    /// Attempt to build configuration from environment variables.
    ///
    /// Returns `Some` only when **all** required `PAPERCLIP_*` variables are
    /// set and non-empty. Returns `None` if any are missing, which signals
    /// that Caro is running in normal (non-agent) mode.
    pub fn from_env() -> Option<Self> {
        let agent_id = non_empty_env(ENV_AGENT_ID)?;
        let api_key = non_empty_env(ENV_API_KEY)?;
        let api_url = non_empty_env(ENV_API_URL)?;
        let run_id = non_empty_env(ENV_RUN_ID)?;

        Some(Self {
            agent_id,
            api_key,
            api_url,
            run_id,
        })
    }

    /// Returns `true` when Paperclip environment variables are detected,
    /// without fully validating them.
    pub fn is_paperclip_env() -> bool {
        env::var(ENV_AGENT_ID).is_ok()
    }
}

/// Helper: read an env var, returning `None` for missing or empty values.
fn non_empty_env(key: &str) -> Option<String> {
    match env::var(key) {
        Ok(val) if !val.is_empty() => Some(val),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;

    #[test]
    #[serial]
    fn from_env_returns_none_when_missing() {
        // Ensure vars are not set (they shouldn't be in test env)
        env::remove_var(ENV_AGENT_ID);
        env::remove_var(ENV_API_KEY);
        env::remove_var(ENV_API_URL);
        env::remove_var(ENV_RUN_ID);

        assert!(PaperclipConfig::from_env().is_none());
    }

    #[test]
    #[serial]
    fn from_env_returns_none_when_partial() {
        env::set_var(ENV_AGENT_ID, "agent-1");
        env::set_var(ENV_API_KEY, "key-123");
        // Deliberately omit API_URL and RUN_ID
        env::remove_var(ENV_API_URL);
        env::remove_var(ENV_RUN_ID);

        assert!(PaperclipConfig::from_env().is_none());

        // Cleanup
        env::remove_var(ENV_AGENT_ID);
        env::remove_var(ENV_API_KEY);
    }

    #[test]
    #[serial]
    fn from_env_returns_config_when_all_set() {
        env::set_var(ENV_AGENT_ID, "agent-1");
        env::set_var(ENV_API_KEY, "key-123");
        env::set_var(ENV_API_URL, "http://localhost:3000");
        env::set_var(ENV_RUN_ID, "run-456");

        let config = PaperclipConfig::from_env().expect("should parse config");
        assert_eq!(config.agent_id, "agent-1");
        assert_eq!(config.api_key, "key-123");
        assert_eq!(config.api_url, "http://localhost:3000");
        assert_eq!(config.run_id, "run-456");

        // Cleanup
        env::remove_var(ENV_AGENT_ID);
        env::remove_var(ENV_API_KEY);
        env::remove_var(ENV_API_URL);
        env::remove_var(ENV_RUN_ID);
    }

    #[test]
    #[serial]
    fn from_env_rejects_empty_values() {
        env::set_var(ENV_AGENT_ID, "agent-1");
        env::set_var(ENV_API_KEY, "");
        env::set_var(ENV_API_URL, "http://localhost:3000");
        env::set_var(ENV_RUN_ID, "run-456");

        assert!(PaperclipConfig::from_env().is_none());

        // Cleanup
        env::remove_var(ENV_AGENT_ID);
        env::remove_var(ENV_API_KEY);
        env::remove_var(ENV_API_URL);
        env::remove_var(ENV_RUN_ID);
    }
}
