//! User profile management for personalized command generation

use crate::models::{SafetyLevel, ShellType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info};
use uuid::Uuid;

/// User profile for personalized command generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    /// Unique profile ID
    pub id: String,

    /// Profile name (e.g., "work", "personal", "devops")
    pub name: String,

    /// Description of this profile's use case
    pub description: String,

    /// Preferred shell type
    pub shell_preference: ShellType,

    /// Safety level preference
    pub safety_level: SafetyLevel,

    /// Frequently used command patterns
    pub command_patterns: Vec<String>,

    /// Custom aliases or shortcuts
    pub aliases: HashMap<String, String>,

    /// Profile creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last used timestamp
    pub last_used: DateTime<Utc>,

    /// Usage statistics
    pub usage_count: u64,
}

impl UserProfile {
    /// Create a new user profile
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        shell_preference: ShellType,
        safety_level: SafetyLevel,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            description: description.into(),
            shell_preference,
            safety_level,
            command_patterns: Vec::new(),
            aliases: HashMap::new(),
            created_at: now,
            last_used: now,
            usage_count: 0,
        }
    }

    /// Create a default profile
    pub fn default_profile() -> Self {
        Self::new(
            "default",
            "Default profile for general command generation",
            ShellType::default(),
            SafetyLevel::default(),
        )
    }

    /// Mark profile as used
    pub fn mark_used(&mut self) {
        self.last_used = Utc::now();
        self.usage_count += 1;
    }

    /// Add a command pattern
    pub fn add_command_pattern(&mut self, pattern: impl Into<String>) {
        let pattern = pattern.into();
        if !self.command_patterns.contains(&pattern) {
            self.command_patterns.push(pattern);
        }
    }

    /// Add an alias
    pub fn add_alias(&mut self, alias: impl Into<String>, command: impl Into<String>) {
        self.aliases.insert(alias.into(), command.into());
    }

    /// Remove an alias
    pub fn remove_alias(&mut self, alias: &str) -> Option<String> {
        self.aliases.remove(alias)
    }

    /// Validate profile data
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Profile name cannot be empty".to_string());
        }

        if self.id.is_empty() {
            return Err("Profile ID cannot be empty".to_string());
        }

        Ok(())
    }
}

/// Manager for user profiles
pub struct UserProfileManager {
    profiles_dir: PathBuf,
    profiles: HashMap<String, UserProfile>,
    active_profile: Option<String>,
}

impl UserProfileManager {
    /// Create a new profile manager
    pub fn new(profiles_dir: impl Into<PathBuf>) -> Result<Self, String> {
        let profiles_dir = profiles_dir.into();

        // Create profiles directory if it doesn't exist
        if !profiles_dir.exists() {
            fs::create_dir_all(&profiles_dir)
                .map_err(|e| format!("Failed to create profiles directory: {}", e))?;
        }

        let mut manager = Self {
            profiles_dir,
            profiles: HashMap::new(),
            active_profile: None,
        };

        // Load existing profiles
        manager.load_profiles()?;

        // Ensure default profile exists
        if !manager.profiles.contains_key("default") {
            let default_profile = UserProfile::default_profile();
            manager.save_profile(&default_profile)?;
            manager.profiles.insert("default".to_string(), default_profile);
        }

        // Set default as active if no profile is active
        if manager.active_profile.is_none() {
            manager.active_profile = Some("default".to_string());
        }

        Ok(manager)
    }

    /// Load all profiles from disk
    fn load_profiles(&mut self) -> Result<(), String> {
        debug!("Loading profiles from {:?}", self.profiles_dir);

        if !self.profiles_dir.exists() {
            return Ok(());
        }

        let entries = fs::read_dir(&self.profiles_dir)
            .map_err(|e| format!("Failed to read profiles directory: {}", e))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                match self.load_profile_from_file(&path) {
                    Ok(profile) => {
                        info!("Loaded profile: {}", profile.name);
                        self.profiles.insert(profile.name.clone(), profile);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load profile from {:?}: {}", path, e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Load a profile from a file
    fn load_profile_from_file(&self, path: &Path) -> Result<UserProfile, String> {
        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read profile file: {}", e))?;

        let profile: UserProfile = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse profile JSON: {}", e))?;

        profile.validate()?;

        Ok(profile)
    }

    /// Save a profile to disk
    pub fn save_profile(&self, profile: &UserProfile) -> Result<(), String> {
        profile.validate()?;

        let filename = format!("{}.json", profile.name);
        let path = self.profiles_dir.join(filename);

        let json = serde_json::to_string_pretty(profile)
            .map_err(|e| format!("Failed to serialize profile: {}", e))?;

        fs::write(&path, json).map_err(|e| format!("Failed to write profile file: {}", e))?;

        debug!("Saved profile '{}' to {:?}", profile.name, path);

        Ok(())
    }

    /// Create a new profile
    pub fn create_profile(
        &mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        shell_preference: ShellType,
        safety_level: SafetyLevel,
    ) -> Result<UserProfile, String> {
        let name = name.into();

        if self.profiles.contains_key(&name) {
            return Err(format!("Profile '{}' already exists", name));
        }

        let profile = UserProfile::new(name.clone(), description, shell_preference, safety_level);

        self.save_profile(&profile)?;
        self.profiles.insert(name, profile.clone());

        info!("Created new profile: {}", profile.name);

        Ok(profile)
    }

    /// Get a profile by name
    pub fn get_profile(&self, name: &str) -> Option<&UserProfile> {
        self.profiles.get(name)
    }

    /// Get a mutable profile by name
    pub fn get_profile_mut(&mut self, name: &str) -> Option<&mut UserProfile> {
        self.profiles.get_mut(name)
    }

    /// Delete a profile
    pub fn delete_profile(&mut self, name: &str) -> Result<(), String> {
        if name == "default" {
            return Err("Cannot delete the default profile".to_string());
        }

        if !self.profiles.contains_key(name) {
            return Err(format!("Profile '{}' not found", name));
        }

        let filename = format!("{}.json", name);
        let path = self.profiles_dir.join(filename);

        if path.exists() {
            fs::remove_file(&path).map_err(|e| format!("Failed to delete profile file: {}", e))?;
        }

        self.profiles.remove(name);

        // Switch to default if active profile was deleted
        if self.active_profile.as_deref() == Some(name) {
            self.active_profile = Some("default".to_string());
        }

        info!("Deleted profile: {}", name);

        Ok(())
    }

    /// List all profiles
    pub fn list_profiles(&self) -> Vec<&UserProfile> {
        self.profiles.values().collect()
    }

    /// Set the active profile
    pub fn set_active_profile(&mut self, name: impl Into<String>) -> Result<(), String> {
        let name = name.into();

        if !self.profiles.contains_key(&name) {
            return Err(format!("Profile '{}' not found", name));
        }

        self.active_profile = Some(name.clone());

        // Mark profile as used and save
        if let Some(profile) = self.profiles.get_mut(&name) {
            profile.mark_used();
            // Clone to avoid borrow issues
            let profile_clone = profile.clone();
            self.save_profile(&profile_clone)?;
        }

        info!("Set active profile: {}", name);

        Ok(())
    }

    /// Get the active profile
    pub fn get_active_profile(&self) -> Option<&UserProfile> {
        self.active_profile
            .as_ref()
            .and_then(|name| self.profiles.get(name))
    }

    /// Get the active profile name
    pub fn active_profile_name(&self) -> Option<&str> {
        self.active_profile.as_deref()
    }

    /// Update a profile
    pub fn update_profile(&mut self, name: &str, profile: UserProfile) -> Result<(), String> {
        if !self.profiles.contains_key(name) {
            return Err(format!("Profile '{}' not found", name));
        }

        self.save_profile(&profile)?;
        self.profiles.insert(name.to_string(), profile);

        info!("Updated profile: {}", name);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_create_default_profile() {
        let profile = UserProfile::default_profile();

        assert_eq!(profile.name, "default");
        assert!(profile.validate().is_ok());
    }

    #[test]
    fn test_profile_manager() {
        let temp_dir = tempdir().unwrap();
        let mut manager = UserProfileManager::new(temp_dir.path()).unwrap();

        // Default profile should exist
        assert!(manager.get_profile("default").is_some());

        // Create a new profile
        let profile = manager
            .create_profile("work", "Work profile", ShellType::Bash, SafetyLevel::Strict)
            .unwrap();

        assert_eq!(profile.name, "work");

        // Switch to the new profile
        manager.set_active_profile("work").unwrap();
        assert_eq!(manager.active_profile_name(), Some("work"));

        // Delete the profile
        manager.delete_profile("work").unwrap();
        assert!(manager.get_profile("work").is_none());

        // Should switch back to default
        assert_eq!(manager.active_profile_name(), Some("default"));
    }

    #[test]
    fn test_profile_persistence() {
        let temp_dir = tempdir().unwrap();

        // Create a profile
        {
            let mut manager = UserProfileManager::new(temp_dir.path()).unwrap();
            manager
                .create_profile("test", "Test profile", ShellType::Zsh, SafetyLevel::Moderate)
                .unwrap();
        }

        // Load profiles again
        {
            let manager = UserProfileManager::new(temp_dir.path()).unwrap();
            let profile = manager.get_profile("test").unwrap();
            assert_eq!(profile.name, "test");
            assert_eq!(profile.shell_preference, ShellType::Zsh);
        }
    }

    #[test]
    fn test_profile_aliases() {
        let mut profile = UserProfile::default_profile();

        profile.add_alias("ll", "ls -la");
        profile.add_alias("gs", "git status");

        assert_eq!(profile.aliases.get("ll"), Some(&"ls -la".to_string()));
        assert_eq!(profile.aliases.get("gs"), Some(&"git status".to_string()));

        profile.remove_alias("ll");
        assert!(profile.aliases.get("ll").is_none());
    }
}
