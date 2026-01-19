//! Generate JSON Schema for caro configuration
//!
//! This binary generates a JSON Schema file that provides autocomplete
//! and validation support in editors like VS Code.
//!
//! Usage:
//!   cargo run --bin generate-schema

use caro::models::UserConfiguration;
use schemars::schema_for;
use std::fs;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate the schema
    let schema = schema_for!(UserConfiguration);
    let schema_json = serde_json::to_string_pretty(&schema)?;

    // Write to .vscode directory
    let vscode_dir = PathBuf::from(".vscode");
    if !vscode_dir.exists() {
        fs::create_dir_all(&vscode_dir)?;
    }

    let schema_path = vscode_dir.join("caro-config.schema.json");
    fs::write(&schema_path, schema_json)?;

    println!("✅ JSON Schema generated at: {}", schema_path.display());
    println!();
    println!("To use this schema in your config.toml:");
    println!("  1. Add this line to the top of ~/.config/caro/config.toml:");
    println!("     # $schema = \"file://{}/{}\"",
        std::env::current_dir()?.display(),
        schema_path.display());
    println!();
    println!("  2. VS Code will provide autocomplete and validation");

    Ok(())
}
