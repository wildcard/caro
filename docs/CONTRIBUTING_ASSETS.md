# Contributing Artwork and Assets to cmdai

> **For Artists and Designers**: A complete guide to uploading your artwork, animations, and assets to the cmdai project.

This guide explains how to contribute your artwork to the cmdai terminal animation system, including file organization, licensing, and best practices for collaborating on GitHub.

## Table of Contents

- [Overview](#overview)
- [Getting Started](#getting-started)
- [Understanding Licensing](#understanding-licensing)
- [Asset Organization](#asset-organization)
- [Step-by-Step Upload Process](#step-by-step-upload-process)
- [Creating Your Asset README](#creating-your-asset-readme)
- [Licensing Your Artwork](#licensing-your-artwork)
- [What Assets to Include](#what-assets-to-include)
- [File Naming Conventions](#file-naming-conventions)
- [Testing Your Assets](#testing-your-assets)
- [Common Questions](#common-questions)
- [Getting Help](#getting-help)

## Overview

When you create artwork for cmdai, there are **two separate things**:

1. **The Code** (written by developers)
   - Licensed under AGPL-3.0 (open source)
   - Anyone can use, modify, and redistribute the code
   - This is the technical infrastructure

2. **Your Artwork** (created by you, the artist)
   - **Your artwork is NOT automatically open source**
   - You retain copyright and control over your art
   - You choose how others can use it with a separate license
   - This includes all sprites, animations, pixel art, ANSI art, etc.

**Important**: The code license (AGPL-3.0) does **NOT** apply to your artwork. Your art needs its own license.

## Getting Started

### Prerequisites

Before you begin, you'll need:

1. **GitHub Account**
   - Create one at [github.com](https://github.com) (it's free)
   - Choose a username that represents you professionally
   - Verify your email address

2. **Repository Access**
   - The project owner will add you as a collaborator
   - Check your email for the invitation
   - Accept the invitation to get access

3. **Your Artwork Files**
   - Have all your files ready to upload
   - See [What Assets to Include](#what-assets-to-include) below

### No Coding Required!

**Good news**: You don't need to know how to code or use Git command-line tools. You can do everything through the GitHub website using your browser.

## Understanding Licensing

### Why Do We Need Separate Licenses?

Think of it like this:
- The **code** is like the frame and engine of a car (open source)
- Your **artwork** is like the custom paint job and design (your intellectual property)

Just because someone can use the engine doesn't mean they can copy your unique design!

### Two Types of Asset Licenses

#### 1. Fully Restrictive License (Recommended for Original Characters)

**Use this if**: You created an original character (like "Kyaro") that is unique to this project.

**What it means**:
- ✅ People can view your art when using the project
- ✅ People can test and contribute to the project code
- ❌ People cannot use your art in other projects
- ❌ People cannot redistribute your art separately
- ❌ People cannot modify and republish your art
- ❌ People cannot use your art commercially

**Example**: The Kyaro character created by Alrezky Caesaria

#### 2. Permissive License (For Generic/Reusable Assets)

**Use this if**: You created generic sprites that could be useful in many projects (like UI elements, common icons, etc.).

**Options**:
- **Creative Commons Attribution (CC BY 4.0)**: Others can use with attribution
- **Creative Commons Attribution-ShareAlike (CC BY-SA 4.0)**: Others can use and modify with attribution
- **Creative Commons Attribution-NonCommercial (CC BY-NC 4.0)**: Non-commercial use only

**Example**: Generic loading spinners, basic UI elements

### What License Should I Use?

Ask yourself:

1. **Is this a unique character or branded artwork?**
   - → Use Fully Restrictive License

2. **Did I create this specifically for this project?**
   - → Use Fully Restrictive License

3. **Is this generic/reusable artwork?**
   - → Consider Creative Commons (CC BY 4.0 or similar)

4. **Am I unsure?**
   - → Start with Fully Restrictive License (you can always relax it later)

**Default recommendation**: Use the **Fully Restrictive License** template provided in this repository.

## Asset Organization

### Directory Structure

All artwork goes in the `assets/` directory with this structure:

```
cmdai/
├── assets/                          # All artwork and assets
│   ├── README.md                    # Overview of all assets
│   ├── [artist-name]/               # Your personal folder
│   │   ├── README.md               # Description of your assets
│   │   ├── LICENSE.md              # License for your specific assets
│   │   ├── [character-name]/       # Folder for each character/project
│   │   │   ├── source/            # Original source files
│   │   │   │   ├── character.ase  # Aseprite file
│   │   │   │   └── sketches/      # Concept art, sketches
│   │   │   ├── export/            # Exported animation files
│   │   │   │   ├── character.ans  # ANSI art
│   │   │   │   ├── character.dur  # DurDraw format
│   │   │   │   └── frames/        # Individual frames
│   │   │   └── README.md          # Character-specific documentation
│   │   └── shared/                # Shared/generic assets
│   └── examples/                   # Example assets (optional)
└── docs/                            # Documentation
```

### Example: Kyaro Character

Here's how the Kyaro character assets would be organized:

```
assets/
├── README.md
├── alrezky/                         # Artist's folder
│   ├── README.md                    # About Alrezky's contributions
│   ├── LICENSE.md                   # Kyaro asset license
│   ├── kyaro/                       # Kyaro character folder
│   │   ├── source/
│   │   │   ├── kyaro-idle.ase      # Aseprite source
│   │   │   ├── kyaro-walk.ase      # Walking animation
│   │   │   └── sketches/
│   │   │       ├── concept-01.png
│   │   │       └── color-studies.png
│   │   ├── export/
│   │   │   ├── kyaro-idle.ans      # ANSI version
│   │   │   ├── kyaro-walk.dur      # DurDraw version
│   │   │   └── frames/
│   │   │       ├── walk-01.png
│   │   │       ├── walk-02.png
│   │   │       └── walk-03.png
│   │   └── README.md               # About Kyaro
│   └── ui-elements/                 # Other assets
│       ├── spinner.ase
│       └── icons.ase
```

## Step-by-Step Upload Process

### Method 1: GitHub Web Interface (Recommended for Beginners)

No Git knowledge required! Use your web browser:

#### Step 1: Navigate to the Repository

1. Go to the cmdai repository on GitHub
2. Make sure you're logged in
3. You should see the repository files

#### Step 2: Create Your Artist Folder

1. Click on the `assets/` folder
2. Click the **"Add file"** button (top right)
3. Select **"Create new file"**
4. In the filename box, type: `your-name/README.md`
   - Replace `your-name` with your actual name or username
   - Example: `alrezky/README.md`
5. This creates the folder automatically!

#### Step 3: Add Your README

In the file editor, paste the README template (see [Creating Your Asset README](#creating-your-asset-readme) below).

Fill in your information:
- Your name
- Brief bio
- Description of your assets
- Contact information (optional)

Click **"Commit new file"** at the bottom.

#### Step 4: Add Your License File

1. While still in your folder (`assets/your-name/`), click **"Add file"** → **"Create new file"**
2. Name it: `LICENSE.md`
3. Copy the license template from `ASSET-LICENSE-TEMPLATE.md` (see [Licensing Your Artwork](#licensing-your-artwork) below)
4. Fill in your information
5. Click **"Commit new file"**

#### Step 5: Upload Your Artwork Files

1. In your folder (`assets/your-name/`), click **"Add file"** → **"Upload files"**
2. Drag and drop your files:
   - Aseprite files (.ase, .aseprite)
   - ANSI art files (.ans)
   - DurDraw files (.dur)
   - PNG exports
   - Concept art
3. Organize them into subfolders if needed (you can create folders while uploading)
4. Add a commit message like: "Add Kyaro character artwork and animations"
5. Click **"Commit changes"**

#### Step 6: Create Character-Specific Documentation

For each character or major asset collection:

1. Create a subfolder: `assets/your-name/character-name/`
2. Add a `README.md` explaining:
   - What the character is
   - What each file contains
   - How the animations work
   - Any special notes

### Method 2: Git Command Line (For Advanced Users)

If you're comfortable with Git:

```bash
# Clone the repository
git clone https://github.com/wildcard/cmdai.git
cd cmdai

# Create a new branch for your assets
git checkout -b add-artwork-your-name

# Create your directory structure
mkdir -p assets/your-name/character-name/{source,export,sketches}

# Copy your files
cp /path/to/your/files/*.ase assets/your-name/character-name/source/
cp /path/to/your/files/*.ans assets/your-name/character-name/export/

# Create README files
nano assets/your-name/README.md
nano assets/your-name/LICENSE.md
nano assets/your-name/character-name/README.md

# Add and commit
git add assets/your-name/
git commit -m "Add artwork by Your Name: Character Name

- Added Aseprite source files
- Added ANSI and DurDraw exports
- Added character documentation
- Added asset license"

# Push to GitHub
git push origin add-artwork-your-name

# Create a Pull Request on GitHub
```

## Creating Your Asset README

### Template for Artist README

Create `assets/your-name/README.md` with this content:

```markdown
# [Your Name] - Artwork for cmdai

**Artist**: [Your Full Name or Artist Name]
**Contact**: [Optional: Email, Website, Social Media]
**GitHub**: [@your-github-username](https://github.com/your-github-username)

## About

[Brief introduction about yourself and your art style]

Example:
> I'm a pixel artist specializing in retro-style character animations. I create
> expressive characters with limited color palettes inspired by classic 8-bit and
> 16-bit games.

## Assets Included

This directory contains my artwork contributions to the cmdai project:

### Characters

- **[Character Name]** (`character-name/`)
  - Description: [What is this character?]
  - Formats: Aseprite (.ase), ANSI (.ans), DurDraw (.dur)
  - Animations: Idle, Walk, [other states]
  - Color Palette: [Palette name or description]
  - Created: [Date]

### UI Elements

- **[Element Name]** (`ui-elements/`)
  - Description: [What are these?]
  - Formats: [List formats]

## File Organization

```
your-name/
├── README.md (this file)
├── LICENSE.md (asset license)
├── character-name/
│   ├── source/ (Aseprite originals)
│   ├── export/ (ANSI, DurDraw, PNG)
│   └── README.md (character documentation)
└── ui-elements/
    └── [various UI assets]
```

## License

All artwork in this directory is licensed under the terms specified in
[LICENSE.md](./LICENSE.md).

**TL;DR**: This artwork is protected by copyright. You can view and use it as
part of this project, but you cannot use it in other projects or redistribute
it without permission.

## Credits

- **Character Design**: [Your Name]
- **Pixel Art**: [Your Name]
- **Animations**: [Your Name]
- **Technical Integration**: [Developer Name, if applicable]

## Inspiration and References

[Optional: Share what inspired your work, references used, etc.]

## Contact

For questions about this artwork or licensing inquiries:
- Email: [your.email@example.com] (optional)
- Portfolio: [yourwebsite.com] (optional)
- Twitter/X: [@yourhandle] (optional)

---

Thank you for viewing my work! 💚
```

### Template for Character README

Create `assets/your-name/character-name/README.md`:

```markdown
# [Character Name]

![Character Preview](export/character-preview.png)

**Created by**: [Your Name]
**Created for**: cmdai terminal animation system
**Created**: [Month Year]

## Character Overview

[Describe the character: personality, appearance, purpose]

Example:
> Kyaro is a friendly dog character designed for terminal-based applications.
> With expressive animations and a warm color palette, Kyaro brings personality
> to command-line interfaces.

## Animations

This character includes the following animation states:

### Idle Animation
- **File**: `export/kyaro-idle.ans`
- **Frames**: 3
- **Frame Duration**: 200ms per frame
- **Loop**: Yes
- **Description**: Gentle breathing animation when character is at rest

### Walk Animation
- **File**: `export/kyaro-walk.dur`
- **Frames**: 4
- **Frame Duration**: 150ms per frame
- **Loop**: Yes
- **Description**: Side-scrolling walk cycle

### [Other Animations]
- ...

## Technical Specifications

- **Dimensions**: 16x16 pixels (or your actual size)
- **Color Depth**: 16 colors (ANSI palette)
- **Color Palette**:
  ```
  #0F380F (Dark outline)
  #FFD700 (Golden fur)
  #8B4513 (Brown details)
  #FFFFFF (White highlights)
  [list all colors]
  ```
- **Formats Available**:
  - Aseprite source (.ase) - `source/kyaro.ase`
  - ANSI art (.ans) - `export/kyaro.ans`
  - DurDraw (.dur) - `export/kyaro.dur`
  - PNG frames - `export/frames/`

## Files Included

### Source Files (`source/`)
- `kyaro-idle.ase` - Aseprite source for idle animation
- `kyaro-walk.ase` - Aseprite source for walk cycle
- `kyaro-master.ase` - Complete sprite sheet
- `sketches/` - Concept art and early designs

### Exported Files (`export/`)
- `kyaro-idle.ans` - ANSI art version (idle)
- `kyaro-walk.dur` - DurDraw version (walk)
- `frames/` - Individual PNG frames for each animation state

## Usage in Code

To use this character in cmdai:

```rust
use cmdai::rendering::{AsepriteParser, Animator, Animation, AnimationMode};

// Load the Aseprite file
let ase_file = AsepriteParser::load_file("assets/your-name/character-name/source/kyaro-idle.ase")?;

// Convert to sprite
let sprite = AsepriteParser::to_sprite(&ase_file)?;

// Create animation
let mut animation = Animation::new(sprite, AnimationMode::Loop);
let animator = Animator::new();

// Play it!
animator.play(&mut animation).await?;
```

Or load ANSI/DurDraw formats:

```rust
use cmdai::rendering::{AnsiParser, DurDrawParser, TerminalRenderer};

// Load ANSI
let (frame, metadata) = AnsiParser::load_file("assets/your-name/character-name/export/kyaro-idle.ans")?;

// Render it
let renderer = TerminalRenderer::new();
renderer.print_ansi_frame(&frame)?;
```

## Design Process

[Optional: Describe your creative process]

Example:
> Kyaro started as rough sketches inspired by classic pixel art dogs from
> 16-bit era games. I wanted to capture a friendly, approachable character
> that would feel at home in a terminal environment. The limited color palette
> was a fun challenge that forced creative use of dithering and contrast.

## License

This character and all associated artwork is licensed under the terms in
[LICENSE.md](../LICENSE.md).

Copyright © [Year] [Your Name]. All rights reserved.

---

Created with ❤️ by [Your Name]
```

## Licensing Your Artwork

### Using the Restrictive Asset License Template

Copy this template to `assets/your-name/LICENSE.md` and fill in your information:

```markdown
# [Character/Asset Name] Artwork License

**Copyright © [Year] [Your Full Name]. All rights reserved.**

This license applies **only** to the artwork and assets in this directory
(`assets/[your-name]/`), including but not limited to:

- All pixel art, sprites, and animations
- ASCII and ANSI artwork
- Aseprite or other source art files
- Concept art and sketches
- PNG exports and derivative files
- Any other visual assets created by [Your Name]

These files are collectively referred to as the "**[Asset Name] Assets**".

## Important Notice

The software code in the cmdai repository is licensed separately under the
AGPL-3.0 license (see the main `LICENSE` file). The [Asset Name] Assets are
**NOT** licensed under AGPL-3.0 or any other open-source license.

## 1. Ownership

The [Asset Name] Assets are owned by:
- **[Your Full Name]** (artist and creator)
- **[Project Owner Name]** (if jointly owned, optional)

All rights are reserved unless explicitly granted below.

## 2. Permitted Use

You are granted a limited, non-exclusive, non-transferable license to:

✅ **View and use** the [Asset Name] Assets **only as part of the original cmdai project**
✅ **Run, test, and contribute** to the cmdai repository including these assets
✅ **Compile and distribute** the cmdai software including these assets for non-commercial purposes

## 3. Prohibited Use

Unless you have prior written permission from the copyright holder(s), you may **NOT**:

❌ Use the [Asset Name] Assets in any other project, product, or service
❌ Redistribute the [Asset Name] Assets separately from the cmdai repository
❌ Create modified versions of the [Asset Name] Assets and distribute them
❌ Use the [Asset Name] Assets for commercial purposes
❌ Remove or obscure copyright notices from the assets
❌ Claim ownership or authorship of the [Asset Name] Assets
❌ Include the [Asset Name] Assets in forks or derivatives presented as separate projects

**Important**: If you fork or redistribute cmdai, you must **exclude** the
[Asset Name] Assets unless you have explicit written permission to include them.

## 4. Attribution

When using the [Asset Name] Assets as part of the cmdai project, you must:

- Preserve all copyright notices
- Credit the artist: "[Asset Name] artwork by [Your Name]"
- Include a link back to this license

## 5. Requests for Additional Use

If you would like to use the [Asset Name] Assets outside the scope of this
license (commercial use, derivative works, separate distribution, etc.), please
contact:

- **Email**: [your.email@example.com] (if you want to provide this)
- **GitHub**: [@your-github-username](https://github.com/your-github-username)

We may grant additional permissions on a case-by-case basis.

## 6. No Warranty

The [Asset Name] Assets are provided "as is", without warranty of any kind,
express or implied, including but not limited to warranties of merchantability,
fitness for a particular purpose, and non-infringement.

## 7. Termination

This license is effective until terminated. Your rights under this license will
terminate automatically if you violate any of these terms.

---

**Summary in Plain English**:

- ✅ You can use this art in the cmdai project
- ✅ You can look at the files and learn from them
- ✅ You can contribute to cmdai including this art
- ❌ You cannot use this art in your own projects
- ❌ You cannot sell or redistribute this art
- ❌ You cannot modify and republish this art

**Questions?** Contact [Your Name] at the email/contact above.

---

*Last updated: [Date]*
```

### For Creative Commons Licensing

If you want to use Creative Commons instead:

1. Visit [creativecommons.org/choose](https://creativecommons.org/choose/)
2. Answer the questions to select a license
3. Copy the license text and badge
4. Place it in your `LICENSE.md`

Popular choices:
- **CC BY 4.0**: Others can use with attribution
- **CC BY-SA 4.0**: Others can use with attribution and same license
- **CC BY-NC 4.0**: Non-commercial use only with attribution

## What Assets to Include

### Required Files

At minimum, include:

1. **README.md** (in your artist folder)
   - Who you are
   - What you contributed
   - How to use your assets

2. **LICENSE.md** (in your artist folder)
   - License terms for your artwork
   - Copyright notice

3. **Source Files**
   - Original Aseprite files (.ase, .aseprite)
   - Original working files
   - Layered formats when possible

### Recommended Files

Also include:

4. **Exported Files**
   - ANSI art (.ans)
   - DurDraw format (.dur)
   - PNG frames
   - Sprite sheets

5. **Documentation**
   - Character-specific READMEs
   - Animation specifications
   - Color palette information

6. **Concept Art** (optional but nice!)
   - Sketches
   - Color studies
   - Design iterations
   - Inspiration boards

### File Format Guidelines

| Format | Extension | Purpose | Include? |
|--------|-----------|---------|----------|
| **Aseprite** | `.ase`, `.aseprite` | Source files | ✅ Required |
| **ANSI Art** | `.ans` | Terminal display | ✅ Recommended |
| **DurDraw** | `.dur` | JSON format | ✅ Recommended |
| **PNG** | `.png` | Frames/exports | ✅ Recommended |
| **GIF** | `.gif` | Preview animations | ⚠️ Optional |
| **JPEG** | `.jpg` | Concept art | ⚠️ Optional |
| **PSD** | `.psd` | Photoshop sources | ⚠️ Optional |

### Don't Include

❌ **Do not include**:
- Temporary files (`.tmp`, `.bak`)
- OS-specific files (`.DS_Store`, `Thumbs.db`)
- Very large files (>10MB without good reason)
- Personal information you don't want public
- Work-in-progress that isn't ready
- Assets you don't own or didn't create

## File Naming Conventions

### Best Practices

✅ **Good naming**:
```
kyaro-idle.ase
kyaro-walk-cycle-v2.ase
ui-button-hover.ans
spinner-loading-16x16.dur
```

❌ **Bad naming**:
```
final-FINAL-v3-USE-THIS-ONE.ase
untitled-1.ase
????.ans
my stuff.dur (spaces!)
```

### Naming Rules

1. **Use lowercase letters**
   - Good: `character-name.ase`
   - Bad: `Character_Name.ase`

2. **Use hyphens, not spaces or underscores**
   - Good: `walk-cycle-left.ase`
   - Bad: `walk_cycle_left.ase` or `walk cycle left.ase`

3. **Be descriptive**
   - Good: `kyaro-idle-breathing.ase`
   - Bad: `animation2.ase`

4. **Include version numbers if needed**
   - Good: `character-v2.ase` or `character-2024-11.ase`
   - Bad: `character-final-FINAL.ase`

5. **Use consistent naming across related files**
   ```
   kyaro-idle.ase      (source)
   kyaro-idle.ans      (export)
   kyaro-idle.dur      (export)
   kyaro-idle.png      (export)
   ```

### Directory Naming

Folder names should be:
- Lowercase
- Hyphenated (not spaces)
- Descriptive

Examples:
```
assets/alrezky/kyaro/          ✅ Good
assets/alrezky/ui-elements/    ✅ Good
assets/My Folder/              ❌ Bad (spaces, capitals)
assets/stuff/                   ❌ Bad (not descriptive)
```

## Testing Your Assets

### After Uploading

1. **Verify Files Are Accessible**
   - Navigate to your folder on GitHub
   - Click on each file to make sure it uploads correctly
   - Check that images display properly

2. **Test with the Demo Applications**

If you included Aseprite files:
```bash
# Clone or pull the latest code
git pull

# Test your Aseprite file
cargo run --example aseprite_demo

# Modify the demo to point to your file
# In examples/aseprite_demo.rs, change the file path to yours
```

If you included ANSI files:
```bash
cargo run --example ansi_art_demo
```

If you included DurDraw files:
```bash
cargo run --example durdraw_demo
```

3. **Visual Validation**
   - Do the colors look right?
   - Are the animations smooth?
   - Is the timing correct?
   - Do all frames render?

4. **Check Documentation**
   - Is your README clear?
   - Are the file descriptions accurate?
   - Can someone understand how to use your assets?

### Getting Feedback

After uploading:
1. Share the GitHub link with the project owner
2. Ask for feedback on your documentation
3. Make any requested changes
4. Celebrate your contribution! 🎉

## Common Questions

### Q: Do I need to know Git or coding?

**A**: No! You can upload files directly through the GitHub website. See [Step-by-Step Upload Process](#step-by-step-upload-process) above.

### Q: What if I make a mistake?

**A**: No problem! You can:
- Edit files directly on GitHub
- Delete and re-upload files
- Ask the project owner for help
- Everything is version-controlled, so mistakes are easy to fix

### Q: Can I update my artwork later?

**A**: Yes! You can upload new versions anytime. Consider:
- Adding version numbers to filenames (`character-v2.ase`)
- Or replacing the old files entirely
- Update your README to explain changes

### Q: What if someone violates my license?

**A**: Contact the project owner immediately. Include:
- Link to the violation
- Explanation of how your license was violated
- What you'd like to happen

### Q: Can I contribute assets under a different name?

**A**: Yes, use whatever name you want associated with your work:
- Your full legal name
- Your artist/brand name
- Your online handle
- Just be consistent!

### Q: What if I created the art with someone else?

**A**: In your LICENSE.md, list all co-creators:
```markdown
## Ownership

The [Asset Name] Assets are jointly owned by:
- **Your Name** (character design and pixel art)
- **Collaborator Name** (animation and technical)

All co-owners must agree to any licensing changes.
```

### Q: Should I include work-in-progress files?

**A**: That's up to you! Options:
- **Yes**: Shows your creative process, helps others learn
- **No**: Only show polished final work
- **Separate folder**: Put WIP in a `sketches/` or `wip/` folder

### Q: What about file size limits?

GitHub has some limits:
- Individual files: 100MB max (but keep under 10MB if possible)
- Repository total: 1GB recommended limit
- Large files: Use Git LFS if needed (ask project owner)

For pixel art and ANSI art, you'll rarely hit these limits.

### Q: Can I remove my artwork later?

**A**: Yes, but:
- If people downloaded the project before removal, they may still have your files
- Your artwork will remain in the Git history
- Coordinate with the project owner
- Update the LICENSE to revoke future use

Better approach: Don't upload anything you might regret!

### Q: What about attribution in the app itself?

**A**: Ask the project owner to include attribution in:
- An "About" or "Credits" screen
- The main README
- The application's help text
- Any promotional materials

Example:
```
Kyaro character artwork by Alrezky Caesaria
https://github.com/cmdai/assets/alrezky/
```

## Getting Help

### If You're Stuck

1. **Read this guide again** - The answer might be here!

2. **Check the examples**
   - Look at `assets/examples/` (if available)
   - See how others organized their files

3. **Ask the project owner**
   - Create a GitHub Issue
   - Tag the maintainer
   - Be specific about what you need help with

4. **Ask in discussions**
   - Use GitHub Discussions
   - Other contributors can help

### Resources

- **GitHub Guides**: [guides.github.com](https://guides.github.com/)
- **Git Basics**: [git-scm.com/book/en/v2/Getting-Started-Git-Basics](https://git-scm.com/book/en/v2/Getting-Started-Git-Basics)
- **Aseprite Docs**: [aseprite.org/docs](https://www.aseprite.org/docs/)
- **Creative Commons**: [creativecommons.org](https://creativecommons.org/)

### Contact

For questions about contributing artwork:

- **Project Owner**: [Contact information]
- **GitHub Issues**: Create an issue with "Asset Question" label
- **Email**: [If provided]

---

## Thank You!

Thank you for contributing your artwork to cmdai! Your creativity makes this project special and unique. We appreciate the time and effort you put into creating beautiful terminal animations.

**Your artwork matters.** By protecting it with a proper license while still sharing it with the project, you're helping build something great while maintaining control over your creative work.

Happy creating! 🎨✨

---

*This guide was last updated: 2025-11-18*
