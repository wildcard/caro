use std::io::Write as _;
use std::time::Instant;
use vte::{Params, Parser, Perform};

#[derive(Debug, Clone, Copy, PartialEq)]
enum BlockEvent {
    PromptStart,
    PromptEnd,
    CommandStart,
    CommandEnd { exit_code: Option<i32> },
}

#[derive(Default)]
struct Screen {
    rows: Vec<String>,
    current: String,
    block_events: Vec<BlockEvent>,
    osc_count: u64,
    bytes_printed: u64,
    sgr_changes: u64,
}

impl Screen {
    fn newline(&mut self) {
        self.rows.push(std::mem::take(&mut self.current));
    }
}

impl Perform for Screen {
    fn print(&mut self, c: char) {
        self.current.push(c);
        self.bytes_printed += c.len_utf8() as u64;
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => self.newline(),
            b'\r' => self.current.clear(),
            b'\t' => self.current.push('\t'),
            _ => {}
        }
    }

    fn osc_dispatch(&mut self, params: &[&[u8]], _bell_terminated: bool) {
        self.osc_count += 1;
        if params.first().is_some_and(|p| *p == b"133") {
            match params.get(1).copied() {
                Some(b"A") => self.block_events.push(BlockEvent::PromptStart),
                Some(b"B") => self.block_events.push(BlockEvent::PromptEnd),
                Some(b"C") => self.block_events.push(BlockEvent::CommandStart),
                Some(b"D") => {
                    let exit_code = params.get(2).and_then(|s| {
                        std::str::from_utf8(s).ok().and_then(|s| s.parse().ok())
                    });
                    self.block_events.push(BlockEvent::CommandEnd { exit_code });
                }
                _ => {}
            }
        }
    }

    fn csi_dispatch(
        &mut self,
        _params: &Params,
        _intermediates: &[u8],
        _ignore: bool,
        action: char,
    ) {
        if action == 'm' {
            self.sgr_changes += 1;
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {}
    fn hook(
        &mut self,
        _params: &Params,
        _intermediates: &[u8],
        _ignore: bool,
        _action: char,
    ) {
    }
    fn put(&mut self, _byte: u8) {}
    fn unhook(&mut self) {}
}

fn run_spike() -> Screen {
    let mut parser = Parser::new();
    let mut screen = Screen::default();

    let stream: &[u8] = b"\x1b]133;A\x07\
                          $ \x1b]133;B\x07\
                          ls -la\n\
                          \x1b]133;C\x07\
                          total 16\n\
                          drwxr-xr-x  4 user staff 128 Apr 30 12:00 .\n\
                          drwxr-xr-x 12 user staff 384 Apr 30 11:55 ..\n\
                          -rw-r--r--  1 user staff  42 Apr 30 12:00 README.md\n\
                          \x1b]133;D;0\x07\
                          \x1b]133;A\x07\
                          $ \x1b]133;B\x07\
                          false\n\
                          \x1b]133;C\x07\
                          \x1b]133;D;1\x07";

    for &b in stream {
        parser.advance(&mut screen, b);
    }
    screen
}

fn benchmark_throughput() -> (u64, std::time::Duration) {
    // Build a 1 MiB stream of mixed content: prompts, commands, output, SGR codes.
    // Goal: get a steady-state throughput number on a realistic stream shape.
    let mut buf = Vec::with_capacity(1024 * 1024);
    while buf.len() < 1024 * 1024 {
        buf.extend_from_slice(b"\x1b]133;A\x07$ \x1b]133;B\x07cmd\n\x1b]133;C\x07");
        buf.extend_from_slice(b"\x1b[1;32mok\x1b[0m line one\n");
        buf.extend_from_slice(b"\x1b[31merror\x1b[0m something happened\n");
        buf.extend_from_slice(b"line two with \x1b[1msome bold\x1b[22m text\n");
        buf.extend_from_slice(b"\x1b]133;D;0\x07");
    }
    buf.truncate(1024 * 1024);

    let mut parser = Parser::new();
    let mut screen = Screen::default();
    let start = Instant::now();
    for &b in &buf {
        parser.advance(&mut screen, b);
    }
    let elapsed = start.elapsed();
    (buf.len() as u64, elapsed)
}

fn main() {
    println!("=== caro-terminal vt-spike (vte crate) ===\n");

    let screen = run_spike();
    println!("Stream parsing");
    println!("  Bytes printed to grid: {}", screen.bytes_printed);
    println!("  OSC sequences seen:   {}", screen.osc_count);
    println!("  SGR changes:          {}", screen.sgr_changes);

    println!("\n--- Grid contents ---");
    for (i, row) in screen.rows.iter().enumerate() {
        println!("{:2}: {row}", i);
    }
    if !screen.current.is_empty() {
        println!("?? (incomplete row): {}", screen.current);
    }

    println!("\n--- Block events detected ---");
    for (i, ev) in screen.block_events.iter().enumerate() {
        println!("{:2}: {ev:?}", i);
    }

    let blocks = screen
        .block_events
        .iter()
        .filter(|e| matches!(e, BlockEvent::PromptStart))
        .count();
    println!("\nBlocks detected: {blocks}");

    let exit_codes: Vec<_> = screen
        .block_events
        .iter()
        .filter_map(|e| match e {
            BlockEvent::CommandEnd { exit_code } => Some(*exit_code),
            _ => None,
        })
        .collect();
    println!("Exit codes:      {exit_codes:?}");

    // Smoke assertions so this is a real spike, not just a print-and-pray.
    assert_eq!(blocks, 2, "expected exactly 2 blocks");
    assert_eq!(
        screen
            .block_events
            .iter()
            .filter(|e| matches!(e, BlockEvent::CommandEnd { .. }))
            .count(),
        2,
        "expected 2 CommandEnd events"
    );
    assert_eq!(exit_codes, vec![Some(0), Some(1)]);

    println!("\n=== Throughput benchmark (1 MiB stream) ===");
    let (bytes, elapsed) = benchmark_throughput();
    let mb_per_sec = (bytes as f64) / elapsed.as_secs_f64() / 1_048_576.0;
    println!("  Bytes:    {bytes}");
    println!("  Duration: {elapsed:?}");
    println!("  Throughput: {mb_per_sec:.1} MiB/s");

    let _ = std::io::stdout().flush();
    println!("\n[OK] vte spike passed: 2 blocks parsed with exit codes [0, 1].");
}
