# Quick Start: Recording cmdai Demo

Get a professional demo recorded in 5 minutes.

## Prerequisites

```bash
# Install asciinema
brew install asciinema  # macOS
# or
apt-get install asciinema  # Linux

# Install agg for GIF conversion
cargo install agg
```

## Option 1: Automated Recording (Recommended)

```bash
# Navigate to demo directory
cd demo

# Make scripts executable
chmod +x *.sh

# Record full demo (2-3 minutes)
make record-full

# Play it back to review
make play-full

# Convert to GIF
make gif-full

# Done! Your GIF is ready: cmdai_demo.gif
```

## Option 2: Quick README GIF (30 seconds)

```bash
cd demo

# Record short version
make record-short

# Convert to GIF (optimized for GitHub)
make gif-short

# Done! Ready for README: cmdai_demo_short.gif
```

## Option 3: Manual Recording

```bash
cd demo

# Start recording
asciinema rec my_demo.cast

# Follow the storyboard
cat DEMO_STORYBOARD.md

# Type commands naturally, paste outputs

# End recording with Ctrl+D

# Convert to GIF
agg my_demo.cast my_demo.gif --speed 2.0 --fps 15
```

## Verify Your Demo

```bash
# Check file size (should be < 10MB for GitHub)
ls -lh *.gif

# Play back
asciinema play cmdai_demo_full.cast

# If too large, optimize
make optimize-gifs
```

## Add to README

```markdown
![cmdai demo](demo/cmdai_demo_short.gif)
```

## Upload to asciinema.org

```bash
make upload-full
# or
asciinema upload cmdai_demo_full.cast
```

## Troubleshooting

**Recording not starting?**
```bash
# Check asciinema is installed
which asciinema

# Check script permissions
ls -l *.sh
```

**GIF too large?**
```bash
# Reduce speed (fewer frames)
make gif-short GIF_SPEED=3.0

# Or reduce FPS
make gif-short GIF_FPS=10
```

**Colors not showing?**
```bash
# Set terminal color support
export TERM=xterm-256color
export COLORTERM=truecolor
```

## Next Steps

- Read [DEMO_STORYBOARD.md](DEMO_STORYBOARD.md) for scene details
- Check [demo_recording_guide.md](demo_recording_guide.md) for advanced tips
- Customize scripts for your needs

## Help

```bash
make help
```

Shows all available commands and options.
