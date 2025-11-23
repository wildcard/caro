//! ML-powered risk prediction for commands
//!
//! This module provides risk prediction using machine learning models.
//! Phase 1: Rule-based predictor (ships now)
//! Phase 2: TensorFlow Lite model (future enhancement)

use anyhow::Result;
use serde::{Deserialize, Serialize};
use super::feature_extractor::{CommandFeatures, PrivilegeLevel, TargetScope};

/// Risk factor contributing to overall risk score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    pub name: String,
    pub severity: f32,      // 0.0-1.0
    pub explanation: String,
}

/// Blast radius indicating scope of potential impact
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlastRadius {
    /// Current directory only
    Local,
    /// Project directory
    Project,
    /// User home directory
    User,
    /// System-wide paths
    System,
    /// Network resources
    Network,
}

impl std::fmt::Display for BlastRadius {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Local => write!(f, "Local (current directory)"),
            Self::Project => write!(f, "Project (project root)"),
            Self::User => write!(f, "User (home directory)"),
            Self::System => write!(f, "System (system-wide)"),
            Self::Network => write!(f, "Network (external resources)"),
        }
    }
}

/// Impact estimate of command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactEstimate {
    pub files_affected: Option<usize>,
    pub data_loss_risk: f32,    // 0.0-1.0
    pub is_reversible: bool,
    pub blast_radius: BlastRadius,
    pub estimated_duration: Option<std::time::Duration>,
}

/// Complete risk prediction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskPrediction {
    pub risk_score: f32,        // 0.0-10.0
    pub confidence: f32,        // 0.0-1.0
    pub risk_factors: Vec<RiskFactor>,
    pub impact: ImpactEstimate,
    pub mitigations: Vec<String>,
}

impl RiskPrediction {
    /// Get risk level category
    pub fn risk_level(&self) -> crate::models::RiskLevel {
        match self.risk_score {
            s if s < 2.0 => crate::models::RiskLevel::Safe,
            s if s < 5.0 => crate::models::RiskLevel::Moderate,
            s if s < 8.0 => crate::models::RiskLevel::High,
            _ => crate::models::RiskLevel::Critical,
        }
    }
}

/// Trait for risk prediction backends
pub trait RiskPredictor {
    /// Predict risk for a command
    fn predict_risk(&self, command: &str, features: &CommandFeatures) -> Result<RiskPrediction>;
}

/// Rule-based risk predictor (Phase 1 implementation)
/// This provides immediate functionality while ML model is being trained
#[derive(Debug, Clone)]
pub struct RuleBasedPredictor {
    /// Maximum risk score to assign
    max_risk_score: f32,
    /// Confidence level for rule-based predictions
    confidence: f32,
}

impl RuleBasedPredictor {
    /// Create new rule-based predictor
    pub fn new() -> Self {
        Self {
            max_risk_score: 10.0,
            confidence: 0.95, // High confidence in rule-based matching
        }
    }

    /// Identify specific risk factors in the command
    fn identify_risk_factors(&self, command: &str, features: &CommandFeatures) -> Vec<RiskFactor> {
        let mut factors = Vec::new();

        // Check for recursive forced deletion
        if features.has_recursive_flag && features.has_force_flag {
            let severity = if features.target_scope == TargetScope::Root {
                1.0
            } else if features.target_scope == TargetScope::System {
                0.95
            } else {
                0.7
            };

            factors.push(RiskFactor {
                name: "Recursive forced deletion".to_string(),
                severity,
                explanation: "Command will delete files recursively without confirmation".to_string(),
            });
        }

        // Check for elevated privileges
        if features.privilege_level == PrivilegeLevel::Elevated {
            factors.push(RiskFactor {
                name: "Elevated privileges".to_string(),
                severity: 0.6,
                explanation: "Command runs with administrator privileges".to_string(),
            });
        } else if features.privilege_level == PrivilegeLevel::Root {
            factors.push(RiskFactor {
                name: "Root privileges".to_string(),
                severity: 0.8,
                explanation: "Command runs as root user".to_string(),
            });
        }

        // Check for system path modifications
        if features.has_system_path && features.destructive_score > 0.3 {
            factors.push(RiskFactor {
                name: "System path modification".to_string(),
                severity: 0.9,
                explanation: "Command modifies critical system directories".to_string(),
            });
        }

        // Check for root path operations
        if features.has_root_path && features.destructive_score > 0.5 {
            factors.push(RiskFactor {
                name: "Root filesystem operation".to_string(),
                severity: 1.0,
                explanation: "Command operates on root filesystem".to_string(),
            });
        }

        // Check for disk operations
        if features.is_disk_command {
            factors.push(RiskFactor {
                name: "Disk/filesystem operation".to_string(),
                severity: 0.85,
                explanation: "Command performs low-level disk operations".to_string(),
            });
        }

        // Check for fork bomb pattern
        if command.contains(":(){ :|:& };:") ||
           (command.contains("fork") && command.contains("while") && command.contains("&")) {
            factors.push(RiskFactor {
                name: "Fork bomb".to_string(),
                severity: 1.0,
                explanation: "Command appears to be a fork bomb (resource exhaustion attack)".to_string(),
            });
        }

        // Check for data loss patterns
        if features.destructive_score > 0.7 {
            let severity = if features.has_wildcard { 0.8 } else { 0.6 };
            factors.push(RiskFactor {
                name: "High data loss risk".to_string(),
                severity,
                explanation: "Command may cause irreversible data loss".to_string(),
            });
        }

        // Check for specific dangerous patterns
        if command.contains("rm -rf /") || command.contains("rm -rf /*") {
            factors.push(RiskFactor {
                name: "Critical filesystem destruction".to_string(),
                severity: 1.0,
                explanation: "Command will delete entire filesystem".to_string(),
            });
        }

        if command.contains("dd if=/dev/zero") || command.contains("dd if=/dev/random") {
            factors.push(RiskFactor {
                name: "Disk wipe operation".to_string(),
                severity: 1.0,
                explanation: "Command will overwrite disk with zeros/random data".to_string(),
            });
        }

        if command.contains("chmod 777") && features.has_recursive_flag {
            factors.push(RiskFactor {
                name: "Insecure permissions".to_string(),
                severity: 0.7,
                explanation: "Command sets world-writable permissions recursively".to_string(),
            });
        }

        factors
    }

    /// Calculate overall risk score from factors and features
    fn calculate_risk_score(&self, factors: &[RiskFactor], features: &CommandFeatures) -> f32 {
        if factors.is_empty() {
            return 0.0;
        }

        // Get maximum severity from factors
        let max_severity = factors.iter()
            .map(|f| f.severity)
            .fold(0.0f32, f32::max);

        // Base score from severity (0-8)
        let mut score = max_severity * 8.0;

        // Add modifiers based on features
        if features.privilege_level == PrivilegeLevel::Root {
            score += 1.0;
        } else if features.privilege_level == PrivilegeLevel::Elevated {
            score += 0.5;
        }

        if features.target_scope == TargetScope::Root {
            score += 1.5;
        } else if features.target_scope == TargetScope::System {
            score += 1.0;
        }

        if features.has_force_flag && features.has_recursive_flag {
            score += 0.5;
        }

        // Cap at max risk score
        score.min(self.max_risk_score)
    }

    /// Estimate impact of command execution
    fn estimate_impact(&self, command: &str, features: &CommandFeatures) -> ImpactEstimate {
        // Determine blast radius
        let blast_radius = match features.target_scope {
            TargetScope::SingleFile => BlastRadius::Local,
            TargetScope::LocalFiles => BlastRadius::Local,
            TargetScope::Recursive => BlastRadius::Project,
            TargetScope::System => BlastRadius::System,
            TargetScope::Root => BlastRadius::System,
            TargetScope::Network => BlastRadius::Network,
        };

        // Estimate files affected (heuristic)
        let files_affected = if features.has_wildcard {
            Some(100) // Approximate for wildcards
        } else if features.has_recursive_flag {
            Some(1000) // Approximate for recursive ops
        } else {
            Some(1) // Single file
        };

        // Calculate data loss risk
        let data_loss_risk = if features.destructive_score > 0.7 {
            0.9
        } else if features.destructive_score > 0.4 {
            0.6
        } else if features.destructive_score > 0.1 {
            0.3
        } else {
            0.0
        };

        // Determine reversibility
        let is_reversible = !features.is_disk_command &&
                           data_loss_risk < 0.7 &&
                           !command.contains("shred") &&
                           !command.contains("dd");

        // Estimate duration (placeholder)
        let estimated_duration = None;

        ImpactEstimate {
            files_affected,
            data_loss_risk,
            is_reversible,
            blast_radius,
            estimated_duration,
        }
    }

    /// Generate mitigation suggestions
    fn suggest_mitigations(&self, command: &str, factors: &[RiskFactor], features: &CommandFeatures) -> Vec<String> {
        let mut mitigations = Vec::new();

        // Suggest dry-run options
        if features.destructive_score > 0.5 {
            if command.starts_with("rm") {
                mitigations.push("Consider using 'rm -i' for interactive confirmation".to_string());
                mitigations.push("Run 'ls' first to preview files that will be deleted".to_string());
            }
        }

        // Suggest sandbox execution
        if factors.iter().any(|f| f.severity > 0.7) {
            mitigations.push("Execute in sandbox first to preview changes".to_string());
        }

        // Suggest removing force flag
        if features.has_force_flag && features.destructive_score > 0.5 {
            mitigations.push("Remove '-f' flag to see error messages and confirmations".to_string());
        }

        // Suggest limiting scope
        if features.has_wildcard || features.has_recursive_flag {
            mitigations.push("Limit operation to specific files instead of wildcards/recursive".to_string());
        }

        // Suggest privilege reduction
        if features.privilege_level != PrivilegeLevel::User {
            mitigations.push("Verify if elevated privileges are truly necessary".to_string());
        }

        mitigations
    }
}

impl Default for RuleBasedPredictor {
    fn default() -> Self {
        Self::new()
    }
}

impl RiskPredictor for RuleBasedPredictor {
    fn predict_risk(&self, command: &str, features: &CommandFeatures) -> Result<RiskPrediction> {
        // Identify specific risk factors
        let risk_factors = self.identify_risk_factors(command, features);

        // Calculate overall risk score
        let risk_score = self.calculate_risk_score(&risk_factors, features);

        // Estimate impact
        let impact = self.estimate_impact(command, features);

        // Generate mitigations
        let mitigations = self.suggest_mitigations(command, &risk_factors, features);

        Ok(RiskPrediction {
            risk_score,
            confidence: self.confidence,
            risk_factors,
            impact,
            mitigations,
        })
    }
}

// Placeholder for future ML-based predictor
/// ML-based risk predictor using TensorFlow Lite (Phase 2)
/// This will replace RuleBasedPredictor once trained model is available
#[allow(dead_code)]
pub struct MLPredictor {
    // model: TfLiteModel,
    // scaler: Scaler,
    fallback: RuleBasedPredictor,
}

#[allow(dead_code)]
impl MLPredictor {
    /// Create new ML predictor
    /// Currently uses rule-based fallback
    pub fn new() -> Result<Self> {
        Ok(Self {
            fallback: RuleBasedPredictor::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_command() {
        let predictor = RuleBasedPredictor::new();
        let features = CommandFeatures::extract("ls -la");
        let prediction = predictor.predict_risk("ls -la", &features).unwrap();

        assert!(prediction.risk_score < 2.0);
        assert_eq!(prediction.risk_level(), crate::models::RiskLevel::Safe);
        assert!(prediction.risk_factors.is_empty());
    }

    #[test]
    fn test_dangerous_rm_rf() {
        let predictor = RuleBasedPredictor::new();
        let features = CommandFeatures::extract("rm -rf /tmp/test");
        let prediction = predictor.predict_risk("rm -rf /tmp/test", &features).unwrap();

        assert!(prediction.risk_score >= 5.0);
        assert!(!prediction.risk_factors.is_empty());
        assert!(!prediction.mitigations.is_empty());
    }

    #[test]
    fn test_critical_rm_rf_root() {
        let predictor = RuleBasedPredictor::new();
        let features = CommandFeatures::extract("rm -rf /");
        let prediction = predictor.predict_risk("rm -rf /", &features).unwrap();

        assert!(prediction.risk_score >= 8.0);
        assert_eq!(prediction.risk_level(), crate::models::RiskLevel::Critical);
        assert!(prediction.risk_factors.iter().any(|f| f.severity >= 0.9));
    }

    #[test]
    fn test_sudo_command() {
        let predictor = RuleBasedPredictor::new();
        let features = CommandFeatures::extract("sudo apt-get install package");
        let prediction = predictor.predict_risk("sudo apt-get install package", &features).unwrap();

        assert!(prediction.risk_score > 0.0);
        assert!(prediction.risk_factors.iter().any(|f| f.name.contains("privilege")));
    }

    #[test]
    fn test_dd_command() {
        let predictor = RuleBasedPredictor::new();
        let features = CommandFeatures::extract("dd if=/dev/zero of=/dev/sda");
        let prediction = predictor.predict_risk("dd if=/dev/zero of=/dev/sda", &features).unwrap();

        assert!(prediction.risk_score >= 8.0);
        assert!(prediction.impact.data_loss_risk > 0.8);
        assert!(!prediction.impact.is_reversible);
    }

    #[test]
    fn test_impact_estimation() {
        let predictor = RuleBasedPredictor::new();
        let features = CommandFeatures::extract("rm -rf /tmp/test");
        let prediction = predictor.predict_risk("rm -rf /tmp/test", &features).unwrap();

        assert!(prediction.impact.files_affected.is_some());
        assert!(prediction.impact.data_loss_risk > 0.0);
    }

    #[test]
    fn test_mitigation_suggestions() {
        let predictor = RuleBasedPredictor::new();
        let features = CommandFeatures::extract("rm -rf /tmp/*");
        let prediction = predictor.predict_risk("rm -rf /tmp/*", &features).unwrap();

        assert!(!prediction.mitigations.is_empty());
    }
}
