# cmdai Demo Recording Guide

This guide provides detailed instructions for recording, producing, and publishing high-quality terminal demos for cmdai.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Environment Setup](#environment-setup)
3. [Recording Process](#recording-process)
4. [Post-Production](#post-production)
5. [Publishing](#publishing)
6. [Best Practices](#best-practices)
7. [Troubleshooting](#troubleshooting)

---

## Prerequisites

### Required Software

#### 1. asciinema (Terminal Recorder)

**macOS**:
```bash
brew install asciinema
```

**Linux (Debian/Ubuntu)**:
```bash
apt-get install asciinema
```

**Linux (Fedora)**:
```bash
dnf install asciinema
```

**From Python**:
```bash
pip install asciinema
```

**Verify installation**:
```bash
asciinema --version
# Should output: asciinema 2.x.x
```

#### 2. agg (GIF Generator) - Recommended

**Using Cargo**:
```bash
cargo install agg
```

**Verify installation**:
```bash
agg --version
```

#### 3. Alternative GIF Tools (Optional)

**asciicast2gif** (deprecated but still works):
```bash
npm install -g asciicast2gif
```

**gifski** (for high-quality GIFs):
```bash
brew install gifski  # macOS
cargo install gifski  # Cross-platform
```

#### 4. svg-term-cli (For SVG Output)

```bash
npm install -g svg-term-cli
```

---

## Environment Setup

### 1. Terminal Configuration

#### Set Terminal Size

For consistent recordings across all platforms:

```bash
# Standard demo size (works well for most screens)
resize -s 30 100  # 30 rows, 100 columns

# Alternative: Set via environment variables
export COLUMNS=100
export LINES=30
```

**Why this size?**
- 100 columns: Wide enough for complex commands
- 30 rows: Enough context without too much scrolling
- Fits well in 1920x1080 displays
- Generates reasonable GIF file sizes

#### Terminal Color Scheme

**Recommended**: Use a dark theme with high contrast

**Popular choices**:
- Dracula
- Nord
- Solarized Dark
- One Dark
- Monokai

**Test your colors**:
```bash
# Run the demo script once to preview
./demo_script.sh
```

#### Font Selection

**Recommended monospace fonts**:
- JetBrains Mono (excellent for code)
- Fira Code (great ligatures)
- SF Mono (native macOS)
- Cascadia Code (Windows Terminal)
- Hack

**Size**: 13-15px works well for most recordings

### 2. Shell Configuration

#### Clean Environment

Create a dedicated demo profile to avoid personal info exposure:

**For Bash** (`~/.bash_profile_demo`):
```bash
# Minimal prompt
export PS1="\[\033[92m\]$\[\033[0m\] "

# Clean environment
export HISTFILE=/dev/null  # Don't save history
export LANG=en_US.UTF-8

# Demo-specific settings
export DEMO_MODE=1
export COLUMNS=100
export LINES=30
```

**For Zsh** (`~/.zshrc_demo`):
```zsh
# Minimal prompt
PROMPT='%F{green}$%f '

# Clean environment
HISTFILE=/dev/null
export LANG=en_US.UTF-8

# Demo settings
export DEMO_MODE=1
export COLUMNS=100
export LINES=30
```

**Use demo profile**:
```bash
bash --rcfile ~/.bash_profile_demo
# or
zsh --rcs ~/.zshrc_demo
```

### 3. Working Directory

```bash
# Create dedicated demo directory
mkdir -p ~/cmdai-demo
cd ~/cmdai-demo

# Copy demo scripts
cp -r /path/to/cmdai/demo/* .

# Make scripts executable
chmod +x *.sh
```

### 4. System Preparation

```bash
# Clear terminal
clear

# Clear command history
history -c

# Close unnecessary applications
# Disable notifications (macOS)
# System Preferences → Notifications → Do Not Disturb

# Check for background processes
jobs
# Kill any if needed

# Verify clean slate
ps aux | grep -i demo
```

---

## Recording Process

### Method 1: Automated Script (Recommended)

#### Full Demo (2-3 minutes)

```bash
# Navigate to demo directory
cd ~/cmdai-demo

# Start recording
asciinema rec cmdai_demo_full.cast -c ./demo_script.sh

# The script will run automatically and exit when done
```

#### Short Demo (30 seconds)

```bash
asciinema rec cmdai_demo_short.cast -c ./demo_short.sh
```

#### Custom Speed

```bash
# Faster typing (80 chars/sec)
DEMO_SPEED=80 asciinema rec fast_demo.cast -c ./demo_script.sh

# Slower typing (30 chars/sec)
DEMO_SPEED=30 asciinema rec slow_demo.cast -c ./demo_script.sh
```

### Method 2: Manual Recording

For more control over timing and pacing:

```bash
# Start recording
asciinema rec cmdai_demo_manual.cast

# Now manually perform the demo:
# 1. Follow DEMO_STORYBOARD.md scene by scene
# 2. Type naturally (50-70 WPM)
# 3. Paste mock outputs from storyboard
# 4. Add natural pauses (2-3 seconds between scenes)

# End recording
# Press Ctrl+D when done
```

**Tips for manual recording**:
- Keep DEMO_STORYBOARD.md open in another window
- Use copy-paste for long outputs (looks natural)
- Take your time - you can speed up later
- Don't worry about mistakes - record until you get it right

### Method 3: Interactive Recording

```bash
# Start interactive recording
asciinema rec cmdai_demo_interactive.cast

# Type commands naturally
# Actually run cmdai (if available)
# Or use mock responses

# End recording
# Ctrl+D
```

---

## Post-Production

### 1. Review Recording

```bash
# Play back your recording
asciinema play cmdai_demo_full.cast

# Play at different speeds
asciinema play -s 1.5 cmdai_demo_full.cast  # 1.5x speed
asciinema play -s 0.8 cmdai_demo_full.cast  # 0.8x speed

# Pause/resume with Space bar
# Quit with Ctrl+C
```

### 2. Trim Recording (If Needed)

```bash
# Cut specific time range (30s to 2m30s)
asciinema cat --start 30 --end 150 cmdai_demo_full.cast > trimmed.cast

# Save to new file
asciinema cat cmdai_demo_full.cast > cmdai_demo_final.cast
```

### 3. Convert to GIF

#### Using agg (Recommended)

```bash
# Standard conversion
agg cmdai_demo_full.cast cmdai_demo.gif

# Custom speed (2x faster)
agg cmdai_demo_full.cast cmdai_demo.gif --speed 2.0

# Custom FPS (reduce file size)
agg cmdai_demo_full.cast cmdai_demo.gif --fps 10

# Custom dimensions (if needed)
agg cmdai_demo_full.cast cmdai_demo.gif --cols 80 --rows 24

# Optimized for GitHub (< 10MB)
agg cmdai_demo_short.cast cmdai_demo.gif --speed 2.0 --fps 15
```

#### Using asciicast2gif (Alternative)

```bash
asciicast2gif cmdai_demo_full.cast cmdai_demo.gif

# With custom theme
asciicast2gif -t monokai cmdai_demo_full.cast cmdai_demo.gif

# Custom speed
asciicast2gif -s 2 cmdai_demo_full.cast cmdai_demo.gif
```

### 4. Convert to SVG

```bash
# Basic conversion
svg-term --in cmdai_demo_full.cast --out cmdai_demo.svg

# With window frame
svg-term --in cmdai_demo_full.cast --out cmdai_demo.svg --window

# Custom dimensions
svg-term --in cmdai_demo_full.cast --out cmdai_demo.svg --width 100 --height 30

# Custom profile (color scheme)
svg-term --in cmdai_demo_full.cast --out cmdai_demo.svg --profile "~/.terminal-profile"
```

### 5. Optimize File Size

#### For GIFs

```bash
# Method 1: Reduce FPS
agg demo.cast demo.gif --fps 10

# Method 2: Increase speed (fewer frames)
agg demo.cast demo.gif --speed 3.0

# Method 3: Reduce dimensions
agg demo.cast demo.gif --cols 80 --rows 24

# Method 4: Use gifsicle for compression
gifsicle -O3 --colors 256 demo.gif -o demo_optimized.gif

# Method 5: Use ImageMagick
convert demo.gif -fuzz 10% -layers Optimize demo_optimized.gif
```

#### Check file size

```bash
ls -lh *.gif

# Target sizes:
# - GitHub README: < 10MB
# - Social media: < 5MB
# - Website embed: Any size
```

---

## Publishing

### 1. Upload to asciinema.org

```bash
# Upload recording
asciinema upload cmdai_demo_full.cast

# Or record and upload in one step
asciinema rec
# Ctrl+D when done, follow prompts

# Get shareable link
# Example: https://asciinema.org/a/123456
```

**Embedding on website**:
```html
<script id="asciicast-123456"
        src="https://asciinema.org/a/123456.js"
        async
        data-autoplay="true"
        data-loop="true">
</script>
```

### 2. Add to GitHub Repository

#### As GIF in README

```bash
# Copy GIF to repository
cp cmdai_demo.gif /path/to/cmdai/demo/

# Add to README.md
```

```markdown
# cmdai

![cmdai demo](demo/cmdai_demo.gif)

Convert natural language to safe shell commands...
```

#### As Cast File

```bash
# Add .cast file to repository
cp cmdai_demo_full.cast /path/to/cmdai/demo/

# Link in README
```

```markdown
## Demo

View the [full demo recording](demo/cmdai_demo_full.cast) or watch below:

[![asciicast](https://asciinema.org/a/123456.svg)](https://asciinema.org/a/123456)
```

### 3. Share on Social Media

#### Twitter/X

```markdown
🚀 Introducing cmdai: Natural language → Shell commands

✓ Safety-first design
✓ Local LLM inference
✓ Interactive confirmations
✓ POSIX-compliant

[GIF embedded]

github.com/wildcard/cmdai
```

#### LinkedIn

```markdown
I'm excited to share cmdai, an open-source CLI tool that converts
natural language to safe shell commands using local LLMs.

Key features:
• Comprehensive safety validation
• Interactive confirmation for risky operations
• Apple Silicon optimized (MLX)
• Multiple output formats
• Cross-shell support

[GIF embedded]

Check it out: github.com/wildcard/cmdai
```

#### Reddit (r/programming, r/rust, r/commandline)

```markdown
Title: [Project] cmdai - Natural language to shell commands with safety validation

Body:
I've been working on cmdai, a Rust CLI tool that converts natural
language descriptions into POSIX shell commands using local LLMs.

[GIF embedded]

Key highlights:
- Safety-first: Blocks dangerous commands, interactive confirmations
- Local inference: Apple Silicon optimized with MLX
- Fast: <2s inference on M1 Mac
- Multiple backends: MLX, Ollama, vLLM

Demo: [link to asciinema]
GitHub: github.com/wildcard/cmdai

Would love feedback!
```

### 4. Embed on Website

#### Using asciinema-player

```html
<!DOCTYPE html>
<html>
<head>
  <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/asciinema-player/3.0.1/asciinema-player.min.css" />
</head>
<body>
  <div id="demo"></div>

  <script src="https://cdnjs.cloudflare.com/ajax/libs/asciinema-player/3.0.1/asciinema-player.min.js"></script>
  <script>
    AsciinemaPlayer.create('/demo/cmdai_demo_full.cast', document.getElementById('demo'), {
      autoPlay: true,
      loop: true,
      fit: 'width',
      speed: 1.5
    });
  </script>
</body>
</html>
```

#### Using GIF

```html
<img src="/demo/cmdai_demo.gif"
     alt="cmdai demo"
     style="max-width: 100%; border-radius: 8px;" />
```

---

## Best Practices

### Before Recording

- [ ] Clean terminal (clear, history -c)
- [ ] Set consistent terminal size (100x30)
- [ ] Use high-contrast theme
- [ ] Close distractions
- [ ] Disable notifications
- [ ] Test demo script once
- [ ] Check audio if recording video
- [ ] Prepare mock data

### During Recording

- [ ] Type at natural speed (50-70 WPM)
- [ ] Pause 2-3 seconds between commands
- [ ] Allow time to read output
- [ ] Don't rush through scenes
- [ ] Avoid corrections/backspaces
- [ ] Check for typos before hitting Enter
- [ ] Monitor file size for GIF conversion

### After Recording

- [ ] Review full recording
- [ ] Check timing and pacing
- [ ] Verify colors are visible
- [ ] Test on different backgrounds
- [ ] Optimize file size
- [ ] Verify no sensitive information
- [ ] Test on mobile (if web-based)
- [ ] Get feedback before publishing

### Quality Checklist

- [ ] Clear, readable text (font size appropriate)
- [ ] Good contrast (text vs background)
- [ ] Proper pacing (not too fast/slow)
- [ ] No personal information visible
- [ ] No errors or glitches
- [ ] Smooth transitions between scenes
- [ ] File size appropriate for platform
- [ ] Loops smoothly (for GIFs)

---

## Troubleshooting

### Recording Issues

#### "asciinema: command not found"

```bash
# Install asciinema
brew install asciinema  # macOS
apt-get install asciinema  # Linux
pip install asciinema  # Python
```

#### Recording shows garbled characters

```bash
# Set proper locale
export LANG=en_US.UTF-8
export LC_ALL=en_US.UTF-8

# Restart terminal
```

#### Terminal size changes during recording

```bash
# Lock terminal size
resize -s 30 100

# Verify
echo $COLUMNS $LINES
# Should output: 100 30
```

### Playback Issues

#### Colors not displaying

```bash
# Check terminal support
echo $TERM  # Should be xterm-256color or similar

# Force color support
export COLORTERM=truecolor
```

#### Playback too fast/slow

```bash
# Adjust speed during playback
asciinema play -s 1.5 demo.cast  # 1.5x speed
asciinema play -s 0.5 demo.cast  # 0.5x speed
```

### Conversion Issues

#### GIF too large (> 10MB)

```bash
# Option 1: Reduce FPS
agg demo.cast demo.gif --fps 10

# Option 2: Increase playback speed
agg demo.cast demo.gif --speed 3.0

# Option 3: Reduce dimensions
agg demo.cast demo.gif --cols 80 --rows 24

# Option 4: Compress with gifsicle
gifsicle -O3 --colors 128 demo.gif -o compressed.gif
```

#### GIF looks pixelated

```bash
# Use higher quality settings
agg demo.cast demo.gif --fps 20

# Or use larger font size in terminal before recording
```

#### SVG not rendering properly

```bash
# Try different theme
svg-term --in demo.cast --out demo.svg --profile solarized-dark

# Verify cast file
asciinema cat demo.cast
```

### Script Execution Issues

#### "Permission denied" when running script

```bash
chmod +x demo_script.sh
./demo_script.sh
```

#### Script exits immediately

```bash
# Check for errors
bash -n demo_script.sh

# Run with debug
bash -x demo_script.sh
```

#### Colors not showing in script output

```bash
# Ensure TERM is set
export TERM=xterm-256color

# Force color output
export COLORTERM=truecolor
```

---

## Advanced Techniques

### Custom Prompts

Create custom prompt for demo clarity:

```bash
export PS1="\[\033[92m\]cmdai-demo\[\033[0m\] $ "
```

### Add Timestamps

```bash
# Show when each scene starts
echo "[$(date +%H:%M:%S)] Scene 1: Simple query"
```

### Multiple Takes

```bash
# Record take 1
asciinema rec demo_take1.cast -c ./demo_script.sh

# Record take 2 with adjustments
DEMO_SPEED=40 asciinema rec demo_take2.cast -c ./demo_script.sh

# Compare
asciinema play demo_take1.cast
asciinema play demo_take2.cast

# Choose best one
```

### A/B Testing

Create variations for different audiences:

```bash
# Technical audience (verbose, shows internals)
SHOW_DEBUG=1 ./demo_script.sh

# Marketing audience (fast, highlights features)
DEMO_SPEED=80 PAUSE_SHORT=0.8 ./demo_script.sh
```

---

## Resources

### Official Documentation

- [asciinema documentation](https://asciinema.org/docs)
- [agg GitHub](https://github.com/asciinema/agg)
- [asciinema-player](https://github.com/asciinema/asciinema-player)
- [svg-term-cli](https://github.com/marionebl/svg-term-cli)

### Inspiration

Popular CLI tools with great demos:
- [bat](https://github.com/sharkdp/bat) - Cat clone with syntax highlighting
- [exa](https://github.com/ogham/exa) - Modern ls replacement
- [ripgrep](https://github.com/BurntSushi/ripgrep) - Fast grep alternative
- [fd](https://github.com/sharkdp/fd) - Simple find alternative
- [bottom](https://github.com/ClementTsang/bottom) - System monitor

### Tools

- [ANSI escape code reference](https://gist.github.com/fnky/458719343aabd01cfb17a3a4f7296797)
- [Terminal color scheme designer](https://terminal.sexy/)
- [Font recommendations](https://www.programmingfonts.org/)

---

## Contributing

To improve this guide:

1. Test on different platforms (macOS, Linux, Windows)
2. Document platform-specific issues
3. Add screenshots/examples
4. Share best practices from your experience
5. Submit PRs to improve scripts

## License

Same as cmdai project (AGPL-3.0).
