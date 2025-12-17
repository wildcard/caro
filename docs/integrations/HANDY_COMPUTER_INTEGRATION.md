# Handy.Computer Integration Plan

## Overview

This document outlines the integration strategy between **caro** (cmdai) and **Handy.Computer**, an open-source push-to-talk speech-to-text application for macOS.

## About Handy.Computer

### Project Information
- **Developer**: CJ Pais ([@cjpais](https://github.com/cjpais))
- **Website**: [handy.computer](https://handy.computer/)
- **Repository**: [github.com/cjpais/Handy](https://github.com/cjpais/Handy)
- **License**: MIT License
- **Contact**: contact@handy.computer
- **Philosophy**: "Isn't trying to be the best speech-to-text app—it's trying to be the most forkable one"

### Attribution & Acknowledgment

**CJ Pais** is an avid open source contributor and software artist who has created Handy.Computer as a privacy-first, offline speech-to-text solution. The project exemplifies excellent open-source practices:
- Privacy-focused: All voice processing happens locally
- Cross-platform: Windows, macOS (Intel/Apple Silicon), Linux
- Extensible: Built to be forked and modified
- Community-driven: Active development with multiple contributors

We deeply appreciate CJ's contributions to the open source community and the excellent work done on Handy.Computer.

## Technical Architecture

### Core Components

**Technology Stack:**
- **Framework**: Tauri (Rust backend + React/TypeScript frontend)
- **Audio Processing**: CPAL (cross-platform audio capture)
- **Voice Activity Detection**: Silero VAD
- **Speech-to-Text Models**:
  - Whisper (Small/Medium/Turbo/Large) with GPU acceleration
  - Parakeet V3 with automatic language detection
- **System Integration**:
  - Global shortcuts via rdev library
  - Platform-specific clipboard text injection

**Architecture Highlights:**
- Offline-first design
- Local model execution (no cloud dependencies)
- GPU-accelerated inference on supported platforms
- Configurable keyboard shortcuts
- Debug mode accessible via keyboard shortcut

### Configuration & Data

**macOS/Linux Configuration Path:**
```
~/.config/com.pais.handy/
```

**Windows Configuration Path:**
```
%APPDATA%\com.pais.handy\
```

## Integration Points

### 1. Process Detection

**Capability**: Detect if Handy is installed and running on the system

**Implementation Strategy:**
```rust
// Check if Handy process is running
fn is_handy_running() -> bool {
    // Platform-specific process detection
    // macOS: Use `pgrep` or process listing
    // Linux: Check /proc filesystem
    // Look for "Handy" or "com.pais.handy" process
}

// Check if Handy is installed
fn is_handy_installed() -> Option<PathBuf> {
    // macOS: Check /Applications/Handy.app
    // Linux: Check common installation paths
    // Also check config directory existence
}
```

### 2. IPC via SIGUSR2 Signal (Primary Integration Method)

**Capability**: Toggle Handy recording externally using Unix signals

**Documentation from Handy README:**
> "Sending SIGUSR2 to the Handy process toggles recording on/off"

**Implementation Strategy:**
```rust
use std::process::Command;

fn toggle_handy_recording() -> Result<(), std::io::Error> {
    // Find Handy process ID
    let output = Command::new("pgrep")
        .arg("Handy")
        .output()?;

    if let Ok(pid_str) = String::from_utf8(output.stdout) {
        if let Ok(pid) = pid_str.trim().parse::<i32>() {
            // Send SIGUSR2 to toggle recording
            Command::new("kill")
                .arg("-SIGUSR2")
                .arg(pid.to_string())
                .status()?;
            return Ok(());
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "Handy process not found"
    ))
}
```

**Platform Support:**
- ✅ macOS: Full support
- ✅ Linux: Full support
- ⚠️ Windows: Not supported (Windows doesn't support SIGUSR2)

### 3. Clipboard Monitoring

**Capability**: Detect transcribed text from Handy via clipboard

**Implementation Strategy:**
```rust
use clipboard::{ClipboardProvider, ClipboardContext};

struct HandyClipboardMonitor {
    last_content: String,
    callback: Box<dyn Fn(String) + Send>,
}

impl HandyClipboardMonitor {
    fn new(callback: impl Fn(String) + Send + 'static) -> Self {
        Self {
            last_content: String::new(),
            callback: Box::new(callback),
        }
    }

    fn poll(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut ctx: ClipboardContext = ClipboardProvider::new()?;
        if let Ok(content) = ctx.get_contents() {
            if content != self.last_content && !content.is_empty() {
                self.last_content = content.clone();
                (self.callback)(content);
            }
        }
        Ok(())
    }
}
```

### 4. Configuration File Integration

**Capability**: Read Handy configuration to understand user preferences

**Configuration Format**: JSON (based on Tauri architecture)

**Potential Integration:**
```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct HandyConfig {
    keyboard_shortcut: Option<String>,
    model: Option<String>,
    paste_method: Option<String>,
}

fn read_handy_config() -> Result<HandyConfig, Box<dyn std::error::Error>> {
    let config_path = if cfg!(target_os = "macos") || cfg!(target_os = "linux") {
        dirs::config_dir()
            .ok_or("Config dir not found")?
            .join("com.pais.handy")
            .join("config.json")
    } else {
        // Windows path
        dirs::config_dir()
            .ok_or("Config dir not found")?
            .join("com.pais.handy")
            .join("config.json")
    };

    let content = std::fs::read_to_string(config_path)?;
    let config: HandyConfig = serde_json::from_str(&content)?;
    Ok(config)
}
```

## Integration Architecture for Caro

### Phase 1: Detection & User Awareness

**Goal**: Inform users about Handy.Computer integration capabilities

**Implementation:**
1. On caro startup, check if Handy is installed
2. Display integration status in terminal UI:
   ```
   ┌─────────────────────────────────────────┐
   │ 🎤 Push-to-talk available via Handy     │
   │ Press Ctrl+Space to activate            │
   │                                         │
   │ Not installed? Download at:             │
   │ https://handy.computer                  │
   └─────────────────────────────────────────┘
   ```

**User Experience:**
```rust
// Example UI integration
fn show_welcome_screen() {
    println!("Welcome to caro - Natural Language Shell Commands");

    if let Some(handy_path) = is_handy_installed() {
        if is_handy_running() {
            println!("✅ Handy.Computer detected and running");
            println!("💡 Press your Handy shortcut to speak commands");
        } else {
            println!("⚠️  Handy.Computer installed but not running");
            println!("   Start Handy to enable voice input");
        }
    } else {
        println!("💡 Enhanced voice input available with Handy.Computer");
        println!("   Download at: https://handy.computer");
    }
}
```

### Phase 2: Terminal UI Integration

**Goal**: Create seamless push-to-talk experience in caro TUI

**UI Components:**

1. **Microphone Button** (if terminal supports mouse input)
   ```
   ┌────────────────────────────────────┐
   │ 🎤 Click to speak | Type command   │
   │ > _                                │
   └────────────────────────────────────┘
   ```

2. **Keyboard Shortcut Indicator**
   ```
   ┌────────────────────────────────────┐
   │ Type: > command                    │
   │ Voice: Press Ctrl+Space (Handy)    │
   └────────────────────────────────────┘
   ```

3. **Recording Status Display**
   ```
   ┌────────────────────────────────────┐
   │ 🔴 Listening... (speak now)        │
   │                                    │
   │ Release to process                 │
   └────────────────────────────────────┘
   ```

**Implementation with TUI Library:**
```rust
use ratatui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, Paragraph},
    layout::{Layout, Constraint, Direction},
    Terminal,
};

struct CaroTUI {
    handy_available: bool,
    recording: bool,
}

impl CaroTUI {
    fn render_input_area(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Input area
                Constraint::Length(1),  // Status bar
            ])
            .split(frame.size());

        // Input area
        let input_block = Block::default()
            .title("Command Input")
            .borders(Borders::ALL);

        // Status bar with Handy integration
        let status = if self.handy_available {
            if self.recording {
                "🔴 Recording via Handy..."
            } else {
                "🎤 Voice input ready (Handy.Computer)"
            }
        } else {
            "Type command or install Handy for voice input"
        };

        let status_bar = Paragraph::new(status)
            .block(Block::default());

        frame.render_widget(input_block, chunks[0]);
        frame.render_widget(status_bar, chunks[1]);
    }
}
```

### Phase 3: Direct Integration (Advanced)

**Goal**: Programmatic control of Handy from caro

**Features:**
1. **Automatic Recording Trigger**
   - Send SIGUSR2 to start/stop Handy recording
   - Monitor clipboard for transcription results
   - Auto-populate command input field

2. **Bidirectional Communication**
   - Start Handy recording when user activates voice mode
   - Detect transcription completion
   - Handle errors and timeouts

**Implementation:**
```rust
use tokio::time::{Duration, timeout};

struct HandyIntegration {
    monitor: HandyClipboardMonitor,
    handy_pid: Option<i32>,
}

impl HandyIntegration {
    async fn capture_voice_command(&mut self) -> Result<String, HandyError> {
        // 1. Clear clipboard
        self.clear_clipboard()?;

        // 2. Trigger Handy recording
        self.toggle_handy_recording()?;

        // 3. Wait for user to speak
        // User will release their push-to-talk key when done

        // 4. Monitor clipboard for transcription (with timeout)
        let transcription = timeout(
            Duration::from_secs(30),
            self.wait_for_clipboard_change()
        ).await??;

        Ok(transcription)
    }

    async fn wait_for_clipboard_change(&mut self) -> Result<String, HandyError> {
        loop {
            if let Some(text) = self.monitor.check_for_new_content()? {
                return Ok(text);
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
}
```

### Phase 4: Enhanced User Experience

**Goal**: Seamless voice-to-command workflow

**Features:**

1. **Visual Feedback**
   ```
   🎤 Speak now...
   ├─ Listening via Handy
   └─ Processing transcription...

   📝 Transcribed: "find all python files"

   🤖 Generating command...
   ├─ Using MLX backend
   └─ Applying safety checks...

   ✅ Command: find . -name "*.py"
   ⚠️  Risk Level: Safe

   Execute? [Y/n]
   ```

2. **Error Handling**
   ```
   ❌ Handy transcription timeout
   💡 Tips:
      • Ensure Handy is running
      • Check your microphone settings
      • Try speaking more clearly

   [R]etry | [T]ype manually | [Q]uit
   ```

3. **Configuration**
   ```toml
   # caro config.toml
   [voice_input]
   enabled = true
   integration = "handy"  # Options: "handy", "whisper", "manual"

   [voice_input.handy]
   auto_detect = true
   clipboard_timeout_ms = 30000
   show_transcription = true
   ```

## Implementation Roadmap

### Milestone 1: Basic Detection (Week 1)
- [ ] Implement Handy process detection
- [ ] Implement Handy installation detection
- [ ] Add welcome screen integration notice
- [ ] Document installation instructions

### Milestone 2: SIGUSR2 Integration (Week 2)
- [ ] Implement signal-based recording toggle
- [ ] Add error handling for process not found
- [ ] Test on macOS and Linux
- [ ] Document platform limitations

### Milestone 3: Clipboard Monitoring (Week 3)
- [ ] Implement clipboard change detection
- [ ] Add timeout handling
- [ ] Test transcription capture
- [ ] Handle edge cases (empty clipboard, etc.)

### Milestone 4: Terminal UI (Week 4-5)
- [ ] Design TUI layout with voice input indicators
- [ ] Implement mouse-based microphone button
- [ ] Add recording status display
- [ ] Create keyboard shortcut hints

### Milestone 5: End-to-End Integration (Week 6)
- [ ] Connect all components
- [ ] Implement voice-to-command workflow
- [ ] Add comprehensive error handling
- [ ] Create user documentation

### Milestone 6: Polish & Testing (Week 7)
- [ ] User testing and feedback
- [ ] Performance optimization
- [ ] Documentation updates
- [ ] Attribution and acknowledgment

## Dependencies

### Rust Crates Needed

```toml
[dependencies]
# Process management
sysinfo = "0.30"           # Cross-platform process detection
libc = "0.2"               # Unix signal handling

# Clipboard integration
copypasta = "0.10"         # Cross-platform clipboard

# TUI components
ratatui = "0.26"           # Terminal UI framework
crossterm = "0.27"         # Terminal manipulation

# Configuration
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Async runtime
tokio = { version = "1.35", features = ["full"] }

# Platform detection
cfg-if = "1.0"
```

## Platform Support Matrix

| Feature | macOS | Linux | Windows |
|---------|-------|-------|---------|
| Handy Detection | ✅ | ✅ | ✅ |
| SIGUSR2 Integration | ✅ | ✅ | ❌ |
| Clipboard Monitoring | ✅ | ✅ | ✅ |
| TUI Integration | ✅ | ✅ | ✅ |
| Mouse Input | Terminal-dependent | Terminal-dependent | Terminal-dependent |

**Note**: Windows support for SIGUSR2 is not available due to OS limitations. Alternative IPC methods would need to be explored for Windows (e.g., named pipes, HTTP API).

## Security Considerations

1. **Process Validation**: Verify Handy process signature before sending signals
2. **Clipboard Security**: Clear sensitive data from clipboard after capture
3. **Permission Checks**: Ensure user consent for clipboard monitoring
4. **Error Handling**: Gracefully handle Handy process crashes or termination

## User Documentation

### Installation Guide

```markdown
# Voice Input with Handy.Computer

caro supports voice input through integration with Handy.Computer,
an open-source push-to-talk speech-to-text application.

## Installing Handy.Computer

1. Download from: https://handy.computer
2. Install the application
3. Launch Handy and configure your preferred keyboard shortcut
4. Start caro - it will automatically detect Handy

## Using Voice Input in caro

### Automatic Detection
caro automatically detects when Handy is running and enables
voice input integration.

### Push-to-Talk Workflow
1. Start caro terminal UI
2. Press your Handy keyboard shortcut (e.g., Ctrl+Space)
3. Speak your command naturally
4. Release the shortcut to process
5. caro receives the transcription and generates the command

### Manual Typing
You can always type commands manually - voice input is optional!

## Troubleshooting

**Handy not detected:**
- Ensure Handy is running
- Check if caro has permission to detect processes

**Transcription not captured:**
- Verify Handy is transcribing correctly (test in another app)
- Check clipboard permissions
- Try increasing the timeout in caro config

## Attribution

Voice integration powered by Handy.Computer by CJ Pais
https://github.com/cjpais/Handy
```

## Acknowledgments & Attribution

### In caro CLI
```
$ caro --version
caro 0.1.0
Voice input integration powered by Handy.Computer by CJ Pais
https://github.com/cjpais/Handy
```

### In README.md
```markdown
## Voice Input

caro integrates with [Handy.Computer](https://handy.computer),
an excellent open-source push-to-talk speech-to-text application
created by [CJ Pais](https://github.com/cjpais).

We're grateful for CJ's contributions to the open source community
and the outstanding work on making local, privacy-first voice input
accessible to everyone.
```

### In About/Help Menu
```
Voice Input Integration
━━━━━━━━━━━━━━━━━━━━━
Powered by: Handy.Computer
Developer:  CJ Pais (@cjpais)
License:    MIT
Website:    https://handy.computer
Repository: https://github.com/cjpais/Handy

Handy is a free, open source, and extensible speech-to-text
application that works completely offline and respects your privacy.
```

## Future Enhancements

### Potential Features
1. **Direct API Integration**: If Handy exposes an API in the future
2. **Custom Model Selection**: Allow users to select Whisper model from caro
3. **Voice Command Chaining**: Support multi-step voice commands
4. **Handy Settings Sync**: Respect Handy's configured shortcuts in caro UI
5. **Windows Support**: Explore alternative IPC methods for Windows
6. **Voice Feedback**: Audio confirmation of command generation

### Community Collaboration
We plan to:
- Contribute improvements back to Handy project
- Share integration patterns with other CLI tools
- Collaborate with CJ Pais on API design if desired
- Document integration as a reference for other projects

## Resources

### Official Documentation
- Handy Website: https://handy.computer
- Handy GitHub: https://github.com/cjpais/Handy
- Handy CLI: https://github.com/cjpais/handy-cli
- Developer Contact: contact@handy.computer

### Related Projects
- Whisper.cpp: https://github.com/ggerganov/whisper.cpp
- Tauri Framework: https://tauri.app/
- Similar tools: Wispr Flow, Willow

### Attribution Requirements (MIT License)
```
MIT License

Copyright (c) 2024 CJ Pais

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.
```

---

**Document Version**: 1.0
**Last Updated**: 2025-12-17
**Author**: caro development team
**Status**: Planning Phase
