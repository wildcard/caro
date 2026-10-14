//! Argv construction, shell rendering and the structural closed-token check.

use super::check::{Pattern, VerifiedIntent};

/// Build the argv. `find` is the executable whose identity was verified.
pub fn argv(intent: &VerifiedIntent, find: &str) -> Vec<String> {
    let mut a: Vec<String> = vec![
        find.to_string(),
        "-P".into(),
        intent.root_emit().to_string(),
        "-type".into(),
        "f".into(),
    ];
    match intent.pattern() {
        None => {}
        Some(Pattern::Name(n)) => a.extend(["-name".into(), n.clone()]),
        Some(Pattern::ExtInsensitive(e)) => a.extend(["-iname".into(), format!("*.{e}")]),
        Some(Pattern::ExtSensitive(e)) => a.extend(["-name".into(), format!("*.{e}")]),
    }
    if let Some(w) = intent.window() {
        a.extend([
            "-newermt".into(),
            format!("@{}", w.cutoff),
            "!".into(),
            "-newermt".into(),
            format!("@{}", w.t0),
        ]);
    }
    a.push("-print0".into());
    a
}

/// Quote one argument for a POSIX shell. A conservative safe set stays bare;
/// everything else, including `!` (history expansion), is single-quoted.
fn quote_arg(a: &str) -> String {
    let safe = !a.is_empty()
        && a.bytes().all(|b| {
            b.is_ascii_alphanumeric()
                || matches!(
                    b,
                    b'_' | b'@' | b'%' | b'+' | b'=' | b':' | b',' | b'.' | b'/' | b'-'
                )
        });
    if safe {
        a.to_string()
    } else {
        format!("'{}'", a.replace('\'', "'\\''"))
    }
}

/// Render the shell string. Every argument is its own shell word.
pub fn render(argv: &[String]) -> String {
    argv.iter()
        .map(|a| quote_arg(a))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Structural check on the emitted argv: only the closed token set, in the
/// canonical order. Policy check, not behavioral evidence.
pub fn check_closed(argv: &[String], find: &str) -> Result<(), String> {
    fn take(s: &[&str], i: &mut usize, want: &str) -> Result<(), String> {
        if s.get(*i).copied() == Some(want) {
            *i += 1;
            Ok(())
        } else {
            Err(format!("expected '{want}' at position {i}"))
        }
    }
    let s: Vec<&str> = argv.iter().map(String::as_str).collect();
    let mut i = 0;
    take(&s, &mut i, find)?;
    take(&s, &mut i, "-P")?;
    if s.len() <= i {
        return Err("missing root".into());
    }
    i += 1; // the root operand; its form is enforced by emit_root
    take(&s, &mut i, "-type")?;
    take(&s, &mut i, "f")?;
    if matches!(s.get(i), Some(&"-name") | Some(&"-iname")) {
        if s.len() <= i + 1 {
            return Err("filter without operand".into());
        }
        i += 2;
    }
    if s.get(i) == Some(&"-newermt") {
        if s.len() <= i + 1 {
            return Err("-newermt without operand".into());
        }
        i += 2;
        take(&s, &mut i, "!")?;
        take(&s, &mut i, "-newermt")?;
        if s.len() <= i {
            return Err("-newermt without operand".into());
        }
        i += 1;
    }
    take(&s, &mut i, "-print0")?;
    if i != s.len() {
        return Err("unexpected trailing arguments".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intent::check::verify;
    use crate::intent::facts::{parse, Outcome};

    const T0: i64 = 1_760_000_000;
    const FIND: &str = "/usr/bin/find";

    fn cmd(req: &str) -> (Vec<String>, String) {
        let Outcome::Accept(f) = parse(req) else {
            panic!("{req}")
        };
        let v = verify(&f, req, T0).unwrap();
        let a = argv(&v, FIND);
        check_closed(&a, FIND).unwrap();
        let r = render(&a);
        assert_eq!(shell_words::split(&r).unwrap(), a, "round trip");
        (a, r)
    }

    #[test]
    fn seven_day_python_exact_command() {
        let (_, r) = cmd("find python files modified in the last 7 days");
        assert_eq!(
            r,
            format!(
                "/usr/bin/find -P . -type f -iname '*.py' -newermt @{} '!' -newermt @{T0} -print0",
                T0 - 604_800
            )
        );
    }

    #[test]
    fn plain_python_and_pdf() {
        assert_eq!(
            cmd("list all python files").1,
            "/usr/bin/find -P . -type f -iname '*.py' -print0"
        );
        assert_eq!(
            cmd("list all PDF files").1,
            "/usr/bin/find -P . -type f -iname '*.pdf' -print0"
        );
    }

    #[test]
    fn explicit_pattern_and_name_are_case_sensitive() {
        assert!(cmd("find *.py files").1.contains("-name '*.py'"));
        assert!(cmd("find files named README").1.contains("-name README"));
    }

    #[test]
    fn hostile_literals_stay_one_shell_word() {
        let (a, r) = cmd("find files named \"a b;rm -rf x$(id)`y`\" in \"-x\"");
        assert!(a.contains(&"./-x".to_string()));
        assert!(a.contains(&"a b;rm -rf x$(id)`y`".to_string()));
        assert!(r.contains("'a b;rm -rf x$(id)`y`'"));
        let (a, _) = cmd("find files in \"!\"");
        assert!(a.contains(&"./!".to_string()));
    }

    #[test]
    fn closed_check_rejects_extras() {
        let (mut a, _) = cmd("list all python files");
        a.insert(a.len() - 1, "-delete".into());
        assert!(check_closed(&a, FIND).is_err());
        let (mut a, _) = cmd("list all python files");
        a.pop();
        a.push("-print".into());
        assert!(check_closed(&a, FIND).is_err());
        let (mut a, _) = cmd("list all python files");
        a.push("-exec".into());
        assert!(check_closed(&a, FIND).is_err());
    }
}
