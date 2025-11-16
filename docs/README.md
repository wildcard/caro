# cmdai Documentation

Welcome to the cmdai documentation! This directory contains comprehensive guides for using and developing with cmdai.

## 📚 Documentation Index

### Sprite Animation System

Complete documentation for the terminal-based sprite animation rendering system:

| Document | Audience | Description | Reading Time |
|----------|----------|-------------|--------------|
| **[Quick Start Guide](QUICKSTART_ANIMATIONS.md)** | Everyone | Get your first animation running in 5 minutes | 5 min |
| **[Animation Guide](ANIMATION_GUIDE.md)** | Developers | Complete technical reference and API documentation | 30 min |
| **[Designer Guide](DESIGNER_GUIDE.md)** | UX/UI Designers | Workflow guide for creating animations with professional tools | 25 min |
| **[Testing Guide](TESTING_ANIMATIONS.md)** | QA/Developers | Comprehensive testing and validation procedures | 30 min |

### Other Documentation

| Document | Description |
|----------|-------------|
| **[QA Test Cases](qa-test-cases.md)** | Test cases for quality assurance |

## 🚀 Getting Started

**New to the animation system?** Start here:

1. **Read**: [Quick Start Guide](QUICKSTART_ANIMATIONS.md) (5 minutes)
2. **Run**: `cargo run --example sprite_demo`
3. **Create**: Follow the "Your First Animation" tutorial in the Quick Start
4. **Explore**: Try the other formats (ANSI, DurDraw, Aseprite)

## 👥 Documentation by Role

### For Developers
You're building Rust applications with terminal animations.

**Learning Path**:
1. [Quick Start Guide](QUICKSTART_ANIMATIONS.md) - Basic concepts
2. [Animation Guide](ANIMATION_GUIDE.md) - Complete API reference
3. [Testing Guide](TESTING_ANIMATIONS.md) - Quality assurance

**Key Sections**:
- Creating animations programmatically
- Loading files from disk
- Async animation playback
- Error handling
- Performance optimization

### For UX/UI Designers
You're creating pixel art animations for terminal applications.

**Learning Path**:
1. [Quick Start Guide](QUICKSTART_ANIMATIONS.md) - See what's possible
2. [Designer Guide](DESIGNER_GUIDE.md) - Tool workflows
3. [Testing Guide](TESTING_ANIMATIONS.md) - Validate your work

**Key Sections**:
- Aseprite workflow (recommended)
- Color palette creation
- Animation principles for terminals
- Testing without writing code
- Design guidelines for readability

### For QA/Testers
You're validating animation features and catching bugs.

**Learning Path**:
1. [Testing Guide](TESTING_ANIMATIONS.md) - Primary reference
2. [Animation Guide](ANIMATION_GUIDE.md) - Understanding the system
3. [Quick Start Guide](QUICKSTART_ANIMATIONS.md) - Running demos

**Key Sections**:
- Running test demos
- Creating test files
- Validation checklists
- Cross-platform testing
- Performance benchmarks

## 📖 Documentation Features

### Quick Start Guide
**Best for**: First-time users, quick demos

**What's inside**:
- 30-second demo command
- 5-minute "Your First Animation" tutorial
- Format overview (ANSI, DurDraw, Aseprite)
- Quick troubleshooting
- Next steps

### Animation Guide
**Best for**: Developers, API reference

**What's inside**:
- Complete API documentation
- All core types (Color, Sprite, Animation, etc.)
- Format specifications (ANSI, DurDraw, Aseprite)
- Creating animations from scratch
- Loading and saving files
- Animation control and timing
- Advanced rendering techniques
- Performance best practices
- Comprehensive troubleshooting

### Designer Guide
**Best for**: Artists, UX designers, non-programmers

**What's inside**:
- Tool selection (Aseprite vs text editors vs JSON)
- Step-by-step Aseprite workflow
- Creating ANSI art by hand
- DurDraw JSON format creation
- Pre-made color palettes
- Animation timing principles
- Design guidelines for terminals
- Testing animations without code
- Common issues and solutions

### Testing Guide
**Best for**: QA engineers, validation, debugging

**What's inside**:
- Running all demo applications
- Creating comprehensive test files
- Manual testing workflow
- Automated testing setup
- Validation checklists
- Performance testing procedures
- Cross-platform compatibility testing
- Debug logging and tools
- Visual regression testing
- Test report templates

## 🎯 Common Tasks

### "I want to see animations in action"
```bash
cargo run --example sprite_demo
cargo run --example ansi_art_demo
cargo run --example durdraw_demo
cargo run --example aseprite_demo
```

### "I want to create my first animation"
1. Read: [Quick Start Guide](QUICKSTART_ANIMATIONS.md#your-first-animation)
2. Copy the code template
3. Run it: `cargo run --example my_animation`

### "I want to use Aseprite to create animations"
1. Read: [Designer Guide - Aseprite Workflow](DESIGNER_GUIDE.md#workflow-1-aseprite-pixel-art-editor-recommended)
2. Create your sprite in Aseprite
3. Export as .ase file
4. Test: Follow the [Testing Guide](TESTING_ANIMATIONS.md#running-demo-applications)

### "I want to add animations to my Rust app"
1. Read: [Animation Guide - Creating Animations](ANIMATION_GUIDE.md#creating-animations)
2. Add dependency: `cmdai` to your `Cargo.toml`
3. Import: `use cmdai::rendering::*;`
4. Create and play your animation

### "I need to test animations thoroughly"
1. Read: [Testing Guide](TESTING_ANIMATIONS.md)
2. Run all demos: `make test-animations` (or manually)
3. Follow validation checklist
4. Test across terminal emulators

### "I'm getting errors"
Check the troubleshooting sections:
- [Quick Start - Common Issues](QUICKSTART_ANIMATIONS.md#common-issues)
- [Animation Guide - Troubleshooting](ANIMATION_GUIDE.md#troubleshooting)
- [Designer Guide - Common Issues](DESIGNER_GUIDE.md#common-issues-and-solutions)
- [Testing Guide - Debugging](TESTING_ANIMATIONS.md#debugging-guide)

## 🔗 External Resources

### Tools
- **[Aseprite](https://www.aseprite.org/)** - Professional pixel art editor
- **[DurDraw](https://github.com/cmang/durdraw)** - Terminal ANSI art editor
- **[Moebius](https://github.com/blocktronics/moebius)** - ANSI/ASCII art editor
- **[PabloDraw](https://picoe.ca/products/pablodraw/)** - ANSI/ASCII editor

### Learning Resources
- **[Pixel Art Tutorial](https://lospec.com/pixel-art-tutorials)** - Lospec tutorials
- **[ANSI Art Tutorial](https://16colo.rs/info/faq)** - 16colors.org FAQ
- **[Animation Principles](https://en.wikipedia.org/wiki/Twelve_basic_principles_of_animation)** - Classic animation principles

### Communities
- **[Lospec](https://lospec.com/)** - Pixel art community and resources
- **[16colors.org](https://16colo.rs/)** - ANSI/ASCII art archive
- **[ANSI Art Discord](https://discord.gg/FfJ9rWN)** - ANSI artists community

## 📝 Contributing to Documentation

Found an issue or want to improve the documentation?

1. **Typos/errors**: Submit a PR with fixes
2. **Missing information**: Open an issue describing what's needed
3. **New examples**: Add them to the appropriate guide
4. **Clarifications**: Suggest improvements in issues

### Documentation Standards
- Use clear, beginner-friendly language
- Include practical code examples
- Add expected output descriptions
- Cross-reference related sections
- Test all code snippets
- Update table of contents

## 📄 License

All documentation is licensed under [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/), except where code examples are provided, which follow the project's AGPL-3.0 license.

---

**Questions?** Check the troubleshooting sections in each guide or open an issue on GitHub.
