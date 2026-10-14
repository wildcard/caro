//! Semantic acceptance for template (static-matcher) answers. Offline.
//!
//! Each case states an outcome, not a snapshot: which values the command must
//! carry, or that the matcher must decline (fall through) because no template
//! can bind the request. The tests call `StaticMatcher` only. They never touch
//! the model loader, the network, or a shell. Strings that name `rm`, `kill`
//! and the like are matcher inputs or outputs and are never executed.
//!
//! Regression guard for the slot-fidelity gate and for word-start keyword
//! matching (acceptance issue: wildcard/caro, "CLI usefulness acceptance").

use caro::backends::static_matcher::StaticMatcher;
use caro::backends::CommandGenerator;
use caro::models::{CommandRequest, ShellType};
use caro::prompts::CapabilityProfile;

/// What a correct answer to the query looks like.
enum Outcome {
    /// A command is returned, holds every `must` value and none of `never`.
    Bound {
        must: &'static [&'static str],
        never: &'static [&'static str],
    },
    /// A command, if any, must not hold `never`. Declining is allowed. The
    /// word-start unit test in `static_matcher` shows the keyword itself.
    Avoid { never: &'static [&'static str] },
    /// No template may answer. The matcher returns an error so a backend that
    /// can bind the request, or the clarify path, gets it.
    Decline,
}

struct Case {
    query: &'static str,
    outcome: Outcome,
}

const CASES: &[Case] = &[
    // Explicit file names must survive into the command.
    Case {
        query: "count lines in README.md",
        outcome: Outcome::Bound {
            must: &["README.md"],
            never: &[],
        },
    },
    Case {
        query: "count lines in 'my file.txt'",
        outcome: Outcome::Decline,
    },
    Case {
        query: "count words in café notes.txt",
        outcome: Outcome::Decline,
    },
    Case {
        query: "count words in -rf.txt",
        outcome: Outcome::Decline,
    },
    Case {
        query: "show first 10 lines of data.csv",
        outcome: Outcome::Decline,
    },
    // Destinations and ports must survive.
    Case {
        query: "find large files over 100MB in ~/Downloads",
        outcome: Outcome::Decline,
    },
    Case {
        query: "find all PDF files larger than 10MB in Downloads",
        outcome: Outcome::Bound {
            must: &["10M"],
            never: &[],
        },
    },
    Case {
        query: "show processes using port 3000",
        outcome: Outcome::Decline,
    },
    Case {
        query: "kill the process on port 3000",
        outcome: Outcome::Decline,
    },
    // A stand-in value must not be presented as an answer.
    Case {
        query: "create a zip archive of the src folder",
        outcome: Outcome::Decline,
    },
    Case {
        query: "show docker containers by memory usage",
        outcome: Outcome::Decline,
    },
    // "remove" is not "move": never answer a delete request with `mv`.
    Case {
        query: "remove old log files",
        outcome: Outcome::Avoid { never: &["mv "] },
    },
    Case {
        query: "remove temp files",
        outcome: Outcome::Avoid { never: &["mv "] },
    },
    // Values the gate must not mistake for literals: apostrophes, "and/or".
    Case {
        query: "what's using the most disk in the user's home",
        outcome: Outcome::Avoid {
            never: &["README.md"],
        },
    },
    Case {
        query: "search for TODO comments in all Python files",
        outcome: Outcome::Bound {
            must: &["TODO"],
            never: &[],
        },
    },
    Case {
        query: "find files larger than 1GB",
        outcome: Outcome::Bound {
            must: &["+1G"],
            never: &[],
        },
    },
    // Controls: plain, fully bound requests keep their template answer.
    Case {
        query: "list all files",
        outcome: Outcome::Bound {
            must: &["ls"],
            never: &[],
        },
    },
    Case {
        query: "show disk usage",
        outcome: Outcome::Bound {
            must: &["df"],
            never: &[],
        },
    },
    Case {
        query: "find files modified in the last 24 hours",
        outcome: Outcome::Bound {
            must: &["-mtime 0"],
            never: &[],
        },
    },
];

/// Judge one matcher result against the stated outcome.
fn judge(case: &Case, result: Result<String, String>) -> Result<(), String> {
    match (&case.outcome, result) {
        (Outcome::Decline, Err(_)) => Ok(()),
        (Outcome::Decline, Ok(cmd)) => Err(format!("`{}` must decline, got `{cmd}`", case.query)),
        // Bound cases may also decline, but only if they hold no value to lose.
        (Outcome::Bound { must, never }, Ok(cmd)) => {
            for m in *must {
                if !cmd.contains(m) {
                    return Err(format!("`{}`: `{cmd}` lacks `{m}`", case.query));
                }
            }
            for n in *never {
                if cmd.contains(n) {
                    return Err(format!("`{}`: `{cmd}` holds forbidden `{n}`", case.query));
                }
            }
            Ok(())
        }
        (Outcome::Avoid { .. }, Err(_)) => Ok(()),
        (Outcome::Avoid { never }, Ok(cmd)) => match never.iter().find(|n| cmd.contains(**n)) {
            Some(n) => Err(format!("`{}`: `{cmd}` holds forbidden `{n}`", case.query)),
            None => Ok(()),
        },
        (Outcome::Bound { .. }, Err(e)) => Err(format!("`{}` was declined: {e}", case.query)),
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
/// v1.5.0 and earlier main gave. If it does not, the table above proves nothing.
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
        ("remove old log files", "mv source.txt /path/to/dest.txt"),
    ];
    for (q, cmd) in wrong {
        let case = CASES.iter().find(|c| c.query == *q).expect("case exists");
        assert!(
            judge(case, Ok((*cmd).to_string())).is_err(),
            "judge accepted wrong answer `{cmd}` for `{q}`"
        );
    }
}

/// Mutation proof for the gate itself: a bound value is accepted, so the
/// gate is not simply declining everything.
#[tokio::test]
async fn bound_value_is_accepted() {
    let cmd = ask("count lines in README.md")
        .await
        .expect("bound request answers");
    assert!(cmd.contains("README.md"), "{cmd}");
}
