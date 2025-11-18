# Terminal Sprite Animation System with Asset Contribution Framework

## Overview

This PR adds a complete terminal-based sprite animation rendering system to cmdai, enabling pixel art characters and animations to be displayed in terminal applications. It includes support for multiple file formats, comprehensive documentation for both developers and designers, and a complete framework for artists to contribute their work with proper licensing protection.

## 🎨 Features Added

### Core Animation System

#### Sprite Rendering Engine (`src/rendering/`)
- **Unicode block rendering** using █, ▀, ▄ for true pixel-based graphics
- **Color palette system** with hex color support (#RRGGBB)
- **Multi-frame animations** with customizable timing
- **Transparency support** for complex sprite compositions
- **True color (24-bit RGB)** and 256-color ANSI mode
- **Animation modes**: Once, Loop, LoopN(times)
- **Async playback** using tokio runtime

#### Core Components
- `sprites.rs` - Data structures for sprites, frames, and color palettes
- `terminal.rs` - Terminal rendering with ANSI escape codes
- `animator.rs` - Animation playback and sequencing
- `examples.rs` - Pre-built sprite examples (idle character, walking, heart, coin, spinner)

### File Format Support

#### 1. ANSI Art Format (`ansi_parser.rs`)
- **Traditional ANSI art files** (.ans)
- Full ANSI escape sequence parsing (SGR, cursor control)
- **SAUCE metadata** extraction (Standard Architecture for Universal Comment Extensions)
- 16-color and 256-color palette support
- Character preservation (€, ‹, ﬂ, etc.)
- Convert ANSI frames to Sprite format

#### 2. DurDraw Format (`durdraw_parser.rs`)
- **Modern JSON-based ANSI art** format (.dur)
- Multiple color formats:
  - RGB arrays: `[255, 128, 64]`
  - Hex strings: `"#FF8040"`
  - Named colors: `"red"`, `"bright_green"`
  - Palette indices
- Full metadata support (title, author, group, date)
- Bidirectional conversion with AnsiFrame
- File loading and saving with JSON serialization

#### 3. Aseprite Format (`aseprite_parser.rs`)
- **Binary .ase/.aseprite file parser**
- Header parsing with magic number validation (0xA5E0)
- Frame-based structure with chunk parsing
- Layer system with visibility, opacity, and blend modes
- **Zlib/DEFLATE decompression** for compressed cel data
- Alpha blending for proper layer compositing
- Color modes: RGBA (32-bit), Grayscale (16-bit), Indexed (8-bit)
- Convert to Sprite format for animation playback

**Dependencies added**: `flate2 = "1.0"` for zlib compression

### Example Applications

Four comprehensive demo applications:
- `examples/sprite_demo.rs` - Pre-built animations
- `examples/ansi_art_demo.rs` - ANSI parsing demonstration
- `examples/durdraw_demo.rs` - DurDraw format demonstration
- `examples/aseprite_demo.rs` - Aseprite format demonstration

## 📚 Documentation

### Animation Documentation (81KB total)

#### For Everyone
**Quick Start Guide** (`docs/QUICKSTART_ANIMATIONS.md`) - 6.1 KB
- 30-second demo commands
- 5-minute "Your First Animation" tutorial
- Format overview and comparison
- Common issues and quick fixes

#### For Developers
**Animation Guide** (`docs/ANIMATION_GUIDE.md`) - 28 KB
- Complete API documentation for all types
- Format specifications (ANSI, DurDraw, Aseprite)
- Creating animations programmatically (15+ examples)
- File loading and saving
- Animation timing and control
- Advanced rendering techniques
- Performance optimization
- Comprehensive troubleshooting

#### For UX/UI Designers
**Designer Guide** (`docs/DESIGNER_GUIDE.md`) - 20 KB
- Three complete workflows:
  - Aseprite (recommended for pixel art)
  - Text editors (for ANSI art)
  - JSON editors (for DurDraw)
- Tool selection guidance
- Pre-made color palettes (Game Boy, Fire, Ocean, CGA)
- Animation principles for terminals
- Design guidelines for readability
- Testing without writing code
- **Asset contribution section** (NEW)

#### For QA/Testers
**Testing Guide** (`docs/TESTING_ANIMATIONS.md`) - 27 KB
- Running all demo applications
- Creating test files (templates provided)
- Validation checklists (Visual, Technical, Cross-platform)
- Performance testing procedures
- Cross-platform compatibility testing
- Debug logging and tools
- Automated testing setup
- Visual regression testing
- Test report templates

#### Documentation Index
**Documentation Hub** (`docs/README.md`) - 7.3 KB
- Complete documentation directory
- Role-based learning paths
- Common tasks quick reference
- External resources and tools

### Asset Contribution Documentation (42KB total)

#### For Artists and Designers
**Contributing Assets Guide** (`docs/CONTRIBUTING_ASSETS.md`) - 19 KB
- **No coding required** - GitHub web interface tutorial
- Understanding licensing (code vs. artwork)
- Two licensing options:
  - Restrictive License (for original characters)
  - Permissive License (Creative Commons)
- Step-by-step upload process (drag & drop)
- File organization best practices
- What to include (required, recommended, optional)
- File naming conventions
- Testing procedures
- Comprehensive FAQ

**Asset License Template** (`ASSET-LICENSE-TEMPLATE.md`) - 7 KB
- Ready-to-use legal template
- Permitted/prohibited use sections
- Attribution requirements
- Plain English summary
- Customizable with `[placeholders]`

**Assets Directory** (`assets/README.md`) - 9 KB
- Licensing notice and compliance guide
- Directory structure documentation
- Contributing artists registry
- Usage examples for all formats
- Fork and redistribution guidelines

**Artist README Template** (`assets/ARTIST_README_TEMPLATE.md`) - 7 KB
- Personal artist profile template
- Asset organization
- License summary
- Usage examples
- Contact and attribution info

### Updated Main Documentation

**Main README** (`README.md`)
- Added comprehensive Sprite Animation System section
- All three format support documented with examples
- **Dual Licensing section** (NEW):
  - Clear separation: AGPL-3.0 (code) vs. Artist licenses (assets)
  - Why separate licenses benefit everyone
  - Fork/redistribution guidelines
  - Artwork contributors acknowledgment section

## 🎯 Use Cases

### For Developers
```rust
use cmdai::rendering::*;

// Load Aseprite file
let ase_file = AsepriteParser::load_file("character.ase")?;
let sprite = AsepriteParser::to_sprite(&ase_file)?;

// Create animation
let mut animation = Animation::new(sprite, AnimationMode::Loop);
let animator = Animator::new();
animator.play(&mut animation).await?;
```

### For Designers
1. Create pixel art in Aseprite
2. Export as .ase file
3. Upload to `assets/your-name/` via GitHub web interface
4. Add license using template
5. Test with demo applications
6. **No Rust knowledge required!**

### For Terminal Applications
- Character animations (NPCs, avatars, mascots)
- Loading indicators and progress animations
- Status icons and notifications
- Interactive terminal UIs
- Retro-style games and demos

## 🔐 Dual Licensing Framework

### Code License: AGPL-3.0
- The cmdai source code remains fully open source
- Anyone can use, modify, and redistribute the code
- Network use requires source disclosure

### Artwork License: Artist-Controlled
- **Artists retain copyright** over their work
- Each artist specifies their own license terms
- **Not automatically open source**
- Typical restrictions:
  - ✅ Can use in cmdai
  - ✅ Can contribute to cmdai
  - ❌ Cannot use in other projects
  - ❌ Cannot redistribute separately
  - ❌ Cannot modify without permission

### Why Separate Licenses?
- ✅ Protects artists' creative work and original characters
- ✅ Allows open-source collaboration on the code
- ✅ Encourages contributions from both developers and artists
- ✅ Ensures proper attribution and copyright respect

## 📁 File Structure

```
cmdai/
├── src/rendering/
│   ├── mod.rs              # Module exports
│   ├── sprites.rs          # Sprite data structures (580+ lines)
│   ├── terminal.rs         # Terminal rendering
│   ├── animator.rs         # Animation engine
│   ├── ansi_parser.rs      # ANSI art parser (580+ lines)
│   ├── durdraw_parser.rs   # DurDraw parser (420+ lines)
│   ├── aseprite_parser.rs  # Aseprite parser (500+ lines)
│   └── examples.rs         # Pre-built sprites
├── examples/
│   ├── sprite_demo.rs      # Sprite animations demo
│   ├── ansi_art_demo.rs    # ANSI parsing demo
│   ├── durdraw_demo.rs     # DurDraw format demo
│   └── aseprite_demo.rs    # Aseprite format demo
├── docs/
│   ├── README.md                     # Documentation index (updated)
│   ├── QUICKSTART_ANIMATIONS.md      # Quick start (6.1 KB)
│   ├── ANIMATION_GUIDE.md            # API reference (28 KB)
│   ├── DESIGNER_GUIDE.md             # Designer workflows (20 KB, updated)
│   ├── TESTING_ANIMATIONS.md         # Testing guide (27 KB)
│   └── CONTRIBUTING_ASSETS.md        # Asset contribution (19 KB, NEW)
├── assets/                           # NEW
│   ├── README.md                     # Assets hub (9 KB, NEW)
│   ├── ARTIST_README_TEMPLATE.md     # Artist template (7 KB, NEW)
│   └── examples/                     # Placeholder
├── ASSET-LICENSE-TEMPLATE.md         # License template (7 KB, NEW)
├── README.md                         # Main README (updated)
├── CHANGELOG.md                      # Changelog (updated)
└── Cargo.toml                        # Dependencies (flate2 added)
```

## 📊 Statistics

### Code Added
- **~2,080 lines** of Rust code (parsing, rendering, animation)
- **4 parser implementations** (sprites, ANSI, DurDraw, Aseprite)
- **4 example applications**
- **6 pre-built sprite examples**

### Documentation Added
- **~123 KB** of documentation
- **9 new documentation files**
- **4 major guides** (Quick Start, Animation, Designer, Testing)
- **4 asset contribution docs** (Guide, Template, README, Artist Template)
- **30+ code examples** across all docs
- **3 updated documentation files**

### Features
- ✅ 3 file format parsers
- ✅ Unicode block rendering
- ✅ Color palette system
- ✅ Multi-frame animation
- ✅ Async playback
- ✅ Transparency support
- ✅ SAUCE metadata
- ✅ Zlib decompression
- ✅ Alpha blending
- ✅ Layer compositing

## 🧪 Testing

### Manual Testing
```bash
# Test all demos
cargo run --example sprite_demo
cargo run --example ansi_art_demo
cargo run --example durdraw_demo
cargo run --example aseprite_demo

# With debug logging
RUST_LOG=debug cargo run --example sprite_demo
```

### Test Coverage
- Unit tests for color conversion
- Frame validation tests
- Parser tests for all formats
- Animation playback tests
- Error handling tests

## 🎨 Pre-Made Examples

The system includes 5 ready-to-use sprite animations:

1. **Idle Character** (8x8) - Humanoid sprite with breathing animation
2. **Walking Animation** (8x8, 4 frames) - Complete walk cycle
3. **Heart Pulse** (6x6, 3 frames) - Beating heart effect
4. **Spinning Coin** (8x8, 4 frames) - 3D coin rotation
5. **Loading Spinner** (5x5, 8 frames) - Circular loading indicator

All with pre-configured color palettes and timing.

## 🚀 Usage Examples

### Programmatic Animation
```rust
use cmdai::rendering::*;

let palette = ColorPalette::from_hex_strings(&[
    "#000000", "#FF5733", "#33FF57"
])?;

let frame = SpriteFrame::new(4, 4, vec![
    0, 1, 1, 0,
    1, 1, 1, 1,
    0, 1, 1, 0,
    0, 2, 2, 0,
], 200)?;

let sprite = Sprite::new("demo", palette, vec![frame])?;
let animator = Animator::new();
let mut animation = Animation::new(sprite, AnimationMode::Loop);
animator.play(&mut animation).await?;
```

### Loading ANSI Art
```rust
let (frame, metadata) = AnsiParser::load_file("artwork.ans")?;
if let Some(sauce) = metadata {
    println!("Title: {}", sauce.title);
    println!("Author: {}", sauce.author);
}
let renderer = TerminalRenderer::new();
renderer.print_ansi_frame(&frame)?;
```

### Loading Aseprite Files
```rust
let ase_file = AsepriteParser::load_file("sprite.ase")?;
println!("Dimensions: {}x{}", ase_file.header.width, ase_file.header.height);
println!("Frames: {}", ase_file.header.frames);
let sprite = AsepriteParser::to_sprite(&ase_file)?;
```

## 🎓 Designer-Friendly Features

### No Coding Required
- Upload files via GitHub web interface (drag & drop)
- Test animations using demo applications
- Complete documentation in plain English
- Step-by-step tutorials with screenshots descriptions
- Templates for all documentation needs

### Workflow Support
- **Aseprite**: Full binary format support
- **Text Editors**: ANSI art with escape sequences
- **JSON Editors**: DurDraw format

### Pre-Made Resources
- 4 color palettes (Game Boy, Fire, Ocean, CGA)
- Animation timing recommendations
- Size guidelines (4x4 to 48x48)
- Character selection guidance

## 📋 Migration Guide

### For Existing Users
No breaking changes. This is a purely additive feature.

### For New Users
1. Read `docs/QUICKSTART_ANIMATIONS.md`
2. Run `cargo run --example sprite_demo`
3. Follow the "Your First Animation" tutorial
4. Explore other formats as needed

### For Artists Contributing Assets
1. Read `docs/CONTRIBUTING_ASSETS.md`
2. Create GitHub account
3. Get added as collaborator
4. Follow step-by-step upload process
5. Use templates for documentation and licensing

## 🔍 Review Checklist

- [x] All parsers handle errors gracefully
- [x] Comprehensive documentation for all audiences
- [x] Examples demonstrate all major features
- [x] Licensing framework protects artists
- [x] No breaking changes to existing code
- [x] Cross-platform compatibility (macOS, Linux, Windows)
- [x] Performance optimization (lazy loading, async)
- [x] Memory safety (no unsafe code in parsers)
- [x] Proper attribution system
- [x] Clear separation of code and asset licenses

## 🎯 Future Enhancements

Potential future additions (not in this PR):
- GIF export from animations
- Real-time editing preview
- Web-based sprite editor
- Animation compression/optimization
- More pre-built examples
- Community asset gallery
- Integration with terminal UI frameworks

## 📝 Commits

This PR includes:
- `8f5c29d` - feat: Add Aseprite binary file format support for sprite animations
- `d1a84e7` - docs: Add comprehensive animation system documentation and user guides
- `ef56670` - docs: Add comprehensive asset contribution and licensing documentation

## 🙏 Acknowledgments

This feature enables:
- **Developers** to add personality to terminal applications
- **Designers** to contribute artwork without coding
- **Artists** to protect their creative work
- **Users** to enjoy beautiful terminal animations

Special thanks to the open-source community for tools like Aseprite, DurDraw, and the ANSI art scene that inspired this work.

---

**Ready for review!** This PR adds a complete terminal animation system with designer-friendly contribution framework while maintaining full protection of artists' rights. 🎨✨
