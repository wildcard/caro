//! Shared policy engine for kernel-level monitoring
//!
//! The policy engine evaluates syscall events against security rules and
//! returns allow/deny decisions. It bridges caro's existing safety patterns
//! (regex-based command validation) with kernel-level syscall enforcement.
//!
//! The same policy engine is used by both the Apple ES and eBPF backends,
//! ensuring consistent security posture across platforms.

use std::collections::HashSet;
use std::path::PathBuf;

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::models::RiskLevel;
use crate::safety::SafetyConfig;

use super::events::{
    NetworkProtocol, PolicyAction, PolicyDecision, SyscallCategory, SyscallDetail, SyscallEvent,
};

/// Security policy configuration
///
/// Loaded from YAML/TOML configuration files, or derived from
/// the existing `SafetyConfig` for backwards compatibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    /// Policy name for identification
    pub name: String,
    /// Ordered list of rules (first match wins)
    pub rules: Vec<PolicyRule>,
    /// Filesystem paths that are always allowed (bypass rules)
    pub allowed_paths: Vec<String>,
    /// Filesystem paths that are always blocked
    pub blocked_paths: Vec<String>,
    /// Network destinations that are always allowed
    pub allowed_network: Vec<NetworkRule>,
    /// Network destinations that are always blocked
    pub blocked_network: Vec<NetworkRule>,
}

/// A single policy rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    /// Rule name for audit logging
    pub name: String,
    /// Which syscall category this rule applies to
    pub category: SyscallCategory,
    /// Condition that triggers this rule
    pub condition: RuleCondition,
    /// Action to take when condition matches
    pub action: PolicyAction,
    /// Risk level for audit logging
    pub risk_level: RiskLevel,
}

/// Conditions for matching syscall events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleCondition {
    /// Match file paths starting with this prefix
    PathPrefix(String),
    /// Match exact file path
    PathExact(String),
    /// Match file path against regex
    PathRegex(String),
    /// Match process name (basename of executable)
    ProcessName(String),
    /// Match network port
    NetworkPort(u16),
    /// Match network CIDR range (e.g., "10.0.0.0/8")
    NetworkCidr(String),
    /// Match command arguments against regex (reuses safety patterns)
    CommandPattern(String),
    /// Always matches
    Always,
}

/// Network allow/block rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRule {
    /// CIDR or specific address
    pub address: String,
    /// Optional port (None = all ports)
    pub port: Option<u16>,
    /// Optional protocol filter
    pub protocol: Option<NetworkProtocol>,
}

/// A compiled policy rule ready for fast matching
struct CompiledRule {
    name: String,
    category: SyscallCategory,
    matcher: RuleMatcher,
    action: PolicyAction,
    risk_level: RiskLevel,
}

/// Pre-compiled matchers for each rule condition type
enum RuleMatcher {
    PathPrefix(String),
    PathExact(String),
    PathRegex(Regex),
    ProcessName(String),
    NetworkPort(u16),
    NetworkCidr(String),
    CommandPattern(Regex),
    Always,
}

/// The policy engine evaluates syscall events against compiled rules
pub struct PolicyEngine {
    policy: SecurityPolicy,
    compiled_rules: Vec<CompiledRule>,
    monitored_pids: HashSet<u32>,
    allowed_path_set: Vec<String>,
    blocked_path_set: Vec<String>,
}

impl PolicyEngine {
    /// Create a new policy engine from a security policy
    pub fn new(policy: SecurityPolicy) -> anyhow::Result<Self> {
        let mut compiled_rules = Vec::with_capacity(policy.rules.len());

        for rule in &policy.rules {
            let matcher = match &rule.condition {
                RuleCondition::PathPrefix(p) => RuleMatcher::PathPrefix(p.clone()),
                RuleCondition::PathExact(p) => RuleMatcher::PathExact(p.clone()),
                RuleCondition::PathRegex(p) => {
                    let regex = Regex::new(p)
                        .map_err(|e| anyhow::anyhow!("Invalid path regex '{}': {}", p, e))?;
                    RuleMatcher::PathRegex(regex)
                }
                RuleCondition::ProcessName(n) => RuleMatcher::ProcessName(n.clone()),
                RuleCondition::NetworkPort(port) => RuleMatcher::NetworkPort(*port),
                RuleCondition::NetworkCidr(cidr) => RuleMatcher::NetworkCidr(cidr.clone()),
                RuleCondition::CommandPattern(p) => {
                    let regex = Regex::new(p)
                        .map_err(|e| anyhow::anyhow!("Invalid command pattern '{}': {}", p, e))?;
                    RuleMatcher::CommandPattern(regex)
                }
                RuleCondition::Always => RuleMatcher::Always,
            };

            compiled_rules.push(CompiledRule {
                name: rule.name.clone(),
                category: rule.category.clone(),
                matcher,
                action: rule.action,
                risk_level: rule.risk_level,
            });
        }

        let allowed_path_set = policy.allowed_paths.clone();
        let blocked_path_set = policy.blocked_paths.clone();

        Ok(Self {
            policy,
            compiled_rules,
            monitored_pids: HashSet::new(),
            allowed_path_set,
            blocked_path_set,
        })
    }

    /// Create a policy engine from an existing `SafetyConfig`
    ///
    /// Converts caro's 52+ safety regex patterns into kernel-level policy rules.
    /// This ensures the same commands blocked at the CLI level are also blocked
    /// at the kernel level if pattern matching is somehow bypassed.
    pub fn from_safety_config(config: &SafetyConfig) -> anyhow::Result<Self> {
        let mut rules = Vec::new();

        // Convert built-in patterns from safety module
        let builtin_patterns = crate::safety::get_patterns_for_shell(crate::models::ShellType::Bash);
        for pattern in builtin_patterns {
            rules.push(PolicyRule {
                name: pattern.description.clone(),
                category: SyscallCategory::ProcessExec,
                condition: RuleCondition::CommandPattern(pattern.pattern.clone()),
                action: match pattern.risk_level {
                    RiskLevel::Critical => PolicyAction::Deny,
                    RiskLevel::High => PolicyAction::Deny,
                    RiskLevel::Moderate => PolicyAction::AuditAllow,
                    RiskLevel::Safe => PolicyAction::Allow,
                },
                risk_level: pattern.risk_level,
            });
        }

        // Convert custom patterns
        for pattern in &config.custom_patterns {
            rules.push(PolicyRule {
                name: pattern.description.clone(),
                category: SyscallCategory::ProcessExec,
                condition: RuleCondition::CommandPattern(pattern.pattern.clone()),
                action: match pattern.risk_level {
                    RiskLevel::Critical | RiskLevel::High => PolicyAction::Deny,
                    RiskLevel::Moderate => PolicyAction::AuditAllow,
                    RiskLevel::Safe => PolicyAction::Allow,
                },
                risk_level: pattern.risk_level,
            });
        }

        // Default path-based rules for defense-in-depth
        let blocked_paths = vec![
            "/dev/sda".to_string(),
            "/dev/sdb".to_string(),
            "/dev/nvme".to_string(),
            "/dev/hda".to_string(),
        ];

        let policy = SecurityPolicy {
            name: format!("auto-from-safety-{}", config.safety_level),
            rules,
            allowed_paths: Vec::new(),
            blocked_paths,
            allowed_network: Vec::new(),
            blocked_network: Vec::new(),
        };

        Self::new(policy)
    }

    /// Evaluate a syscall event against the policy
    ///
    /// Returns a `PolicyDecision` with the action to take and audit metadata.
    /// Uses first-match semantics: the first matching rule determines the action.
    pub fn evaluate(&self, event: &SyscallEvent) -> PolicyDecision {
        // Check if this PID is being monitored (empty set = monitor all)
        if !self.monitored_pids.is_empty() && !self.monitored_pids.contains(&event.pid) {
            return PolicyDecision {
                event_id: event.id,
                action: PolicyAction::Allow,
                reason: "Process not in monitored set".into(),
                matched_rules: vec![],
                risk_level: RiskLevel::Safe,
            };
        }

        // Fast path: check blocked/allowed path lists for file operations
        if let Some(path) = self.extract_path(&event.detail) {
            let path_str = path.to_string_lossy();

            // Blocked paths take priority
            for blocked in &self.blocked_path_set {
                if path_str.starts_with(blocked) {
                    return PolicyDecision {
                        event_id: event.id,
                        action: PolicyAction::Deny,
                        reason: format!("Path {} is in blocked list", path_str),
                        matched_rules: vec!["blocked_path".into()],
                        risk_level: RiskLevel::Critical,
                    };
                }
            }

            // Allowed paths bypass further checks
            for allowed in &self.allowed_path_set {
                if path_str.starts_with(allowed) {
                    return PolicyDecision {
                        event_id: event.id,
                        action: PolicyAction::Allow,
                        reason: format!("Path {} is in allowed list", path_str),
                        matched_rules: vec!["allowed_path".into()],
                        risk_level: RiskLevel::Safe,
                    };
                }
            }
        }

        // Evaluate rules in order (first match wins)
        let mut matched_rules = Vec::new();
        let mut highest_risk = RiskLevel::Safe;
        let mut final_action = PolicyAction::Allow;

        for rule in &self.compiled_rules {
            if rule.category != event.category {
                continue;
            }

            if self.matches_rule(rule, event) {
                matched_rules.push(rule.name.clone());
                if rule.risk_level > highest_risk {
                    highest_risk = rule.risk_level;
                    final_action = rule.action;
                }
            }
        }

        let reason = if matched_rules.is_empty() {
            "No policy rules matched".to_string()
        } else {
            format!(
                "Matched {} rule(s): {}",
                matched_rules.len(),
                matched_rules.join(", ")
            )
        };

        PolicyDecision {
            event_id: event.id,
            action: final_action,
            reason,
            matched_rules,
            risk_level: highest_risk,
        }
    }

    /// Add a PID to the monitored set
    pub fn add_monitored_pid(&mut self, pid: u32) {
        self.monitored_pids.insert(pid);
    }

    /// Remove a PID from the monitored set
    pub fn remove_monitored_pid(&mut self, pid: u32) {
        self.monitored_pids.remove(&pid);
    }

    /// Get current set of monitored PIDs
    pub fn monitored_pids(&self) -> &HashSet<u32> {
        &self.monitored_pids
    }

    /// Get the policy name
    pub fn policy_name(&self) -> &str {
        &self.policy.name
    }

    /// Check if a compiled rule matches a syscall event
    fn matches_rule(&self, rule: &CompiledRule, event: &SyscallEvent) -> bool {
        match &rule.matcher {
            RuleMatcher::PathPrefix(prefix) => {
                self.extract_path(&event.detail)
                    .map(|p| p.to_string_lossy().starts_with(prefix))
                    .unwrap_or(false)
            }
            RuleMatcher::PathExact(exact) => {
                self.extract_path(&event.detail)
                    .map(|p| p.to_string_lossy() == *exact)
                    .unwrap_or(false)
            }
            RuleMatcher::PathRegex(regex) => {
                self.extract_path(&event.detail)
                    .map(|p| regex.is_match(&p.to_string_lossy()))
                    .unwrap_or(false)
            }
            RuleMatcher::ProcessName(name) => {
                event
                    .process_path
                    .file_name()
                    .map(|n| n.to_string_lossy() == *name)
                    .unwrap_or(false)
            }
            RuleMatcher::NetworkPort(port) => match &event.detail {
                SyscallDetail::NetworkConnect { port: p, .. }
                | SyscallDetail::NetworkBind { port: p, .. } => p == port,
                _ => false,
            },
            RuleMatcher::NetworkCidr(cidr) => match &event.detail {
                SyscallDetail::NetworkConnect { address, .. }
                | SyscallDetail::NetworkBind { address, .. } => {
                    // Simple prefix match for now; full CIDR parsing in future
                    address.starts_with(cidr.split('/').next().unwrap_or(cidr))
                }
                _ => false,
            },
            RuleMatcher::CommandPattern(regex) => match &event.detail {
                SyscallDetail::Exec { args, .. } => {
                    let full_command = args.join(" ");
                    regex.is_match(&full_command)
                }
                _ => false,
            },
            RuleMatcher::Always => true,
        }
    }

    /// Extract the primary file path from a syscall detail
    fn extract_path(&self, detail: &SyscallDetail) -> Option<PathBuf> {
        match detail {
            SyscallDetail::Exec { path, .. } => Some(path.clone()),
            SyscallDetail::FileOpen { path, .. } => Some(path.clone()),
            SyscallDetail::FileWrite { path, .. } => Some(path.clone()),
            SyscallDetail::FileDelete { path, .. } => Some(path.clone()),
            SyscallDetail::FileRename { source, .. } => Some(source.clone()),
            SyscallDetail::NetworkConnect { .. }
            | SyscallDetail::NetworkBind { .. }
            | SyscallDetail::Signal { .. } => None,
        }
    }
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            rules: Vec::new(),
            allowed_paths: Vec::new(),
            blocked_paths: vec![
                "/dev/sda".to_string(),
                "/dev/sdb".to_string(),
                "/dev/nvme".to_string(),
            ],
            allowed_network: Vec::new(),
            blocked_network: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monitor::events::*;
    use std::path::PathBuf;

    fn make_exec_event(args: Vec<&str>) -> SyscallEvent {
        let path = PathBuf::from(args.first().copied().unwrap_or("/bin/sh"));
        SyscallEvent::new(
            100, 1, 501, path.clone(),
            SyscallDetail::Exec {
                path,
                args: args.into_iter().map(String::from).collect(),
                env_count: 0,
            },
        )
    }

    fn make_file_event(path: &str) -> SyscallEvent {
        SyscallEvent::new(
            100, 1, 501, PathBuf::from("/bin/cat"),
            SyscallDetail::FileOpen { path: PathBuf::from(path), flags: 0 },
        )
    }

    fn make_net_event(address: &str, port: u16) -> SyscallEvent {
        SyscallEvent::new(
            100, 1, 501, PathBuf::from("/usr/bin/curl"),
            SyscallDetail::NetworkConnect {
                address: address.into(),
                port,
                protocol: NetworkProtocol::Tcp,
            },
        )
    }

    #[test]
    fn test_default_policy_allows_safe_commands() {
        let policy = SecurityPolicy::default();
        let engine = PolicyEngine::new(policy).unwrap();

        let event = make_exec_event(vec!["ls", "-la"]);
        let decision = engine.evaluate(&event);
        assert_eq!(decision.action, PolicyAction::Allow);
    }

    #[test]
    fn test_blocked_path_denies_access() {
        let policy = SecurityPolicy {
            blocked_paths: vec!["/dev/sda".to_string()],
            ..SecurityPolicy::default()
        };
        let engine = PolicyEngine::new(policy).unwrap();

        let event = make_file_event("/dev/sda1");
        let decision = engine.evaluate(&event);
        assert_eq!(decision.action, PolicyAction::Deny);
        assert_eq!(decision.risk_level, RiskLevel::Critical);
    }

    #[test]
    fn test_allowed_path_bypasses_rules() {
        let policy = SecurityPolicy {
            allowed_paths: vec!["/tmp/".to_string()],
            rules: vec![PolicyRule {
                name: "block_all_files".into(),
                category: SyscallCategory::FileOperation,
                condition: RuleCondition::Always,
                action: PolicyAction::Deny,
                risk_level: RiskLevel::High,
            }],
            ..SecurityPolicy::default()
        };
        let engine = PolicyEngine::new(policy).unwrap();

        let event = make_file_event("/tmp/safe_file.txt");
        let decision = engine.evaluate(&event);
        assert_eq!(decision.action, PolicyAction::Allow);
    }

    #[test]
    fn test_command_pattern_matching() {
        let policy = SecurityPolicy {
            rules: vec![PolicyRule {
                name: "block_rm_rf".into(),
                category: SyscallCategory::ProcessExec,
                condition: RuleCondition::CommandPattern(r"rm\s+-rf\s+/".into()),
                action: PolicyAction::Deny,
                risk_level: RiskLevel::Critical,
            }],
            ..SecurityPolicy::default()
        };
        let engine = PolicyEngine::new(policy).unwrap();

        let dangerous = make_exec_event(vec!["rm", "-rf", "/"]);
        let decision = engine.evaluate(&dangerous);
        assert_eq!(decision.action, PolicyAction::Deny);
        assert_eq!(decision.risk_level, RiskLevel::Critical);

        let safe = make_exec_event(vec!["rm", "temp.txt"]);
        let decision = engine.evaluate(&safe);
        assert_eq!(decision.action, PolicyAction::Allow);
    }

    #[test]
    fn test_network_port_rule() {
        let policy = SecurityPolicy {
            rules: vec![PolicyRule {
                name: "block_ssh".into(),
                category: SyscallCategory::NetworkOperation,
                condition: RuleCondition::NetworkPort(22),
                action: PolicyAction::Deny,
                risk_level: RiskLevel::High,
            }],
            ..SecurityPolicy::default()
        };
        let engine = PolicyEngine::new(policy).unwrap();

        let ssh = make_net_event("192.168.1.1", 22);
        let decision = engine.evaluate(&ssh);
        assert_eq!(decision.action, PolicyAction::Deny);

        let https = make_net_event("192.168.1.1", 443);
        let decision = engine.evaluate(&https);
        assert_eq!(decision.action, PolicyAction::Allow);
    }

    #[test]
    fn test_process_name_rule() {
        let policy = SecurityPolicy {
            rules: vec![PolicyRule {
                name: "block_nc".into(),
                category: SyscallCategory::ProcessExec,
                condition: RuleCondition::ProcessName("nc".into()),
                action: PolicyAction::Deny,
                risk_level: RiskLevel::High,
            }],
            ..SecurityPolicy::default()
        };
        let engine = PolicyEngine::new(policy).unwrap();

        let nc_event = SyscallEvent::new(
            100, 1, 501, PathBuf::from("/usr/bin/nc"),
            SyscallDetail::Exec {
                path: PathBuf::from("/usr/bin/nc"),
                args: vec!["nc".into(), "-l".into(), "4444".into()],
                env_count: 0,
            },
        );
        let decision = engine.evaluate(&nc_event);
        assert_eq!(decision.action, PolicyAction::Deny);
    }

    #[test]
    fn test_monitored_pid_filtering() {
        let policy = SecurityPolicy::default();
        let mut engine = PolicyEngine::new(policy).unwrap();

        // With no monitored PIDs, all events are evaluated
        let event = make_exec_event(vec!["ls"]);
        let decision = engine.evaluate(&event);
        assert_eq!(decision.action, PolicyAction::Allow);

        // Add PID 200 to monitored set — PID 100 should be skipped
        engine.add_monitored_pid(200);
        let decision = engine.evaluate(&event); // PID 100
        assert_eq!(decision.action, PolicyAction::Allow);
        assert_eq!(decision.reason, "Process not in monitored set");
    }

    #[test]
    fn test_from_safety_config() {
        let config = SafetyConfig::strict();
        let engine = PolicyEngine::from_safety_config(&config).unwrap();

        // Should have rules derived from built-in safety patterns
        assert!(!engine.compiled_rules.is_empty());
        assert_eq!(engine.policy_name(), "auto-from-safety-strict");
    }

    #[test]
    fn test_invalid_regex_returns_error() {
        let policy = SecurityPolicy {
            rules: vec![PolicyRule {
                name: "bad_regex".into(),
                category: SyscallCategory::ProcessExec,
                condition: RuleCondition::PathRegex("[invalid".into()),
                action: PolicyAction::Deny,
                risk_level: RiskLevel::High,
            }],
            ..SecurityPolicy::default()
        };
        assert!(PolicyEngine::new(policy).is_err());
    }
}
