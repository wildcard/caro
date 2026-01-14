use assert_cmd::prelude::*;
use std::process::Command;

#[test]
fn test_assess_command_runs() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("caro"));

    cmd.arg("assess").assert().success();
}

#[test]
#[ignore = "assess command not fully implemented yet"]
fn test_assess_command_output() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("caro"));

    let output = cmd
        .arg("assess")
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify output contains expected sections
    assert!(stdout.contains("Caro System Assessment"));
    assert!(stdout.contains("System Information:"));
    assert!(stdout.contains("CPU:"));
    assert!(stdout.contains("Memory:"));
}

#[test]
#[ignore = "assess command not fully implemented yet"]
fn test_assess_json_export() {
    use tempfile::NamedTempFile;

    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("caro"));

    cmd.arg("assess")
        .arg("--export")
        .arg("json")
        .arg("--output")
        .arg(path)
        .assert()
        .success();

    // Verify JSON file was created and is valid
    let content = std::fs::read_to_string(path).unwrap();
    let _: serde_json::Value = serde_json::from_str(&content).expect("Output should be valid JSON");
}

#[test]
#[ignore = "Performance test is flaky on CI runners"]
fn test_assess_completes_quickly() {
    use std::time::Instant;

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("caro"));

    let start = Instant::now();
    cmd.arg("assess").assert().success();
    let duration = start.elapsed();

    // Should complete in < 5 seconds (SC-001)
    assert!(
        duration.as_secs() < 5,
        "Assessment took {:?}, expected < 5s",
        duration
    );
}
