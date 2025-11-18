# Designer's Guide to Terminal Sprite Animations

A practical, step-by-step guide for UX designers, artists, and non-developers to create terminal sprite animations for cmdai.

## Who This Guide Is For

This guide is written for:
- UX/UI designers who want to add animations to terminal applications
- Pixel artists familiar with tools like Aseprite
- ANSI art creators
- Anyone who wants to create terminal animations without deep programming knowledge

No Rust programming experience required! We'll focus on design tools and provide simple code templates you can copy and modify.

## Table of Contents

- [Getting Started](#getting-started)
- [Choosing Your Tool](#choosing-your-tool)
- [Workflow 1: Aseprite (Recommended)](#workflow-1-aseprite-recommended)
- [Workflow 2: Text Editors (ANSI Art)](#workflow-2-text-editors-ansi-art)
- [Workflow 3: JSON Editor (DurDraw)](#workflow-3-json-editor-durdraw)
- [Design Guidelines](#design-guidelines)
- [Color Palettes](#color-palettes)
- [Animation Principles](#animation-principles)
- [Testing Your Animations](#testing-your-animations)
- [Common Issues](#common-issues)
- [Resources](#resources)

## Getting Started

### What You Need

1. **A text editor** (VS Code, Sublime, or any code editor)
2. **One of these tools**:
   - **Aseprite** (best for pixel art) - $19.99 or compile from source
   - **Any text editor** (for ANSI art)
   - **VS Code with JSON support** (for DurDraw)

3. **Basic terminal access** to test your animations

### Understanding Terminal Limitations

Terminal animations are different from traditional animations:

- **Low resolution**: Terminals use character cells, not pixels
- **Limited space**: Each "pixel" is a Unicode block character (█)
- **Color modes**: Either 256 colors or true color (16.7M colors)
- **Fixed grid**: Everything aligns to a character grid

Think of it like designing for very low-resolution displays (8x8 to 32x32 typically works best).

## Choosing Your Tool

| Tool | Best For | Difficulty | Features |
|------|----------|------------|----------|
| **Aseprite** | Pixel art, animations | Easy | Layers, animation timeline, onion skinning |
| **Text Editor** | ANSI art, text-based art | Medium | Character-level control, wide compatibility |
| **JSON Editor** | Programmatic art, metadata | Medium | Full color, easy to edit, human-readable |

**Recommendation**: If you're creating pixel art or multi-frame animations, use **Aseprite**. It's the most designer-friendly option.

## Workflow 1: Aseprite (Recommended)

Aseprite is a professional pixel art tool perfect for terminal animations.

### Step 1: Create Your Sprite in Aseprite

1. **Open Aseprite**
2. **Create new sprite**: File → New
   - **Size**: Start with 16x16 pixels (recommended for terminal)
   - **Color Mode**: RGBA (full color)

   ![Aseprite New Sprite]
   ```
   Width:  16px
   Height: 16px
   Color Mode: RGBA
   ```

3. **Design your sprite**:
   - Use the pencil tool to draw
   - Keep it simple - small details may not show well in terminal
   - Use distinct colors

### Step 2: Create Animation Frames

1. **Add frames**: Click the "New Frame" button in the timeline
2. **Set frame duration**:
   - Right-click frame → Frame Properties
   - Set duration (100ms = 0.1 seconds is a good start)
   - 50-200ms per frame feels smooth

3. **Use onion skinning**: View → Show → Onion Skin
   - Helps you see previous/next frames while drawing
   - Makes animation smoother

### Step 3: Animation Tips

**Simple Walk Cycle** (4 frames):
```
Frame 1: Standing position
Frame 2: Left leg forward
Frame 3: Standing position
Frame 4: Right leg forward
```

**Bouncing Ball** (3-4 frames):
```
Frame 1: Ball at top (round)
Frame 2: Ball in middle
Frame 3: Ball at bottom (squashed)
Frame 4: Ball in middle (return)
```

**Blinking** (2 frames):
```
Frame 1: Eyes open (500ms)
Frame 2: Eyes closed (100ms)
```

### Step 4: Export from Aseprite

1. **File → Save As**
2. **Choose format**: `.ase` or `.aseprite`
3. **Save location**: `cmdai/assets/your-sprite.ase`

### Step 5: Load in Rust (Copy This Template)

Create a new file `examples/my_animation.rs`:

```rust
//! My custom animation
//!
//! Run with: cargo run --example my_animation

use cmdai::rendering::{AsepriteParser, Animation, AnimationMode, Animator};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Loading sprite...");

    // Load your .ase file
    let ase_file = AsepriteParser::load_file("assets/your-sprite.ase")?;

    // Convert to sprite
    let sprite = AsepriteParser::to_sprite(&ase_file)?;

    println!("Sprite loaded: {} frames", sprite.frame_count());

    // Create animation
    let mut animation = Animation::new(
        sprite,
        AnimationMode::Loop  // or AnimationMode::Once, AnimationMode::LoopN(5)
    );

    // Play it!
    let animator = Animator::new();
    animator.play(&mut animation).await?;

    Ok(())
}
```

**To test**:
```bash
. "$HOME/.cargo/env"
cargo run --example my_animation
```

### Step 6: Adjust and Iterate

Not happy with the result? Adjust these in Aseprite:

1. **Frame timing**: Make it faster/slower
2. **Colors**: Adjust palette for better terminal visibility
3. **Size**: Try 8x8 or 32x32
4. **Frame count**: Add more frames for smoother animation

Then save and re-run!

## Workflow 2: Text Editors (ANSI Art)

ANSI art is character-based art with color codes. Great for text-based designs.

### Step 1: Understand ANSI Codes

ANSI codes control color in terminals:

```
\x1b[0m      - Reset (clear all formatting)
\x1b[31m     - Red text
\x1b[32m     - Green text
\x1b[33m     - Yellow text
\x1b[34m     - Blue text
\x1b[1;31m   - Bold red
\x1b[44m     - Blue background
```

### Step 2: Create Simple ANSI Art

Example: Colored banner

```rust
//! ANSI art banner
//!
//! Run with: cargo run --example my_ansi_art

use cmdai::rendering::{AnsiParser, TerminalRenderer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create ANSI art
    let ansi_art = "\
\x1b[0m\x1b[1;37;44m╔════════════════╗
\x1b[1;37;44m║   WELCOME!     ║
\x1b[1;37;44m╚════════════════╝\x1b[0m
";

    // Parse it
    let (frame, _) = AnsiParser::parse_bytes(ansi_art.as_bytes())?;

    // Render it
    let renderer = TerminalRenderer::new();
    renderer.print_ansi_frame(&frame)?;

    Ok(())
}
```

### Step 3: Use ANSI Art Tools

**Online tools**:
- **PabloDraw** (Windows) - Classic ANSI editor
- **Moebius** (Cross-platform) - Modern ANSI/ASCII editor
- **ASCII Paint** (Web-based)

### Step 4: Load ANSI Files

If you have an ANSI file (`.ans` or `.txt`):

```rust
use cmdai::rendering::{AnsiParser, TerminalRenderer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load ANSI file
    let (frame, metadata) = AnsiParser::load_file("assets/banner.ans")?;

    // Display metadata (if available)
    if let Some(meta) = metadata {
        println!("Title: {}", meta.title);
        println!("Author: {}", meta.author);
    }

    // Render
    let renderer = TerminalRenderer::new();
    renderer.print_ansi_frame(&frame)?;

    Ok(())
}
```

### Step 5: Box Drawing Characters

Use Unicode box drawing for clean designs:

```
┌─┬─┐  ╔═╦═╗  ╭─┬─╮
├─┼─┤  ╠═╬═╣  ├─┼─┤
└─┴─┘  ╚═╩═╝  ╰─┴─╯

Single: ─ │ ┌ ┐ └ ┘ ├ ┤ ┬ ┴ ┼
Double: ═ ║ ╔ ╗ ╚ ╝ ╠ ╣ ╦ ╩ ╬
Rounded: ─ │ ╭ ╮ ╰ ╯
```

Example:
```
\x1b[36m╔══════════════════╗
║  Terminal Box    ║
╚══════════════════╝\x1b[0m
```

## Workflow 3: JSON Editor (DurDraw)

DurDraw is a JSON format that's easy to edit by hand.

### Step 1: Create a DurDraw File

Create `assets/my-art.dur`:

```json
{
  "version": "1.0",
  "title": "My Pixel Art",
  "author": "Your Name",
  "group": "",
  "date": "20240115",
  "width": 5,
  "height": 3,
  "palette": [],
  "data": [
    {"char": "█", "fg": [255, 0, 0], "bg": [0, 0, 0], "attr": 0},
    {"char": "█", "fg": [255, 0, 0], "bg": [0, 0, 0], "attr": 0},
    {"char": "█", "fg": [255, 0, 0], "bg": [0, 0, 0], "attr": 0},
    {"char": "█", "fg": [255, 0, 0], "bg": [0, 0, 0], "attr": 0},
    {"char": "█", "fg": [255, 0, 0], "bg": [0, 0, 0], "attr": 0},

    {"char": "█", "fg": [0, 255, 0], "bg": [0, 0, 0], "attr": 0},
    {"char": "█", "fg": [0, 255, 0], "bg": [0, 0, 0], "attr": 0},
    {"char": "█", "fg": [0, 255, 0], "bg": [0, 0, 0], "attr": 0},
    {"char": "█", "fg": [0, 255, 0], "bg": [0, 0, 0], "attr": 0},
    {"char": "█", "fg": [0, 255, 0], "bg": [0, 0, 0], "attr": 0},

    {"char": "█", "fg": [0, 0, 255], "bg": [0, 0, 0], "attr": 0},
    {"char": "█", "fg": [0, 0, 255], "bg": [0, 0, 0], "attr": 0},
    {"char": "█", "fg": [0, 0, 255], "bg": [0, 0, 0], "attr": 0},
    {"char": "█", "fg": [0, 0, 255], "bg": [0, 0, 0], "attr": 0},
    {"char": "█", "fg": [0, 0, 255], "bg": [0, 0, 0], "attr": 0}
  ]
}
```

This creates a 5x3 grid:
```
█████  (red row)
█████  (green row)
█████  (blue row)
```

### Step 2: Understanding the Format

**Structure**:
- `width` × `height` = total cells in `data` array
- Each cell has:
  - `char`: The character to display (usually "█" for pixels)
  - `fg`: Foreground color [R, G, B] (0-255)
  - `bg`: Background color [R, G, B]
  - `attr`: 0 = normal, 1 = bold, 2 = dim

**Colors can be**:
```json
// RGB array
"fg": [255, 128, 0]

// Hex string
"fg": "#FF8000"

// Named color
"fg": "red"

// Palette index
"fg": 5
```

### Step 3: Load DurDraw File

```rust
use cmdai::rendering::{DurDrawParser, TerminalRenderer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .dur file
    let (frame, metadata) = DurDrawParser::load_with_metadata("assets/my-art.dur")?;

    println!("Title: {}", metadata.title);
    println!("Author: {}", metadata.author);

    // Render
    let renderer = TerminalRenderer::new();
    renderer.print_ansi_frame(&frame)?;

    Ok(())
}
```

### Step 4: Using a Palette

Create reusable colors:

```json
{
  "version": "1.0",
  "title": "Palette Example",
  "author": "Designer",
  "width": 4,
  "height": 1,
  "palette": [
    [0, 0, 0],
    [255, 0, 0],
    [0, 255, 0],
    [0, 0, 255]
  ],
  "data": [
    {"char": "█", "fg": 1, "bg": 0, "attr": 0},
    {"char": "█", "fg": 2, "bg": 0, "attr": 0},
    {"char": "█", "fg": 3, "bg": 0, "attr": 0},
    {"char": "█", "fg": 1, "bg": 0, "attr": 0}
  ]
}
```

Now `"fg": 1` references `palette[1]` which is `[255, 0, 0]` (red).

## Design Guidelines

### Size Recommendations

| Size | Best For | Visibility |
|------|----------|------------|
| 4x4 - 8x8 | Icons, emoji | Excellent |
| 16x16 | Sprites, small characters | Good |
| 32x32 | Detailed characters | Fair |
| 64x64+ | Background art (static) | Poor (use sparingly) |

**Rule of thumb**: Smaller is better for terminal animations.

### Visual Clarity

1. **Use bold colors**: Pastels may not show well
2. **High contrast**: Dark background, bright foreground (or vice versa)
3. **Simple shapes**: Details get lost in terminals
4. **Test early**: What looks good in Aseprite may need adjustment for terminal

### Character Choices

**For pixel art**:
- Full block: `█` (U+2588) - Best for solid pixels
- Half blocks: `▀` `▄` `▌` `▐` - For details

**For text art**:
- Box drawing: `─ │ ┌ ┐ └ ┘ ├ ┤ ┬ ┴ ┼`
- Blocks: `░ ▒ ▓ █`
- Shapes: `● ○ ◆ ◇ ■ □ ▲ △`

## Color Palettes

### Pre-made Palettes

**Game Boy Classic**:
```
#0F380F  (darkest green)
#306230  (dark green)
#8BAC0F  (light green)
#9BBC0F  (lightest green)
```

**Fire**:
```
#000000  (black)
#8B0000  (dark red)
#FF4500  (orange-red)
#FFA500  (orange)
#FFFF00  (yellow)
#FFFFFF  (white)
```

**Ocean**:
```
#000033  (deep blue)
#003366  (dark blue)
#0066CC  (medium blue)
#3399FF  (light blue)
#66CCFF  (cyan)
#FFFFFF  (white foam)
```

**Retro CGA** (1980s DOS):
```
#000000  (black)
#AA0000  (red)
#00AA00  (green)
#AA5500  (brown)
#0000AA  (blue)
#AA00AA  (magenta)
#00AAAA  (cyan)
#AAAAAA  (light gray)
```

### Creating Your Palette

1. **Start with 4-8 colors**: Enough variety, not overwhelming
2. **Include transparency**: Usually index 0 (black)
3. **Consider terminal backgrounds**: Most are black or white
4. **Test in actual terminal**: Colors appear different than in design tools

**Good palette structure**:
```
0: Transparent (or background)
1: Primary color (main character color)
2: Secondary color (accents)
3: Highlight (lighter version of primary)
4: Shadow (darker version of primary)
5: Outline/detail (usually dark)
```

## Animation Principles

Even in terminal animations, classic animation principles apply!

### Timing

| Effect | Frame Duration | Use Case |
|--------|---------------|----------|
| Very Fast | 30-50ms | Spinning, rapid movement |
| Fast | 50-100ms | Walking, running |
| Normal | 100-200ms | Idle animations, blinking |
| Slow | 200-500ms | Breathing, floating |
| Very Slow | 500ms+ | Emphasis, pause frames |

### Squash and Stretch

Makes animations feel more alive:

```
Bouncing ball:

Frame 1: Normal circle (at top)
         ●●●
         ●●●
         ●●●

Frame 2: Stretched (falling)
         ●●●
         ●●●
         ●●●
         ●●●

Frame 3: Squashed (impact)
         ●●●●●
         ●●●●●
```

### Anticipation

Build up before main action:

```
Jump animation:

Frame 1: Standing
Frame 2: Crouch (anticipation)
Frame 3: Mid-jump
Frame 4: Peak
Frame 5: Landing
```

### Follow-through

Parts continue moving after main action stops:

```
Character stops running:

Frame 1: Running pose
Frame 2: Feet stop, body leans forward
Frame 3: Body straightens, arms still moving
Frame 4: Complete stop
```

### Frame Counts

| Animation Type | Frames | Example |
|---------------|--------|---------|
| Simple blink | 2 | Open, closed |
| Icon pulse | 2-3 | Normal, large, normal |
| Walk cycle | 4-8 | Full walking motion |
| Complex action | 6-12 | Jump, attack, transform |

**Start simple!** Even 2-frame animations can be effective.

## Testing Your Animations

### Quick Test in Terminal

1. **Run your example**:
   ```bash
   cargo run --example my_animation
   ```

2. **Check these**:
   - Does it play smoothly?
   - Are colors visible?
   - Is timing right?
   - Any flickering?

3. **Adjust and repeat**

### Testing Checklist

- [ ] Animation plays at correct speed
- [ ] Colors are visible and distinct
- [ ] No missing frames
- [ ] Loops smoothly (if looping)
- [ ] Size appropriate for terminal
- [ ] Works in different terminal emulators
- [ ] No dimension errors in console

### Test in Different Terminals

Colors and rendering may vary:

**macOS**:
- iTerm2 (best color support)
- Terminal.app (good)

**Linux**:
- GNOME Terminal (good)
- Konsole (good)
- Alacritty (excellent)

**Windows**:
- Windows Terminal (best)
- ConEmu (good)
- Cmd.exe (limited colors)

### Performance Testing

If animation stutters:

1. **Reduce sprite size**: Try 16x16 instead of 32x32
2. **Increase frame duration**: 100ms → 150ms
3. **Reduce frame count**: 8 frames → 4 frames
4. **Simplify colors**: Use fewer palette entries

## Common Issues

### Issue: Colors Don't Match Design

**Cause**: Terminal color mode mismatch

**Solution**:
```bash
# Enable true color support
export COLORTERM=truecolor

# Then test again
cargo run --example my_animation
```

### Issue: Animation Plays Once Then Stops

**Cause**: Using `AnimationMode::Once`

**Solution**:
```rust
// Change from
Animation::new(sprite, AnimationMode::Once)

// To
Animation::new(sprite, AnimationMode::Loop)
// or
Animation::new(sprite, AnimationMode::LoopN(5))  // Loop 5 times
```

### Issue: "Invalid dimensions" Error

**Cause**: Pixel count doesn't match width × height

**Check your sprite**:
```
If width = 4 and height = 3
Then you need exactly 12 pixels (4 × 3 = 12)
```

For DurDraw: Count entries in `data` array
For Aseprite: Check canvas size

### Issue: Sprite Too Small/Large

**Solution**: Resize in your tool

**Aseprite**:
1. Sprite → Sprite Size
2. Enter new dimensions
3. Choose scaling method (keep "Nearest neighbor" for pixel art)

**DurDraw**:
- Change `width` and `height`
- Adjust `data` array size to match

### Issue: Can't Find File

**Cause**: Wrong file path

**Solutions**:
```rust
// Use absolute path
AsepriteParser::load_file("/full/path/to/sprite.ase")?

// Or relative from project root
AsepriteParser::load_file("assets/sprite.ase")?

// Check file exists
let path = std::path::Path::new("assets/sprite.ase");
if !path.exists() {
    println!("File not found: {:?}", path);
}
```

### Issue: Aseprite File Won't Load

**Check**:
1. File saved as `.ase` or `.aseprite` format
2. Not a PNG or other image format
3. File not corrupted
4. File size is reasonable (< 5MB typically)

**In Aseprite**:
- File → Save As
- Select format: "Aseprite" (not PNG, GIF, etc.)

## Resources

### Tools

**Pixel Art / Animation**:
- [Aseprite](https://www.aseprite.org/) - $19.99, or free if compiled from source
- [LibreSprite](https://libresprite.github.io/) - Free, open-source fork of Aseprite
- [Piskel](https://www.piskelapp.com/) - Free, web-based

**ANSI Art**:
- [Moebius](https://github.com/blocktronics/moebius) - Free, modern ANSI editor
- [PabloDraw](http://picoe.ca/products/pablodraw/) - Classic ANSI editor
- [ASCII Paint](http://ascii.alienmelon.com/) - Web-based

**Color Palettes**:
- [Lospec Palette List](https://lospec.com/palette-list) - Thousands of palettes
- [Coolors.co](https://coolors.co/) - Color palette generator

### Tutorials

**Pixel Art Basics**:
- [Pixel Art Tutorial by Saint11](https://blog.studiominiboss.com/pixelart)
- [MortMort's pixel art tutorials](https://www.youtube.com/@MortMort)

**Animation**:
- [Animation Principles by Alan Becker](https://www.youtube.com/user/noogai89)
- [12 Principles of Animation](https://www.creativebloq.com/advice/understand-the-12-principles-of-animation)

**Terminal/ANSI Art**:
- [16 Colors ANSI Art](https://16colo.rs/) - Gallery and community
- [ANSI Art Tutorial](https://www.roysac.com/ansi-art/)

### cmdai Documentation

- **[Quick Start](QUICKSTART_ANIMATIONS.md)**: Get started in 5 minutes
- **[Animation Guide](ANIMATION_GUIDE.md)**: Complete technical reference
- **[Testing Guide](TESTING_ANIMATIONS.md)**: Test and validate animations

### Code Templates

Save time with these ready-to-use templates in `examples/`:

1. **Aseprite loader** - `examples/aseprite_demo.rs`
2. **ANSI art display** - `examples/ansi_art_demo.rs`
3. **DurDraw loader** - `examples/durdraw_demo.rs`
4. **Custom sprites** - `examples/sprite_demo.rs`

Copy, modify, and make them your own!

## Sharing Your Artwork: Contributing to the Repository

Once you've created amazing animations, you may want to contribute them to the cmdai project!

### Why Contribute Your Artwork?

✅ **Showcase your work** - Get your art seen by the community
✅ **Get attributed** - Proper credit in the project
✅ **Help others** - Your art can be used as examples and inspiration
✅ **Protect your rights** - You retain copyright and control

### Important: Licensing Your Artwork

**Your artwork is NOT automatically open source!**

Even though cmdai's code is open source (AGPL-3.0), **your artwork is separate** and you choose how it can be used.

**Two main options**:

1. **Restrictive License** (Recommended for original characters)
   - ✅ People can view your art in cmdai
   - ✅ People can use cmdai including your art
   - ❌ People cannot use your art in other projects
   - ❌ People cannot redistribute your art separately
   - **Best for**: Original characters you want to protect

2. **Permissive License** (Creative Commons)
   - ✅ People can use your art with attribution
   - ✅ More exposure for your work
   - ⚠️ Less control over usage
   - **Best for**: Generic UI elements, reusable sprites

**When in doubt**: Use the restrictive license. You can always make it more permissive later!

### Step-by-Step: Uploading Your Assets

**Complete guide**: See [Contributing Assets Guide](CONTRIBUTING_ASSETS.md) for full details.

**Quick version**:

#### 1. Get Repository Access

1. Create a GitHub account at [github.com](https://github.com)
2. Contact the project owner to be added as a collaborator
3. Accept the invitation email

#### 2. Create Your Artist Folder

Using GitHub's web interface (no Git knowledge needed):

1. Navigate to the `assets/` folder
2. Click **"Add file"** → **"Create new file"**
3. Name it: `your-name/README.md` (this creates the folder!)
4. Fill in the README using the [template](../assets/ARTIST_README_TEMPLATE.md)
5. Click **"Commit new file"**

#### 3. Add Your License

1. In your folder (`assets/your-name/`), click **"Add file"** → **"Create new file"**
2. Name it: `LICENSE.md`
3. Copy the [Asset License Template](../ASSET-LICENSE-TEMPLATE.md)
4. Fill in your information
5. Click **"Commit new file"**

#### 4. Upload Your Files

1. In your folder, click **"Add file"** → **"Upload files"**
2. Drag and drop your files:
   - Aseprite source files (`.ase`)
   - ANSI art (`.ans`)
   - DurDraw files (`.dur`)
   - PNG exports
   - Concept art
3. Organize into subfolders:
   ```
   your-name/
   ├── character-name/
   │   ├── source/      (Aseprite files)
   │   ├── export/      (ANSI, DurDraw, PNG)
   │   └── README.md    (About the character)
   ```
4. Add a commit message like: "Add [Character Name] artwork and animations"
5. Click **"Commit changes"**

### What to Include

**Required**:
- ✅ `README.md` - About you and your assets
- ✅ `LICENSE.md` - How your artwork can be used
- ✅ Source files - Original Aseprite files
- ✅ Exported files - ANSI, DurDraw, or PNG

**Recommended**:
- ⭐ Character documentation - Explain each character
- ⭐ Concept art - Show your design process
- ⭐ Color palette info - Document your color choices

**Optional**:
- 💡 Sketches and early designs
- 💡 Animation specifications
- 💡 Design notes and inspiration

### File Organization Example

Here's how to organize your files:

```
assets/
└── your-name/               # Your artist folder
    ├── README.md           # About you
    ├── LICENSE.md          # Your asset license
    ├── kyaro/              # Example character
    │   ├── source/
    │   │   ├── kyaro-idle.ase
    │   │   └── kyaro-walk.ase
    │   ├── export/
    │   │   ├── kyaro-idle.ans
    │   │   ├── kyaro-walk.dur
    │   │   └── frames/
    │   │       ├── walk-01.png
    │   │       └── walk-02.png
    │   └── README.md       # About Kyaro
    └── ui-elements/
        └── spinner.ase
```

### Testing Before You Upload

Before uploading, make sure:

1. **Files are complete**
   - All source files included
   - Exports are up to date
   - No temporary files

2. **Files work correctly**
   - Test in the demo applications
   - Verify animations play properly
   - Check colors display correctly

3. **Documentation is clear**
   - README explains what each file is
   - License terms are specific
   - Contact information is accurate (if included)

### After Uploading

Once your files are uploaded:

1. **Test them in the project**
   ```bash
   # Pull the latest code
   git pull

   # Test your asset
   cargo run --example aseprite_demo
   ```

2. **Get feedback**
   - Share the link with the project owner
   - Ask for code review
   - Make any requested changes

3. **Celebrate!** 🎉
   - Your artwork is now part of cmdai
   - You'll be credited in the project
   - Others can see and appreciate your work

### Attribution

Your work will be credited in:
- The main README
- The assets directory README
- The application's About/Credits section

**Example attribution**:
```
Kyaro character © 2025 Your Name
Created for cmdai terminal animation system
Licensed under Restrictive Asset License
```

### Getting Help

**Questions about uploading assets?**

1. **Read the complete guide**: [Contributing Assets Guide](CONTRIBUTING_ASSETS.md)
2. **Check the templates**:
   - [Asset License Template](../ASSET-LICENSE-TEMPLATE.md)
   - [Artist README Template](../assets/ARTIST_README_TEMPLATE.md)
3. **Ask for help**:
   - Create a GitHub Issue
   - Tag it with "assets" or "documentation"
   - The community will help!

**Contact the project owner**:
- Email: [If provided]
- GitHub: Create an issue or discussion

## Quick Reference Card

### File Formats Quick Guide

```
.ase/.aseprite  →  AsepriteParser::load_file()
.ans/.txt       →  AnsiParser::load_file()
.dur            →  DurDrawParser::load_with_metadata()
```

### Animation Modes

```rust
AnimationMode::Once      // Play once
AnimationMode::Loop      // Loop forever
AnimationMode::LoopN(5)  // Loop 5 times
```

### Common Frame Durations

```
16ms  ≈ 60 FPS (very smooth)
33ms  ≈ 30 FPS (smooth)
50ms  ≈ 20 FPS (good)
100ms ≈ 10 FPS (standard)
200ms ≈ 5 FPS  (slow)
500ms ≈ 2 FPS  (very slow)
```

### Recommended Sizes

```
Icons:     4x4 to 8x8
Sprites:   8x8 to 16x16
Characters: 16x16 to 32x32
Scenes:    32x32 to 48x48
```

## Need Help?

1. Check the [Testing Guide](TESTING_ANIMATIONS.md) for debugging help
2. Review [Animation Guide](ANIMATION_GUIDE.md) for technical details
3. Look at `examples/` directory for working code
4. File an issue on GitHub with your question

Happy animating! 🎨
