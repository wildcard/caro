# Artist README Template

> **Instructions**: Copy this template to `assets/your-name/README.md` and fill in all the bracketed `[placeholders]` with your information. Delete this instruction block when done.

---

# [Your Name] - Artwork for cmdai

**Artist**: [Your Full Name or Artist/Brand Name]
**GitHub**: [@your-github-username](https://github.com/your-github-username)
**Portfolio**: [your-website.com] (optional)
**Contact**: [your.email@example.com] (optional)

## About Me

[Write 2-4 sentences about yourself and your art style]

**Example**:
> I'm a pixel artist specializing in retro-style character animations and vibrant
> color palettes. I draw inspiration from classic 8-bit and 16-bit games, with a
> focus on expressive characters and smooth animations. I love creating art that
> brings personality to terminal-based applications.

**Specialties**:
- [e.g., Character design]
- [e.g., Pixel art]
- [e.g., Animation]
- [e.g., Color theory]

## Assets Included

This directory contains my artwork contributions to the cmdai terminal animation project.

### Characters

#### [Character Name]
- **Location**: [`[character-name]/`](./[character-name]/)
- **Description**: [What is this character? What's their personality?]
- **Formats**: Aseprite (.ase), ANSI (.ans), DurDraw (.dur)
- **Animations**: [List animation states: Idle, Walk, etc.]
- **Dimensions**: [e.g., 16x16 pixels]
- **Color Palette**: [Palette name or description]
- **Created**: [Month Year]
- **Status**: ✅ Complete / 🚧 Work in Progress

<!-- Add more characters if you have multiple -->

#### [Another Character] (optional)
- ...

### UI Elements

#### [UI Element Set Name]
- **Location**: [`[ui-elements]/`](./[ui-elements]/)
- **Description**: [What are these UI elements?]
- **Includes**: [List what's included: buttons, spinners, icons, etc.]
- **Formats**: [List formats]
- **Style**: [Describe the visual style]
- **Created**: [Month Year]

### Other Assets

[List any other categories of assets you've contributed]

## File Organization

```
[your-name]/
├── README.md (this file)
├── LICENSE.md (license for my assets)
├── [character-name]/
│   ├── source/
│   │   ├── [character]-idle.ase
│   │   ├── [character]-walk.ase
│   │   └── [character]-master.ase
│   ├── export/
│   │   ├── [character]-idle.ans
│   │   ├── [character]-walk.dur
│   │   └── frames/
│   │       ├── frame-01.png
│   │       └── ...
│   ├── sketches/ (optional)
│   │   ├── concept-01.png
│   │   └── color-studies.png
│   └── README.md
├── [ui-elements]/
│   └── ...
└── [other-assets]/
    └── ...
```

## License

All artwork in this directory is licensed under the terms specified in [LICENSE.md](./LICENSE.md).

**TL;DR** (replace with your actual license summary):
> This artwork is protected by copyright. You can view and use it as part of the
> cmdai project, but you cannot use it in other projects or redistribute it
> separately without my permission.

**Full license**: See [LICENSE.md](./LICENSE.md)

## Using My Assets

### Quick Start

To use my assets in cmdai:

```rust
use cmdai::rendering::{AsepriteParser, Animator, Animation, AnimationMode};

// Load the character
let ase_file = AsepriteParser::load_file(
    "assets/[your-name]/[character-name]/source/[character]-idle.ase"
)?;

// Convert to sprite
let sprite = AsepriteParser::to_sprite(&ase_file)?;

// Create and play animation
let mut animation = Animation::new(sprite, AnimationMode::Loop);
let animator = Animator::new();
animator.play(&mut animation).await?;
```

### Available Formats

For each character/asset, I provide:

- **Aseprite (.ase)**: Original source files with layers
  - Best for: Editing and customization
  - Location: `source/` folder

- **ANSI Art (.ans)**: Traditional ANSI terminal art
  - Best for: Direct terminal display
  - Location: `export/` folder

- **DurDraw (.dur)**: Modern JSON-based ANSI format
  - Best for: Programmatic manipulation
  - Location: `export/` folder

- **PNG Frames**: Individual animation frames
  - Best for: Documentation and previews
  - Location: `export/frames/` folder

## Credits and Attribution

### How to Credit My Work

When using my artwork, please include:

**In your application's "About" or "Credits" section**:
```
[Character Name] artwork by [Your Name]
© [Year] [Your Name]
[Link to your portfolio/GitHub] (if you want)
```

**In your project's README**:
```markdown
## Artwork Credits

- **[Character Name]**: Created by [Your Name] ([@your-github](https://github.com/your-github))
  - Licensed under [License Type]
  - © [Year] [Your Name]
```

### Attribution Example

Copy and paste this for quick attribution:

```
[Character Name] artwork © [Year] [Your Name]
Created for cmdai terminal animation system
Licensed under [License Name] - See assets/[your-name]/LICENSE.md
```

## Design Process

[Optional: Share your creative process, inspiration, and design decisions]

**Example**:
> ### Inspiration
>
> [Character Name] was inspired by [describe your inspiration]. I wanted to create
> a character that felt [adjectives describing the character's vibe].
>
> ### Color Palette
>
> The color palette was carefully chosen to:
> - Work well in terminal environments
> - Be accessible (good contrast)
> - Evoke a [feeling/mood]
>
> Colors used:
> - `#HEXCODE` - [Description]
> - `#HEXCODE` - [Description]
>
> ### Animation Principles
>
> The animations follow these principles:
> - [Principle 1]
> - [Principle 2]
>
> ### Technical Decisions
>
> - **Resolution**: [16x16] pixels - Small enough for terminal use, large enough for detail
> - **Frame Rate**: [8 FPS] - Smooth motion without being too fast
> - **Color Count**: [8 colors] - Limited palette for retro aesthetic

## Contact

Want to use my artwork outside of cmdai? Have questions? Want to commission custom work?

**Reach out to me**:
- **Email**: [your.email@example.com] (optional)
- **GitHub**: [@your-github-username](https://github.com/your-github-username)
- **Portfolio**: [your-website.com] (optional)
- **Twitter/X**: [@your-handle] (optional)
- **Instagram**: [@your-handle] (optional)
- **Other**: [Any other contact method]

**Response time**: I usually respond within [X business days]

## FAQ

### Can I use your artwork in my own project?

[Answer based on your license]

**Example for restrictive license**:
> Not without permission. My artwork for cmdai is licensed exclusively for use in
> this project. If you'd like to use it elsewhere, please contact me to discuss
> licensing options.

**Example for permissive license**:
> Yes! My artwork is licensed under [CC BY 4.0 / other], which means you can use
> it in your projects as long as you provide attribution. See LICENSE.md for details.

### Can I modify your artwork?

[Answer based on your license]

**Example**:
> Modifications require my permission. If you have specific needs for your
> contribution to cmdai, let's discuss! Contact me using the information above.

### Can I commission custom artwork?

[Your answer]

**Example**:
> Yes! I'm open to commissions for terminal art and pixel animations. Reach out
> via email with your project details and budget.

### How long did this take to create?

[Optional: Share time investment]

**Example**:
> The [Character Name] character took approximately [X hours/days] to complete,
> including concept sketches, color studies, pixel art, and animations across
> multiple states.

## Support My Work

[Optional: If you want to include ways people can support you]

**Example**:
> If you enjoy my artwork and want to support future contributions:
>
> - ⭐ Star the cmdai repository
> - 💬 Share the project with others
> - ☕ [Buy me a coffee](https://buymeacoffee.com/yourname) (if you have this set up)
> - 🎨 Commission custom work for your projects
> - 💼 [Hire me for your project](mailto:your.email@example.com)

## Acknowledgments

[Optional: Thank people who helped or inspired you]

**Example**:
> Special thanks to:
> - [Project Owner Name] for creating this awesome project and inviting me to contribute
> - [Person Name] for feedback on the character design
> - The pixel art community at [Community Name] for inspiration and support

## Version History

Keep track of major updates to your assets:

- **v1.2** (2025-11-18): Added walk cycle animation
- **v1.1** (2025-10-15): Updated color palette, improved idle animation
- **v1.0** (2025-09-01): Initial character release

---

## Thank You!

Thank you for using my artwork in your projects! I hope [Character Name] brings
joy and personality to your terminal applications. 💚

If you create something cool with my assets, I'd love to see it! Tag me or send
me a link.

Happy coding! 🎨✨

---

*Last updated: [Date]*
*Assets version: [Version Number]*
