// Dangerous command pattern database
// Comprehensive regex patterns for detecting unsafe shell commands

use once_cell::sync::Lazy;
use regex::Regex;

use crate::models::{RiskLevel, ShellType};

use super::DangerPattern;

/// Regex fragments shared by the #1054 patterns and the catastrophic floor.
///
/// Blanks between arguments of one statement: spaces, tabs and `\`-newline
/// continuations. A bare newline ends the statement, so it is not a blank.
pub(crate) const WS: &str = r"(?:[ \t]|\\\n)+";
/// One argument token of the same statement (stops at a bare separator).
pub(crate) const TOK: &str = r"[^\s;&|]+";
/// One option token (`-v`, `--silent`).
pub(crate) const OPT: &str = r"-[^\s;&|]*";
/// A recursive flag for chmod/chown.
pub(crate) const REC: &str = r"(?:-[a-zA-Z]*R[a-zA-Z]*|--recursive)";
/// A world-writable mode: octal with a write bit in the last digit, or a
/// symbolic mode that grants `w` to `o` or `a`.
pub(crate) const WW_MODE: &str =
    r"(?:0?[0-7]?[0-7][0-7][2367]|[ugoa]*[ao][ugoa]*[+=][rwxXst]*w[rwxXst]*)";
/// Root or home as a whole argument, optionally with `/*`. Root may be
/// quoted. `~` expands only unquoted and `$HOME` only unquoted or in double
/// quotes; `'$HOME'` and `"~"` are literal names.
pub(crate) const ROOT_OR_HOME: &str = r#"(?:['"]?/+\*?['"]?|~/?\*?|"?\$\{?HOME\}?/?\*?"?)"#;
/// The rest of a `find` call after the command word: optional `-H/-L/-P`,
/// a root/home start path, then `-delete` or an `-exec rm` that removes the
/// found path (`{}`), in the same statement (`\;` is allowed inside it).
/// The blanks are [`WS`], spelled out because a `const` cannot `format!`.
pub(crate) const FIND_ROOT_OR_HOME_DELETE: &str = r#"(?:-[HLP](?:[ \t]|\\\n)+)*(?:['"]?/+['"]?|~/?|"?\$\{?HOME\}?/?"?)(?:(?:[ \t]|\\\n)+(?:[^\s;&|]|\\;)+)*?(?:[ \t]|\\\n)+(?:-delete\b|-exec(?:dir)?(?:[ \t]|\\\n)+(?:sudo(?:[ \t]|\\\n)+)?rm\b[^;&|\n]*?\{\})"#;
/// The same, but only for a find with no filter: between the start path and
/// the action only non-narrowing options may appear (`-xdev`, `-mindepth N`,
/// `-type X`, `-name '*'`, ...). This one is catastrophic and in the floor; a
/// filtered find on root or home deletes only what matches and is High.
pub(crate) const FIND_ROOT_OR_HOME_DELETE_ALL: &str = r#"(?:-[HLP](?:[ \t]|\\\n)+)*(?:['"]?/+['"]?|~/?|"?\$\{?HOME\}?/?"?)(?:(?:[ \t]|\\\n)+(?:-xdev|-mount|-depth|-follow|-noleaf|-ignore_readdir_race|-mindepth(?:[ \t]|\\\n)+\d+|-type(?:[ \t]|\\\n)+[a-zA-Z]|-i?name(?:[ \t]|\\\n)+['"]?\*['"]?))*(?:[ \t]|\\\n)+(?:-delete\b|-exec(?:dir)?(?:[ \t]|\\\n)+(?:sudo(?:[ \t]|\\\n)+)?rm\b[^;&|\n]*?\{\})"#;

/// Built-in dangerous patterns loaded once at startup
pub static DANGEROUS_PATTERNS: Lazy<Vec<DangerPattern>> = Lazy::new(|| {
    vec![
        // CRITICAL: Filesystem destruction
        DangerPattern {
            // Accept any combination of short (`-rf`, `-fr`, `-r -f`) and GNU
            // long (`--recursive`, `--force`, `--no-preserve-root`) flag tokens
            // between `rm` and the target. The prior `(-[rfRF]*\s+)*` group only
            // matched short flags, so `rm --recursive --force /` slipped past the
            // Critical gate and could be re-enabled by a broad allowlist (#1246).
            pattern: r"rm\s+(-{1,2}\S+\s+)*(/|~|\$HOME|/\*|~/\*|\*|\.\.?/?|\.\./\*|\.\*)"
                .to_string(),
            risk_level: RiskLevel::Critical,
            description: "Recursive deletion of root, home, current, or parent directory"
                .to_string(),
            shell_specific: None,
        },
        DangerPattern {
            pattern: r"rm\s+-rf\s+/".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Force recursive deletion from root".to_string(),
            shell_specific: None,
        },
        DangerPattern {
            pattern: r"rm\s+-rf\s+--no-preserve-root\s+/".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Bypass root protection and delete everything".to_string(),
            shell_specific: None,
        },
        // CRITICAL: Disk operations
        DangerPattern {
            pattern: r"dd\s+.*if=/dev/(zero|random|urandom).*of=/dev/(sd|hd|nvme)".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Overwrite disk with random data".to_string(),
            shell_specific: None,
        },
        DangerPattern {
            pattern: r"dd\s+.*of=/dev/(sd|hd|nvme).*if=/dev/(zero|random|urandom)".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Overwrite disk with random data (reverse arg order)".to_string(),
            shell_specific: None,
        },
        DangerPattern {
            pattern: r"mkfs\.\w+\s+(?:\S+\s+)*?/dev/(sd|hd|nvme)".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Format disk destroying all data".to_string(),
            shell_specific: None,
        },
        // CRITICAL: Fork bombs
        DangerPattern {
            pattern: r":\(\)\s*\{\s*:\s*\|\s*:\s*&\s*\}\s*;\s*:".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Fork bomb - exponential process creation".to_string(),
            shell_specific: Some(ShellType::Bash),
        },
        DangerPattern {
            pattern: r"\|\s*&\s*\|".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Potential fork bomb pattern".to_string(),
            shell_specific: None,
        },
        // HIGH: System directory operations
        DangerPattern {
            pattern: r"(rm|mv|chmod|chown)\s+.*(/bin|/sbin|/usr/bin|/usr/sbin|/etc)".to_string(),
            risk_level: RiskLevel::High,
            description: "Modification of critical system directories".to_string(),
            shell_specific: None,
        },
        DangerPattern {
            // Match `chmod 777 /` with any combination of intervening flag
            // tokens (e.g. `-R`, `-Rfv`, `--recursive`). The optional
            // `(?:\s+-{1,2}\S+)*` group is what closes caro#1034: the prior
            // pattern only matched the bare `chmod 777 /` form and silently
            // let the recursive variant pass under --safety strict.
            pattern: r"chmod(?:\s+-{1,2}\S+)*\s+0?777\s+/".to_string(),
            risk_level: RiskLevel::High,
            description: "Permission change making root world-writable (recursive or direct)"
                .to_string(),
            shell_specific: None,
        },
        DangerPattern {
            pattern: r"chmod\s+-[rR]\s+[0-7]{3,4}\s+/".to_string(),
            risk_level: RiskLevel::High,
            description: "Recursive chmod on root directory".to_string(),
            shell_specific: None,
        },
        // CRITICAL: recursive world-writable chmod on root or home (#1054).
        // Both flag orders: `chmod -R 777 ~` and `chmod 777 -R /`. A mode is
        // world-writable when its last octal digit has the write bit (2, 3,
        // 6, 7), or a symbolic mode grants `w` to `o` or `a`.
        DangerPattern {
            pattern: format!(
                r"\bchmod{WS}(?:{OPT}{WS})*?{REC}{WS}(?:{OPT}{WS})*?{WW_MODE}{WS}(?:{TOK}{WS})*?{ROOT_OR_HOME}(?:\s|$)"
            ),
            risk_level: RiskLevel::Critical,
            description: "Recursive world-writable permission change on root or home".to_string(),
            shell_specific: None,
        },
        DangerPattern {
            pattern: format!(
                r"\bchmod{WS}(?:{OPT}{WS})*?{WW_MODE}{WS}(?:{TOK}{WS})*?{REC}{WS}(?:{TOK}{WS})*?{ROOT_OR_HOME}(?:\s|$)"
            ),
            risk_level: RiskLevel::Critical,
            description: "World-writable permission change applied recursively to root or home"
                .to_string(),
            shell_specific: None,
        },
        // CRITICAL: find-based recursive delete from root or home (#1054).
        DangerPattern {
            pattern: format!(r"\bfind{WS}{FIND_ROOT_OR_HOME_DELETE_ALL}"),
            risk_level: RiskLevel::Critical,
            description: "Recursive delete with find from root or home".to_string(),
            shell_specific: None,
        },
        // HIGH: the same with a filter (`-name`, `-mtime`, ...). It deletes only
        // what matches, so it asks, and a team may allowlist its own cleanup.
        DangerPattern {
            pattern: format!(r"\bfind{WS}{FIND_ROOT_OR_HOME_DELETE}"),
            risk_level: RiskLevel::High,
            description: "Filtered recursive delete with find from root or home".to_string(),
            shell_specific: None,
        },
        // CRITICAL: decoded payload piped into a shell (#1054).
        DangerPattern {
            pattern: r"\bbase64\b[^|;&\n]*\s(?:-d|-D|--decode)\b[^|;&\n]*\|\s*(?:(?:sudo|doas|env|command|exec|nohup)\s+(?:-[ugCD]\s+\S+\s+|-\S+\s+|[A-Za-z_]\w*=\S*\s+)*)*(?:\S*/)?(?:ba|z|da|k|mk|fi)?sh\b"
                .to_string(),
            risk_level: RiskLevel::Critical,
            description: "Decoded base64 payload piped into a shell".to_string(),
            shell_specific: None,
        },
        // HIGH: eval of hidden input (#1054). A variable, or a substitution
        // from the network or a decoder, runs code nobody reviewed.
        DangerPattern {
            pattern: r#"\beval\s+['"]?\$\{?[A-Za-z_]"#.to_string(),
            risk_level: RiskLevel::High,
            description: "eval of a variable runs hidden code".to_string(),
            shell_specific: None,
        },
        DangerPattern {
            pattern: r#"\beval\s+['"]?(?:\$\(|`)[^)`]*\b(?:curl|wget|base64|xxd|openssl|nc)\b"#
                .to_string(),
            risk_level: RiskLevel::High,
            description: "eval of downloaded or decoded content".to_string(),
            shell_specific: None,
        },
        // MODERATE: eval of any other substitution. Common in shell setup
        // (`eval "$(ssh-agent -s)"`), so flagged but not raised to High.
        DangerPattern {
            pattern: r#"\beval\s+['"]?(?:\$\(|`)"#.to_string(),
            risk_level: RiskLevel::Moderate,
            description: "eval of command output".to_string(),
            shell_specific: None,
        },
        // HIGH: interpreter one-liner deleting root, home or a system dir (#1054).
        DangerPattern {
            pattern: r#"\b(?:python[0-9.]*|perl|ruby)\s+(?:-\S+\s+)*?-[a-zA-Z]*[ce]\b[^\n]*?\b(?:rmtree|rm_rf|rm_r|remove_tree)\s*\(\s*['"](?:/|/(?:home|etc|usr|var|bin|sbin|lib|lib64|boot|opt|root|srv|Users|System|Library))/?['"]"#
                .to_string(),
            risk_level: RiskLevel::High,
            description: "Interpreter one-liner deleting root, home or a system directory"
                .to_string(),
            shell_specific: None,
        },
        DangerPattern {
            // Recursive chmod that gives "others" write access, on any path
            // (caro#1592). A world-writable mode is an octal mode whose last
            // digit has the write bit (2, 3, 6, 7), with or without GNU's `+`
            // or `=` operator (`+777`), or a symbolic clause such as `o+w`,
            // `a+rwx` or `o=rwx`. The mode is the first argument
            // that is not an option. `-R` / `--recursive` may come among the
            // options before it, or later in the same command. Spans use
            // `[ \t]` and stop at `;`, `&`, `|` and newlines, so the match
            // stays inside one command.
            pattern: concat!(
                r"\bchmod(?:[ \t]+-\S+)*[ \t]+(?:",
                // a recursive flag among the options, then the mode
                r"(?:-[a-zA-Z]*R[a-zA-Z]*|--recursive)(?:[ \t]+-\S+)*[ \t]+",
                r"(?:[+=]?0?[0-7]{0,3}[2367]|[ugoa+=,rwxXst-]*[oa][ugoa]*[+=][rwxXst]*w[ugoa+=,rwxXst-]*)",
                r"|",
                // the mode, then a recursive flag later in the command
                r"(?:[+=]?0?[0-7]{0,3}[2367]|[ugoa+=,rwxXst-]*[oa][ugoa]*[+=][rwxXst]*w[ugoa+=,rwxXst-]*)",
                r"[ \t](?:[^;&|\n]*[ \t])?(?:-[a-zA-Z]*R[a-zA-Z]*|--recursive)",
                r")(?:\s|$)",
            )
            .to_string(),
            risk_level: RiskLevel::High,
            description: "Recursive chmod that makes files world-writable".to_string(),
            shell_specific: None,
        },
        // HIGH: Privilege escalation
        DangerPattern {
            pattern: r"sudo\s+su\s*$".to_string(),
            risk_level: RiskLevel::High,
            description: "Switch to root user without specific command".to_string(),
            shell_specific: None,
        },
        DangerPattern {
            pattern: r"sudo\s+.*chmod\s+u\+s".to_string(),
            risk_level: RiskLevel::High,
            description: "Add setuid bit with elevated privileges".to_string(),
            shell_specific: None,
        },
        // HIGH: Download and execute
        DangerPattern {
            pattern: r"(curl|wget)\s+.*\|\s*(bash|sh|zsh|fish)".to_string(),
            risk_level: RiskLevel::High,
            description: "Download and execute remote script without inspection".to_string(),
            shell_specific: None,
        },
        DangerPattern {
            pattern: r"(curl|wget)\s+.*\|\s*sudo\s+(bash|sh)".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Download and execute remote script with root privileges".to_string(),
            shell_specific: None,
        },
        // MODERATE: Package management
        DangerPattern {
            pattern: r"(apt|yum|dnf)\s+remove\s+.*--force".to_string(),
            risk_level: RiskLevel::Moderate,
            description: "Force removal of packages bypassing dependencies".to_string(),
            shell_specific: None,
        },
        DangerPattern {
            pattern: r"pip\s+install\s+.*--break-system-packages".to_string(),
            risk_level: RiskLevel::Moderate,
            description: "Install Python packages bypassing system protections".to_string(),
            shell_specific: None,
        },
        // MODERATE: Process manipulation
        DangerPattern {
            pattern: r"kill\s+-9\s+(-1|1)\s*$".to_string(),
            risk_level: RiskLevel::Moderate,
            description: "Force kill all processes or init".to_string(),
            shell_specific: None,
        },
        DangerPattern {
            pattern: r"killall\s+-9\s+\w+".to_string(),
            risk_level: RiskLevel::Moderate,
            description: "Force kill all processes by name".to_string(),
            shell_specific: None,
        },
        // HIGH: Destructive data operations
        DangerPattern {
            pattern: r">\s*/dev/sd[a-z]".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Direct write to disk device".to_string(),
            shell_specific: None,
        },
        DangerPattern {
            pattern: r"shred\s+-[uvz]*\s+/dev/(sd|hd|nvme)".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Securely delete disk device".to_string(),
            shell_specific: None,
        },
        // MODERATE: Network operations
        DangerPattern {
            pattern: r"iptables\s+-F".to_string(),
            risk_level: RiskLevel::Moderate,
            description: "Flush all firewall rules".to_string(),
            shell_specific: None,
        },
        DangerPattern {
            pattern: r"ufw\s+disable".to_string(),
            risk_level: RiskLevel::Moderate,
            description: "Disable firewall".to_string(),
            shell_specific: None,
        },
        // HIGH: Sudo with system modifications
        DangerPattern {
            pattern: r"sudo\s+(systemctl|service)\s+(restart|stop|disable)".to_string(),
            risk_level: RiskLevel::High,
            description: "Modify system services with elevated privileges".to_string(),
            shell_specific: None,
        },
        DangerPattern {
            pattern: r"sudo\s+rm\s".to_string(),
            risk_level: RiskLevel::High,
            description: "Delete files with elevated privileges".to_string(),
            shell_specific: None,
        },
        // HIGH: System file modification
        DangerPattern {
            pattern: r">\s*/etc/".to_string(),
            risk_level: RiskLevel::High,
            description: "Redirect output to system configuration file".to_string(),
            shell_specific: None,
        },
        DangerPattern {
            pattern: r"(echo|cat|printf)\s+.*>\s*/etc/".to_string(),
            risk_level: RiskLevel::High,
            description: "Write to system configuration directory".to_string(),
            shell_specific: None,
        },
        // CRITICAL: Windows path deletion
        DangerPattern {
            pattern: r"rm\s+-r[f]*\s+[A-Z]:\\".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Recursive deletion of Windows drive root (Bash/WSL)".to_string(),
            shell_specific: Some(ShellType::Bash), // When running on WSL
        },
        DangerPattern {
            pattern: r"Remove-Item\s+-Recurse\s+-Force\s+[A-Z]:\\".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Recursive deletion of Windows drive root (PowerShell)".to_string(),
            shell_specific: Some(ShellType::PowerShell),
        },
        DangerPattern {
            pattern: r"Remove-Item\s+(\*|\*\.\*)\s+(.*-Force.*-Recurse|.*-Recurse.*-Force)"
                .to_string(),
            risk_level: RiskLevel::Critical,
            description: "PowerShell recursive deletion of current directory wildcard".to_string(),
            shell_specific: Some(ShellType::PowerShell),
        },
        // HIGH: PowerShell dangerous operations
        DangerPattern {
            pattern: r"Remove-Item\s+.*-Force\s+-Recurse".to_string(),
            risk_level: RiskLevel::High,
            description: "Force recursive deletion in PowerShell".to_string(),
            shell_specific: Some(ShellType::PowerShell),
        },
        DangerPattern {
            pattern: r"Set-ExecutionPolicy\s+Unrestricted".to_string(),
            risk_level: RiskLevel::High,
            description: "Disable PowerShell execution policy protection".to_string(),
            shell_specific: Some(ShellType::PowerShell),
        },
        // MODERATE: Environment manipulation
        DangerPattern {
            pattern: r"export\s+PATH=".to_string(),
            risk_level: RiskLevel::Moderate,
            description: "Modify PATH environment variable".to_string(),
            shell_specific: None,
        },
        DangerPattern {
            pattern: r"alias\s+(rm|mv|cp)=".to_string(),
            risk_level: RiskLevel::Moderate,
            description: "Override critical command with alias".to_string(),
            shell_specific: None,
        },
        // HIGH: Network backdoors
        DangerPattern {
            pattern: r"nc\s+.*-[a-z]*l[a-z]*\s+.*-[a-z]*e".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Netcat bind shell - creates network backdoor".to_string(),
            shell_specific: None,
        },
        DangerPattern {
            pattern: r"nc\s+-[a-z]*e\s+/bin/(ba)?sh".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Netcat shell binding".to_string(),
            shell_specific: None,
        },
        // HIGH: Cron job manipulation
        DangerPattern {
            pattern: r"crontab\s+-r".to_string(),
            risk_level: RiskLevel::High,
            description: "Remove all cron jobs".to_string(),
            shell_specific: None,
        },
        DangerPattern {
            pattern: r"\(crontab\s+-l.*;\s*echo".to_string(),
            risk_level: RiskLevel::High,
            description: "Add malicious cron job".to_string(),
            shell_specific: None,
        },
        // HIGH: Python/Perl exec with dangerous commands
        DangerPattern {
            pattern: r"python[0-9.]*\s+-c\s+.*os\.system.*rm\s+-rf".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Python executing recursive deletion".to_string(),
            shell_specific: None,
        },
        DangerPattern {
            pattern: r"(python[0-9.]*|perl|ruby)\s+-[ec]\s+.*system\s*\(".to_string(),
            risk_level: RiskLevel::High,
            description: "Script language executing shell commands".to_string(),
            shell_specific: None,
        },
        // MODERATE: rm without -rf but still potentially dangerous
        DangerPattern {
            pattern: r"rm\s+[^-\s][^\s]*\.(txt|doc|pdf|xlsx|pptx|zip|tar|sql|bak)".to_string(),
            risk_level: RiskLevel::Moderate,
            description: "Delete important file types".to_string(),
            shell_specific: None,
        },
        // Fix Windows backslash pattern - works in Bash/PowerShell/WSL
        DangerPattern {
            pattern: r"rm\s+-r[f]*\s+[A-Z]:[/\\]".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Recursive deletion of Windows drive root (with backslash)".to_string(),
            shell_specific: None, // Works across shells
        },
        // MODERATE: Borderline commands - changing permissions
        DangerPattern {
            pattern: r"chmod\s+[+\-]x\s+".to_string(),
            risk_level: RiskLevel::Moderate,
            description: "Make files executable".to_string(),
            shell_specific: None,
        },
        DangerPattern {
            pattern: r"chmod\s+[0-7]{3,4}\s+".to_string(),
            risk_level: RiskLevel::Moderate,
            description: "Change file permissions".to_string(),
            shell_specific: None,
        },
        DangerPattern {
            pattern: r"chown\s+[^\s]+\s+".to_string(),
            risk_level: RiskLevel::Moderate,
            description: "Change file ownership".to_string(),
            shell_specific: None,
        },
        // MODERATE: Installing packages with user scope
        DangerPattern {
            pattern: r"pip\s+install\s+--user".to_string(),
            risk_level: RiskLevel::Moderate,
            description: "Install Python packages in user directory".to_string(),
            shell_specific: None,
        },
        // CRITICAL: Windows del command with dangerous flags
        DangerPattern {
            pattern: r"del\s+/[fFsS]\s+".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Windows delete with force/subdirectory flags".to_string(),
            shell_specific: None, // Works in both Cmd and PowerShell
        },
        DangerPattern {
            pattern: r"del\s+.*C:[/\\]".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Windows delete on C drive root".to_string(),
            shell_specific: None, // Works in both Cmd and PowerShell
        },
        // CRITICAL: Windows format command
        DangerPattern {
            pattern: r"format\s+[A-Z]:".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Format disk drive".to_string(),
            shell_specific: None,
        },
        // MODERATE: Docker privileged mode
        DangerPattern {
            pattern: r"docker\s+run\s+.*--privileged".to_string(),
            risk_level: RiskLevel::High,
            description: "Docker container with full host access".to_string(),
            shell_specific: None,
        },
        // MODERATE: Remote access commands
        DangerPattern {
            pattern: r"ssh\s+[^\s]+@[^\s]+".to_string(),
            risk_level: RiskLevel::Moderate,
            description: "SSH connection to remote server".to_string(),
            shell_specific: None,
        },
        DangerPattern {
            pattern: r"scp\s+".to_string(),
            risk_level: RiskLevel::Moderate,
            description: "Secure copy to/from remote server".to_string(),
            shell_specific: None,
        },
        // MODERATE: Force kill specific process
        DangerPattern {
            pattern: r"kill\s+-9\s+\d+".to_string(),
            risk_level: RiskLevel::Moderate,
            description: "Force kill specific process by PID".to_string(),
            shell_specific: None,
        },
        // ──────────────────────────────────────────────────────────────────
        // BSD-family destructive commands (FreeBSD/OpenBSD/NetBSD/macOS)
        //
        // The original 52-pattern set targeted GNU/Linux conventions. The BSD
        // userland ships destructive utilities with no Linux counterpart and
        // BSD-flavored device names (/dev/da*, /dev/ada*, /dev/nvd*, /dev/md*)
        // that the existing /dev/(sd|hd|nvme) patterns do not cover.
        //
        // Inspired by the FreeBSD Device Driver Book ch. 31 (Security Best
        // Practices): every command crossing a privilege boundary must be
        // assumed hostile, regardless of its UNIX dialect.
        // ──────────────────────────────────────────────────────────────────

        // CRITICAL: FreeBSD partition table destruction
        DangerPattern {
            pattern: r"gpart\s+(destroy|delete)\b".to_string(),
            risk_level: RiskLevel::Critical,
            description: "FreeBSD partition table destruction (gpart destroy/delete)".to_string(),
            shell_specific: None,
        },
        // CRITICAL: ZFS recursive/forced dataset destruction
        DangerPattern {
            pattern: r"zfs\s+destroy\s+-[a-zA-Z]*[rRf][a-zA-Z]*".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Recursive or forced ZFS dataset destruction".to_string(),
            shell_specific: None,
        },
        // CRITICAL: dd to BSD-named disk devices (forward arg order)
        DangerPattern {
            pattern: r"dd\s+.*if=/dev/(zero|random|urandom).*of=/dev/(da|ada|nvd|md)\d".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Overwrite BSD disk device with random/zero data".to_string(),
            shell_specific: None,
        },
        // CRITICAL: dd to BSD-named disk devices (reverse arg order)
        DangerPattern {
            pattern: r"dd\s+.*of=/dev/(da|ada|nvd|md)\d.*if=/dev/(zero|random|urandom)".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Overwrite BSD disk device with random/zero data (reverse args)"
                .to_string(),
            shell_specific: None,
        },
        // CRITICAL: mkfs/newfs targeting BSD-named devices.
        // The `(?:\S+\s+)*?` allows flags AND flag-values between the command
        // and the device path (e.g. `newfs -U /dev/ada0p1`,
        // `newfs -L mylabel /dev/da0`). Lazy quantifier prevents the
        // `\S+\s+` group from greedily consuming the device path itself.
        DangerPattern {
            pattern: r"(mkfs\.\w+|newfs)\s+(?:\S+\s+)*?/dev/(da|ada|nvd|md)\d".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Format BSD disk device destroying all data".to_string(),
            shell_specific: None,
        },
        // CRITICAL: Direct write redirect to BSD-named disk devices
        DangerPattern {
            pattern: r">\s*/dev/(da|ada|nvd|md)\d".to_string(),
            risk_level: RiskLevel::Critical,
            description: "Direct write to BSD disk device".to_string(),
            shell_specific: None,
        },
        // HIGH: FreeBSD forced package removal
        DangerPattern {
            pattern: r"pkg\s+delete\s+-[a-zA-Z]*f[a-zA-Z]*".to_string(),
            risk_level: RiskLevel::High,
            description:
                "FreeBSD forced package removal (pkg delete -f) bypasses dependency checks"
                    .to_string(),
            shell_specific: None,
        },
        // HIGH: FreeBSD automated installer invoked outside install media.
        // Anchored to start-of-statement so `man bsdinstall` / `which bsdinstall`
        // are safe. The `^\s*` prefix accepts leading whitespace; `[;&|]+` accepts
        // `&&` and `||` separators (post-fix from safety-pattern-auditor).
        DangerPattern {
            pattern: r"(^\s*|[;&|]+\s*)(sudo\s+)?bsdinstall\b".to_string(),
            risk_level: RiskLevel::High,
            description:
                "FreeBSD automated installer (bsdinstall) — destructive outside install media"
                    .to_string(),
            shell_specific: None,
        },
        // HIGH: chflags removing immutability on system files (security regression).
        // Only `noschg` is matched — `chflags schg /etc/...` is legitimate hardening
        // (sets the system-immutable flag) and must not be blocked.
        DangerPattern {
            pattern: r"chflags\s+noschg\s+/(etc|bin|sbin|boot|usr/bin|usr/sbin)\b".to_string(),
            risk_level: RiskLevel::High,
            description: "BSD chflags removing immutability on system files — security regression"
                .to_string(),
            shell_specific: None,
        },
        // MODERATE: FreeBSD jail removal
        DangerPattern {
            pattern: r"jail\s+-r\s+\w+".to_string(),
            risk_level: RiskLevel::Moderate,
            description: "FreeBSD jail removal (jail -r) terminates running container".to_string(),
            shell_specific: None,
        },
        // ──────────────────────────────────────────────────────────────────
        // BSD-family destructive commands — round 2.
        // Surfaced by the post-fix safety-pattern-auditor pass on PR #1005.
        // The first round caught dataset-level destruction (`zfs destroy -r`)
        // but missed pool-level, boot-environment, GEOM-mirror, and
        // disklabel-rewrite destruction. These add the missing breadth.
        // ──────────────────────────────────────────────────────────────────

        // CRITICAL: ZFS pool destruction or label clearing.
        // Worse than `zfs destroy <dataset>` — wipes the entire pool.
        DangerPattern {
            pattern: r"zpool\s+(destroy|labelclear)\s+\S+".to_string(),
            risk_level: RiskLevel::Critical,
            description: "ZFS pool destruction or label clearing — wipes entire pool".to_string(),
            shell_specific: None,
        },
        // HIGH: FreeBSD Boot Environment destruction.
        // Removes a snapshot-based boot env, foreclosing rollback recovery.
        DangerPattern {
            pattern: r"bectl\s+destroy\s+\S+".to_string(),
            risk_level: RiskLevel::High,
            description: "FreeBSD Boot Environment destruction (bectl destroy)".to_string(),
            shell_specific: None,
        },
        // CRITICAL: GEOM mirror destruction or metadata clearing.
        DangerPattern {
            pattern: r"gmirror\s+(destroy|clear)\s+\w+".to_string(),
            risk_level: RiskLevel::Critical,
            description: "FreeBSD GEOM mirror destruction or metadata wipe".to_string(),
            shell_specific: None,
        },
        // CRITICAL: Disklabel rewrite (FreeBSD bsdlabel + OpenBSD disklabel).
        // The `(?:\S+\s+)*?` allows flags AND device path between the
        // command and the `-w` rewrite flag (e.g. `bsdlabel -B -w -r /dev/da0`).
        DangerPattern {
            pattern: r"(bsdlabel|disklabel)\s+(?:\S+\s+)*?-w\b".to_string(),
            risk_level: RiskLevel::Critical,
            description: "BSD disklabel rewrite (bsdlabel/disklabel -w) destroys partition table"
                .to_string(),
            shell_specific: None,
        },
    ]
});

/// Compile all patterns into regex objects at initialization
/// Returns errors for any patterns that fail to compile
pub fn validate_patterns() -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    for pattern in DANGEROUS_PATTERNS.iter() {
        if let Err(e) = Regex::new(&pattern.pattern) {
            errors.push(format!(
                "Pattern '{}' failed to compile: {}",
                pattern.pattern, e
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Get patterns filtered by shell type
pub fn get_patterns_for_shell(shell: ShellType) -> Vec<&'static DangerPattern> {
    DANGEROUS_PATTERNS
        .iter()
        .filter(|p| p.shell_specific.is_none() || p.shell_specific == Some(shell))
        .collect()
}

/// Get patterns filtered by minimum risk level
pub fn get_patterns_by_risk(min_risk: RiskLevel) -> Vec<&'static DangerPattern> {
    DANGEROUS_PATTERNS
        .iter()
        .filter(|p| p.risk_level >= min_risk)
        .collect()
}

/// Type alias for compiled pattern tuple
type CompiledPattern = (Regex, RiskLevel, String, Option<ShellType>);

/// Compiled regex patterns for performance (cached at startup)
pub static COMPILED_PATTERNS: Lazy<Vec<CompiledPattern>> = Lazy::new(|| {
    DANGEROUS_PATTERNS
        .iter()
        .filter_map(|pattern| {
            Regex::new(&pattern.pattern).ok().map(|regex| {
                (
                    regex,
                    pattern.risk_level,
                    pattern.description.clone(),
                    pattern.shell_specific,
                )
            })
        })
        .collect()
});

/// Get compiled patterns for a specific shell type
pub fn get_compiled_patterns_for_shell(
    shell: ShellType,
) -> Vec<&'static (Regex, RiskLevel, String, Option<ShellType>)> {
    COMPILED_PATTERNS
        .iter()
        .filter(|(_, _, _, shell_specific)| {
            shell_specific.is_none() || *shell_specific == Some(shell)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_patterns_compile() {
        assert!(validate_patterns().is_ok(), "All patterns should compile");
    }

    #[test]
    fn test_pattern_count() {
        assert!(
            DANGEROUS_PATTERNS.len() >= 30,
            "Should have at least 30 dangerous patterns"
        );
    }

    #[test]
    fn test_shell_specific_filtering() {
        let bash_patterns = get_patterns_for_shell(ShellType::Bash);
        let all_patterns = DANGEROUS_PATTERNS.len();
        assert!(bash_patterns.len() <= all_patterns);
    }

    #[test]
    fn test_risk_filtering() {
        let critical = get_patterns_by_risk(RiskLevel::Critical);
        let high = get_patterns_by_risk(RiskLevel::High);
        let moderate = get_patterns_by_risk(RiskLevel::Moderate);

        assert!(critical.len() <= high.len());
        assert!(high.len() <= moderate.len());
    }

    #[test]
    fn test_critical_patterns_exist() {
        let critical = get_patterns_by_risk(RiskLevel::Critical);
        assert!(!critical.is_empty(), "Should have critical risk patterns");
    }

    /// Returns true when at least one built-in pattern of risk High or
    /// Critical matches `command`. Strict safety mode blocks anything at
    /// or above High, so this mirrors the gate that user-facing strict
    /// mode applies.
    fn matches_high_or_critical(command: &str) -> bool {
        DANGEROUS_PATTERNS.iter().any(|p| {
            p.risk_level >= RiskLevel::High
                && regex::Regex::new(&p.pattern)
                    .map(|r| r.is_match(command))
                    .unwrap_or(false)
        })
    }

    /// Regression test for caro#1034: `chmod -R 777 /` and recursive
    /// variants must be blocked under strict mode (High or Critical risk).
    /// Prior to the fix, the pattern at line 75 only matched literal
    /// `chmod 777 /` with no flags, so the recursive form silently passed.
    #[test]
    fn test_chmod_recursive_root_blocked_strict() {
        let must_block = [
            "chmod 777 /",             // existing behavior (regression-protect)
            "chmod -R 777 /",          // canonical bypass from #1034
            "chmod -Rfv 777 /",        // combined short flags
            "chmod --recursive 777 /", // GNU long flag
            "chmod -R 0777 /",         // 4-digit octal, recursive
        ];

        for cmd in must_block {
            assert!(
                matches_high_or_critical(cmd),
                "command {cmd:?} must match at least one High-or-Critical pattern \
                 (would be blocked by --safety strict)"
            );
        }
    }

    /// Regression test for caro#1592: a recursive chmod that makes a tree
    /// world-writable is High risk on any path, not only on `/`. The
    /// landing page's DevOps scenario shows `chmod -R 777 /var/www`.
    #[test]
    fn test_chmod_recursive_world_writable_is_high() {
        let must_flag = [
            "chmod -R 777 /var/www",           // the landing-page scenario
            "sudo chmod -R 777 /var/www/html", // with sudo
            "chmod -Rv 0777 ./uploads",        // relative path, 4-digit octal
            "chmod --recursive 777 storage",   // GNU long flag
            "chmod -R 777 folder/",            // /docs/safety lists this as High
            "chmod 777 -R /srv/app",           // flag after the mode
            "chmod -R 666 data",               // world-writable without execute
            "chmod -R a+rwx /var/www",         // symbolic, all
            "chmod -R o+w public",             // symbolic, others only
            "chmod -R go+w shared",            // symbolic, group and others
            "chmod -R u=rwx,o=rwx public",     // comma-separated clauses
            "cd /srv && chmod -R 777 app",     // inside a command list
            "chmod -v -R 777 app",             // another option first
            "chmod -R -- 777 app",             // end-of-options marker
            "chmod -R +777 uploads",           // GNU operator numeric mode
            "chmod -R =777 uploads",           // GNU operator numeric mode
            "chmod -R 2 shared",               // short octal: others write
        ];
        for cmd in must_flag {
            assert!(
                matches_high_or_critical(cmd),
                "command {cmd:?} makes a tree world-writable and must be High or Critical"
            );
        }

        // Not world-writable, or not recursive: the new pattern must not
        // raise these. Relative paths keep the older root-path patterns out
        // of the result.
        let must_not_flag = [
            "chmod -R 755 public",                      // the landing page's own suggestion
            "chmod -R 775 shared",                      // group-writable only
            "chmod -R 750 app",                         // no access for others
            "chmod -R o-w public",                      // removes world write
            "chmod -R u+w src",                         // owner only
            "chmod -R g+w shared",                      // group only
            "chmod 777 notes.txt",                      // not recursive (stays Moderate)
            "chmod +x deploy.sh",                       // execute bit only
            "chmod -R 755 a; ls 777",                   // `777` is in another command
            "chmod -R 755 o+w",                         // `o+w` is a file name, not the mode
            "chmod 777 notes.txt\nchmod -R 755 public", // two commands, two lines
            "chmod -R 755 backup 777",                  // `777` is a file name
            "chmod -R -002 public",                     // removes others' write
        ];
        for cmd in must_not_flag {
            assert!(
                !matches_high_or_critical(cmd),
                "command {cmd:?} does not make a tree world-writable and must stay below High"
            );
        }
    }
}
