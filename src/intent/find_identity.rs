//! GNU `find` identity, checked once when the backend is selected.
//!
//! The capability profile and its cache are not evidence of GNU findutils, so
//! this module runs a fixed, bounded `find --version` against the very
//! executable that the generator will emit, and requires an affirmative GNU
//! banner. Missing, non-GNU, failing or timed-out `find` refuses before any
//! generation. The result is a receipt, not a cache. It makes no claim about
//! who built the binary or about races after the check.

// Off Linux, `verify` refuses and the rest is unused.
#![cfg_attr(not(target_os = "linux"), allow(dead_code, unused_imports))]

use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Fixed candidates, tried in order; the first regular executable file wins.
pub const FIND_CANDIDATES: &[&str] = &["/usr/bin/find", "/bin/find"];
/// Bound on the `--version` run.
pub const VERSION_TIMEOUT: Duration = Duration::from_secs(2);
/// Bound on the captured output.
pub const MAX_VERSION_BYTES: u64 = 64 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FindReceipt {
    /// Absolute path that is emitted in the command.
    pub path: String,
    /// First banner line, e.g. `find (GNU findutils) 4.9.0`.
    pub banner: String,
    /// Version token, e.g. `4.9.0`.
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum IdentityError {
    #[error("no executable find at {0:?}")]
    Missing(Vec<String>),
    #[error("'{path} --version' did not finish within {secs}s")]
    TimedOut { path: String, secs: u64 },
    #[error("'{path} --version' failed: {why}")]
    Failed { path: String, why: String },
    #[error("'{path}' is not GNU findutils (banner: {banner:?})")]
    NotGnu { path: String, banner: String },
    #[error("select-files supports GNU findutils on Linux only")]
    Unsupported,
}

/// Runs `<exe> --version`. A trait so tests need no real process.
pub trait VersionRunner {
    fn run(&self, exe: &Path) -> Result<Vec<u8>, RunFailure>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunFailure {
    TimedOut,
    Other(String),
}

/// Real runner: cleared environment, null stdin, bounded time and output.
pub struct SystemRunner {
    pub timeout: Duration,
}

impl Default for SystemRunner {
    fn default() -> Self {
        Self {
            timeout: VERSION_TIMEOUT,
        }
    }
}

impl VersionRunner for SystemRunner {
    /// One fixed deadline covers spawn, execution and output collection.
    ///
    /// stdout is drained on a thread while the child runs, so a verbose child
    /// never blocks on a full pipe. The thread keeps at most
    /// [`MAX_VERSION_BYTES`]; more output kills the child. If the child has
    /// exited but a descendant still holds the pipe open, the deadline still
    /// ends the wait (the reader thread is detached and ends when the pipe
    /// closes; it owns nothing the verifier waits on).
    fn run(&self, exe: &Path) -> Result<Vec<u8>, RunFailure> {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::{mpsc, Arc};

        let mut child = Command::new(exe)
            .arg("--version")
            .env_clear()
            .env("LC_ALL", "C")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| RunFailure::Other(format!("spawn: {e}")))?;
        let deadline = Instant::now() + self.timeout;
        let overflow = Arc::new(AtomicBool::new(false));
        let (tx, rx) = mpsc::channel::<Vec<u8>>();
        if let Some(mut so) = child.stdout.take() {
            let overflow = overflow.clone();
            std::thread::spawn(move || {
                let mut kept = Vec::new();
                let mut chunk = [0u8; 4096];
                loop {
                    match so.read(&mut chunk) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => {
                            if kept.len() + n > MAX_VERSION_BYTES as usize {
                                overflow.store(true, Ordering::SeqCst);
                                // keep draining so the child is not blocked
                                // while the verifier kills it
                                continue;
                            }
                            kept.extend_from_slice(&chunk[..n]);
                        }
                    }
                }
                let _ = tx.send(kept);
            });
        } else {
            let _ = tx.send(Vec::new());
        }
        // Terminate and reap; never leave the child behind.
        fn stop(child: &mut std::process::Child) {
            let _ = child.kill();
            let _ = child.wait();
        }
        let status = loop {
            if overflow.load(Ordering::SeqCst) {
                stop(&mut child);
                return Err(RunFailure::Other("output exceeds the size bound".into()));
            }
            match child.try_wait() {
                Ok(Some(s)) => break s,
                Ok(None) if Instant::now() >= deadline => {
                    stop(&mut child);
                    return Err(RunFailure::TimedOut);
                }
                Ok(None) => std::thread::sleep(Duration::from_millis(5)),
                Err(e) => {
                    stop(&mut child);
                    return Err(RunFailure::Other(format!("wait: {e}")));
                }
            }
        };
        let remaining = deadline.saturating_duration_since(Instant::now());
        let out = match rx.recv_timeout(remaining) {
            Ok(v) => v,
            Err(_) => {
                // Pipe still held open after the child exited: fail closed.
                stop(&mut child);
                return Err(RunFailure::TimedOut);
            }
        };
        if overflow.load(Ordering::SeqCst) {
            return Err(RunFailure::Other("output exceeds the size bound".into()));
        }
        if !status.success() {
            return Err(RunFailure::Other(format!("exit status {status}")));
        }
        Ok(out)
    }
}

/// Parse the first line; require `find (GNU findutils) <digits>.<digits>[...]`.
pub fn parse_gnu_banner(stdout: &[u8]) -> Option<(String, String)> {
    let text = std::str::from_utf8(stdout).ok()?;
    let first = text.lines().next()?.trim_end();
    let rest = first.strip_prefix("find (GNU findutils) ")?;
    let ok = rest.split('.').count() >= 2
        && rest
            .split('.')
            .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()));
    ok.then(|| (first.to_string(), rest.to_string()))
}

#[cfg(unix)]
fn is_executable_file(p: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(p)
        .map(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable_file(p: &Path) -> bool {
    std::fs::metadata(p).map(|m| m.is_file()).unwrap_or(false)
}

/// Resolve the executable among `candidates`, then verify it.
pub fn verify_with(
    runner: &dyn VersionRunner,
    candidates: &[&str],
) -> Result<FindReceipt, IdentityError> {
    let path = candidates
        .iter()
        .find(|c| is_executable_file(Path::new(c)))
        .ok_or_else(|| {
            IdentityError::Missing(candidates.iter().map(|s| s.to_string()).collect())
        })?;
    // Verify and emit the canonical path, so the checked file is the emitted one.
    let canon = std::fs::canonicalize(path)
        .ok()
        .and_then(|p| p.to_str().map(str::to_string))
        .ok_or_else(|| IdentityError::Failed {
            path: path.to_string(),
            why: "cannot canonicalize".into(),
        })?;
    let path = &canon;
    let out = runner.run(Path::new(path)).map_err(|f| match f {
        RunFailure::TimedOut => IdentityError::TimedOut {
            path: path.to_string(),
            secs: VERSION_TIMEOUT.as_secs(),
        },
        RunFailure::Other(why) => IdentityError::Failed {
            path: path.to_string(),
            why,
        },
    })?;
    match parse_gnu_banner(&out) {
        Some((banner, version)) => Ok(FindReceipt {
            path: path.to_string(),
            banner,
            version,
        }),
        None => Err(IdentityError::NotGnu {
            path: path.to_string(),
            banner: String::from_utf8_lossy(&out)
                .lines()
                .next()
                .unwrap_or("")
                .chars()
                .take(80)
                .collect(),
        }),
    }
}

/// Production check, run at backend selection.
#[cfg(target_os = "linux")]
pub fn verify() -> Result<FindReceipt, IdentityError> {
    verify_with(&SystemRunner::default(), FIND_CANDIDATES)
}

/// select-files is GNU/Linux only; other platforms refuse at selection.
#[cfg(not(target_os = "linux"))]
pub fn verify() -> Result<FindReceipt, IdentityError> {
    Err(IdentityError::Unsupported)
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;

    struct Fake(Result<Vec<u8>, RunFailure>);
    impl VersionRunner for Fake {
        fn run(&self, _: &Path) -> Result<Vec<u8>, RunFailure> {
            self.0.clone()
        }
    }

    // /bin/sh exists on every supported host and is a stand-in "executable".
    const EXE: &[&str] = &["/bin/sh"];

    #[test]
    fn gnu_banner_is_accepted() {
        let r = verify_with(
            &Fake(Ok(b"find (GNU findutils) 4.9.0\nCopyright\n".to_vec())),
            EXE,
        )
        .unwrap();
        assert_eq!(r.version, "4.9.0");
        assert_eq!(
            r.path,
            std::fs::canonicalize("/bin/sh").unwrap().to_str().unwrap()
        );
        assert_eq!(r.banner, "find (GNU findutils) 4.9.0");
    }

    #[test]
    fn non_gnu_and_spoofs_are_refused() {
        for out in [
            &b"find: unrecognized option '--version'\n"[..],
            b"BusyBox v1.36 multi-call binary.\n",
            b"",
            b"GNU findutils 4.9.0\n",
            b"find (GNU findutils)\n",
            b"find (GNU findutils) x.y\n",
            b"find (GNU findutils) 4.\n",
            // GNU text only on a later line must not count
            b"usage: find\nfind (GNU findutils) 4.9.0\n",
            b"\xff\xfe",
        ] {
            let e = verify_with(&Fake(Ok(out.to_vec())), EXE).unwrap_err();
            assert!(matches!(e, IdentityError::NotGnu { .. }), "{out:?}: {e:?}");
        }
    }

    #[test]
    fn failure_timeout_and_missing_refuse() {
        let e = verify_with(&Fake(Err(RunFailure::TimedOut)), EXE).unwrap_err();
        assert!(matches!(e, IdentityError::TimedOut { .. }));
        let e = verify_with(&Fake(Err(RunFailure::Other("boom".into()))), EXE).unwrap_err();
        assert!(matches!(e, IdentityError::Failed { .. }));
        let e = verify_with(&Fake(Ok(vec![])), &["/nonexistent/find"]).unwrap_err();
        assert!(matches!(e, IdentityError::Missing(_)));
        // a directory is not an executable file
        let e = verify_with(&Fake(Ok(vec![])), &["/usr"]).unwrap_err();
        assert!(matches!(e, IdentityError::Missing(_)));
    }

    #[test]
    fn real_runner_terminates_and_reaps_on_timeout() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let script = dir.path().join("slowfind");
        let pidfile = dir.path().join("pid");
        std::fs::write(
            &script,
            format!(
                "#!/bin/sh\necho $$ > '{}'\nexec sleep 30\n",
                pidfile.display()
            ),
        )
        .unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
        let r = SystemRunner {
            timeout: Duration::from_millis(400),
        };
        let t = Instant::now();
        assert_eq!(r.run(&script), Err(RunFailure::TimedOut));
        assert!(t.elapsed() < Duration::from_secs(5));
        let pid = std::fs::read_to_string(&pidfile)
            .unwrap()
            .trim()
            .to_string();
        // reaped: no zombie and no live process under that pid
        assert!(
            !Path::new(&format!("/proc/{pid}")).exists(),
            "child {pid} still present"
        );
    }

    fn script(dir: &Path, name: &str, body: &str) -> std::path::PathBuf {
        use std::os::unix::fs::PermissionsExt;
        let p = dir.join(name);
        std::fs::write(&p, format!("#!/bin/sh\n{body}\n")).unwrap();
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
        p
    }

    #[test]
    fn exited_child_with_held_pipe_ends_at_the_deadline() {
        // The script exits at once, but a background `sleep` inherits stdout
        // and keeps the pipe open. The verifier must not wait for EOF.
        let dir = tempfile::tempdir().unwrap();
        let pidfile = dir.path().join("sleeper.pid");
        let s = script(
            dir.path(),
            "heldfind",
            &format!(
                "echo 'find (GNU findutils) 4.9.0'\nsleep 20 &\necho $! > '{}'\nexit 0",
                pidfile.display()
            ),
        );
        let r = SystemRunner {
            timeout: Duration::from_millis(500),
        };
        let t = Instant::now();
        assert_eq!(r.run(&s), Err(RunFailure::TimedOut));
        assert!(t.elapsed() < Duration::from_secs(5), "{:?}", t.elapsed());
        // clean up the synthetic sleeper (own pid from the fixture)
        if let Ok(pid) = std::fs::read_to_string(&pidfile) {
            let _ = Command::new("kill").arg(pid.trim()).status();
        }
    }

    #[test]
    fn verbose_output_is_drained_and_bounded() {
        let dir = tempfile::tempdir().unwrap();
        // 40 KiB of padding after a valid banner: under the cap, must pass
        // and must not depend on the child exiting before a pipe fills.
        let ok = script(
            dir.path(),
            "bigok",
            "echo 'find (GNU findutils) 4.9.0'\nhead -c 40000 /dev/zero | tr '\\0' x\necho",
        );
        let r = SystemRunner::default();
        let out = r.run(&ok).unwrap();
        assert!(out.starts_with(b"find (GNU findutils)"));
        // 1 MiB: over the cap. Must fail promptly, never hang or buffer all.
        let big = script(
            dir.path(),
            "bigbad",
            "echo 'find (GNU findutils) 4.9.0'\nhead -c 1048576 /dev/zero | tr '\\0' x",
        );
        let t = Instant::now();
        let e = r.run(&big).unwrap_err();
        assert!(
            matches!(e, RunFailure::Other(ref m) if m.contains("size bound")),
            "{e:?}"
        );
        assert!(t.elapsed() < Duration::from_secs(5));
        // endless output: killed and reaped at the bound or the deadline
        let endless = script(dir.path(), "endless", "yes");
        let r2 = SystemRunner {
            timeout: Duration::from_millis(800),
        };
        let t = Instant::now();
        assert!(r2.run(&endless).is_err());
        assert!(t.elapsed() < Duration::from_secs(5));
    }

    #[test]
    fn real_runner_failure_and_nonzero() {
        let r = SystemRunner::default();
        assert!(matches!(
            r.run(Path::new("/nonexistent/find")),
            Err(RunFailure::Other(_))
        ));
        assert!(matches!(
            r.run(Path::new("/bin/false")),
            Err(RunFailure::Other(_))
        ));
    }

    #[test]
    fn real_runner_environment_is_cleared() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let script = dir.path().join("envfind");
        std::fs::write(&script, "#!/bin/sh\n/usr/bin/env\n").unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
        std::env::set_var("SELECT_FILES_LEAK_PROBE", "1");
        let out = SystemRunner::default().run(&script).unwrap();
        let s = String::from_utf8_lossy(&out);
        assert!(s.contains("LC_ALL=C"));
        assert!(!s.contains("SELECT_FILES_LEAK_PROBE"));
    }

    #[test]
    fn installed_find_is_gnu_on_this_host() {
        // Boundary test against the installed binary; skipped visibly if the
        // host `find` is not GNU (the product would refuse there as well).
        match verify() {
            Ok(r) => {
                assert!(r.banner.starts_with("find (GNU findutils)"));
                // the emitted path is the canonical, verified file
                assert_eq!(
                    std::fs::canonicalize(&r.path).unwrap().to_str(),
                    Some(r.path.as_str())
                );
            }
            Err(e) => eprintln!("SKIPPED: host find is not usable GNU find: {e}"),
        }
    }
}
