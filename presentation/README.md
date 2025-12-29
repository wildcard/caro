# Caro Presentations

Slidev presentations for the Caro project (formerly cmdai):

- **Main Presentation** (`slides.md`) - Technical overview and capabilities
- **Roadmap Presentation** (`roadmap-slides.md`) - 2026 development roadmap

**Featuring Caro** 🐕 - The friendly Shiba Inu mascot representing Caro's mission to make shell commands safe and accessible!

## Setup

```bash
npm install
```

## Run Presentations

### Main Presentation

```bash
# Development mode with hot reload
npm run dev

# Build for production
npm run build

# Export as PDF
npm run export
```

### Roadmap Presentation

```bash
# Development mode with hot reload
npm run roadmap

# Build for production
npm run roadmap:build

# Export as PDF
npm run roadmap:export
```

### Build Both (for Deployment)

```bash
# Build both presentations to dist/
npm run build:all
```

## Deployment

See [DEPLOYMENT.md](./DEPLOYMENT.md) for complete Vercel deployment instructions.

**Quick Deploy**:
```bash
vercel --prod
```

Deployment structure:
- Main presentation: `/` (root)
- Roadmap presentation: `/roadmap/`

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
- ✅ `mascot.gif` - **Caro the Shiba Inu** (already included!)
- Any additional images or media

See `MASCOT.md` for more about Caro, the cmdai mascot.

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
