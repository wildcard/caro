//! Explainer Prompt System
//!
//! This module provides a prompt system optimized for educational command explanations.
//! Unlike the generator prompt which focuses on quick command synthesis, this system
//! provides detailed explanations of commands, options, and alternatives.
//!
//! # Design Philosophy
//!
//! The explainer prompt is inspired by how expert developers explain commands:
//!
//! 1. **Identify the Tool**: Recognize the relevant Unix utility for the task
//! 2. **Explain the Command**: Break down what the command does
//! 3. **Describe Options**: Explain each flag and option used
//! 4. **Show Examples**: Provide variations for different use cases
//!
//! # Example Output
//!
//! For "find files modified in the last 24 hours":
//!
//! ```text
//! Use `find` with time-based filters:
//!
//!   # Last 24 hours
//!   find . -type f -mtime -1
//!
//!   # With file details
//!   find . -type f -mtime -1 -ls
//!
//! The `-mtime -1` means "modified less than 1 day ago".
//! Use `-mmin` for minute precision (1440 minutes = 24 hours).
//! ```

use super::capability_profile::{CapabilityProfile, ProfileType};
use super::profiles::{
    AlternativeCommand, CommandExplanation, OptionExplanation, ProfileConfig, UsageExample,
};

/// Builder for explainer-mode prompts
///
/// This prompt builder generates responses that explain commands in detail,
/// suitable for users who want to learn how shell commands work.
pub struct ExplainerPromptBuilder {
    profile: CapabilityProfile,
    #[allow(dead_code)]
    config: ProfileConfig,
    #[allow(dead_code)]
    current_directory: Option<String>,
    #[allow(dead_code)]
    available_context: Option<String>,
}

impl ExplainerPromptBuilder {
    /// Create a new explainer prompt builder with the given capability profile
    pub fn new(profile: CapabilityProfile) -> Self {
        Self {
            profile,
            config: ProfileConfig::explainer(),
            current_directory: None,
            available_context: None,
        }
    }

    /// Create with custom profile config
    pub fn with_config(profile: CapabilityProfile, config: ProfileConfig) -> Self {
        Self {
            profile,
            config,
            current_directory: None,
            available_context: None,
        }
    }

    /// Create an Ubuntu-optimized explainer prompt builder
    pub fn ubuntu() -> Self {
        Self::new(CapabilityProfile::ubuntu())
    }

    /// Set current working directory context
    pub fn current_directory(mut self, dir: impl Into<String>) -> Self {
        self.current_directory = Some(dir.into());
        self
    }

    /// Set additional context
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.available_context = Some(context.into());
        self
    }

    /// Build the system prompt for explanation mode
    pub fn build_system_prompt(&self) -> String {
        let mut prompt = String::new();

        // Role definition
        prompt.push_str(&self.build_role_section());
        prompt.push('\n');

        // Output format for explanations
        prompt.push_str(&self.build_output_format());
        prompt.push('\n');

        // Tool knowledge base
        prompt.push_str(&self.build_tool_knowledge());
        prompt.push('\n');

        // Platform context
        prompt.push_str(&self.build_platform_context());
        prompt.push('\n');

        // Few-shot examples
        prompt.push_str(&self.build_examples());
        prompt.push('\n');

        prompt.push_str("\nRespond to the next user message using the format above.\n");

        prompt
    }

    /// Format a complete chat prompt with system and user messages
    pub fn format_chat(&self, user_query: &str) -> String {
        let system_prompt = self.build_system_prompt();

        format!(
            "<|im_start|>system\n{}<|im_end|>\n<|im_start|>user\n{}<|im_end|>\n<|im_start|>assistant\n",
            system_prompt, user_query
        )
    }

    fn build_role_section(&self) -> String {
        r#"You are ShellCommandExplainer, an expert Unix systems educator.

GOAL: Given a user's question about shell commands, provide:
1. The BEST command for their task
2. A CLEAR explanation of how it works
3. Examples showing common variations

Explain like a senior engineer who helps a colleague. Follow the WRITING RULES."#
            .to_string()
    }

    fn build_output_format(&self) -> String {
        r#"OUTPUT FORMAT:

Start with a brief intro identifying the relevant tool, then show commands with comments:

```
Use `<tool>` with <key concept>:

  # <description>
  <command>

  # <variation description>
  <command variation>
```

After commands, add a brief explanation of key options.

RULES:
1. Start response with "Use `<tool>`..." identifying the main Unix utility
2. Show 2-4 command examples with # comments explaining each
3. End with 1-2 sentences explaining the most important options
4. Keep total response under 15 lines
5. Focus on POSIX-compatible commands when possible

WRITING RULES (STE-lite, from ASD-STE100):
1. Write 20 words or fewer in an instruction, 25 or fewer in a description
2. Give one instruction in each sentence
3. Use the active voice and the present tense
4. Use one word for one meaning, and use simple words ("use", not "utilize")
5. Do not use filler words such as "simply", "just" or "basically"
6. If a command can delete or change data, start with "Caution:" and the risk"#
            .to_string()
    }

    fn build_tool_knowledge(&self) -> String {
        let mut section = String::from("[TOOL KNOWLEDGE]\n");
        section.push_str("Common tasks and their primary tools:\n\n");

        section.push_str("FINDING FILES:\n");
        section.push_str("- find: locate files by name, type, size, date, permissions\n");
        section.push_str("  Key options: -name, -type f/d, -size +/-N, -mtime +/-N, -exec\n\n");

        section.push_str("SEARCHING TEXT:\n");
        section.push_str("- grep: search file contents for patterns\n");
        section.push_str("  Key options: -r (recursive), -i (ignore case), -n (line numbers), -l (files only)\n\n");

        section.push_str("LISTING & SORTING:\n");
        section.push_str("- ls: list directory contents\n");
        section.push_str(
            "  Key options: -l (long), -a (all), -h (human sizes), -t (by time), -S (by size)\n",
        );
        section.push_str("- sort: sort lines of text\n");
        section.push_str(
            "  Key options: -n (numeric), -r (reverse), -k N (by column N), -h (human sizes)\n\n",
        );

        section.push_str("DISK & SIZE:\n");
        section.push_str("- du: disk usage by directory\n");
        section.push_str("  Key options: -s (summary), -h (human), -d N (depth)\n");
        section.push_str("- df: disk space on filesystems\n");
        section.push_str("  Key options: -h (human readable)\n\n");

        section.push_str("TEXT PROCESSING:\n");
        section.push_str("- awk: field-based text processing\n");
        section.push_str("  Pattern: awk '{print $1}' (print first field)\n");
        section.push_str("- sed: stream editing/substitution\n");
        section.push_str("  Pattern: sed 's/old/new/g' (global substitution)\n");
        section.push_str("- cut: extract columns\n");
        section.push_str("  Pattern: cut -d',' -f1 (first comma-separated field)\n\n");

        section.push_str("PROCESSES:\n");
        section.push_str("- ps: list processes\n");
        section.push_str("  Pattern: ps aux (all processes, detailed)\n");
        section.push_str("- top/htop: interactive process viewer\n\n");

        section.push_str("NETWORK:\n");
        section.push_str("- netstat/ss: network connections and ports\n");
        section.push_str("- lsof: list open files (including network)\n");
        section.push_str("  Pattern: lsof -i :PORT (find process on port)\n");

        section
    }

    fn build_platform_context(&self) -> String {
        let mut section = format!(
            "[PLATFORM]\nOS: {} {}\nProfile: {}\n",
            self.profile.os_name, self.profile.os_version, self.profile.profile_type
        );

        // Add platform-specific notes
        match self.profile.profile_type {
            ProfileType::Bsd => {
                section.push_str("\nBSD/macOS notes:\n");
                section.push_str("- Use `stat -f '%z %N'` instead of `stat -c '%s %n'`\n");
                section.push_str("- find doesn't support -printf, use -exec stat instead\n");
                section.push_str("- sed -i requires '' argument: sed -i '' 's/...'\n");
            }
            ProfileType::GnuLinux => {
                section.push_str("\nGNU/Linux notes:\n");
                section.push_str("- Full GNU coreutils available\n");
                section.push_str("- find supports -printf for custom output\n");
                section.push_str("- grep supports -P for Perl regex\n");
            }
            ProfileType::Busybox => {
                section.push_str("\nBusyBox notes:\n");
                section.push_str("- Limited flag support, prefer POSIX options\n");
                section.push_str("- Some GNU extensions unavailable\n");
            }
            _ => {}
        }

        if let Some(ctx) = &self.available_context {
            section.push_str(&format!("\n[CONTEXT]\n{}\n", ctx));
        }

        section
    }

    fn build_examples(&self) -> String {
        let mut section = String::from("[EXAMPLES]\n\n");

        // Example 1: Finding files
        section.push_str("User: \"how to find files modified in last 24 hours\"\n");
        section.push_str("Assistant: Use `find` with time-based filters:\n\n");
        section.push_str("  # Last 24 hours\n");
        section.push_str("  find . -type f -mtime -1\n\n");
        section.push_str("  # Last 24 hours with details\n");
        section.push_str("  find . -type f -mtime -1 -ls\n\n");
        section.push_str("  # Specific directory, last 24 hours\n");
        section.push_str("  find /path/to/dir -type f -mtime -1\n\n");
        section.push_str("  # Last N minutes (more precise)\n");
        section.push_str("  find . -type f -mmin -1440  # 1440 minutes = 24 hours\n\n");
        section.push_str(
            "The `-mtime -1` means \"modified less than 1 day ago\". Use `-mmin` for minute precision.\n\n",
        );

        // Example 2: Disk usage
        section.push_str("User: \"show disk usage by folder sorted by size\"\n");
        section.push_str("Assistant: Use `du` with `sort` for size-ordered output:\n\n");
        section.push_str("  # Top 10 largest directories\n");
        section.push_str("  du -sh */ | sort -rh | head -10\n\n");
        section.push_str("  # All subdirectories with human-readable sizes\n");
        section.push_str("  du -h --max-depth=1 | sort -rh\n\n");
        section.push_str("  # Specific path\n");
        section.push_str("  du -sh /var/log/* | sort -rh\n\n");
        section.push_str(
            "The `-s` flag summarizes each argument, `-h` gives human-readable sizes, and `sort -rh` sorts largest first.\n\n",
        );

        // Example 3: Searching text
        section.push_str("User: \"search for TODO in all python files\"\n");
        section.push_str("Assistant: Use `grep` for text search across files:\n\n");
        section.push_str("  # Basic recursive search\n");
        section.push_str("  grep -rn 'TODO' --include='*.py' .\n\n");
        section.push_str("  # Case-insensitive\n");
        section.push_str("  grep -rni 'todo' --include='*.py' .\n\n");
        section.push_str("  # Just show matching files\n");
        section.push_str("  grep -rl 'TODO' --include='*.py' .\n\n");
        section.push_str(
            "The `-r` flag searches recursively, `-n` shows line numbers, and `--include` filters by filename pattern.\n",
        );

        section
    }

    /// Generate a structured explanation for a command (for programmatic use)
    ///
    /// All text follows STE-lite (see [`super::ste`]). The contract test
    /// `tests/explain_ste_contract.rs` checks it.
    pub fn create_explanation(&self, command: &str, intent: &str) -> CommandExplanation {
        // This would typically be generated by the LLM, but we can provide
        // static explanations for common patterns
        let tool = self.identify_primary_tool(command);

        CommandExplanation {
            command: command.to_string(),
            // Displayed after "Use `<tool>`", so it must not repeat the tool.
            summary: summary_from_intent(intent),
            detailed_explanation: self.generate_explanation_for_command(command, &tool),
            option_breakdown: self.extract_options(command, &tool),
            examples: self.generate_examples(&tool, intent),
            alternatives: self.generate_alternatives(&tool),
            tool_used: tool,
            use_cases: vec![intent.to_string()],
        }
    }

    fn identify_primary_tool(&self, command: &str) -> String {
        let command = command.trim();
        // Get first word (the command name)
        command
            .split_whitespace()
            .next()
            .unwrap_or("unknown")
            .to_string()
    }

    fn generate_explanation_for_command(&self, command: &str, tool: &str) -> String {
        let words = first_segment_words(command);
        let mut text = match tool {
            "find" => find_description(&find_predicates(&words)),
            "grep" => "`grep` reads files and shows each line that matches a pattern. \
                       It does not change the files."
                .to_string(),
            "du" => "`du` shows how much disk space files and directories use. \
                     It does not change the files."
                .to_string(),
            "ls" => "`ls` shows the files and directories in a directory. \
                     It does not change the files."
                .to_string(),
            _ => format!(
                "This command uses `{}`. Read each part of the command before you run it.",
                tool
            ),
        };

        // STE warnings come first, before the description.
        if tool == "find" {
            let predicates = find_predicates(&words);
            let mut cautions = Vec::new();
            if predicates.contains(&"-delete") {
                cautions.push(
                    "Caution: `-delete` removes each file that matches, and you cannot undo it. \
                     Run the command without `-delete` first to see the list of files.",
                );
            }
            if predicates.iter().any(|w| FIND_EXEC_ACTIONS.contains(w)) {
                cautions.push(
                    "Caution: `-exec` runs a command on each file that matches. \
                     Run the command without `-exec` first to see the list of files.",
                );
            }
            if !cautions.is_empty() {
                text = format!("{}\n\n{text}", cautions.join("\n\n"));
            }
        }

        text
    }

    fn extract_options(&self, command: &str, tool: &str) -> Vec<OptionExplanation> {
        let words = first_segment_words(command);
        let short = match tool {
            "grep" => short_flags(&words, GREP_ARG_FLAGS),
            "ls" => short_flags(&words, LS_ARG_FLAGS),
            _ => Vec::new(),
        };
        let long = long_options(&words);
        let mut options = Vec::new();
        let mut add = |option: &str, description: &str, example: Option<&str>| {
            options.push(OptionExplanation {
                option: option.to_string(),
                description: description.to_string(),
                example_value: example.map(str::to_string),
            });
        };

        match tool {
            "find" => {
                // Words inside `-exec ... ;` belong to the executed command.
                let words = find_predicates(&words);
                // find uses whole words (`-name`), not combined short flags.
                let has_type = |kind: &str| {
                    words
                        .windows(2)
                        .any(|pair| pair[0] == "-type" && pair[1] == kind)
                };
                if has_type("f") {
                    add("-type f", "Match only regular files, not directories", None);
                }
                if has_type("d") {
                    add("-type d", "Match only directories", None);
                }
                if words.contains(&"-name") {
                    add(
                        "-name",
                        "Match the file name to a pattern. The match is case-sensitive.",
                        Some("'*.txt'"),
                    );
                }
                if words.contains(&"-iname") {
                    add(
                        "-iname",
                        "Match the file name to a pattern. The match ignores case.",
                        Some("'*.jpg'"),
                    );
                }
                if words.contains(&"-mtime") {
                    add(
                        "-mtime",
                        "Match by the time of the last change, in days. Use +N for older, -N for newer.",
                        Some("-1 (last 24 hours)"),
                    );
                }
                if words.contains(&"-mmin") {
                    add(
                        "-mmin",
                        "Match by the time of the last change, in minutes",
                        Some("-60 (last hour)"),
                    );
                }
                if words.contains(&"-size") {
                    add(
                        "-size",
                        "Match by file size. Use +N for larger, -N for smaller.",
                        Some("+100M (larger than 100 MB)"),
                    );
                }
                if words.contains(&"-delete") {
                    add(
                        "-delete",
                        "Delete each file that matches. You cannot undo this.",
                        None,
                    );
                }
                if words.contains(&"-exec") {
                    add("-exec", "Run a command on each file that matches", None);
                }
            }
            "grep" => {
                if short.contains(&'r') || short.contains(&'R') {
                    add(
                        "-r/-R",
                        "Search all files in each directory and its subdirectories",
                        None,
                    );
                }
                if short.contains(&'n') {
                    add("-n", "Show the line number of each match", None);
                }
                if short.contains(&'i') {
                    add(
                        "-i",
                        "Ignore the difference between uppercase and lowercase letters",
                        None,
                    );
                }
                if short.contains(&'l') {
                    add("-l", "Show only the names of files that match", None);
                }
                if short.contains(&'v') {
                    add("-v", "Show the lines that do not match", None);
                }
                if short.contains(&'w') {
                    add("-w", "Match only whole words", None);
                }
                if short.contains(&'E') {
                    add("-E", "Use extended regular expressions", None);
                }
                if long.contains(&"include") {
                    add(
                        "--include",
                        "Search only the files with names that match the pattern",
                        Some("'*.py'"),
                    );
                }
            }
            "ls" => {
                if short.contains(&'l') {
                    add(
                        "-l",
                        "Show details: permissions, owner, size and date",
                        None,
                    );
                }
                if short.contains(&'a') {
                    add(
                        "-a",
                        "Show hidden files, which have names that start with a dot",
                        None,
                    );
                }
                if short.contains(&'h') {
                    add("-h", "Show sizes in K, M and G", None);
                }
                if short.contains(&'t') {
                    add(
                        "-t",
                        "Sort by the time of the last change, newest first",
                        None,
                    );
                }
                if short.contains(&'S') {
                    add("-S", "Sort by size, largest first", None);
                }
                if short.contains(&'d') {
                    add(
                        "-d",
                        "Show each directory as one entry, not its contents",
                        None,
                    );
                }
            }
            _ => {}
        }

        options
    }

    fn generate_examples(&self, tool: &str, _intent: &str) -> Vec<UsageExample> {
        let example = |description: &str, command: &str| UsageExample {
            description: description.to_string(),
            command: command.to_string(),
        };
        match tool {
            "find" => vec![
                example("Find all Python files", "find . -name '*.py' -type f"),
                example(
                    "Find files larger than 100 MB",
                    "find . -type f -size +100M",
                ),
                // Read-only on purpose: examples are not safety-validated.
                example("List empty directories", "find . -type d -empty"),
            ],
            "grep" => vec![
                example(
                    "Search all files and show line numbers",
                    "grep -rn 'pattern' .",
                ),
                example("Search and ignore case", "grep -ri 'pattern' ."),
                example(
                    "Search only JavaScript files",
                    "grep -rn 'pattern' --include='*.js' .",
                ),
            ],
            _ => vec![],
        }
    }

    fn generate_alternatives(&self, tool: &str) -> Vec<AlternativeCommand> {
        let alternative = |command: &str, reason: &str| AlternativeCommand {
            command: command.to_string(),
            reason: reason.to_string(),
        };
        match tool {
            "find" => vec![
                alternative("fd", "Faster, with a simpler syntax. You must install it."),
                alternative(
                    "locate",
                    "Faster for file names. It uses a database that can be out of date.",
                ),
            ],
            "grep" => vec![
                alternative(
                    "rg (ripgrep)",
                    "Much faster. It skips the files in .gitignore. You must install it.",
                ),
                alternative(
                    "ag (silver searcher)",
                    "Faster than grep. You must install it.",
                ),
            ],
            _ => vec![],
        }
    }
}

/// The headline text after "Use `<tool>`": "to <intent>". A leading
/// "how to", "how do I", "how can I" or "to" is removed, so the headline
/// does not read "to how to ..." or "to to ...".
fn summary_from_intent(intent: &str) -> String {
    let mut rest = intent.trim().trim_end_matches(['?', '.']);
    for prefix in ["how do i ", "how can i ", "how to ", "to "] {
        // `get` returns None if the prefix length splits a multi-byte char.
        if rest
            .get(..prefix.len())
            .is_some_and(|head| head.eq_ignore_ascii_case(prefix))
        {
            rest = rest[prefix.len()..].trim_start();
            break;
        }
    }
    if rest.is_empty() {
        return String::new();
    }
    // Lowercase the first letter, but keep acronyms such as "PDF".
    let mut chars = rest.chars();
    let first = chars.next().unwrap_or_default();
    let second_is_upper = chars.next().is_some_and(char::is_uppercase);
    if first.is_uppercase() && !second_is_upper {
        format!("to {}{}", first.to_lowercase(), &rest[first.len_utf8()..])
    } else {
        format!("to {rest}")
    }
}

/// find actions that run another command.
const FIND_EXEC_ACTIONS: &[&str] = &["-exec", "-execdir", "-ok", "-okdir"];

/// find actions that print file names.
const FIND_PRINT_ACTIONS: &[&str] = &["-print", "-print0", "-printf", "-ls", "-fprint", "-fls"];

/// find words without the arguments of `-exec`-style actions. The action
/// word stays; the command after it, up to `;` or `+`, is removed.
fn find_predicates<'a>(words: &[&'a str]) -> Vec<&'a str> {
    let mut predicates = Vec::new();
    let mut in_exec = false;
    for &word in words {
        if in_exec {
            in_exec = !matches!(word, "\\;" | "';'" | "\";\"" | ";" | "+");
            continue;
        }
        in_exec = FIND_EXEC_ACTIONS.contains(&word);
        predicates.push(word);
    }
    predicates
}

/// What a find command does. An explicit action (`-delete`, `-exec`)
/// turns off the implicit print, unless a print action is also given.
fn find_description(predicates: &[&str]) -> String {
    let deletes = predicates.contains(&"-delete");
    let runs = predicates.iter().any(|w| FIND_EXEC_ACTIONS.contains(w));
    let prints = predicates.iter().any(|w| FIND_PRINT_ACTIONS.contains(w));

    let mut text = String::from(
        "`find` searches a directory tree for files. It starts at the path that you give.",
    );
    if deletes {
        text.push_str(" It deletes each file that matches all of the filters.");
    }
    if runs {
        text.push_str(" It runs a command on each file that matches all of the filters.");
    }
    if !deletes && !runs {
        text.push_str(" It shows each file that matches all of the filters.");
    } else if prints {
        text.push_str(" It also shows the name of each file.");
    } else {
        text.push_str(" It does not show the file names unless you add `-print`.");
    }
    text
}

/// Words of the first command only. Flags after `|`, `||`, `&&` or `;`
/// belong to another command.
fn first_segment_words(command: &str) -> Vec<&str> {
    command
        .split_whitespace()
        .take_while(|w| !matches!(*w, "|" | "||" | "&&" | ";"))
        .collect()
}

/// Short flags, with combined flags split: `-rn` gives `r` and `n`.
/// Long options (`--include`) are not short flags. Parsing stops at `--`.
/// A flag in `takes_arg` (grep `-e PATTERN`) uses the rest of its word,
/// or the next word, as its argument, so that argument is not a flag.
fn short_flags(words: &[&str], takes_arg: &[char]) -> Vec<char> {
    let mut flags = Vec::new();
    let mut skip_next = false;
    for word in words {
        if std::mem::take(&mut skip_next) {
            continue;
        }
        if *word == "--" {
            break;
        }
        let Some(cluster) = word.strip_prefix('-') else {
            continue;
        };
        if cluster.starts_with('-') {
            continue;
        }
        for (i, c) in cluster.char_indices() {
            if !c.is_ascii_alphabetic() {
                break;
            }
            flags.push(c);
            if takes_arg.contains(&c) {
                skip_next = i + 1 == cluster.len();
                break;
            }
        }
    }
    flags
}

/// grep short options that take an argument.
const GREP_ARG_FLAGS: &[char] = &['e', 'f', 'm', 'A', 'B', 'C', 'd', 'D'];

/// ls short options that take an argument.
const LS_ARG_FLAGS: &[char] = &['I', 'w', 'T'];

/// Long option names without the value: `--include='*.py'` gives `include`.
fn long_options<'a>(words: &[&'a str]) -> Vec<&'a str> {
    words
        .iter()
        .filter_map(|w| w.strip_prefix("--"))
        .map(|w| w.split('=').next().unwrap_or(w))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explainer_prompt_builder() {
        let builder = ExplainerPromptBuilder::ubuntu();
        let prompt = builder.build_system_prompt();

        assert!(prompt.contains("ShellCommandExplainer"));
        assert!(prompt.contains("TOOL KNOWLEDGE"));
        assert!(prompt.contains("find"));
        assert!(prompt.contains("grep"));
    }

    #[test]
    fn test_format_chat() {
        let builder = ExplainerPromptBuilder::ubuntu();
        let chat = builder.format_chat("find files modified today");

        assert!(chat.starts_with("<|im_start|>system"));
        assert!(chat.contains("find files modified today"));
        assert!(chat.ends_with("<|im_start|>assistant\n"));
    }

    #[test]
    fn test_create_explanation() {
        let builder = ExplainerPromptBuilder::ubuntu();
        let explanation =
            builder.create_explanation("find . -type f -mtime -1", "find recent files");

        assert_eq!(explanation.tool_used, "find");
        assert!(!explanation.option_breakdown.is_empty());
        assert!(explanation
            .option_breakdown
            .iter()
            .any(|o| o.option == "-type f"));
    }

    #[test]
    fn test_identify_tool() {
        let builder = ExplainerPromptBuilder::ubuntu();

        assert_eq!(builder.identify_primary_tool("find . -type f"), "find");
        assert_eq!(
            builder.identify_primary_tool("grep -rn 'pattern' ."),
            "grep"
        );
        assert_eq!(builder.identify_primary_tool("ls -la"), "ls");
    }
}
