# Contributing to Terminal Sprite Animations

> **Welcome!** This project needs your help to grow into an amazing ecosystem for terminal animations. Whether you're a developer, designer, or just enthusiastic about TUI apps, there's a place for you here.

## Table of Contents

- [Why Contribute?](#why-contribute)
- [Ways to Contribute](#ways-to-contribute)
- [For Developers](#for-developers)
- [For Designers/Artists](#for-designersartists)
- [For Documentation Writers](#for-documentation-writers)
- [Good First Issues](#good-first-issues)
- [Development Workflow](#development-workflow)
- [Code Review Process](#code-review-process)
- [Getting Help](#getting-help)

## Why Contribute?

This project has the potential to become **the standard** for terminal animations in Rust. Your contributions can:

✅ **Help thousands of developers** add personality to their CLI tools
✅ **Showcase your work** - Your sprites/code will be used by many projects
✅ **Learn Rust** - Great codebase to learn from
✅ **Build your portfolio** - Real open-source contribution
✅ **Shape the ecosystem** - Influence how terminal UIs evolve

**Current status**: Early stage, high impact potential!

## Ways to Contribute

### 1. Code Contributions 💻

**What we need**:
- New file format parsers (GIF, PNG sprite sheets)
- Performance optimizations
- Additional Ratatui widgets
- Game engine integrations
- Bug fixes
- Test coverage improvements

**Skill level**: Intermediate to Advanced Rust

### 2. Artwork & Sprites 🎨

**What we need**:
- More pre-made sprite examples
- Themed sprite packs (sci-fi, fantasy, emojis)
- UI elements (buttons, progress bars, icons)
- Character sets for games
- Loading animations

**Skill level**: Pixel art skills, no coding required!

### 3. Documentation 📚

**What we need**:
- More tutorials and examples
- Video tutorials / GIFs
- Blog posts about using the system
- API documentation improvements
- Translation to other languages

**Skill level**: Beginner friendly!

### 4. Examples & Demos 🎮

**What we need**:
- More progressive tutorials
- Real-world integration examples
- Game examples
- Dashboard examples
- Creative use cases

**Skill level**: Beginner to Intermediate

### 5. Testing & QA 🐛

**What we need**:
- Bug reports with reproduction steps
- Testing on different terminals
- Performance benchmarks
- Cross-platform testing (Windows, Linux, macOS)

**Skill level**: Beginner friendly!

### 6. Community Building 🌟

**What we need**:
- Answer questions in issues/discussions
- Share your projects using this system
- Write blog posts
- Give conference talks
- Spread the word on social media

**Skill level**: Everyone can help!

## For Developers

### Setting Up Development Environment

```bash
# Clone the repository
git clone https://github.com/wildcard/cmdai.git
cd cmdai

# Ensure you have Rust 1.75+
rustc --version

# Build the project
cargo build

# Run tests
cargo test

# Run a demo to verify everything works
cargo run --example ratatui_sprite_demo --features tui
```

### Architecture Overview

```
src/rendering/
├── mod.rs                  # Module exports
├── sprites.rs              # Core data structures
├── terminal.rs             # Raw terminal rendering
├── animator.rs             # Animation engine
├── ansi_parser.rs          # ANSI art file parser
├── durdraw_parser.rs       # DurDraw JSON parser
├── aseprite_parser.rs      # Aseprite binary parser
└── ratatui_widget.rs       # Ratatui integration
```

**Key traits and structs**:
- `Sprite` - Container for frames and palette
- `SpriteFrame` - Single animation frame
- `ColorPalette` - Color definitions
- `Animation` - Animation state machine
- `AnimationController` - High-level animation management

### Code Style

We follow standard Rust conventions:

```rust
// ✅ Good: Descriptive names, documented
/// Renders a sprite frame to a string with ANSI colors
pub fn render_frame(
    frame: &SpriteFrame,
    palette: &ColorPalette,
) -> RenderResult<String> {
    // Implementation...
}

// ❌ Bad: Unclear names, no docs
pub fn render(f: &SF, p: &CP) -> Res<Str> {
    // Implementation...
}
```

**Guidelines**:
- Use `rustfmt` for formatting: `cargo fmt`
- Use `clippy` for lints: `cargo clippy`
- Write doc comments for public APIs
- Add examples in doc comments
- Prefer explicit types over `auto`/`_`
- Use `Result` instead of panicking

### Adding a New Feature

**Example**: Adding GIF file format support

#### Step 1: Create an Issue

```markdown
Title: Add GIF file format parser

Description:
Add support for loading animated GIF files as sprites.

Goals:
- Parse GIF files using the `gif` crate
- Extract frames and timing
- Convert to Sprite format
- Handle transparency

Implementation plan:
1. Add `gif` dependency to Cargo.toml
2. Create `src/rendering/gif_parser.rs`
3. Implement GifParser with load_file() method
4. Add conversion to Sprite
5. Write tests
6. Add example: `examples/gif_demo.rs`
7. Update documentation
```

#### Step 2: Fork and Branch

```bash
# Fork the repo on GitHub first, then:
git clone https://github.com/YOUR_USERNAME/cmdai.git
cd cmdai
git checkout -b feature/gif-parser
```

#### Step 3: Implement

Create `src/rendering/gif_parser.rs`:

```rust
use std::path::Path;
use crate::rendering::{Sprite, SpriteFrame, ColorPalette, Color, RenderResult};

pub struct GifParser;

impl GifParser {
    /// Load a GIF file and convert to Sprite
    pub fn load_file<P: AsRef<Path>>(path: P) -> RenderResult<Sprite> {
        // Implementation using gif crate
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_gif() {
        // Test implementation
    }
}
```

#### Step 4: Add Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gif_loading() {
        let sprite = GifParser::load_file("test_data/test.gif").unwrap();
        assert_eq!(sprite.frame_count(), 5);
        assert_eq!(sprite.dimensions(), (16, 16));
    }

    #[test]
    fn test_gif_transparency() {
        // Test transparent pixels are handled correctly
    }
}
```

#### Step 5: Update Documentation

Add to `docs/ANIMATION_GUIDE.md`:

```markdown
### GIF Format Support

Load animated GIF files:

'''rust
use cmdai::rendering::GifParser;

let sprite = GifParser::load_file("animation.gif")?;
'''
```

#### Step 6: Create PR

```bash
git add .
git commit -m "feat: Add GIF file format parser

- Add gif dependency
- Implement GifParser with load_file()
- Convert GIF frames to Sprite format
- Handle transparency and timing
- Add tests and example
- Update documentation"

git push origin feature/gif-parser
```

Then create a Pull Request on GitHub!

### Testing Guidelines

**Unit tests**:
```rust
#[test]
fn test_frame_dimensions() {
    let frame = SpriteFrame::new(8, 8, vec![0; 64], 100).unwrap();
    assert_eq!(frame.dimensions(), (8, 8));
}
```

**Integration tests**:
```rust
#[test]
fn test_load_and_animate() {
    let sprite = create_heart_animation().unwrap();
    let mut ctrl = AnimationController::new(sprite, AnimationMode::Loop);

    ctrl.update();
    assert!(ctrl.current_frame().get_pixel(0, 0).is_some());
}
```

**Performance tests**:
```rust
#[test]
fn bench_rendering() {
    let sprite = create_heart_animation().unwrap();
    let start = Instant::now();

    for _ in 0..1000 {
        render_frame(/* ... */);
    }

    assert!(start.elapsed() < Duration::from_millis(100));
}
```

## For Designers/Artists

### Contributing Sprites

See [CONTRIBUTING_ASSETS.md](CONTRIBUTING_ASSETS.md) for the complete guide!

**Quick version**:

1. **Create your sprite** in Aseprite or your favorite pixel art tool
2. **Export** as `.ase`, `.ans`, or `.dur` format
3. **Upload** to `assets/your-name/` via GitHub web interface
4. **Add a license** using [ASSET-LICENSE-TEMPLATE.md](../ASSET-LICENSE-TEMPLATE.md)
5. **Done!** You'll be credited in the project

**We especially need**:
- ⭐ UI elements (buttons, borders, icons)
- ⭐ Loading animations (spinners, progress bars)
- ⭐ Character sets (emotions, poses)
- ⭐ Themed packs (cyberpunk, fantasy, etc.)

**No coding required!**

### Sprite Guidelines

**Size recommendations**:
- Icons: 4x4 to 8x8
- UI elements: 8x8 to 16x16
- Characters: 16x16 to 32x32
- Scenes: 32x32 to 48x48

**Color guidelines**:
- Use limited palettes (4-16 colors)
- Ensure good contrast for terminal visibility
- Test in multiple terminals
- Consider color-blind users

**Animation guidelines**:
- Keep frame counts low (3-8 frames typical)
- Smooth frame rates: 8-12 FPS for slow, 24-30 for fast
- Loop cleanly (last frame → first frame)
- Use consistent timing

## For Documentation Writers

### What Makes Good Documentation?

**✅ Good documentation**:
- Assumes minimal prior knowledge
- Shows complete, working examples
- Explains WHY, not just WHAT
- Includes common mistakes section
- Has "next steps" links

**❌ Bad documentation**:
- Assumes expert knowledge
- Incomplete code snippets
- No context or explanation
- No troubleshooting help

### Documentation Needs

**High priority**:
- More beginner tutorials (Tutorial 04, 05, ...)
- Video tutorials / screencasts
- Integration examples with popular TUI apps
- Troubleshooting guide expansion
- Performance optimization guide

**Medium priority**:
- API reference completeness
- Architecture decision records
- Migration guides
- Comparison with alternatives

**Low priority** (nice to have):
- Translation to other languages
- Cheat sheets / quick references
- Blog posts and articles

### Writing Style Guide

**For tutorials**:
- Use "you" and "we" (conversational)
- Short paragraphs (3-4 lines max)
- Code first, explanation after
- Show expected output
- List common mistakes

**For API docs**:
- Start with one-sentence summary
- Show basic example
- Explain parameters
- Document errors
- Link to related functions

**Example**:
```rust
/// Loads an Aseprite file and converts it to a Sprite.
///
/// # Example
///
/// '''rust
/// let sprite = AsepriteParser::load_file("character.ase")?;
/// println!("Loaded {} frames", sprite.frame_count());
/// '''
///
/// # Errors
///
/// Returns an error if:
/// - File doesn't exist
/// - File is not a valid Aseprite file
/// - Unsupported Aseprite features are used
///
/// # See Also
///
/// - [`AnsiParser::load_file`] for ANSI art files
/// - [`DurDrawParser::load_file`] for DurDraw files
pub fn load_file<P: AsRef<Path>>(path: P) -> RenderResult<Sprite> {
    // ...
}
```

## Good First Issues

Looking for somewhere to start? Try these!

### Beginner-Friendly (No Rust experience needed)

1. **Add more sprite examples**
   - Create a new pre-made sprite in `examples.rs`
   - Skills needed: Pixel art
   - Difficulty: ⭐☆☆☆☆

2. **Improve error messages**
   - Make error messages more helpful
   - Skills needed: English writing
   - Difficulty: ⭐☆☆☆☆

3. **Write a blog post**
   - Share your experience using this system
   - Skills needed: Writing
   - Difficulty: ⭐☆☆☆☆

### Intermediate (Some Rust knowledge)

4. **Add Tutorial 04: Interactive Scene**
   - Create a tutorial showing moving sprites
   - Skills needed: Basic Rust, Ratatui
   - Difficulty: ⭐⭐☆☆☆

5. **Add FPS counter widget**
   - Create a reusable FPS display widget
   - Skills needed: Rust, Ratatui
   - Difficulty: ⭐⭐⭐☆☆

6. **Test on Windows/Linux**
   - Run all examples and report issues
   - Skills needed: Testing mindset
   - Difficulty: ⭐⭐☆☆☆

### Advanced (Rust experience required)

7. **Add PNG sprite sheet parser**
   - Parse PNG files with multiple sprites
   - Skills needed: Rust, image processing
   - Difficulty: ⭐⭐⭐⭐☆

8. **Optimize rendering performance**
   - Profile and optimize hot paths
   - Skills needed: Rust, performance tuning
   - Difficulty: ⭐⭐⭐⭐☆

9. **Add Bevy integration example**
   - Show how to use with Bevy game engine
   - Skills needed: Rust, Bevy, ECS
   - Difficulty: ⭐⭐⭐⭐⭐

## Development Workflow

### 1. Find or Create an Issue

Browse [Good First Issues](https://github.com/wildcard/cmdai/labels/good%20first%20issue)

Or create a new issue describing what you want to add.

### 2. Fork and Clone

```bash
# Fork on GitHub, then:
git clone https://github.com/YOUR_USERNAME/cmdai.git
cd cmdai
git remote add upstream https://github.com/wildcard/cmdai.git
```

### 3. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/bug-description
```

**Branch naming**:
- `feature/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation
- `refactor/` - Code improvements
- `test/` - Adding tests

### 4. Make Changes

- Write code
- Add tests
- Update documentation
- Run `cargo fmt` and `cargo clippy`

### 5. Commit

```bash
git add .
git commit -m "feat: Add feature description

- Bullet point 1
- Bullet point 2
- Closes #123"
```

**Commit message format**:
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation
- `test:` - Tests
- `refactor:` - Code improvement
- `perf:` - Performance improvement

### 6. Push and Create PR

```bash
git push origin feature/your-feature-name
```

Then create a Pull Request on GitHub!

**PR checklist**:
- [ ] Tests pass locally
- [ ] Code is formatted (`cargo fmt`)
- [ ] No clippy warnings
- [ ] Documentation updated
- [ ] Examples added if needed
- [ ] CHANGELOG.md updated

## Code Review Process

### What to Expect

1. **Initial review** (24-48 hours)
   - Maintainer will review your PR
   - May request changes
   - May suggest improvements

2. **Iteration**
   - Make requested changes
   - Push to same branch
   - PR updates automatically

3. **Approval**
   - Once approved, maintainer will merge
   - Your contribution is live!

### Review Criteria

**Code quality**:
- Follows Rust best practices
- No clippy warnings
- Properly formatted
- Has appropriate tests

**Documentation**:
- Public APIs are documented
- Examples are included
- README/guides updated if needed

**Functionality**:
- Solves the stated problem
- Doesn't break existing features
- Performs well

## Getting Help

### I'm stuck!

**Options**:
1. **GitHub Discussions** - Ask questions
2. **Discord** (if we have one) - Chat with community
3. **Issues** - Report blockers
4. **Email maintainers** - For private concerns

### I found a bug!

**Create an issue with**:
1. What you expected to happen
2. What actually happened
3. Steps to reproduce
4. Your environment (OS, Rust version, terminal)
5. Error messages (if any)

**Example**:
```markdown
Title: Sprite doesn't render in Windows Terminal

Expected: Heart sprite appears in color
Actual: Just see squares

Steps to reproduce:
1. Run `cargo run --example tutorial_01_hello_animated --features tui`
2. Using Windows Terminal 1.18

Environment:
- Windows 11
- Rust 1.75
- Windows Terminal 1.18.10821.0

Error: None, but sprites appear as white squares
```

### I have an idea!

**Create an issue with**:
1. What problem it solves
2. Proposed solution
3. Alternative approaches considered
4. Willingness to implement

We love new ideas!

## Recognition

**All contributors will be**:
- Listed in CONTRIBUTORS.md
- Credited in release notes
- Thanked in the project README
- Given a digital high-five 🙌

**Significant contributors may**:
- Become maintainers
- Get early access to new features
- Shape the project direction

---

## Quick Links

- [Good First Issues](https://github.com/wildcard/cmdai/labels/good%20first%20issue)
- [Documentation Index](README.md)
- [Asset Contribution Guide](CONTRIBUTING_ASSETS.md)
- [Getting Started](GETTING_STARTED_TUI.md)
- [Roadmap](ROADMAP.md)

---

**Thank you for contributing!** Every contribution, no matter how small, makes this project better. We're excited to see what you'll build! 🚀💚
