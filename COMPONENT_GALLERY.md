# 🎨 TUI Component Gallery

> **A Visual Catalog of All Available Terminal UI Components**

Welcome to the TUI Component Showcase Gallery! This document provides visual previews and comprehensive information about all 16+ production-ready terminal UI components available in this project. Each component is designed to demonstrate different TUI patterns and can be viewed live in the interactive showcase.

## 📋 Quick Navigation

**Jump to Category:**
- [Display Components](#-display-components) (7 components, 40 stories)
- [Input Components](#-input-components) (3 components, 18 stories)
- [Feedback Components](#-feedback-components) (3 components, 18 stories)
- [Workflow Components](#-workflow-components) (1 component, 6 stories)
- [Help Components](#-help-components) (1 component, 4 stories)
- [File System Components](#-file-system-components) (1 component, 7 stories)

**Quick Links:**
- [Component Statistics](#-component-statistics)
- [Component Patterns](#-component-patterns)
- [Building Block Guide](#-building-block-guide)
- [Running the Showcase](#-running-the-showcase)

---

## 📊 Quick Reference Table

| Component | Category | Stories | Best For |
|-----------|----------|---------|----------|
| **SimpleText** | Display | 3 | Basic text rendering with styles |
| **CommandPreview** | Display | 3 | Shell command visualization |
| **TableSelector** | Display | 7 | Interactive data tables with selection |
| **CommandOutputViewer** | Display | 7 | Scrollable command output with syntax highlighting |
| **HistoryTimeline** | Display | 7 | Timeline views with filtering |
| **GenerationComparison** | Display | 6 | Side-by-side command alternative comparison |
| **MetricDashboard** | Display | 7 | System monitoring with metrics and alerts |
| **ConfirmationDialog** | Input | 4 | Modal confirmation dialogs |
| **CommandEditor** | Input | 7 | Multi-line command editing |
| **CommandRating** | Input | 7 | Community voting and rating system |
| **SafetyIndicator** | Feedback | 4 | Risk level visualization |
| **ProgressSpinner** | Feedback | 6 | Loading animations |
| **NotificationToast** | Feedback | 8 | Toasts and banner notifications |
| **CommandFlow** | Workflow | 6 | Complete workflow visualization |
| **KeyboardShortcuts** | Help | 4 | Keyboard reference displays |
| **FileBrowser** | File System | 7 | Hierarchical file/directory tree |

**Total**: 16 components | 87 stories | 6 categories

---

## 🎯 Display Components

Components that present information to users in various formats.

### SimpleText

**Category:** Display
**Stories:** 3 (Default, Styled, MultiLine)
**Use Case:** Demonstrate basic text rendering with various styling options including colors, modifiers (bold, italic, underline), and multi-line layouts.

#### Visual Preview

```
┌─ Simple Text ─────────────────────────────┐
│                                            │
│       Hello, Ratatui Showcase!             │
│                                            │
└────────────────────────────────────────────┘

┌─ Styled Text ─────────────────────────────┐
│                                            │
│     Bold Italic Underline                  │
│                                            │
│     Red Green Blue                         │
│                                            │
└────────────────────────────────────────────┘

┌─ Multi-Line Text ─────────────────────────┐
│                                            │
│  Line 1: First line of text               │
│  Line 2: Second line of text              │
│  Line 3: Third line of text               │
│                                            │
└────────────────────────────────────────────┘
```

#### Stories Available
1. **Default** - Basic centered text
2. **Styled** - Bold, italic, underline, and colored text
3. **MultiLine** - Multiple lines with different styles

#### Code Example
```rust
ShowcaseStory::new("Default", "Simple centered text", |frame, area| {
    let text = Paragraph::new("Hello, World!")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Simple Text"));
    frame.render_widget(text, area);
});
```

#### Related Components
- CommandPreview (for syntax-highlighted text)
- CommandOutputViewer (for scrollable multi-line content)

---

### CommandPreview

**Category:** Display
**Stories:** 3 (Simple Command, Complex Command, With Description)
**Use Case:** Display generated shell commands with syntax highlighting, perfect for showing AI-generated commands before execution.

#### Visual Preview

```
┌─ Command Preview ─────────────────────────┐
│ Generated Command:                         │
│                                            │
│ $ find ~/Downloads -name "*.pdf" -size +10M│
│                                            │
└────────────────────────────────────────────┘

┌─ Command Preview - Complex ───────────────┐
│ Generated Command:                         │
│                                            │
│ $ find . -name '*.rs' \                    │
│     | grep -v target \                     │
│     | xargs wc -l                          │
│                                            │
│ Description:                               │
│ Count lines in all Rust files, excluding  │
│ the target directory                       │
└────────────────────────────────────────────┘
```

#### Stories Available
1. **Simple Command** - Basic one-line command
2. **Complex Command** - Multi-line pipeline with backslashes
3. **With Description** - Command with explanatory text

#### Code Example
```rust
let command = "find . -name '*.rs'";
let preview = Paragraph::new(vec![
    Line::from(vec![
        Span::styled("$ ", Style::default().fg(Color::Green)),
        Span::raw(command),
    ])
]);
```

#### Related Components
- CommandEditor (for editing commands)
- SafetyIndicator (for showing command risk level)
- CommandOutputViewer (for showing command results)

---

### TableSelector

**Category:** Display
**Stories:** 7 (various selection states and highlighting patterns)
**Use Case:** Interactive data tables with row selection, sorting, and dangerous command highlighting. Perfect for displaying command history or results lists.

#### Visual Preview

```
┌─ Command History ─────────────────────────────────────────────────────┐
│ Time                 Query              Command                Status │
│────────────────────────────────────────────────────────────────────────│
│ 2025-01-19 14:32:15  list all PDFs      find . -name '*.pdf'   ✓ Success│
│ 2025-01-19 14:30:42  show disk usage    df -h                  ✓ Success│
│ 2025-01-19 14:28:19  find large logs    find /var/log ...      ✓ Success│
│ 2025-01-19 14:25:33  compress images    find . -name '*.jpg'   ⚠ Cancelled│
│ 2025-01-19 14:22:01  delete temp files  rm -rf /tmp/*          ✗ Blocked│
└────────────────────────────────────────────────────────────────────────┘
                                     ▲ Selected row (cyan background)
                                     ▼ Dangerous command (red text)
```

#### Stories Available
1. **Default Table** - Basic table with headers and data
2. **First Row Selected** - Selection at the top
3. **Middle Row Selected** - Selection in the middle
4. **Last Row Selected** - Selection at the bottom
5. **Dangerous Rows Highlighted** - Red highlighting for risky commands
6. **No Header** - Headerless table variant
7. **Selected Dangerous** - Combined selection and danger highlighting

#### Code Example
```rust
let rows = vec![
    Row::new(vec!["Time", "Query", "Command", "Status"]),
    Row::new(vec!["14:32:15", "list PDFs", "find . -name '*.pdf'", "✓ Success"])
        .style(Style::default().bg(Color::Cyan)), // Selected
];
let table = Table::new(rows)
    .header(Row::new(vec!["Time", "Query", "Command", "Status"]))
    .widths(&[Constraint::Length(20), ...]);
```

#### Related Components
- HistoryTimeline (alternative timeline view)
- CommandRating (for voting on commands)

---

### CommandOutputViewer

**Category:** Display
**Stories:** 7 (Success, Error, Long outputs with scrolling, Tree view, etc.)
**Use Case:** Display scrollable command output with syntax highlighting, line numbers, and colored output for errors/warnings. Essential for showing command execution results.

#### Visual Preview

```
┌─ Command ─────────────────────────────────────────┐
│ $ find . -name '*.rs' | wc -l                     │
└───────────────────────────────────────────────────┘

┌─ Output (Lines 1-12/12) ──────────────────────────┐
│    1 │ src/main.rs                                 │
│    2 │ src/lib.rs                                  │
│    3 │ src/backends/mod.rs                         │
│    4 │ src/backends/remote/ollama.rs               │
│    5 │ src/backends/remote/vllm.rs                 │
│    6 │ src/backends/embedded/cpu.rs                │
│    7 │ src/safety/mod.rs                           │
│    8 │ src/safety/patterns.rs                      │
│    9 │ src/tui/mod.rs                              │
│   10 │ src/tui/showcase.rs                         │
│   11 │                                             │
│   12 │ Total: 42 Rust files                        │
└───────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────┐
│ Exit Code: 0  │  Duration: 0.2s  │  Lines: 12  │
└─────────────────────────────────────────────────┘
```

```
┌─ Command ─────────────────────────────────────────┐
│ $ cat nonexistent_file.txt                        │
└───────────────────────────────────────────────────┘

┌─ Output (Lines 1-8/8) ────────────────────────────┐
│    1 │ cat: nonexistent_file.txt: No such file... │
│    2 │                                             │
│    3 │ Error: Failed to read file                 │ (Red)
│    4 │   at main.rs:42:5                          │
│    5 │   Caused by:                               │
│    6 │     File not found: nonexistent_file.txt   │
│    7 │                                             │
│    8 │ Suggestion: Check if the file exists...    │
└───────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────┐
│ Exit Code: 1  │  Duration: 0.2s  │  Lines: 8   │
└─────────────────────────────────────────────────┘
```

#### Stories Available
1. **Success Output** - Normal command output with line numbers
2. **Error Output** - Error messages with red highlighting
3. **Long Output - Top** - Scrollable content at the top
4. **Long Output - Middle** - Scrolled to middle position
5. **Long Output - Bottom** - Scrolled to bottom
6. **Tree View** - Directory tree with box drawing characters
7. **No Line Numbers** - Cleaner output without line numbering

#### Code Example
```rust
let output_lines = vec!["Line 1", "Line 2", "Line 3"];
let lines: Vec<Line> = output_lines.iter().enumerate().map(|(i, line)| {
    Line::from(vec![
        Span::styled(format!(" {:4} │ ", i + 1), Style::default().fg(Color::DarkGray)),
        Span::raw(*line),
    ])
}).collect();
let output = Paragraph::new(lines).block(Block::default().borders(Borders::ALL));
```

#### Related Components
- CommandPreview (for showing command before execution)
- ProgressSpinner (for showing command is running)
- NotificationToast (for showing completion status)

---

### HistoryTimeline

**Category:** Display
**Stories:** 7 (Compact, Detailed, Filtered, Statistics views)
**Use Case:** Timeline visualization of command history with filtering by status, detailed information display, and session statistics. Great for reviewing past command usage patterns.

#### Visual Preview

```
┌─────────────────────────────────────────────────────────────┐
│  Command History Timeline  │  Showing all commands          │
└─────────────────────────────────────────────────────────────┘

┌─ Timeline ──────────────────────────────────────────────────┐
│ ✓  14:32:15  list all PDF files larger than 10MB      0.8s │
│ ✓  14:30:42  show disk usage in human readable...     0.1s │
│ ✓  14:28:19  find large log files over 100MB          1.2s │
│ ⚠  14:25:33  compress all images to 85% quality       0.0s │
│ ✗  14:22:01  delete all temporary files               0.0s │
│ ✓  14:18:45  count lines in all Rust files            0.3s │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│ Showing 6 of 7 commands  │  Use ↑↓ to navigate, F to filter│
└─────────────────────────────────────────────────────────────┘
```

```
┌─ Timeline ──────────────────────────────────────────────────┐
│ ┌ ✓  2025-01-19 14:32:15  SUCCESS                          │
│ │  Query: list all PDF files larger than 10MB              │
│ │  $ find . -name '*.pdf' -size +10M -ls                   │
│ │  Duration: 0.8s                                           │
│ │                                                            │
│ ├ ✓  2025-01-19 14:30:42  SUCCESS                          │
│ │  Query: show disk usage in human readable format          │
│ │  $ df -h                                                  │
│ │  Duration: 0.1s                                           │
│ │                                                            │
│ └ ✗  2025-01-19 14:22:01  BLOCKED                          │
│    Query: delete all temporary files                        │
│    $ rm -rf /tmp/*                                          │
│    Duration: 0.0s                                           │
└─────────────────────────────────────────────────────────────┘
```

```
┌─ Timeline ──────────────────────────────────────────────────┐
│                                                              │
│                    Session Statistics                        │
│                                                              │
│                                                              │
│   Total Commands: 7                                          │
│                                                              │
│   ✓ Success:     4  ( 57%)                                  │
│   ✗ Blocked:     1  ( 14%)                                  │
│   ⚠ Cancelled:   1  ( 14%)                                  │
│   ✗ Failed:      1  ( 14%)                                  │
│                                                              │
│                                                              │
│   Success Rate: 57.1%                                        │
└─────────────────────────────────────────────────────────────┘
```

#### Stories Available
1. **Compact View** - List format showing all commands
2. **Compact with Selection** - Compact view with selected item
3. **Detailed View** - Full information with timeline connectors
4. **Detailed with Selection** - Detailed view with highlighted command
5. **Filter: Success Only** - Show only successful commands
6. **Filter: Blocked Only** - Show only blocked commands
7. **Statistics View** - Session summary with charts

#### Code Example
```rust
let timeline_connector = if is_first { "┌" } else if is_last { "└" } else { "├" };
Line::from(vec![
    Span::styled(format!(" {} ", timeline_connector), Style::default().fg(Color::Cyan)),
    Span::styled(format!(" {} ", status_icon), Style::default().fg(status_color)),
    Span::styled(timestamp, Style::default().fg(Color::DarkGray)),
    Span::styled(status_label, Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
])
```

#### Related Components
- TableSelector (alternative tabular view)
- CommandOutputViewer (for viewing command details)
- CommandRating (for rating historical commands)

---

### GenerationComparison

**Category:** Display
**Stories:** 6 (Side-by-side, Detailed, Safety comparisons)
**Use Case:** Compare multiple AI-generated command alternatives side-by-side with pros/cons, safety ratings, and community votes. Helps users choose the best command variant.

#### Visual Preview

```
┌───────────────────────────────────────────────────────────────────────┐
│  Comparing Alternatives for: find large files                         │
└───────────────────────────────────────────────────────────────────────┘

┌─ ► Alternative 1 (Selected) ───┬─ Alternative 2 ──────────────────────┐
│                                 │                                      │
│ Command:                        │ Command:                             │
│ $ find . -type f -size +100M \  │ $ find . -type f -size +100M -ls     │
│     -exec ls -lh {} \;          │                                      │
│                                 │                                      │
│ Safety: SAFE                    │ Safety: SAFE                         │
│ Model: MLX Qwen2.5-Coder        │ Model: Ollama CodeLlama              │
│ Votes: ▲ 47                     │ Votes: ▲ 32                          │
│                                 │                                      │
│ Pros:                           │ Pros:                                │
│   ✓ POSIX compliant             │   ✓ Faster execution                 │
│   ✓ Works on all systems        │   ✓ Single process                   │
│   ✓ Shows file details          │   ✓ Clean output                     │
│                                 │                                      │
│ Cons:                           │ Cons:                                │
│   ✗ Slower due to multiple ls   │   ✗ Less portable (not all find...)  │
│   ✗ Verbose output              │   ✗ Fixed format                     │
│                                 │                                      │
└─────────────────────────────────┴──────────────────────────────────────┘
```

```
┌───────────────────────────────────────────────────────────────────────┐
│  Query: delete temp files                                              │
└───────────────────────────────────────────────────────────────────────┘

┌─ Alternative 1 of 3 ──────────────────────────────────────────────────┐
│                                                                        │
│ Command:                                                               │
│ $ find /tmp -type f -name '*.tmp' -mtime +7 -delete                    │
│                                                                        │
│ Explanation: Safely deletes only .tmp files older than 7 days in /tmp │
│                                                                        │
│ Safety: SAFE  │  Performance: Fast  │  Model: MLX Qwen2.5-Coder       │
└────────────────────────────────────────────────────────────────────────┘

┌─ All Alternatives (↑↓ to navigate) ────────────────────────────────────┐
│ ► 1. find /tmp -type f -name '*.tmp' -mtime +7 -delete         SAFE   │
│   2. rm -rf /tmp/*                                              RISKY  │
│   3. find ~/Downloads -type f -name '*.tmp' -o -name '*.cache' MODERATE│
└────────────────────────────────────────────────────────────────────────┘
```

#### Stories Available
1. **Side-by-Side: Find Files** - Compare two safe alternatives
2. **Side-by-Side: Selected Alt 2** - Second alternative selected
3. **Detailed View: Safe Command** - Detailed view with explanation
4. **Detailed View: All Alternatives** - Shows all three options
5. **Dangerous Command Warning** - Highlighting risky alternative
6. **Safety Comparison** - Safe vs risky side-by-side

#### Code Example
```rust
let alt1_panel = Layout::default()
    .direction(Direction::Vertical)
    .constraints([...])
    .split(left_area);

let lines = vec![
    Line::from(vec![Span::styled("Command:", Style::default().fg(Color::Cyan))]),
    Line::from(vec![Span::styled("$ ", Style::default().fg(Color::Green)), Span::raw(command)]),
    Line::from(""),
    Line::from("Pros:"),
    Line::from(vec![Span::styled("  ✓ ", Style::default().fg(Color::Green)), Span::raw(pro)]),
];
```

#### Related Components
- CommandRating (for community voting)
- SafetyIndicator (for detailed safety analysis)
- CommandPreview (for single command display)

---

### MetricDashboard

**Category:** Display
**Stories:** 7 (Basic, Sparklines, Color-coded, Alerts, Historical)
**Use Case:** System monitoring dashboard with metrics, sparklines, and color-coded alerts. Displays CPU, memory, disk, network stats with visual indicators.

#### Visual Preview

```
┌─────────────────────┬─────────────────────┬─────────────────────┬─────────────────────┐
│ CPU Usage           │ Memory Usage        │ Disk Space          │ Network I/O         │
├─────────────────────┼─────────────────────┼─────────────────────┼─────────────────────┤
│ 45.0%               │ 78.0%               │ 234.0 GB / 512.0 GB │ 2.3MB/s             │
│ Normal              │ High                │ Normal              │ Normal              │
└─────────────────────┴─────────────────────┴─────────────────────┴─────────────────────┘
  (Green)               (Yellow)              (Green)               (Green)
```

```
┌─────────────────────┬─────────────────────┬─────────────────────┬─────────────────────┐
│ CPU Usage           │ Memory Usage        │ Disk Space          │ Network I/O         │
├─────────────────────┼─────────────────────┼─────────────────────┼─────────────────────┤
│ 45.0%               │ 78.0%               │ 234.0 GB / 512.0 GB │ 2.3MB/s             │
│ ░▒▓▓█▓▒             │ ▒▓▓█████            │ ░░▒▒▓▓█             │ ░▒▒█▓▒▒             │
│ Normal              │ High                │ Normal              │ Normal              │
└─────────────────────┴─────────────────────┴─────────────────────┴─────────────────────┘
    ▲ ASCII sparklines showing trends over time
```

```
┌─────────────────────┬─────────────────────┬─────────────────────┬─────────────────────┐
│ CPU Usage           │ Memory Usage        │ Disk Space          │ Network I/O         │
├─────────────────────┼─────────────────────┼─────────────────────┼─────────────────────┤
│ 45.0%               │ 78.0%               │ 92.0%               │ 2.3MB/s             │
│ ▓▓▓▓▓░░░░░          │ ▓▓▓▓▓▓▓▓░░          │ ▓▓▓▓▓▓▓▓▓▓          │ ▓▓░░░░░░░░          │
│ Normal              │ High                │ Critical            │ Normal              │
└─────────────────────┴─────────────────────┴─────────────────────┴─────────────────────┘
  (Green)               (Yellow)              (Red - ALERT!)        (Green)
```

```
┌─────────────────────┬─────────────────────┬─────────────────────┬─────────────────────┐
│ CPU Usage           │ Memory Usage        │ Disk Space          │ Network I/O         │
├─────────────────────┼─────────────────────┼─────────────────────┼─────────────────────┤
│ 45.0%               │ 78.0%               │ 234.0 GB / 512.0 GB │ 2.3MB/s             │
│ Normal              │ High                │ Normal              │ Normal              │
│ ↑ +5.2%             │ ↓ -2.1%             │ ↑ +0.5%             │ ↓ -15.3%            │
└─────────────────────┴─────────────────────┴─────────────────────┴─────────────────────┘
    ▲ Change indicators (↑ increase, ↓ decrease)
```

#### Stories Available
1. **Basic Metrics** - Simple dashboard with 4 key metrics
2. **With Sparklines** - Mini graphs showing recent trends
3. **Color-Coded** - Red/yellow/green thresholds
4. **Compact Layout** - Dense 2x2 grid for small screens
5. **With Units** - Various units (%, GB, MB/s)
6. **Critical Alert** - Multiple metrics in critical state
7. **Historical Comparison** - Change indicators over time

#### Code Example
```rust
let metric = Metric::new("CPU Usage", 45.0, "%")
    .with_status(MetricStatus::Healthy)
    .with_sparkline(vec![30.0, 35.0, 40.0, 42.0, 45.0])
    .with_change(5.2);

let progress_bar = metric.progress_bar(10); // "▓▓▓▓▓░░░░░"
let sparkline = metric.render_sparkline();  // "░▒▓▓█"
let status_color = metric.status_color();   // Color::Green
```

#### Related Components
- ProgressSpinner (for loading states)
- NotificationToast (for threshold alerts)

---

## 💡 Input Components

Components that capture user input and interactions.

### ConfirmationDialog

**Category:** Input
**Stories:** 4 (Yes/No Selected, Dangerous Command, Long Message)
**Use Case:** Modal confirmation dialogs for Yes/No decisions with button selection and dangerous command warnings.

#### Visual Preview

```
                    ┌─ Confirm Execution ──────┐
                    │                          │
                    │ Do you want to execute   │
                    │ this command?            │
                    │                          │
                    │ ls -la /home/user        │
                    │                          │
                    ├──────────────────────────┤
                    │                          │
                    │    [Yes]     No          │
                    │                          │
                    └──────────────────────────┘
                         ▲ Yes selected (green background)
```

```
                    ┌─ Confirm Execution ──────┐
                    │                          │
                    │ Do you want to execute   │
                    │ this command?            │
                    │                          │
                    │ ls -la /home/user        │
                    │                          │
                    ├──────────────────────────┤
                    │                          │
                    │     Yes     [No]         │
                    │                          │
                    └──────────────────────────┘
                              ▲ No selected (red background)
```

```
                    ┌─ ⚠ DANGEROUS COMMAND ────┐
                    │                          │
                    │ WARNING: This command    │
                    │ is dangerous!            │
                    │                          │
                    │ rm -rf /                 │
                    │                          │
                    │ This will delete all     │
                    │ files on your system!    │
                    │                          │
                    ├──────────────────────────┤
                    │                          │
                    │     Yes     [No]         │
                    │                          │
                    └──────────────────────────┘
                        ▲ Red border for dangerous commands
```

#### Stories Available
1. **Yes Selected** - Green highlighted Yes button
2. **No Selected** - Red highlighted No button
3. **Dangerous Command** - Warning style with red border
4. **Long Message** - Multi-line message text

#### Code Example
```rust
let buttons = vec![
    Span::styled("  [Yes]  ", Style::default().bg(Color::Green).fg(Color::Black)),
    Span::raw("  "),
    Span::styled("   No   ", Style::default().fg(Color::White)),
];
let button_line = Line::from(buttons).alignment(Alignment::Center);
```

#### Related Components
- SafetyIndicator (for showing risk level)
- CommandPreview (for showing the command being confirmed)

---

### CommandEditor

**Category:** Input
**Stories:** 7 (Simple, Multi-line, Cursor positions, Syntax highlighting)
**Use Case:** Multi-line command editor with syntax highlighting, line numbers, and cursor positioning. Ideal for editing complex shell scripts.

#### Visual Preview

```
┌─ Command Editor ──────────────────────────────────────────┐
│  Editor (Syntax Highlighting)                             │
├───────────────────────────────────────────────────────────┤
│   1 │ find . -name '*.rs' \                               │
│   2 │   | grep -v target \                                │
│   3 │   | xargs wc -l \                          ▍        │
│   4 │   | sort -n                                         │
│                                                            │
└────────────────────────────────────────────────────────────┘
│ ↑↓: Navigate | Enter: Execute | Ctrl+E: Edit | Esc: Cancel│
└────────────────────────────────────────────────────────────┘
         ▲ Line 3 selected with cursor indicator
```

```
┌─ Command Editor ──────────────────────────────────────────┐
│  Editor (No Syntax Highlighting)                          │
├───────────────────────────────────────────────────────────┤
│   1 │ #!/bin/bash                                         │
│   2 │                                                     │
│   3 │ for file in *.txt; do                      ▍        │
│   4 │     echo "Processing $file"                         │
│   5 │     wc -l "$file"                                   │
│   6 │ done                                                │
│                                                            │
└────────────────────────────────────────────────────────────┘
│ ↑↓: Navigate | Enter: Execute | Ctrl+E: Edit | Esc: Cancel│
└────────────────────────────────────────────────────────────┘
```

```
┌─ Command Editor ──────────────────────────────────────────┐
│  Editor                                                   │
├───────────────────────────────────────────────────────────┤
│ find . -name '*.rs' \                                     │
│   | grep -v target \                                      │
│   | xargs wc -l \                               ▍         │
│   | sort -n                                               │
│                                                            │
└────────────────────────────────────────────────────────────┘
│ ↑↓: Navigate | Enter: Execute | Ctrl+E: Edit | Esc: Cancel│
└────────────────────────────────────────────────────────────┘
    ▲ No line numbers variant
```

#### Stories Available
1. **Simple Command** - Single-line command
2. **Multi-line Pipeline** - Shell pipeline with backslashes
3. **With Cursor (Start)** - Cursor at line 1
4. **With Cursor (Middle)** - Cursor at line 3
5. **With Cursor (End)** - Cursor at last line
6. **No Line Numbers** - Clean view without numbering
7. **Complex Shell Script** - Multi-line bash script

#### Code Example
```rust
let lines: Vec<Line> = command_lines.iter().enumerate().map(|(i, line)| {
    let mut spans = vec![
        Span::styled(format!(" {:3} │ ", i + 1), Style::default().fg(Color::DarkGray)),
        Span::raw(*line),
    ];
    if i == cursor_line {
        spans.push(Span::styled(" ▍", Style::default().fg(Color::Yellow)));
    }
    Line::from(spans)
}).collect();
```

#### Related Components
- CommandPreview (for read-only display)
- ConfirmationDialog (for confirming edits)

---

### CommandRating

**Category:** Input
**Stories:** 7 (List, Selection, Sorted views, Voting details)
**Use Case:** Community voting and rating system for commands with upvotes/downvotes, comments, and alternative suggestions. Enables collective intelligence.

#### Visual Preview

```
┌─────────────────────────────────────────────────────────────┐
│              Community-Rated Commands                        │
└─────────────────────────────────────────────────────────────┘

┌─ All Commands ──────────────────────────────────────────────┐
│ ▲  +44 ▼  │  find large files over 100MB                    │
│ ▲  +24 ▼  │  count lines in all Rust files                  │
│ ▲  +26 ▼  │  show disk usage sorted by size                 │
│ ▲  -10 ▼  │  find and remove node_modules                   │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│ ↑↓: Navigate │ Space: Vote │ C: Comments │ A: Alternatives │
└─────────────────────────────────────────────────────────────┘
   ▲ Vote arrows (gray when not voted, colored when voted)
```

```
┌─────────────────────────────────────────────────────────────┐
│              Community-Rated Commands                        │
└─────────────────────────────────────────────────────────────┘

┌─ Top Rated ─────────────────────────────────────────────────┐
│ ▲  +44 ▼  │  find large files over 100MB                    │
│        │  $ find . -type f -size +100M -exec ls -lh {} \;   │
│        │  94% upvoted  │  5 comments  │  3 alternatives     │
│        │  ▲ 47  ▼ 3                                          │
│                                                              │
│ ▲  +26 ▼  │  show disk usage sorted by size                 │
│        │  $ du -ah | sort -hr | head -20                    │
│        │  93% upvoted  │  3 comments  │  2 alternatives     │
│        │  ▲ 28  ▼ 2                                          │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│ ↑↓: Navigate │ Space: Vote │ C: Comments │ A: Alternatives │
└─────────────────────────────────────────────────────────────┘
```

```
┌─ Command ───────────────────────────────────────────────────┐
│ Query: find large files over 100MB                          │
│                                                              │
│ $ find . -type f -size +100M -exec ls -lh {} \;             │
└─────────────────────────────────────────────────────────────┘

┌─ Voting Statistics ─────────────────────────────────────────┐
│                                                              │
│  Score: +44  (94% upvoted)                                  │
│                                                              │
│  ▲ Upvotes:   47                                            │
│  ▼ Downvotes: 3                                             │
│  Total Votes: 50                                            │
└─────────────────────────────────────────────────────────────┘

┌─ Comments (5) ──────────────────────────────────────────────┐
│                                                              │
│ @rustdev42 2h ago                                           │
│   This is a great command! Much better than using du.       │
│                                                              │
│ @shellmaster 5h ago                                         │
│   Consider adding -prune to avoid traversing excluded dirs. │
│                                                              │
│   3 more comments...                                        │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│ ↑: Upvote │ ↓: Downvote │ C: View All Comments │ Esc: Back │
└─────────────────────────────────────────────────────────────┘
```

#### Stories Available
1. **Command List** - List with scores and voting arrows
2. **With Selection** - Selected command with details
3. **Top Rated** - Sorted by highest score
4. **Controversial** - Most mixed voting
5. **Voting Detail View** - Full details with comments
6. **User Voted Up** - Command user upvoted (green arrow)
7. **User Voted Down** - Command user downvoted (red arrow)

#### Code Example
```rust
let score = upvotes as i32 - downvotes as i32;
let score_color = if score > 20 { Color::Green } else if score > 0 { Color::Yellow } else { Color::Red };

Line::from(vec![
    Span::styled(" ▲ ", if user_voted_up { Style::default().fg(Color::Green).add_modifier(Modifier::BOLD) } else { Style::default().fg(Color::DarkGray) }),
    Span::styled(format!("{:4}", score), Style::default().fg(score_color).add_modifier(Modifier::BOLD)),
    Span::styled(" ▼", if user_voted_down { Style::default().fg(Color::Red).add_modifier(Modifier::BOLD) } else { Style::default().fg(Color::DarkGray) }),
])
```

#### Related Components
- GenerationComparison (for comparing alternatives)
- HistoryTimeline (for viewing command history)
- TableSelector (alternative list view)

---

## 🔔 Feedback Components

Components that provide feedback to users about system state and actions.

### SafetyIndicator

**Category:** Feedback
**Stories:** 4 (Safe, Moderate, High Risk, Critical)
**Use Case:** Visual indicator for command safety levels with color coding (green/yellow/red) and risk descriptions.

#### Visual Preview

```
┌─ Safety Level ──────┐     ┌─ Safety Level ──────┐     ┌─ Safety Level ──────┐
│                     │     │                     │     │                     │
│  ✓ SAFE             │     │  ⚠ HIGH RISK        │     │  ✗ CRITICAL         │
│                     │     │                     │     │                     │
└─────────────────────┘     └─────────────────────┘     └─────────────────────┘
 (Green border)              (Orange/Red border)        (Red border)

┌─ Command ───────────┐     ┌─ Command ───────────┐     ┌─ Command ───────────┐
│ $ ls -la            │     │ $ rm -rf ./target   │     │ $ sudo rm -rf /     │
└─────────────────────┘     └─────────────────────┘     └─────────────────────┘

┌─ Description ───────┐     ┌─ Description ───────┐     ┌─ Description ───────┐
│ This command is     │     │ This command may    │     │ This command is     │
│ safe to execute     │     │ cause unintended    │     │ dangerous and should│
│                     │     │ changes             │     │ not be executed     │
└─────────────────────┘     └─────────────────────┘     └─────────────────────┘
```

#### Stories Available
1. **Safe Command** - Green indicator, ls command
2. **Moderate Risk** - Yellow indicator, moderate warning
3. **High Risk** - Orange/red indicator, rm command
4. **Critical Risk** - Red indicator, dangerous system command

#### Code Example
```rust
let (icon, color, label) = match risk_level {
    RiskLevel::Safe => ("✓", Color::Green, "SAFE"),
    RiskLevel::Moderate => ("⚠", Color::Yellow, "MODERATE"),
    RiskLevel::High => ("⚠", Color::LightRed, "HIGH RISK"),
    RiskLevel::Critical => ("✗", Color::Red, "CRITICAL"),
};

let indicator = Paragraph::new(format!("{} {}", icon, label))
    .style(Style::default().fg(color).add_modifier(Modifier::BOLD))
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(color)));
```

#### Related Components
- ConfirmationDialog (used together for confirmations)
- CommandPreview (shows command being evaluated)
- GenerationComparison (shows safety of alternatives)

---

### ProgressSpinner

**Category:** Feedback
**Stories:** 6 (Animation frames, different contexts)
**Use Case:** Animated spinner for loading states and progress indication. Uses Braille characters for smooth animation.

#### Visual Preview

```
┌─ Loading ───────────────────┐
│                             │
│       ⠋ Loading model...    │
│                             │
└─────────────────────────────┘

┌─ Loading ───────────────────┐
│                             │
│       ⠙ Loading model...    │
│                             │
└─────────────────────────────┘

┌─ Loading ───────────────────┐
│                             │
│       ⠹ Loading model...    │
│                             │
└─────────────────────────────┘

┌─ Loading ───────────────────┐
│                             │
│       ⠸ Loading model...    │
│                             │
└─────────────────────────────┘
    Animation frames: ⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏
```

```
┌─ Generating Command ────────┐
│                             │
│    ⠼ Generating command...  │
│                             │
└─────────────────────────────┘

┌─ Processing ────────────────┐
│                             │
│    ⠦ Processing input...    │
│                             │
└─────────────────────────────┘
```

#### Stories Available
1. **Frame 0** - First animation frame
2. **Frame 1** - Second animation frame
3. **Frame 2** - Third animation frame
4. **Frame 3** - Fourth animation frame
5. **Generating Command** - Context: command generation
6. **Processing** - Context: processing input

#### Code Example
```rust
const SPINNER_FRAMES: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

let frame = SPINNER_FRAMES[frame_index % SPINNER_FRAMES.len()];
let spinner = Paragraph::new(format!("{} {}", frame, message))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::ALL));
```

#### Related Components
- CommandOutputViewer (shown after loading completes)
- NotificationToast (for completion notification)

---

### NotificationToast

**Category:** Feedback
**Stories:** 8 (Toast/Banner styles, Info/Success/Warning/Error levels)
**Use Case:** Temporary notifications with different styles (toast/banner), positions (top/bottom/center), and severity levels.

#### Visual Preview

```
Toast (Centered):                        Banner (Top):
    ┌────────────────────────┐          ╔═══════════════════════════════╗
    │                        │          ║ ✓ SUCCESS Configuration saved ║
    │ ℹ INFO: Loading...     │          ╚═══════════════════════════════╝
    │                        │
    └────────────────────────┘

Toast (Top-Right):                       Banner (Bottom):
                  ┌──────────────────┐   ╔═══════════════════════════════╗
                  │                  │   ║ ✗ ERROR Network connection lost║
                  │ ✗ ERROR: Failed  │   ╚═══════════════════════════════╝
                  │ Press key...     │
                  └──────────────────┘

Toast Styles:
┌────────────────────────┐  ┌────────────────────────┐  ┌────────────────────────┐  ┌────────────────────────┐
│ ℹ INFO                 │  │ ✓ SUCCESS              │  │ ⚠ WARNING              │  │ ✗ ERROR                │
│ Processing...          │  │ Command executed!      │  │ High memory usage      │  │ Failed to connect      │
└────────────────────────┘  └────────────────────────┘  └────────────────────────┘  └────────────────────────┘
 (Blue)                      (Green)                     (Yellow)                    (Red)
```

#### Stories Available
1. **Info Toast** - Blue info notification
2. **Success Toast** - Green success notification
3. **Warning Toast** - Yellow warning notification
4. **Error Toast** - Red error notification
5. **Info Banner** - Full-width info banner
6. **Success Banner** - Full-width success banner
7. **Warning Banner** - Full-width warning banner
8. **Error Banner** - Full-width error banner

#### Code Example
```rust
let (icon, color, border_set) = match notification_type {
    NotificationType::Info => ("ℹ", Color::Cyan, BorderSet::ROUNDED),
    NotificationType::Success => ("✓", Color::Green, BorderSet::DOUBLE),
    NotificationType::Warning => ("⚠", Color::Yellow, BorderSet::ROUNDED),
    NotificationType::Error => ("✗", Color::Red, BorderSet::ROUNDED),
};

let toast = Paragraph::new(format!("{} {} {}", icon, level, message))
    .block(Block::default().borders(Borders::ALL).border_set(border_set).border_style(Style::default().fg(color)))
    .alignment(Alignment::Center);
```

#### Related Components
- ProgressSpinner (for ongoing operations)
- SafetyIndicator (for command safety feedback)

---

## 🔄 Workflow Components

Components that orchestrate multi-step processes.

### CommandFlow

**Category:** Workflow
**Stories:** 6 (Step 1-6 progression through workflow)
**Use Case:** Complete command generation workflow visualization from input through generation, safety check, confirmation, execution, to completion.

#### Visual Preview

```
┌─ Command Generation Workflow ─────────────────────────────────────┐
│                  Command Generation Workflow                       │
└────────────────────────────────────────────────────────────────────┘

┌─ Progress ─────────────────┐
│                            │
│  ✓ 1. Input                │
│  ○ 2. Generating           │
│  ○ 3. Safety Check         │
│  ○ 4. Confirmation         │
│  ○ 5. Executing            │
│  ○ 6. Complete             │
│                            │
└────────────────────────────┘

┌─ Current Step ─────────────┐
│                            │
│ Enter your query:          │
│                            │
│ > find large PDF files_    │
│                            │
└────────────────────────────┘
```

```
┌─ Command Generation Workflow ─────────────────────────────────────┐
│                  Command Generation Workflow                       │
└────────────────────────────────────────────────────────────────────┘

┌─ Progress ─────────────────┐
│                            │
│  ✓ 1. Input                │
│  ✓ 2. Generating           │
│  ▶ 3. Safety Check         │ ← Current step (green)
│  ○ 4. Confirmation         │
│  ○ 5. Executing            │
│  ○ 6. Complete             │
│                            │
└────────────────────────────┘

┌─ Current Step ─────────────┐
│                            │
│ Generated Command:         │
│                            │
│ $ find . -name '*.pdf'     │
│   -size +10M -ls           │
│                            │
│ ✓ SAFE - This command is  │
│   safe to execute          │
│                            │
└────────────────────────────┘
```

```
┌─ Command Generation Workflow ─────────────────────────────────────┐
│                  Command Generation Workflow                       │
└────────────────────────────────────────────────────────────────────┘

┌─ Progress ─────────────────┐
│                            │
│  ✓ 1. Input                │
│  ✓ 2. Generating           │
│  ✓ 3. Safety Check         │
│  ✓ 4. Confirmation         │
│  ✓ 5. Executing            │
│  ✓ 6. Complete             │ ← All steps complete (green)
│                            │
└────────────────────────────┘

┌─ Current Step ─────────────┐
│                            │
│ ✓ Command executed!        │
│                            │
│ Exit code: 0               │
│ Duration: 0.8s             │
│ Output: 15 files found     │
│                            │
└────────────────────────────┘
```

#### Stories Available
1. **Step 1: Input** - User entering query
2. **Step 2: Generating** - AI generating command
3. **Step 3: Safety Check** - Validating command safety
4. **Step 4: Confirmation** - User confirming execution
5. **Step 5: Executing** - Command running
6. **Step 6: Complete** - Workflow finished

#### Code Example
```rust
let steps = vec![
    ("Input", StepStatus::Completed),
    ("Generating", StepStatus::Completed),
    ("Safety Check", StepStatus::Current),
    ("Confirmation", StepStatus::Pending),
    ("Executing", StepStatus::Pending),
    ("Complete", StepStatus::Pending),
];

let step_lines: Vec<Line> = steps.iter().enumerate().map(|(i, (name, status))| {
    let (icon, color) = match status {
        StepStatus::Completed => ("✓", Color::Green),
        StepStatus::Current => ("▶", Color::Yellow),
        StepStatus::Pending => ("○", Color::DarkGray),
    };
    Line::from(vec![
        Span::styled(format!(" {} {}. {}", icon, i + 1, name), Style::default().fg(color))
    ])
}).collect();
```

#### Related Components
- ProgressSpinner (for in-progress steps)
- SafetyIndicator (for safety check step)
- ConfirmationDialog (for confirmation step)
- CommandOutputViewer (for results display)

---

## ❓ Help Components

Components that provide help and reference information.

### KeyboardShortcuts

**Category:** Help
**Stories:** 4 (Compact, Categorized, Detailed, Grid layouts)
**Use Case:** Keyboard shortcuts reference displayed in various layouts (compact list, categorized, detailed with descriptions, grid).

#### Visual Preview

```
Compact View:
┌─ Keyboard Shortcuts ────┐
│ Ctrl+C         Exit     │
│ Enter          Confirm  │
│ Esc            Cancel   │
│ ↑/↓            Navigate │
│ Ctrl+E         Edit     │
│ Ctrl+H         History  │
│ F1             Help     │
└──────────────────────────┘
```

```
Compact with Categories:
┌─ Keyboard Shortcuts ────┐
│ ╔═══ General ═══        │
│ ║ Ctrl+C       Exit     │
│ ║ Enter        Confirm  │
│ ║ Esc          Cancel   │
│ ╚════════════════       │
│                         │
│ ╔═══ Navigation ═══     │
│ ║ ↑/↓          Move     │
│ ║ ←/→          Switch   │
│ ╚════════════════       │
│                         │
│ ╔═══ Editing ═══        │
│ ║ Ctrl+E       Edit     │
│ ║ Ctrl+R       Regen    │
│ ╚════════════════       │
└──────────────────────────┘
```

```
Grid Layout:
┌─ Keyboard Shortcuts Reference ─────────────────────────────┐
│              Keyboard Shortcuts Reference                   │
├─────────────────────────────┬───────────────────────────────┤
│ Essential                   │ Advanced                      │
├─────────────────────────────┼───────────────────────────────┤
│ Ctrl+C                      │ Ctrl+E                        │
│ Cancel/Exit                 │ Edit command                  │
│                             │                               │
│ Enter                       │ Ctrl+H                        │
│ Confirm/Execute             │ Show history                  │
│                             │                               │
│ Esc                         │ Ctrl+R                        │
│ Cancel/Go back              │ Regenerate                    │
└─────────────────────────────┴───────────────────────────────┘
│ Press F1 anytime to show help                               │
└─────────────────────────────────────────────────────────────┘
```

```
Detailed View:
┌─ Help ───────────────────────────────────────────────────┐
│                                                           │
│ Available Shortcuts                                       │
│                                                           │
│ ╔═══ General ═══                                          │
│ ║                                                         │
│ ║  Ctrl+C         → Exit the application                 │
│ ║                   Immediately quit without saving       │
│ ║                                                         │
│ ║  Enter          → Confirm action                       │
│ ║                   Execute selected command             │
│ ║                                                         │
│ ║  Esc            → Cancel or go back                    │
│ ║                   Return to previous screen            │
│ ╚════════════════════                                     │
│                                                           │
│ ╔═══ Navigation ═══                                       │
│ ║                                                         │
│ ║  ↑/↓ or j/k     → Navigate up/down                     │
│ ║                   Move through list items              │
│ ╚════════════════════                                     │
└───────────────────────────────────────────────────────────┘
```

#### Stories Available
1. **Compact List** - Simple two-column list
2. **Compact with Categories** - Grouped by category
3. **Detailed View** - Full descriptions for each shortcut
4. **Grid Layout** - Two-column grid with categories

#### Code Example
```rust
let shortcuts = vec![
    ("Ctrl+C", "Exit", "General"),
    ("Enter", "Confirm", "General"),
    ("↑/↓", "Navigate", "Navigation"),
    ("Ctrl+E", "Edit", "Editing"),
];

let lines: Vec<Line> = shortcuts.iter().map(|(key, action, _)| {
    Line::from(vec![
        Span::styled(format!(" {:15}", key), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled(action, Style::default().fg(Color::White)),
    ])
}).collect();
```

#### Related Components
- NotificationToast (for showing help hints)
- ConfirmationDialog (for help confirmation)

---

## 📁 File System Components

Components for file system navigation and display.

### FileBrowser

**Category:** File System
**Stories:** 7 (Simple tree, Deep nesting, Large directory, Icons, Selection, Empty, Search)
**Use Case:** Hierarchical file/directory tree with expandable folders, file type icons, and selection highlighting. Perfect for file navigation interfaces.

#### Visual Preview

```
┌─ File Browser - Simple Tree ──────────────────────────┐
│ ▼ 📁 src                                               │
│     🦀 main.rs                                         │
│     🦀 lib.rs                                          │
│ ▶ 📁 tests                                             │
│ ⚙️  Cargo.toml                                         │
│ 📝 README.md                                           │
└────────────────────────────────────────────────────────┘
  ▲ Expanded folder (▼)  ▲ Collapsed folder (▶)
```

```
┌─ File Browser - Deep Nesting ─────────────────────────┐
│ ▼ 📁 project                                           │
│   ▼ 📁 src                                             │
│     ▼ 📁 components                                    │
│       ▼ 📁 ui                                          │
│         ▼ 📁 buttons                                   │
│           🦀 PrimaryButton.rs                          │
│           🦀 SecondaryButton.rs                        │
└────────────────────────────────────────────────────────┘
      ▲ 5+ levels of nesting with proper indentation
```

```
┌─ File Browser - With Icons ───────────────────────────┐
│ ▼ 📁 project                                           │
│   🦀 main.rs          (Rust - crab emoji)             │
│   🐍 script.py        (Python - snake emoji)          │
│   ⚙️  config.json     (Config - gear emoji)           │
│   📝 README.md        (Markdown - memo emoji)         │
│   🖼️  logo.png        (Image - picture emoji)         │
│   📕 manual.pdf       (PDF - book emoji)              │
│   📦 archive.zip      (Archive - package emoji)       │
│   📜 index.js         (JavaScript - scroll emoji)     │
└────────────────────────────────────────────────────────┘
```

```
┌─ File Browser - Selected Item ────────────────────────┐
│ ▼ 📁 src                                               │
│   🦀 main.rs                                           │
│   🦀 lib.rs                                            │
│   🦀 utils.rs                                          │
│ ⚙️  Cargo.toml                                         │
└────────────────────────────────────────────────────────┘
    ▲ Selected item (cyan background, bold text)
```

```
┌─ File Browser - Large Directory ──────────────────────┐
│ ▼ 📁 logs                                              │
│   📄 app-0001.log                                      │
│   📄 app-0002.log                                      │
│   📄 app-0003.log                                      │
│   ...                                                  │
│   📄 app-0020.log                                      │
│   📄 app-0021.log                                      │
│   📄 app-0022.log                                      │
└────────────────────────────────────────────────────────┘
    ▲ 20+ files in directory
```

```
┌─ File Browser ─────────────────────────────────────────┐
│                                                        │
│                  Empty Directory                       │
│                                                        │
│           No files or folders to display               │
│                                                        │
└────────────────────────────────────────────────────────┘
    ▲ Empty state handling
```

```
┌─ File Browser - Search Results ───────────────────────┐
│ ▼ 📁 Search Results: *.rs                              │
│   🦀 src/main.rs                                       │
│   🦀 src/lib.rs                                        │
│   🦀 src/utils.rs                                      │
│   🦀 tests/integration.rs                              │
│   🦀 tests/unit.rs                                     │
└────────────────────────────────────────────────────────┘
    ▲ Filtered view showing specific file types
```

#### Stories Available
1. **Simple Tree** - Basic 3-4 files and folders
2. **Deep Nesting** - 5+ levels of folder hierarchy
3. **Large Directory** - 20+ files in a single folder
4. **With Icons** - File type icons using Unicode
5. **Selected Item** - Highlighted selection
6. **Empty Directory** - Empty state display
7. **Search Results** - Filtered file list

#### Code Example
```rust
let file_tree = vec![
    FileNode::directory("src")
        .expanded()
        .with_child(FileNode::file("main.rs"))
        .with_child(FileNode::file("lib.rs").selected()),
    FileNode::file("Cargo.toml"),
];

fn get_icon(node: &FileNode) -> &'static str {
    if node.is_directory { return "📁"; }
    match node.extension() {
        "rs" => "🦀",
        "py" => "🐍",
        "json" => "⚙️",
        "md" => "📝",
        _ => "📄",
    }
}
```

#### Related Components
- TableSelector (alternative list view)
- CommandOutputViewer (for tree command output)

---

## 🎯 Component Patterns

### Common UI Patterns

#### Progress Indicators
Multiple components for showing progress:
- **ProgressSpinner** - Indeterminate progress with animation
- **MetricDashboard** - Progress bars with percentages
- **CommandFlow** - Step-by-step workflow progress

#### Status Displays
Components for showing system state:
- **SafetyIndicator** - Command risk levels
- **MetricDashboard** - System metrics and alerts
- **NotificationToast** - Temporary status messages

#### Selection UIs
Interactive selection components:
- **TableSelector** - Row-based selection in tables
- **FileBrowser** - File/folder selection
- **CommandRating** - Voting and selection
- **ConfirmationDialog** - Binary choice (Yes/No)

#### Information Displays
Read-only information components:
- **SimpleText** - Basic text display
- **CommandPreview** - Formatted command display
- **CommandOutputViewer** - Scrollable multi-line output
- **HistoryTimeline** - Chronological event display
- **KeyboardShortcuts** - Reference information

---

## 🔨 Building Block Guide

### How Components Can Be Combined

#### Command Execution Flow
Complete workflow using multiple components:

1. **Input** → CommandEditor (edit command)
2. **Preview** → CommandPreview (show what will run)
3. **Safety** → SafetyIndicator (check risk level)
4. **Confirm** → ConfirmationDialog (get user approval)
5. **Progress** → ProgressSpinner (show execution)
6. **Output** → CommandOutputViewer (display results)
7. **Notify** → NotificationToast (completion message)

#### Monitoring Dashboard
Combining metrics and alerts:

1. **MetricDashboard** (main metrics grid)
2. **NotificationToast** (threshold alerts)
3. **HistoryTimeline** (historical trends)

#### Command History Browser
Full-featured history interface:

1. **HistoryTimeline** or **TableSelector** (list view)
2. **CommandOutputViewer** (view past output)
3. **CommandRating** (rate and vote)
4. **GenerationComparison** (compare alternatives)

#### File Management Interface
IDE-like file browser:

1. **FileBrowser** (directory tree)
2. **CommandEditor** (edit selected file)
3. **CommandOutputViewer** (file contents)

#### Help System
Multi-level help interface:

1. **KeyboardShortcuts** (quick reference)
2. **NotificationToast** (contextual hints)
3. **ConfirmationDialog** (help confirmation)

---

## 📊 Component Statistics

### Overall Metrics
- **Total Components**: 16
- **Total Stories**: 87
- **Total Categories**: 6 (Display, Input, Feedback, Workflow, Help, File System)
- **Total Lines of Code**: ~5,500 lines
- **Average Stories per Component**: 5.4

### By Category
| Category | Components | Stories | Percentage |
|----------|-----------|---------|------------|
| Display | 7 | 40 | 46% |
| Input | 3 | 18 | 21% |
| Feedback | 3 | 18 | 21% |
| Workflow | 1 | 6 | 7% |
| Help | 1 | 4 | 5% |
| File System | 1 | 7 | 8% |

### Complexity Distribution
| Stories per Component | Count | Components |
|----------------------|-------|------------|
| 3-4 stories | 5 | SimpleText, CommandPreview, ConfirmationDialog, SafetyIndicator, KeyboardShortcuts |
| 6-7 stories | 10 | TableSelector, CommandEditor, CommandOutputViewer, HistoryTimeline, GenerationComparison, MetricDashboard, CommandRating, FileBrowser, CommandFlow, ProgressSpinner |
| 8+ stories | 1 | NotificationToast |

### Test Coverage
- **Total Tests**: 87+ test cases
- **Components with Tests**: 12/16 (75%)
- **Test Types**: Unit tests, metadata validation, rendering tests

---

## 🚀 Running the Showcase

### Quick Start

```bash
# Basic run
cargo run --bin tui-showcase

# With hot-reload for development
cargo watch -x 'run --bin tui-showcase'

# Build optimized version
cargo build --release --bin tui-showcase
./target/release/tui-showcase
```

### Navigation

In the showcase:
- **↑/↓** or **j/k**: Navigate components and stories
- **Enter**: Select component or view story
- **Backspace**: Go back to previous view
- **h**: Toggle help screen
- **q** or **Esc**: Quit application or close help

### Finding Components

1. Launch the showcase
2. Use arrow keys to browse categories
3. Press Enter to see components in a category
4. Press Enter again to view component stories
5. Navigate through stories to see different states

---

## 🎨 Design Philosophy

All components in this gallery follow consistent design principles:

### 1. Isolation
Each component is completely independent and self-contained. Components don't depend on external state or other components.

### 2. Stories
Multiple variations (stories) demonstrate different states:
- Default/basic state
- Loading/processing states
- Error/warning states
- Empty states
- Edge cases (very long content, etc.)

### 3. Consistency
Uniform styling and interaction patterns across all components:
- Color scheme (cyan for primary, green for success, red for danger, yellow for warnings)
- Border styles (rounded for standard, double for emphasis)
- Spacing and alignment conventions

### 4. Accessibility
Keyboard-first navigation and clear visual feedback:
- All interactions via keyboard
- Clear focus indicators
- High contrast colors
- Screen reader friendly (where applicable)

### 5. Documentation
Self-documenting through examples:
- Component metadata describes purpose
- Story names explain what's being shown
- Visual previews match actual output
- Code examples show implementation

---

## 🔮 Usage Recommendations

### When to Use Each Component

**For displaying text:**
- Simple content → SimpleText
- Commands → CommandPreview
- Output → CommandOutputViewer
- Timeline → HistoryTimeline

**For user input:**
- Binary choice → ConfirmationDialog
- Multi-line text → CommandEditor
- Voting → CommandRating

**For feedback:**
- Risk level → SafetyIndicator
- Loading → ProgressSpinner
- Notifications → NotificationToast

**For workflows:**
- Multi-step processes → CommandFlow

**For help:**
- Keyboard reference → KeyboardShortcuts

**For file systems:**
- Directory trees → FileBrowser

---

## 📚 Additional Resources

- **Development Guide**: See [TUI_SHOWCASE.md](TUI_SHOWCASE.md) for component creation tutorial
- **Contribution Guide**: See [CONTRIBUTING_TUI.md](CONTRIBUTING_TUI.md) for submission guidelines
- **Onboarding Guide**: See [TUI_ONBOARDING.md](TUI_ONBOARDING.md) for newcomer tutorial
- **Source Code**: All components in `src/tui/components/`
- **Showcase Binary**: Run with `cargo run --bin tui-showcase`

---

**Built with ❤️ using [Ratatui](https://ratatui.rs/)**

**Last Updated**: 2025-01-19
**Component Count**: 16
**Story Count**: 87
**Total LOC**: ~5,500
