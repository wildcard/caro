//! Normalization views for the safety scanners (spec 011 §P2.6).
//!
//! The pattern scanners read a command as text. Three encodings hide a
//! dangerous command from that text while the shell still runs it:
//!
//! - **terminal control sequences** inside the command line
//!   (`rm\x1b[0m -rf /`), which a shell ignores or a terminal swallows;
//! - **ANSI-C quoting** (`$'rm -rf /'`, `$'\x72m' -rf /`), which the shell
//!   unescapes before execution;
//! - **a base64 literal decoded into an exec sink**
//!   (`eval "$(echo cm0gLXJmIC8= | base64 -d)"`), where the payload only
//!   exists after decoding.
//!
//! [`views`] returns extra strings for the scanners: the command with
//! control characters stripped, with `$'…'` unescaped, and with such a
//! payload decoded. A literal that is merely decoded and printed
//! (`echo X | base64 -d`) is data, not code, and is not decoded here.

use base64::Engine as _;
use once_cell::sync::Lazy;
use regex::Regex;

/// Largest base64 literal that is decoded.
const MAX_B64_LITERAL: usize = 4096;

/// Control sequences and control characters other than `\n`/`\t`:
/// CSI (`ESC [ … final`), OSC (`ESC ] … BEL/ST`), a lone ESC with one
/// following byte, and C0/DEL.
static CONTROL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?x)
        \x1b\[[0-9;?]*[\x20-\x2f]*[\x40-\x7e]   # CSI
      | \x1b\][^\x07\x1b]*(?:\x07|\x1b\\)       # OSC
      | \x1b[\x20-\x7e]                          # ESC + one byte
      | [\x00-\x08\x0b\x0c\x0e-\x1f\x7f]         # C0 except \t \n \r, and DEL
      | \r
    ",
    )
    .expect("valid control regex")
});

/// `$'…'` with its body (no unescaped `'` inside; `\'` is allowed).
static ANSI_C: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\$'((?:[^'\\]|\\.)*)'").expect("valid ansi-c regex"));

/// `echo|printf [-neE] <base64> | base64 -d|-D|--decode`, the literal
/// captured.
static B64_DECODE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?:\becho|\bprintf)\s+(?:-[neE]+\s+)*['"]?([A-Za-z0-9+/]{8,}={0,2})['"]?\s*\|\s*base64\s+(?:-d|-D|--decode)\b"#,
    )
    .expect("valid base64 regex")
});

/// After the decode: a pipe into a shell (optionally via sudo/doas/env).
static PIPE_TO_SHELL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"^\s*\|\s*(?:(?:sudo|doas|env|command|exec|nohup)\s+(?:-\S+\s+|[A-Za-z_]\w*=\S*\s+)*)*(?:\S*/)?(?:ba|z|da|k|mk|fi)?sh\b",
    )
    .expect("valid pipe regex")
});

/// Before the decode: an `eval`, a command substitution, or a shell `-c`
/// whose argument the substitution feeds.
static EXEC_PREFIX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?:\beval\b|\$\(|`|(?:ba|z|da|k)?sh\s+(?:-\S+\s+)*-c\s+["']?\s*\$\()[^|]*$"#)
        .expect("valid exec-prefix regex")
});

/// The normalization views of `command`, without `command` itself and
/// without duplicates.
pub fn views(command: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut push = |v: String| {
        if v != command && !v.trim().is_empty() && !out.contains(&v) {
            out.push(v);
        }
    };

    if CONTROL.is_match(command) {
        push(CONTROL.replace_all(command, "").into_owned());
    }
    if let Some(unq) = unescape_ansi_c(command) {
        push(unq);
    }
    for payload in decoded_payloads(command) {
        push(payload);
    }
    out
}

/// Replace every `$'…'` with its unescaped body. `None` when there is none.
fn unescape_ansi_c(command: &str) -> Option<String> {
    if !ANSI_C.is_match(command) {
        return None;
    }
    Some(
        ANSI_C
            .replace_all(command, |caps: &regex::Captures| unescape_body(&caps[1]))
            .into_owned(),
    )
}

/// Bash `$'…'` escapes: `\n \t \r \a \e \\ \' \" \xHH \NNN`.
fn unescape_body(body: &str) -> String {
    let mut out = String::with_capacity(body.len());
    let mut chars = body.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some('a') => out.push('\x07'),
            Some('e') | Some('E') => out.push('\x1b'),
            Some('\\') => out.push('\\'),
            Some('\'') => out.push('\''),
            Some('"') => out.push('"'),
            Some('x') => {
                let mut hex = String::new();
                while hex.len() < 2 {
                    match chars.peek() {
                        Some(h) if h.is_ascii_hexdigit() => {
                            hex.push(*h);
                            chars.next();
                        }
                        _ => break,
                    }
                }
                match u8::from_str_radix(&hex, 16) {
                    Ok(b) if !hex.is_empty() => out.push(b as char),
                    _ => {
                        out.push_str("\\x");
                        out.push_str(&hex);
                    }
                }
            }
            Some(d) if d.is_digit(8) => {
                let mut oct = String::from(d);
                while oct.len() < 3 {
                    match chars.peek() {
                        Some(o) if o.is_digit(8) => {
                            oct.push(*o);
                            chars.next();
                        }
                        _ => break,
                    }
                }
                match u8::from_str_radix(&oct, 8) {
                    Ok(b) => out.push(b as char),
                    Err(_) => out.push_str(&oct),
                }
            }
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

/// Decoded base64 literals that feed an exec sink.
fn decoded_payloads(command: &str) -> Vec<String> {
    let mut out = Vec::new();
    for caps in B64_DECODE.captures_iter(command) {
        let whole = caps.get(0).expect("match");
        let after = &command[whole.end()..];
        let before = &command[..whole.start()];
        let executes = PIPE_TO_SHELL.is_match(after) || EXEC_PREFIX.is_match(before);
        if !executes {
            continue;
        }
        let literal = &caps[1];
        if literal.len() > MAX_B64_LITERAL {
            continue;
        }
        let engine = base64::engine::general_purpose::STANDARD;
        let Ok(bytes) = engine
            .decode(literal)
            .or_else(|_| base64::engine::general_purpose::STANDARD_NO_PAD.decode(literal))
        else {
            continue;
        };
        if let Ok(text) = String::from_utf8(bytes) {
            let text = text.trim().to_string();
            if !text.is_empty() {
                out.push(text);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_control_sequences() {
        let v = views("rm\x1b[0m -rf \x1b[1m/\x1b[0m");
        assert!(v.contains(&"rm -rf /".to_string()), "{v:?}");
        assert!(views("ls -la").is_empty());
        let v = views("rm\x00 -rf\x7f /");
        assert!(v.contains(&"rm -rf /".to_string()), "{v:?}");
    }

    #[test]
    fn unescapes_ansi_c_quoting() {
        assert!(views("$'rm -rf /'").contains(&"rm -rf /".to_string()));
        assert!(views("$'\\x72m' -rf /").contains(&"rm -rf /".to_string()));
        assert!(views("$'\\162m' -rf /").contains(&"rm -rf /".to_string()));
        assert!(views("bash -c $'rm -rf /'").contains(&"bash -c rm -rf /".to_string()));
        assert_eq!(unescape_body("a\\nb\\'c\\\\d"), "a\nb'c\\d");
        // A plain single-quoted string is not ANSI-C quoting.
        assert!(views("echo 'rm -rf /'").is_empty());
    }

    #[test]
    fn decodes_base64_only_into_an_exec_sink() {
        // cm0gLXJmIC8= is "rm -rf /"
        for cmd in [
            "echo cm0gLXJmIC8= | base64 -d | sh",
            "echo cm0gLXJmIC8= | base64 --decode | sudo bash",
            "eval \"$(echo cm0gLXJmIC8= | base64 -d)\"",
            "bash -c \"$(echo cm0gLXJmIC8= | base64 -d)\"",
            "x=$(printf cm0gLXJmIC8= | base64 -D); $x",
        ] {
            let v = views(cmd);
            assert!(v.contains(&"rm -rf /".to_string()), "{cmd}: {v:?}");
        }
        // Decoded and printed, or written to a file: data, not code.
        for cmd in [
            "echo cm0gLXJmIC8= | base64 -d",
            "echo cm0gLXJmIC8= | base64 -d > out.txt",
            "echo aGk= | base64 -d | grep h",
        ] {
            assert!(views(cmd).is_empty(), "{cmd}");
        }
    }

    #[test]
    fn invalid_or_binary_payloads_are_ignored() {
        assert!(views("echo not-base64!! | base64 -d | sh").is_empty());
        // 0xff 0xfe is not UTF-8.
        assert!(views("echo //4= | base64 -d | sh").is_empty());
    }
}
