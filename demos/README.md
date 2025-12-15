# Caro.sh VHS Demo Tapes

This directory contains VHS tape files for generating terminal demos inspired by [Atuin](https://github.com/atuinsh/atuin) and [Crush](https://github.com/charmbracelet/crush).

## 📼 Available Demos

### 1. **caro-quickstart.tape** - Quick Start Demo
- **Duration**: ~30 seconds
- **Purpose**: GitHub README, documentation
- **Content**: Basic command generation showcase
- **Theme**: Dracula

### 2. **caro-features.tape** - Feature Showcase
- **Duration**: ~60 seconds  
- **Purpose**: Feature overview, product page
- **Content**: System monitoring, file ops, security, networking
- **Theme**: Catppuccin Mocha

### 3. **vancouver-dev-demo.tape** - Vancouver.Dev Presentation
- **Duration**: ~90 seconds
- **Purpose**: Community presentation, emphasizing open source
- **Content**: Full demo with community messaging
- **Theme**: Tokyo Night

### 4. **caro-before-after.tape** - Before/After Comparison
- **Duration**: ~45 seconds
- **Purpose**: Showing productivity gains, investor pitch
- **Content**: Traditional workflow vs Caro workflow
- **Theme**: Nord

### 5. **caro-social.tape** - Social Media Version
- **Duration**: ~20 seconds
- **Purpose**: Twitter, LinkedIn, short-form content
- **Content**: Rapid-fire command generation
- **Theme**: Catppuccin Mocha

## 🚀 Generating Demos

### Prerequisites

Install VHS:
```bash
# macOS
brew install vhs

# Other platforms
go install github.com/charmbracelet/vhs@latest
```

Ensure binary is built:
```bash
cargo build --release --features embedded-mlx
```

### Generate Single Demo

**Important: Run commands from the `demos/` directory!**

```bash
# Navigate to demos directory
cd demos

# Generate GIF
vhs caro-quickstart.tape

# Or use make
make quickstart

# Generate MP4 instead
# Edit the tape file: Change "Output demos/caro-quickstart.gif" to ".mp4"
```

### Generate All Demos

```bash
# From demos/ directory
cd demos
make all
```

Or manually:
```bash
cd demos
vhs caro-quickstart.tape
vhs caro-features.tape
vhs vancouver-dev-demo.tape
vhs caro-before-after.tape
vhs caro-social.tape
```

## 📊 Demo Specifications

| Demo | Duration | Size (approx) | Use Case |
|------|----------|---------------|----------|
| quickstart | 30s | 1-2MB | README, docs |
| features | 60s | 2-3MB | Product pages |
| vancouver-dev | 90s | 3-4MB | Presentations |
| before-after | 45s | 2MB | Investor pitch |
| social | 20s | 1MB | Social media |

## 🎨 Customization

### Change Theme

Edit the tape file and modify the `Set Theme` line:

```bash
Set Theme "Dracula"           # Dark theme
Set Theme "Catppuccin Mocha"  # Catppuccin dark
Set Theme "Nord"              # Nord theme
Set Theme "Tokyo Night"       # Tokyo Night
Set Theme "Monokai"           # Monokai
```

See all themes: https://github.com/charmbracelet/vhs#themes

### Change Output Format

```bash
# GIF (default)
Output demos/my-demo.gif

# MP4 (better for presentations)
Output demos/my-demo.mp4

# WebM (better for web)
Output demos/my-demo.webm
```

### Adjust Timing

```bash
Set TypingSpeed 80ms          # Typing speed
Set PlaybackSpeed 1.0         # Playback multiplier
Sleep 2s                      # Pause duration
```

## 📝 Creating New Demos

1. **Copy a template**:
   ```bash
   cp demos/caro-quickstart.tape demos/my-demo.tape
   ```

2. **Edit the tape file**:
   - Change output filename
   - Modify theme/dimensions
   - Update commands
   - Adjust timing

3. **Test it**:
   ```bash
   vhs demos/my-demo.tape
   ```

4. **Preview**:
   ```bash
   open demos/my-demo.gif
   ```

## 🎯 Best Practices

### For Documentation
- Keep under 30 seconds
- Focus on 3-4 key commands
- Use clear, simple prompts
- Show actual output

### For Social Media
- Keep under 20 seconds
- High energy, fast pacing
- Bold text for key messages
- End with clear CTA

### For Presentations
- Allow 60-90 seconds
- Tell a story
- Include context/messaging
- Show real-world scenarios

### For Investor Pitch
- Emphasize ROI/value
- Show before/after
- Include stats/numbers
- Professional theme

## 🐛 Troubleshooting

### Demo Fails to Generate

**Issue**: `cmdai not found`
```bash
# Solution: Ensure binary is built
cargo build --release --features embedded-mlx

# Or update Require line in tape:
Require /absolute/path/to/target/release/cmdai
```

**Issue**: Commands show "Unable to generate"
```bash
# Solution: Warm up model first
./target/release/cmdai "test command"

# Or add warmup to tape:
Hide
Type "./target/release/cmdai 'test'"
Enter
Sleep 5s
Type "clear"
Enter
Show
```

### Theme Doesn't Apply

```bash
# List available themes
vhs themes

# Use exact theme name (case-sensitive)
Set Theme "Dracula"  # ✅
Set Theme "dracula"  # ❌
```

### Output Too Large

```bash
# Reduce dimensions
Set Width 1200   # Instead of 1800
Set Height 700   # Instead of 1000

# Reduce font size
Set FontSize 16  # Instead of 22

# Increase playback speed
Set PlaybackSpeed 1.5  # Play 1.5x faster

# Convert to MP4 (better compression)
Output demos/demo.mp4  # Instead of .gif
```

## 📚 Resources

- **VHS Documentation**: https://github.com/charmbracelet/vhs
- **VHS Examples**: https://github.com/charmbracelet/vhs/tree/main/examples
- **Atuin Demo**: https://github.com/atuinsh/atuin#demo
- **Crush Demo**: https://github.com/charmbracelet/crush

## 🎬 Demo Guidelines

### Content
- ✅ Show real commands that work
- ✅ Use realistic prompts
- ✅ Include actual output
- ❌ Don't fake success
- ❌ Don't hide failures

### Design
- ✅ Consistent theming
- ✅ Readable font sizes (18-24)
- ✅ Clear visual hierarchy
- ✅ Professional appearance
- ❌ Don't overcrowd

### Pacing
- ✅ Give time to read
- ✅ Natural typing speed
- ✅ Appropriate pauses
- ❌ Don't rush through
- ❌ Don't drag on

## 🚀 Next Steps

After generating demos:

1. **Add to README**:
   ```markdown
   ![Caro Demo](demos/caro-quickstart.gif)
   ```

2. **Share on Social**:
   - Twitter: Upload directly
   - LinkedIn: Convert to MP4
   - Reddit: GIF works best

3. **Use in Presentations**:
   - MP4 for slides
   - GIF for web embeds
   - WebM for optimal web

4. **Update Documentation**:
   - Feature pages
   - Tutorial walkthroughs
   - Blog posts

## 📄 License

Same as main project (AGPL-3.0)

---

**Created with** [VHS](https://github.com/charmbracelet/vhs) by [Charm](https://charm.sh)
