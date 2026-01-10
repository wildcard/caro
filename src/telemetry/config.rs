//! Telemetry configuration

use serde::{Deserialize, Serialize};

/// Telemetry configuration
///
/// Controls telemetry behavior including enable/disable, level, and air-gapped mode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TelemetryConfig {
    /// Telemetry enabled
    ///
    /// - Beta (v1.1.0-beta): Default `true` (opt-out)
    /// - GA (v1.1.0+): Default `false` (opt-in)
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Telemetry level
    ///
    /// Controls verbosity of telemetry collection:
    /// - `Minimal`: Only critical events (errors, safety blocks)
    /// - `Normal`: Standard events (sessions, commands, errors)
    /// - `Verbose`: Detailed events (performance metrics, debug info)
    #[serde(default)]
    pub level: TelemetryLevel,

    /// Air-gapped mode
    ///
    /// When enabled:
    /// - Events stored locally only
    /// - No automatic uploads
    /// - Use `caro telemetry export` to manually extract data
    #[serde(default)]
    pub air_gapped: bool,

    /// Upload endpoint
    ///
    /// Default: `https://telemetry.caro.sh/api/events`
    #[serde(default = "default_endpoint")]
    pub endpoint: String,

    /// First run flag
    ///
    /// Used to determine if we should show consent prompt
    #[serde(default = "default_first_run")]
    pub first_run: bool,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            level: TelemetryLevel::default(),
            air_gapped: false,
            endpoint: default_endpoint(),
            first_run: true,
        }
    }
}

fn default_enabled() -> bool {
    // Beta: opt-out (true), GA: opt-in (false)
    let version = env!("CARGO_PKG_VERSION");
    version.contains("beta")
}

fn default_endpoint() -> String {
    "https://telemetry.caro.sh/api/events".to_string()
}

fn default_first_run() -> bool {
    true
}

/// Telemetry collection level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum TelemetryLevel {
    /// Minimal telemetry - only critical events
    ///
    /// Collects:
    /// - Safety validation blocks
    /// - Fatal errors
    /// - Backend failures
    Minimal,

    /// Normal telemetry - standard events
    ///
    /// Collects:
    /// - All Minimal events
    /// - Session start/end
    /// - Command generation success/failure
    /// - Non-fatal errors
    #[default]
    Normal,

    /// Verbose telemetry - detailed debug events
    ///
    /// Collects:
    /// - All Normal events
    /// - Performance metrics (latency percentiles)
    /// - Model inference details
    /// - Backend fallback chains
    Verbose,
}

impl std::fmt::Display for TelemetryLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TelemetryLevel::Minimal => write!(f, "minimal"),
            TelemetryLevel::Normal => write!(f, "normal"),
            TelemetryLevel::Verbose => write!(f, "verbose"),
        }
    }
}

impl std::str::FromStr for TelemetryLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "minimal" => Ok(TelemetryLevel::Minimal),
            "normal" => Ok(TelemetryLevel::Normal),
            "verbose" => Ok(TelemetryLevel::Verbose),
            _ => Err(format!("Invalid telemetry level: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = TelemetryConfig::default();

        // Beta version should have telemetry enabled by default
        let version = env!("CARGO_PKG_VERSION");
        if version.contains("beta") {
            assert!(config.enabled);
        } else {
            assert!(!config.enabled);
        }

        assert_eq!(config.level, TelemetryLevel::Normal);
        assert!(!config.air_gapped);
        assert_eq!(config.endpoint, "https://telemetry.caro.sh/api/events");
    }

    #[test]
    fn test_telemetry_level_parsing() {
        assert_eq!(
            "minimal".parse::<TelemetryLevel>().unwrap(),
            TelemetryLevel::Minimal
        );
        assert_eq!(
            "normal".parse::<TelemetryLevel>().unwrap(),
            TelemetryLevel::Normal
        );
        assert_eq!(
            "verbose".parse::<TelemetryLevel>().unwrap(),
            TelemetryLevel::Verbose
        );
        assert_eq!(
            "MINIMAL".parse::<TelemetryLevel>().unwrap(),
            TelemetryLevel::Minimal
        );

        assert!("invalid".parse::<TelemetryLevel>().is_err());
    }

    #[test]
    fn test_telemetry_level_display() {
        assert_eq!(TelemetryLevel::Minimal.to_string(), "minimal");
        assert_eq!(TelemetryLevel::Normal.to_string(), "normal");
        assert_eq!(TelemetryLevel::Verbose.to_string(), "verbose");
    }

    #[test]
    fn test_config_serialization() {
        let config = TelemetryConfig {
            enabled: true,
            level: TelemetryLevel::Verbose,
            air_gapped: true,
            endpoint: "https://custom.endpoint.com/api".to_string(),
            first_run: false,
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"enabled\":true"));
        assert!(json.contains("\"verbose\""));
        assert!(json.contains("\"air_gapped\":true"));

        let deserialized: TelemetryConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.enabled, config.enabled);
        assert_eq!(deserialized.level, config.level);
        assert_eq!(deserialized.air_gapped, config.air_gapped);
    }
}
