//! Shell Command Generation Prompt System
//!
//! This module provides a comprehensive prompt system for generating shell commands
//! from natural language queries. It is optimized for small language models like
//! SmolLM-135M-Instruct while maintaining compatibility with larger models.
//!
//! # Architecture
//!
//! The prompt system consists of four main components:
//!
//! 1. **Capability Profile** (`capability_profile`): Detects platform capabilities
//!    and generates a profile describing available tools and supported flags.
//!
//! 2. **Command Templates** (`command_templates`): A library of command templates
//!    organized by intent category and filtered by platform capabilities.
//!
//! 3. **SmolLM Prompt** (`smollm_prompt`): The main prompt builder that generates
//!    system prompts optimized for small language models.
//!
//! 4. **Validation** (`validation`): Validates generated commands for safety,
//!    platform compatibility, and proper formatting.
//!
//! # Quick Start
//!
//! ```rust
//! use caro::prompts::{
//!     CapabilityProfile, SmolLMPromptBuilder, CommandValidator,
//! };
//!
//! // Detect platform capabilities (or use pre-defined profiles)
//! let profile = CapabilityProfile::ubuntu();
//!
//! // Build the system prompt
//! let builder = SmolLMPromptBuilder::new(profile.clone());
//! let chat_prompt = builder.format_chat("list all files larger than 100MB");
//!
//! // After getting model output, validate the command
//! let validator = CommandValidator::new(profile);
//! let result = validator.validate("find . -type f -size +100M");
//! assert!(result.is_valid());
//! ```
//!
//! # Platform Support
//!
//! The prompt system supports four platform profiles:
//!
//! - **gnu-linux**: Ubuntu, Debian, Fedora, RHEL, Arch (GNU coreutils + findutils)
//! - **bsd**: macOS, FreeBSD, OpenBSD (BSD utilities)
//! - **busybox**: Alpine Linux, embedded systems (BusyBox utilities)
//! - **hybrid**: Git Bash, MSYS2, Cygwin (mixed environments)
//!
//! # Safety
//!
//! The system includes comprehensive safety features:
//!
//! - Dangerous command detection (rm -rf /, fork bombs, etc.)
//! - Destructive command confirmation requirement
//! - Output hallucination detection
//! - Tool allowlist enforcement
//!
//! # Model Compatibility
//!
//! While optimized for SmolLM-135M-Instruct, the prompt system is designed to work
//! with any instruction-following model that supports the ChatML format:
//!
//! ```text
//! <|im_start|>system
//! {system_prompt}
//! <|im_end|>
//! <|im_start|>user
//! {user_message}
//! <|im_end|>
//! <|im_start|>assistant
//! ```
//!
//! For best results with small models (< 500M parameters):
//!
//! - Use temperature ~0.2 and top_p ~0.9
//! - Keep prompts short and explicit
//! - Use the template-based approach
//! - Validate all outputs

pub mod capability_profile;
pub mod command_templates;
pub mod smollm_prompt;
pub mod validation;

// Re-export main types for convenient access
pub use capability_profile::{AwkType, CapabilityProfile, DetectedShell, ProfileType, StatFormat};
pub use command_templates::{CommandTemplate, TemplateLibrary};
pub use smollm_prompt::{CommandOutput, PromptResponse, RepairPromptBuilder, SmolLMPromptBuilder};
pub use validation::{
    CommandValidator, RiskLevel, ValidationError, ValidationErrorCode, ValidationResult,
    ValidationWarning,
};

/// Convenience function to create an Ubuntu-optimized prompt for a query
///
/// # Example
///
/// ```rust
/// use caro::prompts::create_ubuntu_prompt;
///
/// let prompt = create_ubuntu_prompt("list all files");
/// // prompt is ready to be sent to SmolLM or similar model
/// ```
pub fn create_ubuntu_prompt(query: &str) -> String {
    let builder = SmolLMPromptBuilder::ubuntu();
    builder.format_chat(query)
}

/// Convenience function to create a prompt with JSON output format
///
/// This variant pre-fills the start of the JSON output to guide the model
/// toward producing valid JSON.
///
/// # Example
///
/// ```rust
/// use caro::prompts::create_ubuntu_prompt_json;
///
/// let prompt = create_ubuntu_prompt_json("find python files");
/// // prompt ends with {"cmd": " to guide JSON generation
/// ```
pub fn create_ubuntu_prompt_json(query: &str) -> String {
    let builder = SmolLMPromptBuilder::ubuntu();
    builder.format_chat_json(query)
}

/// Convenience function to validate a command for Ubuntu
///
/// # Example
///
/// ```rust
/// use caro::prompts::validate_ubuntu_command;
///
/// let result = validate_ubuntu_command("ls -la");
/// assert!(result.is_valid());
/// ```
pub fn validate_ubuntu_command(command: &str) -> ValidationResult {
    let profile = CapabilityProfile::ubuntu();
    let validator = CommandValidator::new(profile);
    validator.validate(command)
}

/// Detect the current platform's capabilities
///
/// This function runs capability probes to determine what tools and flags
/// are available on the current system.
///
/// # Example
///
/// ```rust
/// use caro::prompts::detect_capabilities;
///
/// #[tokio::main]
/// async fn main() {
///     let profile = detect_capabilities().await;
///     println!("Profile: {}", profile.profile_type);
///     println!("find -printf: {}", profile.find_printf);
/// }
/// ```
pub async fn detect_capabilities() -> CapabilityProfile {
    CapabilityProfile::detect_or_cached().await
}

/// Generate a shell script that outputs a capability profile
///
/// This can be used to generate a probe script that runs on a target system
/// and outputs the capability profile in a parseable format.
///
/// # Example
///
/// ```rust
/// use caro::prompts::generate_probe_script;
///
/// let script = generate_probe_script();
/// // Save to file and run on target system
/// // Parse the output to create a CapabilityProfile
/// ```
pub fn generate_probe_script() -> String {
    r#"#!/bin/sh
# Capability Probe Script for Caro Shell Command Generation
# Run this script and capture its output to get the capability profile
set -eu

have() { command -v "$1" >/dev/null 2>&1; }

# OS identity
OS_NAME="unknown"
OS_VERSION="unknown"
if [ -r /etc/os-release ]; then
  . /etc/os-release
  OS_NAME="${NAME:-unknown}"
  OS_VERSION="${VERSION_ID:-unknown}"
elif have sw_vers; then
  OS_NAME="macOS"
  OS_VERSION="$(sw_vers -productVersion 2>/dev/null || echo unknown)"
fi

UNAME="$(uname -srm 2>/dev/null || echo unknown)"
SHELL_SH="$( (readlink -f /bin/sh 2>/dev/null) || echo /bin/sh )"

# Tool presence
TOOLS_CANDIDATES="ls find grep awk sed sort head tail xargs stat du wc cat tar gzip curl cut tr uniq"
TOOLS=""
for t in $TOOLS_CANDIDATES; do
  if have "$t"; then TOOLS="${TOOLS}${TOOLS:+,}$t"; fi
done

# Detect userland/profile
PROFILE="unknown"
if have busybox; then
  PROFILE="busybox"
fi

# GNU probes
is_gnu_coreutils=false
if have ls && ls --version >/dev/null 2>&1; then
  is_gnu_coreutils=true
fi

is_gnu_findutils=false
if have find && find --version >/dev/null 2>&1; then
  is_gnu_findutils=true
fi

if [ "$PROFILE" = "unknown" ]; then
  if [ "$is_gnu_coreutils" = true ] || [ "$is_gnu_findutils" = true ]; then
    PROFILE="gnu-linux"
  else
    if [ "$(uname -s 2>/dev/null || true)" = "Darwin" ]; then
      PROFILE="bsd"
    else
      PROFILE="hybrid"
    fi
  fi
fi

# Feature probes
FIND_PRINTF=false
if have find; then
  if find . -maxdepth 0 -printf '%p\n' >/dev/null 2>&1; then
    FIND_PRINTF=true
  fi
fi

FIND_PRINT0=false
if have find; then
  if find . -maxdepth 0 -print0 >/dev/null 2>&1; then
    FIND_PRINT0=true
  fi
fi

SORT_H=false
if have sort; then
  if printf "1K\n2K\n" | sort -h >/dev/null 2>&1; then
    SORT_H=true
  fi
fi

XARGS_0=false
if have xargs; then
  if printf "x\0" | xargs -0 printf "%s" >/dev/null 2>&1; then
    XARGS_0=true
  fi
fi

GREP_R=false
if have grep; then
  if grep -R --help >/dev/null 2>&1; then
    GREP_R=true
  fi
fi

GREP_P=false
if have grep; then
  if echo test | grep -P 'test' >/dev/null 2>&1; then
    GREP_P=true
  fi
fi

STAT_FORMAT="none"
if have stat; then
  if stat --version >/dev/null 2>&1; then
    STAT_FORMAT="gnu"
  elif stat -f "%N" . >/dev/null 2>&1; then
    STAT_FORMAT="bsd"
  fi
fi

SED_INPLACE_GNU=false
if have sed; then
  if sed --version >/dev/null 2>&1; then
    SED_INPLACE_GNU=true
  fi
fi

DU_MAX_DEPTH=false
if have du; then
  if du --max-depth=0 . >/dev/null 2>&1; then
    DU_MAX_DEPTH=true
  fi
fi

DATE_GNU=false
if have date; then
  if date --date=now >/dev/null 2>&1; then
    DATE_GNU=true
  fi
fi

READLINK_F=false
if have readlink; then
  if readlink -f . >/dev/null 2>&1; then
    READLINK_F=true
  fi
fi

PS_SORT=false
if have ps; then
  if ps --sort=pid -e >/dev/null 2>&1; then
    PS_SORT=true
  fi
fi

LS_SORT=false
if have ls; then
  if ls --sort=size . >/dev/null 2>&1; then
    LS_SORT=true
  fi
fi

# Output capability profile
cat <<EOF
PROFILE=$PROFILE
OS_NAME=$OS_NAME
OS_VERSION=$OS_VERSION
UNAME=$UNAME
SHELL_SH=$SHELL_SH
TOOLS=$TOOLS
FIND_PRINTF=$FIND_PRINTF
FIND_PRINT0=$FIND_PRINT0
SORT_H=$SORT_H
XARGS_0=$XARGS_0
GREP_R=$GREP_R
GREP_P=$GREP_P
STAT_FORMAT=$STAT_FORMAT
SED_INPLACE_GNU=$SED_INPLACE_GNU
DU_MAX_DEPTH=$DU_MAX_DEPTH
DATE_GNU=$DATE_GNU
READLINK_F=$READLINK_F
PS_SORT=$PS_SORT
LS_SORT=$LS_SORT
EOF
"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_ubuntu_prompt() {
        let prompt = create_ubuntu_prompt("list all files");
        assert!(prompt.contains("<|im_start|>system"));
        assert!(prompt.contains("list all files"));
        assert!(prompt.contains("<|im_start|>assistant"));
    }

    #[test]
    fn test_create_ubuntu_prompt_json() {
        let prompt = create_ubuntu_prompt_json("list files");
        assert!(prompt.contains("<|im_start|>system"));
        assert!(prompt.ends_with("{\"cmd\": \""));
    }

    #[test]
    fn test_validate_ubuntu_command() {
        let result = validate_ubuntu_command("ls -la");
        assert!(result.is_valid());

        let result = validate_ubuntu_command("rm -rf /");
        assert!(!result.is_valid());
    }

    #[test]
    fn test_generate_probe_script() {
        let script = generate_probe_script();
        assert!(script.contains("#!/bin/sh"));
        assert!(script.contains("FIND_PRINTF"));
        assert!(script.contains("SORT_H"));
    }

    #[tokio::test]
    async fn test_detect_capabilities() {
        let profile = detect_capabilities().await;
        // Should detect something
        assert!(!profile.tools.is_empty() || profile.profile_type != ProfileType::Unknown);
    }
}
