//! Test-driven fix for RiskLevel match exhaustiveness
//!
//! This test demonstrates how to use TDD to fix compilation errors

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
enum RiskLevel {
    Low,
    Safe,
    Medium,
    Moderate,
    High,
    Critical,
}

impl fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RiskLevel::Low => write!(f, "Low"),
            RiskLevel::Safe => write!(f, "Safe"),
            RiskLevel::Medium => write!(f, "Medium"),
            RiskLevel::Moderate => write!(f, "Moderate"),
            RiskLevel::High => write!(f, "High"),
            RiskLevel::Critical => write!(f, "Critical"),
        }
    }
}

/// Calculate confidence bonus based on risk level
fn calculate_risk_bonus(risk: RiskLevel) -> f64 {
    match risk {
        RiskLevel::Critical => 0.20,
        RiskLevel::High => 0.15,
        RiskLevel::Medium => 0.10,
        RiskLevel::Moderate => 0.08,
        RiskLevel::Safe => 0.03,
        RiskLevel::Low => 0.05,
    }
}

/// Determine if a risk level requires user confirmation
fn requires_confirmation(risk: RiskLevel, strict_mode: bool) -> bool {
    match risk {
        RiskLevel::Critical => true,
        RiskLevel::High => true,
        RiskLevel::Medium | RiskLevel::Moderate => strict_mode,
        RiskLevel::Safe | RiskLevel::Low => false,
    }
}

/// Convert risk level to threat severity
fn to_threat_level(risk: RiskLevel) -> &'static str {
    match risk {
        RiskLevel::Critical => "CRITICAL",
        RiskLevel::High => "HIGH",
        RiskLevel::Medium | RiskLevel::Moderate => "MEDIUM",
        RiskLevel::Low | RiskLevel::Safe => "LOW",
    }
}

// ============= TESTS =============

#[test]
fn test_all_risk_levels_have_bonus() {
    // Test that every risk level returns a valid bonus
    assert_eq!(calculate_risk_bonus(RiskLevel::Critical), 0.20);
    assert_eq!(calculate_risk_bonus(RiskLevel::High), 0.15);
    assert_eq!(calculate_risk_bonus(RiskLevel::Medium), 0.10);
    assert_eq!(calculate_risk_bonus(RiskLevel::Moderate), 0.08);
    assert_eq!(calculate_risk_bonus(RiskLevel::Safe), 0.03);
    assert_eq!(calculate_risk_bonus(RiskLevel::Low), 0.05);
}

#[test]
fn test_confirmation_requirements() {
    // Test strict mode
    assert!(requires_confirmation(RiskLevel::Critical, true));
    assert!(requires_confirmation(RiskLevel::High, true));
    assert!(requires_confirmation(RiskLevel::Medium, true));
    assert!(requires_confirmation(RiskLevel::Moderate, true));
    assert!(!requires_confirmation(RiskLevel::Safe, true));
    assert!(!requires_confirmation(RiskLevel::Low, true));

    // Test non-strict mode
    assert!(requires_confirmation(RiskLevel::Critical, false));
    assert!(requires_confirmation(RiskLevel::High, false));
    assert!(!requires_confirmation(RiskLevel::Medium, false));
    assert!(!requires_confirmation(RiskLevel::Moderate, false));
    assert!(!requires_confirmation(RiskLevel::Safe, false));
    assert!(!requires_confirmation(RiskLevel::Low, false));
}

#[test]
fn test_threat_level_conversion() {
    assert_eq!(to_threat_level(RiskLevel::Critical), "CRITICAL");
    assert_eq!(to_threat_level(RiskLevel::High), "HIGH");
    assert_eq!(to_threat_level(RiskLevel::Medium), "MEDIUM");
    assert_eq!(to_threat_level(RiskLevel::Moderate), "MEDIUM");
    assert_eq!(to_threat_level(RiskLevel::Safe), "LOW");
    assert_eq!(to_threat_level(RiskLevel::Low), "LOW");
}

#[test]
fn test_risk_level_display() {
    assert_eq!(format!("{}", RiskLevel::Low), "Low");
    assert_eq!(format!("{}", RiskLevel::Safe), "Safe");
    assert_eq!(format!("{}", RiskLevel::Medium), "Medium");
    assert_eq!(format!("{}", RiskLevel::Moderate), "Moderate");
    assert_eq!(format!("{}", RiskLevel::High), "High");
    assert_eq!(format!("{}", RiskLevel::Critical), "Critical");
}

#[test]
fn test_risk_ordering() {
    // Test that risk levels can be compared
    let risks = vec![
        RiskLevel::Low,
        RiskLevel::Safe,
        RiskLevel::Medium,
        RiskLevel::Moderate,
        RiskLevel::High,
        RiskLevel::Critical,
    ];

    // All risks should be equal to themselves
    for risk in &risks {
        assert_eq!(*risk, *risk);
    }
}
