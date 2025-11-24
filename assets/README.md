# cmdai Assets Directory

This directory contains artwork, animations, and other visual assets contributed to the cmdai terminal animation system.

## ⚠️ Important Licensing Notice

**The assets in this directory are NOT covered by the project's main AGPL-3.0 license.**

Each artist retains copyright over their work and specifies their own license terms. Please check the `LICENSE.md` file in each artist's directory before using any assets.

### Licensing Summary

| Component | License | Can I Use It? |
|-----------|---------|---------------|
| **cmdai Code** | AGPL-3.0 | ✅ Yes, freely (with GPL obligations) |
| **Artist Assets** | Various (see individual LICENSE.md) | ⚠️ Check each artist's license |

**Default assumption**: Unless otherwise stated, all artwork is **copyrighted and not open source**. You can view and use it within the cmdai project, but you cannot redistribute it separately or use it in other projects without permission.

## Directory Structure

```
assets/
├── README.md (this file)
├── examples/                    # Example/tutorial assets (if any)
├── [artist-name-1]/            # Assets by first artist
│   ├── README.md              # About the artist and their work
│   ├── LICENSE.md             # License for this artist's assets
│   ├── [character-name]/      # Character-specific assets
│   │   ├── source/           # Original source files (.ase, etc.)
│   │   ├── export/           # Exported formats (.ans, .dur, .png)
│   │   └── README.md         # Character documentation
│   └── [other-assets]/
├── [artist-name-2]/            # Assets by second artist
│   └── ...
└── [artist-name-3]/
    └── ...
```

## Contributing Artists

This section lists all artists who have contributed to cmdai. Thank you for making this project visually special! 🎨

<!-- Artists: Add your entry here when you contribute -->

### Active Contributors

#### [Artist Name]
- **Assets**: [Brief description of what they contributed]
- **Folder**: [`[artist-folder-name]/`](./ [artist-folder-name]/)
- **License**: [License type, e.g., "Restrictive Asset License" or "CC BY 4.0"]
- **Contact**: [Optional: GitHub, website, etc.]

<!-- Example entry:

#### Alrezky Caesaria
- **Assets**: Kyaro character design and animations
- **Folder**: [`alrezky/`](./alrezky/)
- **License**: Restrictive Asset License (not open source)
- **Contact**: [@alrezky](https://github.com/alrezky)

-->

## Asset Categories

### Characters
Original characters designed specifically for cmdai.

| Character | Artist | Folder | License | Preview |
|-----------|--------|--------|---------|---------|
| [Character Name] | [Artist] | [Link] | [License Type] | [Link to preview] |
<!-- Add rows as characters are contributed -->

### UI Elements
Reusable user interface components (buttons, spinners, icons, etc.).

| Element | Artist | Folder | License | Preview |
|---------|--------|--------|---------|---------|
| [Element Name] | [Artist] | [Link] | [License Type] | [Link to preview] |
<!-- Add rows as UI elements are contributed -->

### Generic Sprites
Generic sprites and animations that aren't character-specific.

| Sprite | Artist | Folder | License | Preview |
|--------|--------|--------|---------|---------|
| [Sprite Name] | [Artist] | [Link] | [License Type] | [Link to preview] |
<!-- Add rows as sprites are contributed -->

## How to Contribute

**Artists**: Want to contribute your artwork to cmdai? Please read:

📖 **[Contributing Assets Guide](../docs/CONTRIBUTING_ASSETS.md)**

This comprehensive guide explains:
- How to organize and upload your files
- How to protect your work with a proper license
- What file formats to include
- How to document your assets
- How to test your animations

### Quick Start for Contributors

1. **Create a GitHub account** (if you don't have one)
2. **Get added as a collaborator** (ask the project owner)
3. **Create your folder**: `assets/your-name/`
4. **Add three files**:
   - `README.md` - About you and your assets
   - `LICENSE.md` - How your assets can be used
   - Your artwork files
5. **Test your assets** using the demo applications
6. **Done!** 🎉

See the [full guide](../docs/CONTRIBUTING_ASSETS.md) for detailed instructions.

## Using Assets in Your Code

### Loading Aseprite Files

```rust
use cmdai::rendering::{AsepriteParser, Animator, Animation, AnimationMode};

// Load from assets directory
let ase_file = AsepriteParser::load_file(
    "assets/artist-name/character-name/source/character.ase"
)?;

// Convert to sprite
let sprite = AsepriteParser::to_sprite(&ase_file)?;

// Animate it
let mut animation = Animation::new(sprite, AnimationMode::Loop);
let animator = Animator::new();
animator.play(&mut animation).await?;
```

### Loading ANSI Art

```rust
use cmdai::rendering::{AnsiParser, TerminalRenderer};

// Load ANSI file
let (frame, metadata) = AnsiParser::load_file(
    "assets/artist-name/character-name/export/character.ans"
)?;

// Render it
let renderer = TerminalRenderer::new();
renderer.print_ansi_frame(&frame)?;
```

### Loading DurDraw Files

```rust
use cmdai::rendering::{DurDrawParser, TerminalRenderer};

// Load DurDraw file
let dur_file = DurDrawParser::load_file(
    "assets/artist-name/character-name/export/character.dur"
)?;

// Convert to frame
let frame = DurDrawParser::to_ansi_frame(&dur_file)?;

// Render it
let renderer = TerminalRenderer::new();
renderer.print_ansi_frame(&frame)?;
```

## License Compliance

### For Users

When using cmdai:
- ✅ You can run the software including all assets
- ✅ You can contribute to the project
- ✅ You can compile and distribute cmdai for non-commercial use
- ⚠️ Check individual asset licenses for specific restrictions

### For Redistributors

When redistributing cmdai:
- ✅ The AGPL-3.0 code license applies to the source code
- ⚠️ Individual asset licenses apply to artwork
- ❌ You may need to **exclude** assets with restrictive licenses
- ✅ You can **replace** restricted assets with your own artwork

**Important**: Before redistributing, check each `assets/*/LICENSE.md` file to understand what you can and cannot include.

### For Fork Creators

If you're forking cmdai:
- ✅ Fork the code freely under AGPL-3.0
- ⚠️ Check asset licenses before including artwork
- ✅ Replace restricted assets with your own
- ✅ Link to the original project for asset downloads

**Recommended approach**: Fork without assets, then:
1. Create your own original artwork, or
2. Use assets with permissive licenses (CC BY, etc.), or
3. Get permission from the original artists

## Attribution

### In the Application

If you use cmdai with contributed assets, please provide attribution:

**In a Credits or About screen**:
```
cmdai Terminal Animation System
https://github.com/wildcard/cmdai

Artwork contributions:
- [Character Name] by [Artist Name]
  © [Year] [Artist Name]
  Licensed under [License Type]
```

**In the README**:
```markdown
## Credits

**Code**: AGPL-3.0 License
**Artwork**:
- [Character Name] © [Artist Name] - [License]
```

### In Documentation

When documenting or presenting cmdai:
- Always credit the artists
- Link to their portfolios (if provided)
- Respect their license terms
- Don't claim their work as your own

## Support for Artists

### Why This Structure?

This dual-licensing approach (code vs. assets) allows:

✅ **For Developers**:
- Freely use and modify the code
- Collaborate on the open-source project
- Learn from the implementation

✅ **For Artists**:
- Retain copyright over their work
- Control how their art is used
- Get proper attribution
- Protect their original characters

✅ **For Users**:
- Enjoy beautiful terminal animations
- See high-quality artwork in action
- Support both developers and artists

### Respecting Artists' Rights

Please remember:
- Creating artwork takes time, skill, and creativity
- Artists deserve recognition and control over their work
- Not everything in an open-source project is automatically open source
- Respecting licenses helps encourage more contributions

**If you're unsure whether you can use an asset**, ask the artist!

## Frequently Asked Questions

### Can I use these assets in my own project?

**Check the license!** Each artist specifies their own terms in `assets/[artist-name]/LICENSE.md`.

- **Restrictive licenses**: Usually no, only within cmdai
- **Creative Commons (CC BY)**: Yes, with attribution
- **Public Domain**: Yes, freely
- **When in doubt**: Contact the artist

### Can I modify the assets?

Again, **check the license**:
- Some licenses prohibit modifications
- Some allow modifications with attribution
- Some require modifications be shared under the same license

### What if I want to fork cmdai?

You have several options:

1. **Fork without assets**: Most common approach
   - Fork the code
   - Create your own artwork
   - Link to original project for assets

2. **Fork with permissive assets only**:
   - Include only CC BY or similar licensed assets
   - Exclude restrictive assets
   - Clearly document what was excluded

3. **Get permission**:
   - Contact the artists
   - Ask for permission to include their work
   - Get written confirmation

### How do I contact an artist?

Check their `README.md` file in `assets/[artist-name]/README.md` for contact information.

### Can I commission artwork for cmdai?

Yes! If you want custom artwork:
1. Contact an artist directly
2. Commission the work
3. Work with them to add it to the repository
4. They'll specify the license terms

## Resources

### For Artists
- [Contributing Assets Guide](../docs/CONTRIBUTING_ASSETS.md) - Complete guide
- [Asset License Template](../ASSET-LICENSE-TEMPLATE.md) - License template
- [Designer Guide](../docs/DESIGNER_GUIDE.md) - Workflow guide
- [Creative Commons Chooser](https://creativecommons.org/choose/) - CC license selector

### For Developers
- [Animation Guide](../docs/ANIMATION_GUIDE.md) - API documentation
- [Quick Start](../docs/QUICKSTART_ANIMATIONS.md) - Getting started
- [Testing Guide](../docs/TESTING_ANIMATIONS.md) - Testing procedures

### External Tools
- [Aseprite](https://www.aseprite.org/) - Pixel art editor
- [DurDraw](https://github.com/cmang/durdraw) - ANSI art editor
- [Moebius](https://github.com/blocktronics/moebius) - ANSI/ASCII editor

## Acknowledgments

Thank you to all artists who have contributed to cmdai! Your work brings personality and charm to terminal-based applications. 🎨✨

Special thanks to:
- [Artist names will be listed here as they contribute]

---

**Questions about assets or licensing?**
- Create a [GitHub Issue](https://github.com/wildcard/cmdai/issues)
- Tag it with "assets" or "licensing"
- The community will help!

---

*This directory was last updated: 2025-11-18*
