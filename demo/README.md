# cmdai Demo Scripts

This directory contains ASCII cinema demo scripts and storyboards for showcasing cmdai's features.

## Files

- **DEMO_STORYBOARD.md** - Comprehensive storyboard with scene-by-scene breakdown
- **demo_script.sh** - Full demo script (2-3 minutes)
- **demo_short.sh** - Short loop demo (30 seconds, optimized for GIFs)
- **demo_recording_guide.md** - Instructions for recording and publishing demos

## Quick Start

### Record Full Demo

```bash
# Make script executable
chmod +x demo_script.sh

# Record with asciinema
asciinema rec cmdai_demo.cast -c ./demo_script.sh

# Optional: Convert to GIF
agg cmdai_demo.cast cmdai_demo.gif --speed 1.5

# Optional: Convert to SVG
svg-term --in cmdai_demo.cast --out cmdai_demo.svg --window
```

### Record Short Loop (for README)

```bash
chmod +x demo_short.sh
asciinema rec cmdai_demo_short.cast -c ./demo_short.sh
agg cmdai_demo_short.cast cmdai_demo_short.gif --speed 2.0
```

## Requirements

### For Recording

- **asciinema** - Terminal session recorder
  ```bash
  # macOS
  brew install asciinema

  # Linux
  apt-get install asciinema

  # Python
  pip install asciinema
  ```

### For GIF Conversion

- **agg** - asciinema GIF generator (recommended)
  ```bash
  cargo install agg
  ```

- **Alternative: asciicast2gif** (deprecated)
  ```bash
  npm install -g asciicast2gif
  ```

### For SVG Conversion

- **svg-term-cli**
  ```bash
  npm install -g svg-term-cli
  ```

## Terminal Setup

For best results, configure your terminal:

```bash
# Terminal size
export COLUMNS=100
export LINES=30

# Colors (for testing)
export TERM=xterm-256color

# Optional: Set demo speed
export DEMO_SPEED=50  # characters per second
```

## Demo Content

### Full Demo (demo_script.sh)

**Duration**: ~2-3 minutes

**Scenes**:
1. Title card with branding
2. Simple file listing query
3. Complex multi-condition query
4. Dangerous command blocked (safety feature)
5. Interactive confirmation workflow
6. Verbose mode with debug info
7. JSON output format
8. Cross-shell support (Fish example)
9. Real-world workflow example
10. Feature summary and call-to-action

### Short Demo (demo_short.sh)

**Duration**: ~30 seconds (loops seamlessly)

**Scenes**:
1. Simple query
2. Complex query
3. Safety blocking
4. Interactive confirmation
5. Feature banner

Optimized for:
- GitHub README header GIF
- Social media posts
- Quick feature previews
- File size < 10MB

## Customization

### Adjust Typing Speed

```bash
# Faster (80 chars/second)
DEMO_SPEED=80 ./demo_script.sh

# Slower (30 chars/second)
DEMO_SPEED=30 ./demo_script.sh
```

### Modify Pause Durations

Edit the script variables:
```bash
PAUSE_SHORT=1.5   # Short pauses between actions
PAUSE_MEDIUM=2.5  # Medium pauses for reading
PAUSE_LONG=4.0    # Long pauses for emphasis
```

### Custom Terminal Colors

The scripts use standard ANSI color codes. To customize:

1. Edit color definitions in the script header
2. Adjust your terminal's color scheme
3. Use a custom `asciinema` theme

## Publishing

### To asciinema.org

```bash
# Record and upload directly
asciinema rec
# Press Ctrl+D when done, follow upload prompts

# Or upload existing recording
asciinema upload cmdai_demo.cast
```

### To GitHub README

```bash
# Convert to GIF
agg cmdai_demo_short.cast demo.gif --speed 2.0

# Add to README.md
# ![cmdai demo](demo/demo.gif)
```

### To Website

```bash
# Embed player
<script id="asciicast-123456"
        src="https://asciinema.org/a/123456.js"
        async>
</script>

# Or use asciinema-player
<asciinema-player src="/demo.cast"></asciinema-player>
```

## Tips for Great Recordings

### Before Recording

1. **Clear your terminal history**: `clear && history -c`
2. **Set consistent terminal size**: `resize -s 30 100`
3. **Check colors**: Run `demo_script.sh` once to verify
4. **Close distractions**: Disable notifications, close other apps

### During Recording

1. **Don't rush**: Let viewers read the output
2. **Avoid interruptions**: No accidental keypresses
3. **Check timing**: Watch for natural pacing
4. **Monitor file size**: Keep GIFs under 10MB for GitHub

### After Recording

1. **Review the output**: Play it back with `asciinema play demo.cast`
2. **Trim if needed**: Use `asciinema cat` with timestamps
3. **Optimize GIF size**: Use compression and frame reduction
4. **Test on different backgrounds**: Ensure readability

## Troubleshooting

### Colors Not Showing

```bash
# Check terminal support
echo $TERM  # Should be xterm-256color or similar

# Force color support
export COLORTERM=truecolor
```

### Timing Issues

```bash
# Speed up playback
asciinema play -s 2 demo.cast  # 2x speed

# Slow down
asciinema play -s 0.5 demo.cast  # 0.5x speed
```

### GIF Too Large

```bash
# Reduce frames per second
agg demo.cast demo.gif --fps 10

# Increase speed (fewer frames)
agg demo.cast demo.gif --speed 3.0

# Reduce dimensions (if terminal was too large)
# Re-record with smaller terminal: resize -s 24 80
```

### Recording Artifacts

```bash
# Clear screen before starting
clear && sleep 1 && asciinema rec demo.cast

# Reset terminal state
reset

# Check for lingering background processes
jobs
```

## Alternative: Manual Recording

If you want more control:

1. Start recording: `asciinema rec manual_demo.cast`
2. Manually type commands from DEMO_STORYBOARD.md
3. Use mock responses (copy-paste with slight delays)
4. End recording: Press `Ctrl+D`

This gives you complete control over timing and pacing.

## Automated Testing

Test that scripts run without errors:

```bash
# Quick syntax check
bash -n demo_script.sh

# Dry run (fast execution)
DEMO_SPEED=500 PAUSE_SHORT=0.1 PAUSE_MEDIUM=0.1 PAUSE_LONG=0.1 ./demo_script.sh

# Full run
./demo_script.sh
```

## Resources

- [asciinema documentation](https://asciinema.org/docs)
- [agg (GIF generator)](https://github.com/asciinema/agg)
- [svg-term-cli](https://github.com/marionebl/svg-term-cli)
- [Terminal color codes](https://gist.github.com/fnky/458719343aabd01cfb17a3a4f7296797)

## Contributing

To improve the demo:

1. Update `DEMO_STORYBOARD.md` with new scenes
2. Modify `demo_script.sh` with new mock responses
3. Test thoroughly before committing
4. Record and attach sample output
5. Update this README with any new requirements

## License

Same as the main cmdai project (AGPL-3.0).
