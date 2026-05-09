//! Caro's voice — pager-era / teletype codes used as decorative epilogue
//! lines on success messages.
//!
//! The five canonical codes (anchored on the user's design intent):
//!
//! | Code | Meaning                            | When |
//! |------|------------------------------------|------|
//! | 143  | "I love you" (1-4-3 letters)       | After a long successful run, or first-time success on a new platform |
//! | 371  | "I love you too"                    | After accepting an adopt suggestion |
//! | 607  | "I miss you"                        | After `caro upgrade` regenerates a stale variant |
//! | 42   | The Hitchhiker's Guide answer       | After multi-iteration validation loops converge |
//! | 111111 | Binary all-ones; "all green"      | After all JOBs in a Carofile complete green |
//!
//! Always opt-out, never opt-in. Callers gate output on `eggs_enabled()`.

/// One of the five canonical voice codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Code {
    Love143,
    LoveBack371,
    Miss607,
    Hitchhiker42,
    AllOnes,
}

impl Code {
    /// The decorative number rendered to the terminal.
    pub fn rendered(self) -> &'static str {
        match self {
            Self::Love143 => "143",
            Self::LoveBack371 => "371",
            Self::Miss607 => "607",
            Self::Hitchhiker42 => "42",
            Self::AllOnes => "111111",
        }
    }

    /// Human-readable meaning (used in `caro --help-eggs` output, v0.2).
    pub fn meaning(self) -> &'static str {
        match self {
            Self::Love143 => "I love you (1-4-3 letters)",
            Self::LoveBack371 => "I love you too",
            Self::Miss607 => "I miss you",
            Self::Hitchhiker42 => "The answer to life, the universe, and everything",
            Self::AllOnes => "Binary all-ones — all green",
        }
    }
}

/// Pick a code appropriate for the situation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Occasion {
    /// Successful execution.
    SuccessfulRun,
    /// `caro adopt` accepted.
    Adopted,
    /// `caro upgrade` produced a fresh lock for a stale variant.
    UpgradedFromStale,
    /// Multi-iteration validation loop converged on a clean command.
    LoopConverged,
    /// All JOBs in a Carofile completed green.
    AllJobsGreen,
}

impl Occasion {
    pub fn code(self) -> Code {
        match self {
            Self::SuccessfulRun => Code::Love143,
            Self::Adopted => Code::LoveBack371,
            Self::UpgradedFromStale => Code::Miss607,
            Self::LoopConverged => Code::Hitchhiker42,
            Self::AllJobsGreen => Code::AllOnes,
        }
    }
}

/// Render an epilogue line for `occasion`. Returns an empty string when
/// `eggs` is false.
///
/// The output format is `"  <code>"` (two spaces of leading padding) so
/// callers can append to a status line cleanly.
pub fn epilogue(occasion: Occasion, eggs: bool) -> String {
    if !eggs {
        return String::new();
    }
    format!("  {}", occasion.code().rendered())
}

/// Read the `--no-eggs` policy. Reads from the env var `CARO_NO_EGGS` (any
/// non-empty value disables) and falls back to the per-call argument.
///
/// Order of precedence: env > arg.
pub fn eggs_enabled(arg_no_eggs: bool) -> bool {
    if std::env::var("CARO_NO_EGGS")
        .map(|v| !v.is_empty())
        .unwrap_or(false)
    {
        return false;
    }
    !arg_no_eggs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_code_renders_distinctly() {
        let codes = [
            Code::Love143.rendered(),
            Code::LoveBack371.rendered(),
            Code::Miss607.rendered(),
            Code::Hitchhiker42.rendered(),
            Code::AllOnes.rendered(),
        ];
        let unique: std::collections::HashSet<_> = codes.iter().collect();
        assert_eq!(unique.len(), 5);
    }

    #[test]
    fn occasions_map_to_canonical_codes() {
        assert_eq!(Occasion::SuccessfulRun.code(), Code::Love143);
        assert_eq!(Occasion::Adopted.code(), Code::LoveBack371);
        assert_eq!(Occasion::UpgradedFromStale.code(), Code::Miss607);
        assert_eq!(Occasion::LoopConverged.code(), Code::Hitchhiker42);
        assert_eq!(Occasion::AllJobsGreen.code(), Code::AllOnes);
    }

    #[test]
    fn epilogue_empty_when_eggs_disabled() {
        assert_eq!(epilogue(Occasion::SuccessfulRun, false), "");
    }

    #[test]
    fn epilogue_has_padding_and_code_when_enabled() {
        let s = epilogue(Occasion::SuccessfulRun, true);
        assert_eq!(s, "  143");
    }

    #[test]
    fn meanings_are_non_empty() {
        for c in [
            Code::Love143,
            Code::LoveBack371,
            Code::Miss607,
            Code::Hitchhiker42,
            Code::AllOnes,
        ] {
            assert!(!c.meaning().is_empty());
        }
    }
}
