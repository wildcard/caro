//! Migration support for V1 to V2 upgrade
//!
//! Handles data migration when upgrading from cmdai V1 to V2.

use anyhow::Result;
use std::path::PathBuf;

/// Migrate from V1 to V2
///
/// V1 had no learning data, so this primarily:
/// - Creates the learning database if it doesn't exist
/// - Informs user about the new learning features
/// - Sets up initial configuration
pub async fn migrate_v1_to_v2() -> Result<MigrationResult> {
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let cmdai_dir = home_dir.join(".cmdai");
    let patterns_db = cmdai_dir.join("patterns.db");

    // Check if this is a fresh install or upgrade
    let is_upgrade = cmdai_dir.exists() && !patterns_db.exists();

    if is_upgrade {
        // This is an upgrade from V1 - inform user
        println!("🎉 Upgrading to cmdai V2!");
        println!("   New features enabled:");
        println!("   • Learning engine - cmdai now remembers your patterns");
        println!("   • Command explanations - understand what commands do");
        println!("   • Interactive tutorials - learn shell commands");
        println!("   • Achievement system - track your progress");
        println!();
        println!("   All learning data is stored locally in: {}", cmdai_dir.display());
        println!("   You can clear it anytime with: cmdai --clear-history");
        println!();
    }

    // Ensure .cmdai directory exists
    if !cmdai_dir.exists() {
        tokio::fs::create_dir_all(&cmdai_dir).await?;
    }

    // Create initial config if it doesn't exist
    let config_file = cmdai_dir.join("config.toml");
    if !config_file.exists() {
        create_default_config(&config_file).await?;
    }

    Ok(MigrationResult {
        is_upgrade,
        cmdai_dir,
        patterns_db,
    })
}

/// Result of migration
#[derive(Debug)]
pub struct MigrationResult {
    pub is_upgrade: bool,
    pub cmdai_dir: PathBuf,
    pub patterns_db: PathBuf,
}

/// Create default configuration file
async fn create_default_config(config_path: &PathBuf) -> Result<()> {
    let default_config = r#"# cmdai V2 Configuration

[learning]
# Enable learning from user edits
learn_from_edits = true

# Enable similarity search
enable_similarity = true

# Enable achievements
enable_achievements = true

# Maximum patterns to store (disk space management)
max_patterns = 100000

[privacy]
# Enable telemetry (anonymized, opt-in only)
telemetry_enabled = false

# Encrypt database at rest
encrypt_database = false
"#;

    tokio::fs::write(config_path, default_config).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_create_default_config() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.toml");

        create_default_config(&config_path).await.unwrap();

        assert!(config_path.exists());
        let content = tokio::fs::read_to_string(&config_path).await.unwrap();
        assert!(content.contains("[learning]"));
    }
}
