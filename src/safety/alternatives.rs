//! Safer alternatives for dangerous command patterns
//!
//! When safety validation blocks a command, these alternatives provide
//! users with guidance on what to do instead.

/// A safer alternative to a dangerous command
#[derive(Debug, Clone)]
pub struct SaferAlternative {
    pub safer_command: String,
    pub explanation: String,
}

/// Get safer alternatives for a command that was blocked by safety validation.
///
/// Returns an empty vec if no specific alternative is known.
pub fn get_alternatives(command: &str) -> Vec<SaferAlternative> {
    let cmd = command.trim().to_lowercase();
    let mut alternatives = Vec::new();

    // Full system deletion: rm -rf /
    if (cmd.contains("rm") && cmd.contains("-rf") && cmd.contains("/ ")
        && !cmd.contains("./") && !cmd.contains("~/"))
        || cmd == "rm -rf /"
        || cmd == "rm -rf /*"
    {
        alternatives.push(SaferAlternative {
            safer_command: "find ~/projects -name '*.tmp' -type f -mtime +30 -ls".to_string(),
            explanation: "Preview files to delete first. Then use 'rm' on specific directories, not the entire filesystem.".to_string(),
        });
        return alternatives;
    }

    // Home directory deletion: rm -rf ~
    if cmd.contains("rm") && cmd.contains("-rf") && (cmd.contains("~") || cmd.contains("$HOME")) {
        alternatives.push(SaferAlternative {
            safer_command: "find ~ -name '*.tmp' -type f -mtime +30 -ls".to_string(),
            explanation: "Target specific file types in subdirectories instead of deleting everything.".to_string(),
        });
        return alternatives;
    }

    // Mass recursive deletion: rm -rf * in dangerous locations
    if cmd.contains("rm") && cmd.contains("-rf") && cmd.contains("*") {
        alternatives.push(SaferAlternative {
            safer_command: "ls -la".to_string(),
            explanation: "Preview what would be deleted first. Then use 'rm -rf ./specific-directory' for targeted deletion.".to_string(),
        });
        return alternatives;
    }

    // chmod 777 on root or system paths
    if cmd.contains("chmod") && cmd.contains("777") {
        if cmd.contains("/") && !cmd.contains("./") {
            alternatives.push(SaferAlternative {
                safer_command: "chmod 755 directories && chmod 644 files".to_string(),
                explanation: "Use 755 for directories (rwxr-xr-x) and 644 for files (rw-r--r--). Never use 777 on system paths.".to_string(),
            });
            return alternatives;
        }
    }

    // dd to disk
    if cmd.contains("dd") && cmd.contains("/dev/zero") && cmd.contains("/dev/") {
        alternatives.push(SaferAlternative {
            safer_command: "# This operation is not recommended".to_string(),
            explanation: "Writing zeros to a disk device destroys all data. Use proper partitioning tools (fdisk, gdisk) instead.".to_string(),
        });
        return alternatives;
    }

    // Fork bomb
    if cmd.contains(":(){") || cmd.contains(":|:&") {
        alternatives.push(SaferAlternative {
            safer_command: "ulimit -u 100".to_string(),
            explanation: "Set process limits with 'ulimit -u' to prevent runaway process creation.".to_string(),
        });
        return alternatives;
    }

    // sudo su / privilege escalation
    if cmd.contains("sudo su") {
        alternatives.push(SaferAlternative {
            safer_command: "sudo -i".to_string(),
            explanation: "Use 'sudo -i' for an interactive root shell, or 'sudo <command>' to run specific commands with elevated privileges.".to_string(),
        });
        return alternatives;
    }

    // curl | bash / pipe to shell
    if (cmd.contains("curl") || cmd.contains("wget")) && cmd.contains("|") && (cmd.contains("sh") || cmd.contains("bash")) {
        alternatives.push(SaferAlternative {
            safer_command: "curl -sSfL https://example.com/setup.sh -o setup.sh && less setup.sh && bash setup.sh".to_string(),
            explanation: "Download the script first, review it, then execute. Never pipe directly to a shell without inspection.".to_string(),
        });
        return alternatives;
    }

    // Delete logs (non-critical but suggest preview)
    if (cmd.contains("delete") || cmd.contains("rm") || cmd.contains("remove"))
        && cmd.contains("log")
        && !cmd.contains("-rf /")
    {
        alternatives.push(SaferAlternative {
            safer_command: "find . -name '*.log' -type f -mtime +30 -ls".to_string(),
            explanation: "Preview which log files would be deleted first, then use '-delete' if the list looks correct.".to_string(),
        });
        return alternatives;
    }

    // Generic recursive deletion (not already caught above)
    if cmd.contains("rm") && (cmd.contains("-r") || cmd.contains("-rf")) {
        alternatives.push(SaferAlternative {
            safer_command: "ls -la <target-directory>".to_string(),
            explanation: "Preview what would be deleted first. Then use 'rm -r <specific-path>' for targeted deletion.".to_string(),
        });
    }

    alternatives
}
