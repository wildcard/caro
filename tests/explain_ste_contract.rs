//! Explain-mode contract: the text is correct and legible.
//!
//! Guards two properties of `caro --explain` output:
//! 1. Correctness: the option breakdown names only flags that the command
//!    has. A wrong explanation is worse than no explanation.
//! 2. Legibility: all text passes the STE-lite checker (ASD-STE100
//!    sentence, paragraph, word and voice rules).
//!
//! Background: docs/research/2026-10-03-legible-output-ste100.md

use caro::prompts::ste::{self, TextKind};
use caro::prompts::{CapabilityProfile, ExplainerPromptBuilder};

fn explainer() -> ExplainerPromptBuilder {
    ExplainerPromptBuilder::new(CapabilityProfile::ubuntu())
}

fn options(command: &str) -> Vec<String> {
    explainer()
        .create_explanation(command, "test intent")
        .option_breakdown
        .into_iter()
        .map(|o| o.option)
        .collect()
}

/// Commands that cover each tool branch and each option in the explainer.
const COMMANDS: &[&str] = &[
    "find . -type f -mtime -1",
    "find . -type d -name 'build'",
    "find . -iname '*.jpg' -size +1M -mmin -60",
    "find . -name '*.tmp' -delete",
    "find . -name '*.log' -exec gzip {} +",
    "grep -rn 'TODO' --include='*.py' .",
    "grep -rilvwE 'a|b' src",
    "ls -lahtSd .*",
    "du -sh */",
    "awk '{print $1}' access.log",
];

#[test]
fn headline_does_not_repeat_the_tool() {
    let e = explainer().create_explanation("find . -mtime 0", "find files modified today");
    // Displayed as "Use `find` <summary>:".
    assert_eq!(e.summary, "to find files modified today");
}

#[test]
fn long_option_is_not_read_as_short_flag() {
    // Regression: `--include` matched the substring "-i" and the output
    // said "case-insensitive" for a case-sensitive search.
    let opts = options("grep -rn 'TODO' --include='*.py' .");
    assert!(!opts.contains(&"-i".to_string()), "{opts:?}");
    assert!(opts.contains(&"--include".to_string()), "{opts:?}");
}

#[test]
fn combined_short_flags_are_split() {
    // Regression: `-rn` did not match "-n", `-la` did not match "-a".
    let grep = options("grep -rn 'TODO' .");
    assert!(grep.contains(&"-r/-R".to_string()) && grep.contains(&"-n".to_string()));

    let ls = options("ls -la");
    assert!(ls.contains(&"-l".to_string()) && ls.contains(&"-a".to_string()));
}

#[test]
fn flags_of_a_later_pipeline_stage_are_ignored() {
    // `-r` belongs to `sort`, not to `grep`.
    let opts = options("grep 'x' file.txt | sort -r");
    assert!(opts.is_empty(), "{opts:?}");
}

#[test]
fn destructive_find_gets_a_caution_first() {
    let e = explainer().create_explanation("find . -name '*.tmp' -delete", "delete tmp files");
    assert!(e.detailed_explanation.contains("Caution: `-delete`"));
}

#[test]
fn examples_are_read_only() {
    // Examples are shown without safety validation, so none may delete.
    for tool_cmd in ["find .", "grep x ."] {
        for ex in explainer().create_explanation(tool_cmd, "x").examples {
            assert!(!ex.command.contains("-delete"), "{}", ex.command);
            assert!(!ex.command.contains("rm "), "{}", ex.command);
        }
    }
}

#[test]
fn all_explanation_text_passes_ste_lite() {
    let mut failures = Vec::new();
    let mut check = |command: &str, field: &str, text: &str, kind: TextKind| {
        let issues = ste::check(text, kind);
        if !issues.is_empty() {
            failures.push(format!("{command} / {field}: {text:?}\n    {issues:?}"));
        }
    };

    for command in COMMANDS {
        let e = explainer().create_explanation(command, "do the task");
        check(
            command,
            "detailed_explanation",
            &e.detailed_explanation,
            TextKind::Descriptive,
        );
        for o in &e.option_breakdown {
            check(command, &o.option, &o.description, TextKind::Descriptive);
        }
        for ex in &e.examples {
            check(command, "example", &ex.description, TextKind::Procedural);
        }
        for alt in &e.alternatives {
            check(command, &alt.command, &alt.reason, TextKind::Descriptive);
        }
    }

    assert!(
        failures.is_empty(),
        "STE-lite violations:\n{}",
        failures.join("\n")
    );
}

#[test]
fn every_command_gets_an_option_breakdown_where_flags_exist() {
    // Guards against a parser change that silently drops all options.
    for command in COMMANDS
        .iter()
        .filter(|c| !c.starts_with("du") && !c.starts_with("awk"))
    {
        assert!(!options(command).is_empty(), "no options for {command}");
    }
}

#[test]
fn system_prompt_carries_the_writing_rules() {
    let prompt = explainer().build_system_prompt();
    assert!(prompt.contains("WRITING RULES (STE-lite"));
    assert!(prompt.contains("20 words or fewer"));
    assert!(prompt.contains("Caution:"));
}

#[test]
fn grep_pattern_after_terminator_or_e_is_not_a_flag() {
    // Review finding: in `grep -- -v file` and `grep -e -v file`, `-v` is
    // the pattern, not "show non-matching lines".
    for command in ["grep -- -v file", "grep -e -v file", "grep -ie -v file"] {
        let opts = options(command);
        assert!(!opts.contains(&"-v".to_string()), "{command}: {opts:?}");
    }
    // `-i` before `-e` is still a real flag.
    assert!(options("grep -ie -v file").contains(&"-i".to_string()));
}

#[test]
fn find_with_explicit_action_does_not_claim_to_show_files() {
    // Review finding: `-delete` and `-exec` turn off find's implicit print.
    let delete = explainer().create_explanation("find . -name '*.tmp' -delete", "x");
    assert!(
        !delete.detailed_explanation.contains("shows each file"),
        "{}",
        delete.detailed_explanation
    );
    assert!(delete.detailed_explanation.contains("deletes each file"));

    let exec = explainer().create_explanation("find . -name '*.log' -exec gzip {} +", "x");
    assert!(!exec.detailed_explanation.contains("shows each file"));
    assert!(exec
        .detailed_explanation
        .contains("runs a command on each file"));

    let plain = explainer().create_explanation("find . -name '*.log'", "x");
    assert!(plain.detailed_explanation.contains("shows each file"));
}
