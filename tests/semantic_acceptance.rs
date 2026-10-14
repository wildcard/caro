//! Semantic acceptance for template (static-matcher) answers. Offline.
//!
//! SCOPE: this is intermediate correctness ("reject known-wrong templates").
//! It is NOT the A2 end-to-end pass of the acceptance umbrella (#1662): a
//! declined request still reaches the CPU stub or the model downloader.
//!
//! Each case states an outcome, not a snapshot: which values the command must
//! carry, or that the matcher must decline for a named reason because no
//! template can bind the request. Fixture tests run read-only allowlisted
//! commands on a disposable directory and check real counts and files.
//!
//! The tests call `StaticMatcher` only. They never touch the model loader or
//! the network. Strings that name `rm`, `mv` and the like are matcher inputs
//! or outputs and are never executed.

use caro::backends::static_matcher::StaticMatcher;
use caro::backends::CommandGenerator;
use caro::models::{CommandRequest, ShellType};
use caro::prompts::CapabilityProfile;
use std::path::{Path, PathBuf};

/// What a correct answer to the query looks like.
enum Outcome {
    /// A command is returned and holds every `must` value.
    Bound { must: &'static [&'static str] },
    /// Either a command that holds every `must` value, or a decline whose
    /// reason names the part that no template could bind.
    BoundOrDecline {
        must: &'static [&'static str],
        why: &'static str,
    },
    /// No template may answer. The error must be the gate's own decline and
    /// name `why`, not any error.
    Decline { why: &'static str },
    /// Regression only: a command, if any, must not hold `never`. This is not
    /// a complete-operation pass.
    Avoid { never: &'static [&'static str] },
}

struct Case {
    query: &'static str,
    outcome: Outcome,
}

const GATE: &str = "template cannot bind the request";

const CASES: &[Case] = &[
    // Explicit file names must survive into the command, with exact case.
    Case {
        query: "count lines in README.md",
        outcome: Outcome::Bound {
            must: &["README.md"],
        },
    },
    Case {
        query: "count lines in README.MD",
        outcome: Outcome::Decline { why: "README.MD" },
    },
    Case {
        query: "count lines in 'my file.txt'",
        outcome: Outcome::Decline { why: "my file.txt" },
    },
    Case {
        query: "count words in café notes.txt",
        outcome: Outcome::Decline { why: "notes.txt" },
    },
    Case {
        query: "count words in -rf.txt",
        outcome: Outcome::Decline { why: "-rf.txt" },
    },
    Case {
        query: "show first 10 lines of data.csv",
        outcome: Outcome::Decline { why: "data.csv" },
    },
    Case {
        query: "count HTTP status codes in logs/access.log",
        outcome: Outcome::Decline {
            why: "logs/access.log",
        },
    },
    // Destinations and ports must survive.
    Case {
        query: "find large files over 100MB in ~/Downloads",
        outcome: Outcome::Decline { why: "~/Downloads" },
    },
    Case {
        query: "find all PDF files larger than 10MB in Downloads",
        outcome: Outcome::BoundOrDecline {
            must: &["pdf", "10M", "Downloads"],
            why: "Downloads",
        },
    },
    Case {
        query: "show processes using port 3000",
        outcome: Outcome::Decline { why: "3000" },
    },
    Case {
        query: "kill the process on port 3000",
        outcome: Outcome::Decline { why: "3000" },
    },
    // File-type words must survive.
    Case {
        query: "list all Python files",
        outcome: Outcome::Decline { why: "python" },
    },
    Case {
        query: "find all rust files modified today",
        outcome: Outcome::Decline { why: "rust" },
    },
    Case {
        query: "replace all tabs with spaces in python files",
        outcome: Outcome::Decline { why: "python" },
    },
    // A stand-in value must not be presented as an answer.
    Case {
        query: "create a zip archive of the src folder",
        outcome: Outcome::Decline { why: "src" },
    },
    Case {
        query: "show docker containers by memory usage",
        outcome: Outcome::Decline {
            why: "container_name",
        },
    },
    // "remove" is not "move": regression only.
    Case {
        query: "remove old log files",
        outcome: Outcome::Avoid { never: &["mv "] },
    },
    Case {
        query: "remove temp files",
        outcome: Outcome::Avoid { never: &["mv "] },
    },
    // Controls that bind a type word and a size (apostrophes are unit-tested in slot_fidelity).
    Case {
        query: "search for TODO comments in all Python files",
        outcome: Outcome::Bound { must: &["TODO"] },
    },
    Case {
        query: "find files larger than 1GB",
        outcome: Outcome::Bound { must: &["+1G"] },
    },
    // Controls: plain, fully bound requests keep their template answer.
    Case {
        query: "list all files",
        outcome: Outcome::Bound { must: &["ls"] },
    },
    Case {
        query: "show disk usage",
        outcome: Outcome::Bound { must: &["df"] },
    },
    Case {
        query: "find files modified in the last 24 hours",
        outcome: Outcome::Bound {
            must: &["-mtime 0"],
        },
    },
];

/// Judge one matcher result against the stated outcome.
fn judge(case: &Case, result: Result<String, String>) -> Result<(), String> {
    let holds = |cmd: &str, must: &[&str]| -> Result<(), String> {
        let lower = cmd.to_lowercase();
        for m in must {
            // Type words match as text; names and directories match exactly.
            let found = if m.chars().all(|c| c.is_ascii_lowercase()) {
                lower.contains(m)
            } else {
                cmd.contains(m)
            };
            if !found {
                return Err(format!("`{}`: `{cmd}` lacks `{m}`", case.query));
            }
        }
        Ok(())
    };
    match (&case.outcome, result) {
        (Outcome::Bound { must }, Ok(cmd)) => holds(&cmd, must),
        (Outcome::Bound { .. }, Err(e)) => Err(format!("`{}` was declined: {e}", case.query)),
        (Outcome::BoundOrDecline { must, .. }, Ok(cmd)) => holds(&cmd, must),
        (Outcome::BoundOrDecline { why, .. }, Err(e)) | (Outcome::Decline { why }, Err(e)) => {
            if e.contains(GATE) && e.contains(why) {
                Ok(())
            } else {
                Err(format!(
                    "`{}`: decline `{e}` does not name `{why}` from the gate",
                    case.query
                ))
            }
        }
        (Outcome::Decline { .. }, Ok(cmd)) => {
            Err(format!("`{}` must decline, got `{cmd}`", case.query))
        }
        (Outcome::Avoid { never }, Ok(cmd)) => match never.iter().find(|n| cmd.contains(**n)) {
            Some(n) => Err(format!("`{}`: `{cmd}` holds forbidden `{n}`", case.query)),
            None => Ok(()),
        },
        (Outcome::Avoid { .. }, Err(_)) => Ok(()),
    }
}

async fn ask(query: &str) -> Result<String, String> {
    let matcher = StaticMatcher::new(CapabilityProfile::ubuntu());
    matcher
        .generate_command(&CommandRequest::new(query, ShellType::Bash))
        .await
        .map(|g| g.command)
        .map_err(|e| e.to_string())
}

fn case(query: &str) -> &'static Case {
    CASES
        .iter()
        .find(|c| c.query == query)
        .expect("case exists")
}

#[tokio::test]
async fn template_answers_carry_the_users_values() {
    let mut failures = Vec::new();
    for case in CASES {
        if let Err(msg) = judge(case, ask(case.query).await) {
            failures.push(msg);
        }
    }
    assert!(
        failures.is_empty(),
        "semantic failures:\n{}",
        failures.join("\n")
    );
}

/// Negative control: the judge must reject the exact wrong answers that
/// v1.5.0 and earlier main gave. If it does not, the table proves nothing.
#[test]
fn judge_rejects_the_known_wrong_answers() {
    let wrong: &[(&str, &str)] = &[
        ("count lines in 'my file.txt'", "wc -l README.md"),
        ("count words in café notes.txt", "wc -w file.txt"),
        ("show processes using port 3000", "lsof -i :8080"),
        ("kill the process on port 3000", "kill PID"),
        (
            "create a zip archive of the src folder",
            "tar -czf archive.tar.gz directory/",
        ),
        ("list all Python files", "ls -la"),
        ("remove old log files", "mv source.txt /path/to/dest.txt"),
    ];
    for (q, cmd) in wrong {
        assert!(
            judge(case(q), Ok((*cmd).to_string())).is_err(),
            "judge accepted wrong answer `{cmd}` for `{q}`"
        );
    }
}

/// Mutation control: removing any one constraint from a faithful PDF answer
/// must turn the judge red. `10M` alone is not enough.
#[test]
fn pdf_answer_needs_type_threshold_and_directory() {
    let q = case("find all PDF files larger than 10MB in Downloads");
    let faithful = "find Downloads -type f -iname '*.pdf' -size +10M";
    assert!(judge(q, Ok(faithful.into())).is_ok());
    for mutant in [
        "find Downloads -type f -size +10M",        // PDF filter removed
        "find Downloads -type f -iname '*.pdf'",    // threshold removed
        "find . -type f -iname '*.pdf' -size +10M", // directory removed
        "find . -type f -size +10M",                // the old template answer
    ] {
        assert!(
            judge(q, Ok(mutant.into())).is_err(),
            "judge accepted `{mutant}`"
        );
    }
}

/// Mutation control for the decline arm: an unrelated error is not a decline.
#[test]
fn decline_must_come_from_the_gate_and_name_the_value() {
    let q = case("count lines in 'my file.txt'");
    assert!(judge(
        q,
        Err("Backend is not available: No static pattern match found".into())
    )
    .is_err());
    assert!(judge(
        q,
        Err("template cannot bind the request: something else".into())
    )
    .is_err());
    assert!(judge(
        q,
        Err("template cannot bind the request: user value `my file.txt` is missing".into())
    )
    .is_ok());
}

// ---------------------------------------------------------------------------
// Native fixture outcomes (disposable directory, read-only allowlisted commands)
// ---------------------------------------------------------------------------

struct Fixture(PathBuf);

impl Fixture {
    /// A fresh directory this process creates and owns. `create_dir` fails if
    /// the path already exists, so nothing is deleted before ownership is clear.
    fn new(tag: &str) -> Self {
        use std::sync::atomic::{AtomicU32, Ordering};
        static N: AtomicU32 = AtomicU32::new(0);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!(
            "caro-semantic-{tag}-{}-{nanos}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&dir).expect("fresh fixture dir must not exist");
        Fixture(dir)
    }
    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Fixture {
    /// Removes only the directory this fixture created.
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// The exact read-only commands the fixture tests may run, as program + argv.
/// No shell is involved, so operators, substitution and redirection have no
/// meaning. A command that is not in this map is rejected, never run.
fn approved_argv(cmd: &str) -> Option<(&'static str, &'static [&'static str])> {
    Some(match cmd {
        "wc -l README.md" => ("wc", &["-l", "README.md"]),
        "wc -l README.MD" => ("wc", &["-l", "README.MD"]),
        "wc -c README.md" => ("wc", &["-c", "README.md"]),
        r#"find . -name "*.py" -type f -mtime -7"# => (
            "find",
            &[".", "-name", "*.py", "-type", "f", "-mtime", "-7"],
        ),
        r#"find . -name "*.py" -type f"# => ("find", &[".", "-name", "*.py", "-type", "f"]),
        "find . -type f -mtime -7" => ("find", &[".", "-type", "f", "-mtime", "-7"]),
        _ => return None,
    })
}

/// Run an approved command in `dir` and return stdout. Checks the exit status.
fn run_readonly(cmd: &str, dir: &Path) -> Result<String, String> {
    let (program, argv) =
        approved_argv(cmd).ok_or_else(|| format!("`{cmd}` is not on the read-only allowlist"))?;
    let out = std::process::Command::new(program)
        .args(argv)
        .current_dir(dir)
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(format!("`{cmd}` exited with {}", out.status));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Rejection tests call only the lookup, so no unsafe string is ever executed.
#[test]
fn allowlist_rejects_shell_tricks_without_running_them() {
    for unsafe_cmd in [
        "find . -print; touch SENTINEL",
        "find . -fprint SENTINEL",
        "find . -type f -delete",
        "find . -exec true {} +",
        "find / -type f",
        "find ../ -type f",
        "wc -l /etc/hostname",
        "wc -l ../README.md",
        "wc -l README.md; touch SENTINEL",
        "wc -l README.md > SENTINEL",
        "wc -l $(touch SENTINEL)",
        "wc -l `touch SENTINEL`",
        "wc -l README.md | cat",
        "sh -c true",
    ] {
        assert!(
            approved_argv(unsafe_cmd).is_none(),
            "allowlist accepted `{unsafe_cmd}`"
        );
    }
}

#[test]
fn fixture_dirs_are_unique_and_owned() {
    let (a, b) = (Fixture::new("own"), Fixture::new("own"));
    assert_ne!(a.path(), b.path());
    assert!(a.path().is_dir() && b.path().is_dir());
}

/// The README count must be the real number of lines in the fixture file.
fn counted_lines(cmd: &str, dir: &Path) -> Result<(), String> {
    let out = run_readonly(cmd, dir)?;
    let n: usize = out
        .split_whitespace()
        .next()
        .unwrap_or("")
        .parse()
        .map_err(|_| format!("no count in `{out}`"))?;
    if n == 3 && cmd.contains("-l ") {
        Ok(())
    } else {
        Err(format!("`{cmd}` gave {n}, fixture has 3 lines"))
    }
}

#[tokio::test]
async fn readme_count_is_the_real_line_count() {
    let fx = Fixture::new("count");
    std::fs::write(fx.path().join("README.md"), "one\ntwo\nthree\n").unwrap();
    // A decoy with a different line count, so naming the wrong file is caught.
    // On a case-insensitive file system (macOS default) the two names are one
    // file, so the decoy is only written, and checked, when they stay distinct.
    let decoy = !fx.path().join("README.MD").exists();
    if decoy {
        std::fs::write(fx.path().join("README.MD"), "x\n").unwrap();
    }
    let decoy = decoy
        && std::fs::read_to_string(fx.path().join("README.md")).unwrap() == "one\ntwo\nthree\n";
    let cmd = ask("count lines in README.md")
        .await
        .expect("bound request answers");
    counted_lines(&cmd, fx.path()).unwrap();
    // Mutation controls: wrong operation and wrong file both turn it red.
    assert!(counted_lines("wc -c README.md", fx.path()).is_err());
    if decoy {
        assert!(counted_lines("wc -l README.MD", fx.path()).is_err());
    }
}

#[tokio::test]
async fn recent_python_files_are_the_requested_set() {
    let fx = Fixture::new("py");
    std::fs::write(fx.path().join("recent.py"), "x").unwrap();
    std::fs::write(fx.path().join("note.txt"), "x").unwrap();
    let old = std::fs::File::create(fx.path().join("stale.py")).unwrap();
    let thirty_days = std::time::Duration::from_secs(30 * 24 * 3600);
    old.set_modified(std::time::SystemTime::now() - thirty_days)
        .unwrap();

    let oracle = |cmd: &str| -> Result<(), String> {
        let out = run_readonly(cmd, fx.path())?;
        if out == "./recent.py" {
            Ok(())
        } else {
            Err(format!("`{cmd}` listed `{out}`, expected only ./recent.py"))
        }
    };
    let cmd = ask("find all python files modified in the last 7 days")
        .await
        .expect("bound request answers");
    oracle(&cmd).unwrap();
    // Mutation controls: dropping the type or the age turns it red.
    assert!(oracle("find . -type f -mtime -7").is_err());
    assert!(oracle("find . -name \"*.py\" -type f").is_err());
}

/// Two distinct files differ only by case on a case-sensitive file system. A
/// command that names the other one is the wrong answer; a decline is fine.
#[tokio::test]
async fn exact_case_file_is_the_requested_target() {
    let fx = Fixture::new("case");
    for name in ["README.md", "README.MD"] {
        std::fs::write(fx.path().join(name), "one\ntwo\n").unwrap();
    }
    let distinct = std::fs::read_dir(fx.path()).unwrap().count() == 2;
    for (query, wanted) in [
        ("count lines in README.MD", "README.MD"),
        ("count lines in README.md", "README.md"),
    ] {
        if let Ok(cmd) = ask(query).await {
            let target = cmd.split_whitespace().last().unwrap_or_default();
            assert!(
                !distinct || target == wanted,
                "`{query}` answered `{cmd}`, which names a different file than `{wanted}`"
            );
        }
    }
}
