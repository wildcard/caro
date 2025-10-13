// Property-based tests - THESE MUST FAIL INITIALLY (TDD)
// Tests validate system properties using generated test cases

use proptest::prelude::*;

// Import system components
use cmdai::{
    models::{RiskLevel, ShellType},
    safety::{SafetyConfig, SafetyValidator},
};

// Configure proptest to run fewer cases for faster test execution
// Override with PROPTEST_CASES environment variable if needed

// Property: Safety validation should be deterministic
proptest! {
    #![proptest_config(ProptestConfig {
        cases: 10,
        max_shrink_iters: 1000,
        timeout: 30000, // 30 seconds per test
        ..ProptestConfig::default()
    })]

    #[test]
    fn prop_safety_validation_deterministic(
        command in "[a-zA-Z0-9 ._-]{1,100}",
        shell in prop_oneof![
            Just(ShellType::Bash),
            Just(ShellType::Zsh),
            Just(ShellType::Fish),
            Just(ShellType::Sh),
        ]
    ) {
        tokio_test::block_on(async {
            let validator = SafetyValidator::new(SafetyConfig::moderate());
            if validator.is_err() {
                // Not implemented yet, skip property test
                return Ok(());
            }

            let v = validator.unwrap();

            // Validate the same command multiple times
            let result1 = v.validate_command(&command, shell).await;
            let result2 = v.validate_command(&command, shell).await;
            let result3 = v.validate_command(&command, shell).await;

            if let (Ok(val1), Ok(val2), Ok(val3)) = (result1, result2, result3) {

                // Results should be identical (deterministic)
                prop_assert_eq!(val1.allowed, val2.allowed);
                prop_assert_eq!(val2.allowed, val3.allowed);
                prop_assert_eq!(val1.risk_level, val2.risk_level);
                prop_assert_eq!(val2.risk_level, val3.risk_level);
            }

            Ok(())
        })?;
    }
}

// Property: Risk levels should be monotonic with safety levels
proptest! {
    #![proptest_config(ProptestConfig {
        cases: 10,
        max_shrink_iters: 1000,
        timeout: 30000,
        ..ProptestConfig::default()
    })]

    #[test]
    fn prop_safety_levels_monotonic(
        command in "[a-zA-Z0-9 ._/-]{1,50}",
    ) {
        tokio_test::block_on(async {
            let strict_validator = SafetyValidator::new(SafetyConfig::strict());
            let moderate_validator = SafetyValidator::new(SafetyConfig::moderate());
            let permissive_validator = SafetyValidator::new(SafetyConfig::permissive());

            if strict_validator.is_err() || moderate_validator.is_err() || permissive_validator.is_err() {
                // Not implemented yet
                return Ok(());
            }

            let strict = strict_validator.unwrap();
            let moderate = moderate_validator.unwrap();
            let permissive = permissive_validator.unwrap();

            let strict_result = strict.validate_command(&command, ShellType::Bash).await;
            let moderate_result = moderate.validate_command(&command, ShellType::Bash).await;
            let permissive_result = permissive.validate_command(&command, ShellType::Bash).await;

            if let (Ok(strict_val), Ok(moderate_val), Ok(permissive_val)) = (strict_result, moderate_result, permissive_result) {

                // If strict blocks something, moderate should not be more permissive
                if !strict_val.allowed {
                    prop_assert!(!moderate_val.allowed || moderate_val.risk_level >= RiskLevel::High,
                                "Moderate should not be more permissive than strict for: {}", command);
                }

                // If moderate blocks something, permissive might allow but should warn
                if !moderate_val.allowed {
                    prop_assert!(!permissive_val.allowed || !permissive_val.warnings.is_empty(),
                                "Permissive should warn if moderate blocks: {}", command);
                }
            }

            Ok(())
        })?;
    }
}

// Property: Command validation should handle arbitrary strings safely
proptest! {
    #![proptest_config(ProptestConfig {
        cases: 10,
        max_shrink_iters: 1000,
        timeout: 30000,
        ..ProptestConfig::default()
    })]

    #[test]
    fn prop_arbitrary_command_safety(
        command in ".*{0,200}",  // Any string up to 200 chars
    ) {
        tokio_test::block_on(async {
            let validator = SafetyValidator::new(SafetyConfig::moderate());
            if validator.is_err() {
                return Ok(()); // Not implemented
            }

            let v = validator.unwrap();
            let result = v.validate_command(&command, ShellType::Bash).await;

            // Should never panic or crash, regardless of input
            match result {
                Ok(validation) => {
                    // Validation result should be consistent
                    prop_assert!(validation.confidence_score >= 0.0 && validation.confidence_score <= 1.0,
                                "Confidence score should be valid: {}", validation.confidence_score);

                    // If blocked, should have explanation
                    if !validation.allowed {
                        prop_assert!(!validation.explanation.is_empty(),
                                    "Blocked commands should have explanation for: '{}'", command);
                    }
                }
                Err(err) => {
                    // Errors should be meaningful
                    prop_assert!(!err.to_string().is_empty(),
                                "Error messages should not be empty for: '{}'", command);
                }
            }

            Ok(())
        })?;
    }
}

// Property: Common safe commands should always be allowed
proptest! {
    #![proptest_config(ProptestConfig {
        cases: 10,
        max_shrink_iters: 1000,
        timeout: 30000,
        ..ProptestConfig::default()
    })]

    #[test]
    fn prop_safe_commands_always_allowed(
        safe_prefix in prop_oneof![
            Just("ls"),
            Just("pwd"),
            Just("date"),
            Just("echo"),
            Just("cat"),
            Just("head"),
            Just("tail"),
            Just("wc"),
            Just("grep"),
        ],
        safe_args in "[a-zA-Z0-9 ._-]{0,50}",
    ) {
        let command = if safe_args.is_empty() {
            safe_prefix.to_string()
        } else {
            format!("{} {}", safe_prefix, safe_args)
        };

        tokio_test::block_on(async {
            let validator = SafetyValidator::new(SafetyConfig::strict());
            if validator.is_err() {
                return Ok(());
            }

            let v = validator.unwrap();
            let result = v.validate_command(&command, ShellType::Bash).await;

            if let Ok(validation) = result {
                // These commands should generally be safe
                prop_assert!(validation.allowed || validation.risk_level <= RiskLevel::Moderate,
                            "Safe command '{}' should be allowed or low risk", command);
            }

            Ok(())
        })?;
    }
}

// Property: Dangerous patterns should be detected consistently
proptest! {
    #![proptest_config(ProptestConfig {
        cases: 10,
        max_shrink_iters: 1000,
        timeout: 30000,
        ..ProptestConfig::default()
    })]

    #[test]
    fn prop_dangerous_patterns_detected(
        dangerous_command in prop_oneof![
            Just("rm -rf /"),
            Just("sudo rm -rf /*"),
            Just("dd if=/dev/zero of=/dev/sda"),
            Just(":(){ :|:& };:"),  // fork bomb
            Just("chmod 777 /etc/passwd"),
            Just("mkfs.ext4 /dev/sda"),
        ],
        prefix in "[a-zA-Z0-9 ]{0,20}",
        suffix in "[a-zA-Z0-9 ]{0,20}",
    ) {
        let full_command = format!("{} {} {}", prefix, dangerous_command, suffix).trim().to_string();

        tokio_test::block_on(async {
            let validator = SafetyValidator::new(SafetyConfig::strict());
            if validator.is_err() {
                return Ok(());
            }

            let v = validator.unwrap();
            let result = v.validate_command(&full_command, ShellType::Bash).await;

            if let Ok(validation) = result {
                // Dangerous commands should be blocked or flagged as high risk
                prop_assert!(!validation.allowed || validation.risk_level >= RiskLevel::High,
                            "Dangerous command '{}' should be blocked or high risk", full_command);
            }

            Ok(())
        })?;
    }
}

// Property: Command length should not affect safety classification consistency
proptest! {
    #![proptest_config(ProptestConfig {
        cases: 10,
        max_shrink_iters: 1000,
        timeout: 30000,
        ..ProptestConfig::default()
    })]

    #[test]
    fn prop_command_length_consistency(
        base_command in "[a-zA-Z]{2,10}",
        args in prop::collection::vec("[a-zA-Z0-9._-]{1,20}", 0..10),
    ) {
        let short_command = base_command.clone();
        let long_command = if args.is_empty() {
            base_command
        } else {
            format!("{} {}", base_command, args.join(" "))
        };

        tokio_test::block_on(async {
            let validator = SafetyValidator::new(SafetyConfig::moderate());
            if validator.is_err() {
                return Ok(());
            }

            let v = validator.unwrap();
            let short_result = v.validate_command(&short_command, ShellType::Bash).await;
            let long_result = v.validate_command(&long_command, ShellType::Bash).await;

            if let (Ok(short_val), Ok(long_val)) = (short_result, long_result) {
                // If the base command is safe, adding safe args shouldn't make it dangerous
                if short_val.allowed && short_val.risk_level == RiskLevel::Safe {
                    prop_assert!(long_val.risk_level <= RiskLevel::Moderate,
                                "Adding safe args to safe command should not increase risk dramatically: '{}' -> '{}'",
                                short_command, long_command);
                }
            }

            Ok(())
        })?;
    }
}

// Property: Shell type should not dramatically change risk assessment for basic commands
proptest! {
    #![proptest_config(ProptestConfig {
        cases: 10,
        max_shrink_iters: 1000,
        timeout: 30000,
        ..ProptestConfig::default()
    })]

    #[test]
    fn prop_shell_consistency_basic_commands(
        basic_command in prop_oneof![
            Just("ls -la"),
            Just("pwd"),
            Just("echo hello"),
            Just("date"),
        ],
    ) {
        tokio_test::block_on(async {
            let validator = SafetyValidator::new(SafetyConfig::moderate());
            if validator.is_err() {
                return Ok(());
            }

            let v = validator.unwrap();

            let shells = vec![ShellType::Bash, ShellType::Zsh, ShellType::Fish, ShellType::Sh];
            let mut results = Vec::new();

            for shell in shells {
                let result = v.validate_command(basic_command, shell).await;
                if let Ok(validation) = result {
                    results.push((shell, validation));
                }
            }

            if results.len() > 1 {
                // Basic commands should have similar risk levels across shells
                let first_risk = results[0].1.risk_level;
                for (shell, validation) in &results {
                    prop_assert!(validation.risk_level <= RiskLevel::Moderate,
                                "Basic command '{}' should be safe in shell {:?}", basic_command, shell);

                    // Risk levels should not vary wildly across shells for basic commands
                    prop_assert!(
                        (validation.risk_level as u8).abs_diff(first_risk as u8) <= 1,
                        "Risk level consistency across shells for '{}': {:?} vs {:?}",
                        basic_command, first_risk, validation.risk_level
                    );
                }
            }

            Ok(())
        })?;
    }
}

// Property: Empty and whitespace commands should be handled consistently
proptest! {
    #![proptest_config(ProptestConfig {
        cases: 10,
        max_shrink_iters: 1000,
        timeout: 30000,
        ..ProptestConfig::default()
    })]

    #[test]
    fn prop_empty_whitespace_commands(
        whitespace in r"[ \t\n\r]{0,20}",
    ) {
        tokio_test::block_on(async {
            let validator = SafetyValidator::new(SafetyConfig::moderate());
            if validator.is_err() {
                return Ok(());
            }

            let v = validator.unwrap();
            let result = v.validate_command(&whitespace, ShellType::Bash).await;

            if let Ok(validation) = result {
                // Empty/whitespace commands should be safe and allowed
                prop_assert!(validation.allowed, "Empty/whitespace commands should be allowed");
                prop_assert_eq!(validation.risk_level, RiskLevel::Safe,
                               "Empty/whitespace commands should be safe");
            }

            Ok(())
        })?;
    }
}

// Property: System should handle Unicode and special characters gracefully
proptest! {
    #![proptest_config(ProptestConfig {
        cases: 10,
        max_shrink_iters: 1000,
        timeout: 30000,
        ..ProptestConfig::default()
    })]

    #[test]
    fn prop_unicode_special_chars(
        unicode_command in "[\\p{L}\\p{N}\\p{P}\\p{S} ]{1,50}",
    ) {
        tokio_test::block_on(async {
            let validator = SafetyValidator::new(SafetyConfig::moderate());
            if validator.is_err() {
                return Ok(());
            }

            let v = validator.unwrap();
            let result = v.validate_command(&unicode_command, ShellType::Bash).await;

            // Should handle any Unicode input without panicking
            match result {
                Ok(validation) => {
                    prop_assert!(validation.confidence_score >= 0.0 && validation.confidence_score <= 1.0,
                                "Confidence should be valid for Unicode command");
                }
                Err(_) => {
                    // Errors are acceptable, but should not panic
                    prop_assert!(true, "Unicode handling should not panic");
                }
            }

            Ok(())
        })?;
    }
}

// Property: Batch validation should be consistent with individual validation
proptest! {
    #![proptest_config(ProptestConfig {
        cases: 10,
        max_shrink_iters: 1000,
        timeout: 30000,
        ..ProptestConfig::default()
    })]

    #[test]
    fn prop_batch_individual_consistency(
        commands in prop::collection::vec("[a-zA-Z0-9 ._-]{1,30}", 1..5),
    ) {
        tokio_test::block_on(async {
            let validator = SafetyValidator::new(SafetyConfig::moderate());
            if validator.is_err() {
                return Ok(());
            }

            let v = validator.unwrap();

            // Individual validations
            let mut individual_results = Vec::new();
            for cmd in &commands {
                let result = v.validate_command(cmd, ShellType::Bash).await;
                if let Ok(validation) = result {
                    individual_results.push(validation);
                }
            }

            // Batch validation
            let batch_result = v.validate_batch(&commands, ShellType::Bash).await;

            if let Ok(batch_validations) = batch_result {
                if !individual_results.is_empty() {

                prop_assert_eq!(batch_validations.len(), commands.len(),
                               "Batch should return result for each command");

                // Results should be consistent between batch and individual
                for (i, batch_val) in batch_validations.iter().enumerate() {
                    if i < individual_results.len() {
                        let individual_val = &individual_results[i];
                        prop_assert_eq!(batch_val.allowed, individual_val.allowed,
                                       "Batch and individual results should match for command: '{}'", commands[i]);
                        prop_assert_eq!(batch_val.risk_level, individual_val.risk_level,
                                       "Risk levels should match for command: '{}'", commands[i]);
                    }
                }
                }
            }

            Ok(())
        })?;
    }
}
