//! S1b behavioral layer for `--backend select-files`.
//!
//! These tests run the *compiler-generated* read-only `find` commands (never a
//! model command) against disposable fixture trees, two ways: as argv with no
//! shell, and as the rendered string through `sh -c` / `bash -c`. They need
//! GNU findutils on Linux. Nothing here writes outside the temp directories.
//!
//! Reference files for the precision oracle live in a second temp directory,
//! outside the searched tree, and their mtimes are read back before use.
#![cfg(target_os = "linux")]

use std::collections::BTreeSet;
use std::fs::{self, File};
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use caro::intent::clock::FixedClock;
use caro::intent::find_identity;
use caro::intent::generator::SelectFilesGenerator;
use tempfile::TempDir;

/// Fixed reference instant (2023-11-14 22:13:20 UTC), never the real clock.
const T0: i64 = 1_700_000_000;
const WEEK: i64 = 7 * 24 * 3600;
const C: i64 = T0 - WEEK;

fn generator() -> SelectFilesGenerator {
    let receipt = find_identity::verify().expect("these tests need GNU findutils on Linux");
    SelectFilesGenerator::with_parts(receipt, Box::new(FixedClock::new(T0)))
}

fn st(secs: i64, nanos: u32) -> SystemTime {
    if secs >= 0 {
        UNIX_EPOCH + Duration::new(secs as u64, nanos)
    } else {
        UNIX_EPOCH - Duration::new((-secs) as u64, 0) + Duration::new(0, nanos)
    }
}

fn ns_of(t: SystemTime) -> i128 {
    match t.duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_nanos() as i128,
        Err(e) => -(e.duration().as_nanos() as i128),
    }
}

fn touch(path: &Path, t: SystemTime) {
    let f = File::create(path).unwrap();
    f.set_modified(t).unwrap();
}

fn mtime_ns(path: &Path) -> i128 {
    ns_of(fs::metadata(path).unwrap().modified().unwrap())
}

type Records = Vec<Vec<u8>>;

/// Split NUL-framed output; every record must be non-empty, the stream must
/// end with NUL, and no path may repeat.
fn records(out: &[u8]) -> Records {
    if out.is_empty() {
        return vec![];
    }
    assert_eq!(*out.last().unwrap(), 0, "output must end with NUL");
    let mut recs: Records = out[..out.len() - 1]
        .split(|b| *b == 0)
        .map(<[u8]>::to_vec)
        .collect();
    assert!(recs.iter().all(|r| !r.is_empty()), "empty record");
    recs.sort();
    let set: BTreeSet<_> = recs.iter().collect();
    assert_eq!(set.len(), recs.len(), "duplicate path in output");
    recs
}

/// Run a command; require exit 0 and empty stderr (traversal failures fail).
fn ok_records(mut c: Command) -> Result<Records, String> {
    let o = c.output().map_err(|e| e.to_string())?;
    if !o.status.success() || !o.stderr.is_empty() {
        return Err(format!(
            "status={:?} stderr={:?}",
            o.status,
            String::from_utf8_lossy(&o.stderr)
        ));
    }
    Ok(records(&o.stdout))
}

fn run_argv(argv: &[String], cwd: &Path) -> Result<Records, String> {
    let mut c = Command::new(&argv[0]);
    c.args(&argv[1..])
        .current_dir(cwd)
        .env_clear()
        .env("LC_ALL", "C");
    ok_records(c)
}

fn run_shell(shell: &str, cmd: &str, cwd: &Path) -> Result<Records, String> {
    let mut c = Command::new(shell);
    c.arg("-c")
        .arg(cmd)
        .current_dir(cwd)
        .env_clear()
        .env("LC_ALL", "C");
    ok_records(c)
}

fn shells() -> Vec<&'static str> {
    let mut v = vec!["/bin/sh"];
    if Path::new("/bin/bash").exists() || Path::new("/usr/bin/bash").exists() {
        v.push("bash");
    }
    v
}

/// Plan a request, then run it as argv and through each shell; all must agree.
fn select(request: &str, cwd: &Path) -> Records {
    let plan = generator()
        .plan(request, T0)
        .unwrap_or_else(|e| panic!("{request}: {e}"));
    assert!(plan.command.ends_with("-print0"));
    let by_argv = run_argv(&plan.argv, cwd).unwrap_or_else(|e| panic!("{request}: argv: {e}"));
    for sh in shells() {
        let by_sh = run_shell(sh, &plan.command, cwd)
            .unwrap_or_else(|e| panic!("{request}: {sh}: {e}\n{}", plan.command));
        assert_eq!(
            by_argv, by_sh,
            "{request}: {sh} differs from argv\n{}",
            plan.command
        );
    }
    by_argv
}

fn rec(s: &str) -> Vec<u8> {
    s.as_bytes().to_vec()
}

fn sorted(mut v: Records) -> Records {
    v.sort();
    v
}

fn ns_supported(dir: &Path) -> bool {
    let p = dir.join("ns_probe");
    touch(&p, st(T0, 123));
    let ok = mtime_ns(&p) == i128::from(T0) * 1_000_000_000 + 123;
    fs::remove_file(&p).unwrap();
    ok
}

// ---------------------------------------------------------------- time ----

struct Row {
    label: &'static str,
    secs: i64,
    nanos: u32,
    needs_ns: bool,
}

fn rows() -> Vec<Row> {
    let r = |label, secs, nanos, needs_ns| Row {
        label,
        secs,
        nanos,
        needs_ns,
    };
    vec![
        r("c_minus_1s", C - 1, 0, false),
        r("c_minus_1ns", C - 1, 999_999_999, true),
        r("c_exact", C, 0, false),
        r("c_plus_1ns", C, 1, true),
        r("c_plus_1s", C + 1, 0, false),
        r("mid", C + WEEK / 2, 0, false),
        r("t0_minus_1s", T0 - 1, 0, false),
        r("t0_minus_1ns", T0 - 1, 999_999_999, true),
        r("t0_exact", T0, 0, false),
        r("t0_plus_1ns", T0, 1, true),
        r("t0_plus_1s", T0 + 1, 0, false),
        r("t0_plus_1day", T0 + 86_400, 0, false),
        r("t0_plus_10years", T0 + 10 * 365 * 86_400, 0, false),
        r("epoch0", 0, 0, false),
        r("pre1970", -1_000_000, 0, false),
    ]
}

#[test]
fn window_boundaries_match_two_independent_oracles() {
    let tree = TempDir::new().unwrap();
    let refs = TempDir::new().unwrap(); // outside the searched tree
    let ns_ok = ns_supported(tree.path());
    if !ns_ok {
        println!("SKIPPED(window_boundaries): nanosecond rows (file system keeps whole seconds)");
    }
    let mut graded: Vec<(String, i128)> = vec![];
    for row in rows() {
        if row.needs_ns && !ns_ok {
            continue;
        }
        let name = format!("{}.py", row.label);
        touch(&tree.path().join(&name), st(row.secs, row.nanos));
        graded.push((name, 0));
    }
    // Decoys: wrong extension and a directory, both inside the window.
    touch(&tree.path().join("mid_decoy.txt"), st(C + 10, 0));
    fs::create_dir(tree.path().join("dir.py")).unwrap();
    File::open(tree.path().join("dir.py"))
        .unwrap()
        .set_modified(st(C + 10, 0))
        .unwrap();

    // Read back every mtime before grading precision rows.
    for g in &mut graded {
        g.1 = mtime_ns(&tree.path().join(&g.0));
    }
    let row_by_name: std::collections::HashMap<_, _> = rows()
        .into_iter()
        .map(|r| (format!("{}.py", r.label), r))
        .collect();
    for (name, got) in &graded {
        let want = i128::from(row_by_name[name].secs) * 1_000_000_000
            + i128::from(row_by_name[name].nanos);
        if name == "pre1970.py" && *got != want {
            eprintln!("SKIPPED: pre-1970 row not representable here");
            continue;
        }
        assert_eq!(*got, want, "mtime read-back for {name}");
    }

    // Oracle 1: hand-written arithmetic over the read-back mtimes.
    let c_ns = i128::from(C) * 1_000_000_000;
    let t0_ns = i128::from(T0) * 1_000_000_000;
    let expected: Records = sorted(
        graded
            .iter()
            .filter(|(_, m)| c_ns < *m && *m <= t0_ns)
            .map(|(n, _)| rec(&format!("./{n}")))
            .collect(),
    );
    // Hand-written anchors for the rows that matter most.
    let must_in = ["c_plus_1s.py", "mid.py", "t0_minus_1s.py", "t0_exact.py"];
    let must_out = [
        "c_minus_1s.py",
        "c_exact.py",
        "t0_plus_1s.py",
        "t0_plus_1day.py",
        "t0_plus_10years.py",
        "epoch0.py",
    ];
    for n in must_in {
        assert!(expected.contains(&rec(&format!("./{n}"))), "{n} must be in");
    }
    for n in must_out {
        assert!(
            !expected.contains(&rec(&format!("./{n}"))),
            "{n} must be out"
        );
    }
    if ns_ok {
        assert!(expected.contains(&rec("./c_plus_1ns.py")));
        assert!(expected.contains(&rec("./t0_minus_1ns.py")));
        assert!(!expected.contains(&rec("./c_minus_1ns.py")));
        assert!(!expected.contains(&rec("./t0_plus_1ns.py")));
    }

    let got = select("find python files modified in the last 7 days", tree.path());
    assert_eq!(got, expected, "compiled command vs arithmetic oracle");

    // Oracle 2: a different spelling (-newer against reference files).
    let ref_c = refs.path().join("ref_c");
    let ref_t0 = refs.path().join("ref_t0");
    touch(&ref_c, st(C, 0));
    touch(&ref_t0, st(T0, 0));
    assert_eq!(mtime_ns(&ref_c), c_ns, "reference mtime read-back");
    assert_eq!(mtime_ns(&ref_t0), t0_ns, "reference mtime read-back");
    let mut o2 = Command::new("/usr/bin/find");
    o2.args(["-P", ".", "-type", "f", "-iname", "*.py", "-newer"])
        .arg(&ref_c)
        .arg("!")
        .arg("-newer")
        .arg(&ref_t0)
        .arg("-print0")
        .current_dir(tree.path());
    assert_eq!(
        ok_records(o2).unwrap(),
        got,
        "-newermt pair vs -newer oracle"
    );
}

#[test]
fn equivalent_durations_select_the_same_files() {
    let tree = TempDir::new().unwrap();
    touch(&tree.path().join("a.py"), st(T0 - 5 * 86_400, 0));
    touch(&tree.path().join("b.py"), st(T0 - 7 * 86_400 + 5, 0));
    touch(&tree.path().join("c.py"), st(T0 - 7 * 86_400 - 5, 0));
    let g = generator();
    let argv = |r: &str| g.plan(r, T0).unwrap().argv;
    assert_eq!(
        argv("python files modified in the last 6 days"),
        argv("python files modified in the last 144 hours")
    );
    assert_eq!(
        argv("python files modified in the last 144 hours"),
        argv("python files modified in the last 8640 minutes")
    );
    assert_eq!(
        argv("python files modified in the last 7 days"),
        argv("python files modified in the last 168 hours")
    );
    assert_eq!(
        select("python files modified in the last 7 days", tree.path()),
        vec![rec("./a.py"), rec("./b.py")]
    );
    assert_eq!(
        select("python files modified in the last 168 hours", tree.path()),
        vec![rec("./a.py"), rec("./b.py")]
    );
    // 6 days = 144 hours: only the 5-day-old file is inside.
    assert_eq!(
        select("python files modified in the last 6 days", tree.path()),
        vec![rec("./a.py")]
    );
}

#[test]
fn counts_10000_and_10001() {
    let g = generator();
    assert!(g
        .plan("files modified in the last 10000 minutes", T0)
        .is_ok());
    assert!(g
        .plan("files modified in the last 10001 minutes", T0)
        .is_err());
}

// ------------------------------------------------------ names and shells ----

#[test]
fn hostile_names_stay_data_through_every_layer() {
    let tree = TempDir::new().unwrap();
    let t = st(T0 - 100, 0);
    let names = [
        "plain.py",
        "with space.py",
        "it's.py",
        "dq\"uote.py",
        "dollar$HOME.py",
        "semi;colon.py",
        "back`tick`.py",
        "glob[a]*?.py",
        "-dash.py",
        "!bang.py",
        "ünï.py",
        ".hidden.py",
        ".py",
        "new\nline.py",
        "UPPER.PY",
    ];
    for n in names {
        touch(&tree.path().join(n), t);
    }
    touch(&tree.path().join("other.txt"), t);
    // symlinks are never listed
    symlink("plain.py", tree.path().join("link_to_file.py")).unwrap();
    symlink(".", tree.path().join("link_to_dir.py")).unwrap();
    symlink("missing", tree.path().join("dangling.py")).unwrap();
    symlink("loop.py", tree.path().join("loop.py")).unwrap();
    let want: Records = sorted(names.iter().map(|n| rec(&format!("./{n}"))).collect());
    let got = select("list all python files", tree.path());
    assert_eq!(got, want);
    // Explicit pattern is case-sensitive; the table word is not.
    assert!(!select("find *.py files", tree.path()).contains(&rec("./UPPER.PY")));
    assert!(got.contains(&rec("./UPPER.PY")));
    // A file named exactly ".py" is selected by '*.py'.
    assert!(got.contains(&rec("./.py")));
}

#[test]
fn quoted_literals_with_shell_syntax_are_exact_names_and_roots() {
    let tree = TempDir::new().unwrap();
    let t = st(T0 - 100, 0);
    let weird = [
        "a b;rm -rf x.txt",
        "$(id).txt",
        "`id`.txt",
        "q'uote.txt",
        "x*y.txtz",
    ];
    for n in &weird[..4] {
        touch(&tree.path().join(n), t);
    }
    touch(&tree.path().join("decoy.txt"), t);
    for n in &weird[..4] {
        let req = format!("find files named \"{n}\"");
        assert_eq!(
            select(&req, tree.path()),
            vec![rec(&format!("./{n}"))],
            "{req}"
        );
    }
    // roots whose names look like find expressions or shell syntax
    for d in ["-x", "!", "(", ")", ",", "a b", "$(id)", "x;y"] {
        let dir = tree.path().join(d);
        fs::create_dir(&dir).unwrap();
        touch(&dir.join("in.py"), t);
        let req = format!("list python files in \"{d}\"");
        assert_eq!(
            select(&req, tree.path()),
            vec![rec(&format!("./{d}/in.py"))],
            "{req}"
        );
    }
}

#[test]
fn name_decoys_and_other_roots_are_excluded() {
    let tree = TempDir::new().unwrap();
    fs::create_dir_all(tree.path().join("docs/sub")).unwrap();
    fs::create_dir_all(tree.path().join("src")).unwrap();
    let inw = st(T0 - 100, 0);
    let old = st(T0 - 2 * WEEK, 0);
    touch(&tree.path().join("docs/sub/a.py"), inw);
    touch(&tree.path().join("docs/old.py"), old);
    touch(&tree.path().join("docs/b.txt"), inw);
    touch(&tree.path().join("src/c.py"), inw);
    assert_eq!(
        select(
            "find python files in docs modified in the last 7 days",
            tree.path()
        ),
        vec![rec("./docs/sub/a.py")]
    );
    assert_eq!(
        select("find files named a.py in the docs folder", tree.path()),
        vec![rec("./docs/sub/a.py")]
    );
    assert_eq!(
        select("find files in ./src", tree.path()),
        vec![rec("./src/c.py")]
    );
}

#[test]
fn symlink_roots_and_absolute_roots() {
    let tree = TempDir::new().unwrap();
    let real = tree.path().join("real");
    fs::create_dir(&real).unwrap();
    touch(&real.join("a.py"), st(T0 - 10, 0));
    symlink("real", tree.path().join("link")).unwrap();
    // A root that is itself a symlink lists nothing (-P follows nothing).
    assert_eq!(
        select("list python files in \"link\"", tree.path()),
        Records::new()
    );
    // A symlink as an intermediate component is resolved by the OS.
    fs::create_dir(real.join("sub")).unwrap();
    touch(&real.join("sub/b.py"), st(T0 - 10, 0));
    assert_eq!(
        select("list python files in \"link/sub\"", tree.path()),
        vec![rec("./link/sub/b.py")]
    );
    // Absolute root: emitted unchanged.
    let abs = real.display().to_string();
    let got = select(&format!("list python files in \"{abs}\""), tree.path());
    assert_eq!(
        got,
        sorted(vec![
            rec(&format!("{abs}/a.py")),
            rec(&format!("{abs}/sub/b.py"))
        ])
    );
}

// --------------------------------------------------------------- errors ----

#[test]
fn traversal_errors_are_failures_not_partial_results() {
    let tree = TempDir::new().unwrap();
    let plan = generator()
        .plan("list python files in nonexistent", T0)
        .unwrap();
    let r = run_argv(&plan.argv, tree.path());
    assert!(r.is_err(), "a missing root must fail: {r:?}");
}

fn runner_bypasses_permissions() -> bool {
    fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("Uid:"))
                .map(|l| l.split_whitespace().nth(2) == Some("0"))
        })
        .unwrap_or(true)
}

#[test]
fn permission_denied_directory_is_a_failure() {
    use std::os::unix::fs::PermissionsExt;
    if runner_bypasses_permissions() {
        // Rust has no native skip: say so on both streams, and the receipt
        // run uses --nocapture and greps for this marker. Never a pass.
        println!("SKIPPED(permission_denied_directory): runner bypasses permissions (uid 0)");
        eprintln!("SKIPPED(permission_denied_directory): runner bypasses permissions (uid 0)");
        return;
    }
    let tree = TempDir::new().unwrap();
    touch(&tree.path().join("ok.py"), st(T0 - 10, 0));
    let denied = tree.path().join("denied");
    fs::create_dir(&denied).unwrap();
    touch(&denied.join("hidden.py"), st(T0 - 10, 0));
    fs::set_permissions(&denied, fs::Permissions::from_mode(0o000)).unwrap();
    let plan = generator().plan("list python files", T0).unwrap();
    let r = run_argv(&plan.argv, tree.path());
    fs::set_permissions(&denied, fs::Permissions::from_mode(0o755)).unwrap();
    assert!(r.is_err(), "partial results must not pass: {r:?}");
}

// ------------------------------------------------------------ empty ----

#[test]
fn no_match_is_a_clean_empty_result() {
    let tree = TempDir::new().unwrap();
    touch(&tree.path().join("a.txt"), st(T0 - 10, 0));
    assert_eq!(select("list all pdf files", tree.path()), Records::new());
}

#[allow(dead_code)]
fn _unused(_: PathBuf) {}
