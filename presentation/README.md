# cmdai Presentation

Slidev presentation showcasing cmdai's capabilities, roadmap, and call to action for contributors.

## Setup

```bash
npm install
```

## Run Presentation

```bash
# Development mode with hot reload
npm run dev

# Build for production
npm run build

# Export as PDF
npm run export-pdf
```

## Presentation Structure

1. **Introduction** - What is cmdai?
2. **Problem & Solution** - Why cmdai matters
3. **Working Demo** - MLX test suite results
4. **Architecture** - Technical design
5. **Safety Validation** - The critical feature
6. **Performance** - Real benchmarks
7. **Backends** - Flexible inference options
8. **Roadmap** - Vision and phases
9. **Future Ideas** - Self-maintenance, governance, static generation
10. **Community Governance** - Democratic safety decisions
11. **Static Generation** - Pre-compiled intelligence
12. **Open Source** - Principles and contribution areas
13. **Call to Action** - How to get involved
14. **Contact** - Resources and quick wins

## Key Highlights

- ✅ Working MLX demo with Qwen2.5-Coder
- ✅ Real performance benchmarks (0.7s inference)
- ✅ 100% safety detection validation
- 🎯 Clear roadmap and contribution opportunities
- 🚀 Vision for self-maintenance and community governance

## Assets Needed

Place in `public/` directory:
- `mascot.gif` - Project mascot animation
- Any additional images or media

## Export Options

```bash
# PDF (recommended for sharing)
npm run export-pdf

# PNG slides
slidev export slides.md --format png

# HTML (self-contained)
npm run build
# Output in dist/
```

## Customization

Edit `slides.md` to customize:
- Content and messaging
- GitHub URLs
- Contact information
- Project statistics
- Demo results

## Theme

Uses Slidev's `seriph` theme with custom styling for:
- Code blocks
- Mermaid diagrams
- Animations and transitions
- Color scheme (green accents for safety)

## Notes

- Presenter notes included in HTML comments
- Keyboard shortcuts: Space/Arrow keys to navigate
- Press `O` for overview mode
- Press `F` for fullscreen

## License

Same as cmdai project: AGPL-3.0
