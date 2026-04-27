//! Current-platform detection — maps `std::env::consts::OS` to the
//! lowercase platform identifiers CaroML uses everywhere
//! (`"macos"` / `"linux"` / `"windows"` / `"posix"`).

/// Detect the current platform identifier.
///
/// Returns one of:
/// - `"macos"`
/// - `"linux"`
/// - `"windows"`
/// - `"posix"` (FreeBSD, NetBSD, OpenBSD, illumos, etc.)
pub fn current() -> &'static str {
    match std::env::consts::OS {
        "macos" => "macos",
        "linux" => "linux",
        "windows" => "windows",
        _ => "posix",
    }
}

/// True if `s` is a CaroML-recognized platform identifier.
pub fn is_known(s: &str) -> bool {
    matches!(s, "macos" | "linux" | "windows" | "posix")
}

/// Iterate the four known platforms in canonical order. Useful for
/// `caro upgrade --all-platforms` and the lock's `supported_platforms`.
pub fn all() -> [&'static str; 4] {
    ["macos", "linux", "windows", "posix"]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_is_known() {
        assert!(is_known(current()));
    }

    #[test]
    fn all_are_known() {
        for p in all() {
            assert!(is_known(p));
        }
    }

    #[test]
    fn unknown_strings_rejected() {
        assert!(!is_known("solaris"));
        assert!(!is_known("MACOS"));
        assert!(!is_known(""));
    }
}
