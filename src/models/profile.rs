//! User profile types for personalized knowledge management
//!
//! Profiles allow users to maintain separate command knowledge for different contexts
//! (work, personal, devops) with scoped queries and isolated namespaces.

use chrono::{DateTime, Utc};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Type of user profile
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum ProfileType {
    /// Work-related commands and patterns
    Work,
    /// Personal projects and experiments
    Personal,
    /// DevOps and infrastructure commands
    DevOps,
}

impl fmt::Display for ProfileType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProfileType::Work => write!(f, "work"),
            ProfileType::Personal => write!(f, "personal"),
            ProfileType::DevOps => write!(f, "devops"),
        }
    }
}

impl std::str::FromStr for ProfileType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "work" => Ok(ProfileType::Work),
            "personal" => Ok(ProfileType::Personal),
            "devops" => Ok(ProfileType::DevOps),
            _ => Err(format!("Invalid profile type: {}", s)),
        }
    }
}

/// User profile for scoped knowledge management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    /// Unique profile name (e.g., "work", "personal-laptop", "company-devops")
    pub name: String,

    /// Type of profile
    #[serde(rename = "type")]
    pub profile_type: ProfileType,

    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// When this profile was created
    pub created: DateTime<Utc>,

    /// Last time this profile was active
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used: Option<DateTime<Utc>>,

    /// Command execution count for this profile
    #[serde(default)]
    pub command_count: usize,
}

impl UserProfile {
    /// Create a new user profile
    pub fn new(name: String, profile_type: ProfileType) -> Self {
        Self {
            name,
            profile_type,
            description: None,
            created: Utc::now(),
            last_used: None,
            command_count: 0,
        }
    }

    /// Create a new profile with description
    pub fn with_description(name: String, profile_type: ProfileType, description: String) -> Self {
        Self {
            name,
            profile_type,
            description: Some(description),
            created: Utc::now(),
            last_used: None,
            command_count: 0,
        }
    }

    /// Mark this profile as used
    pub fn mark_used(&mut self) {
        self.last_used = Some(Utc::now());
        self.command_count += 1;
    }
}

/// Profile configuration stored in config file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileConfig {
    /// All available profiles
    pub profiles: Vec<UserProfile>,

    /// Currently active profile name
    pub active_profile: Option<String>,
}

impl ProfileConfig {
    /// Create empty profile configuration
    pub fn new() -> Self {
        Self {
            profiles: Vec::new(),
            active_profile: None,
        }
    }

    /// Get the active profile
    pub fn get_active(&self) -> Option<&UserProfile> {
        self.active_profile
            .as_ref()
            .and_then(|name| self.profiles.iter().find(|p| &p.name == name))
    }

    /// Get a profile by name
    pub fn get_profile(&self, name: &str) -> Option<&UserProfile> {
        self.profiles.iter().find(|p| p.name == name)
    }

    /// Get a mutable profile by name
    pub fn get_profile_mut(&mut self, name: &str) -> Option<&mut UserProfile> {
        self.profiles.iter_mut().find(|p| p.name == name)
    }

    /// Add a new profile
    pub fn add_profile(&mut self, profile: UserProfile) -> Result<(), String> {
        // Check for duplicate names
        if self.profiles.iter().any(|p| p.name == profile.name) {
            return Err(format!("Profile '{}' already exists", profile.name));
        }

        self.profiles.push(profile);
        Ok(())
    }

    /// Remove a profile by name
    pub fn remove_profile(&mut self, name: &str) -> Result<(), String> {
        let index = self
            .profiles
            .iter()
            .position(|p| p.name == name)
            .ok_or_else(|| format!("Profile '{}' not found", name))?;

        self.profiles.remove(index);

        // If this was the active profile, clear it
        if self.active_profile.as_ref() == Some(&name.to_string()) {
            self.active_profile = None;
        }

        Ok(())
    }

    /// Switch to a different profile
    pub fn switch_profile(&mut self, name: &str) -> Result<(), String> {
        // Verify profile exists
        if !self.profiles.iter().any(|p| p.name == name) {
            return Err(format!("Profile '{}' not found", name));
        }

        self.active_profile = Some(name.to_string());
        Ok(())
    }

    /// List all profile names
    pub fn list_profiles(&self) -> Vec<&str> {
        self.profiles.iter().map(|p| p.name.as_str()).collect()
    }
}

impl Default for ProfileConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_type_from_str() {
        assert_eq!("work".parse::<ProfileType>().unwrap(), ProfileType::Work);
        assert_eq!(
            "personal".parse::<ProfileType>().unwrap(),
            ProfileType::Personal
        );
        assert_eq!(
            "devops".parse::<ProfileType>().unwrap(),
            ProfileType::DevOps
        );
        assert!("invalid".parse::<ProfileType>().is_err());
    }

    #[test]
    fn test_profile_creation() {
        let profile = UserProfile::new("work".to_string(), ProfileType::Work);
        assert_eq!(profile.name, "work");
        assert_eq!(profile.profile_type, ProfileType::Work);
        assert!(profile.description.is_none());
        assert!(profile.last_used.is_none());
        assert_eq!(profile.command_count, 0);
    }

    #[test]
    fn test_profile_mark_used() {
        let mut profile = UserProfile::new("work".to_string(), ProfileType::Work);
        assert_eq!(profile.command_count, 0);

        profile.mark_used();
        assert_eq!(profile.command_count, 1);
        assert!(profile.last_used.is_some());
    }

    #[test]
    fn test_profile_config_add_remove() {
        let mut config = ProfileConfig::new();

        // Add profiles
        let work = UserProfile::new("work".to_string(), ProfileType::Work);
        let personal = UserProfile::new("personal".to_string(), ProfileType::Personal);

        assert!(config.add_profile(work).is_ok());
        assert!(config.add_profile(personal).is_ok());
        assert_eq!(config.profiles.len(), 2);

        // Try to add duplicate
        let duplicate = UserProfile::new("work".to_string(), ProfileType::Work);
        assert!(config.add_profile(duplicate).is_err());

        // Remove profile
        assert!(config.remove_profile("work").is_ok());
        assert_eq!(config.profiles.len(), 1);

        // Try to remove non-existent
        assert!(config.remove_profile("nonexistent").is_err());
    }

    #[test]
    fn test_profile_config_switch() {
        let mut config = ProfileConfig::new();

        let work = UserProfile::new("work".to_string(), ProfileType::Work);
        let personal = UserProfile::new("personal".to_string(), ProfileType::Personal);

        config.add_profile(work).unwrap();
        config.add_profile(personal).unwrap();

        // Switch to work
        assert!(config.switch_profile("work").is_ok());
        assert_eq!(config.active_profile, Some("work".to_string()));
        assert_eq!(config.get_active().unwrap().name, "work");

        // Switch to personal
        assert!(config.switch_profile("personal").is_ok());
        assert_eq!(config.get_active().unwrap().name, "personal");

        // Try to switch to non-existent
        assert!(config.switch_profile("nonexistent").is_err());
    }

    #[test]
    fn test_profile_config_remove_active() {
        let mut config = ProfileConfig::new();

        let work = UserProfile::new("work".to_string(), ProfileType::Work);
        config.add_profile(work).unwrap();
        config.switch_profile("work").unwrap();

        assert!(config.active_profile.is_some());

        // Remove active profile should clear active_profile
        config.remove_profile("work").unwrap();
        assert!(config.active_profile.is_none());
    }
}
