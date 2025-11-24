# Quick Start Guide

## Installation

```bash
cd presentation
npm install
```

## Run Presentation

```bash
npm run dev
```

The presentation will start on **http://localhost:3030**

Open your browser to:
- **Slides**: http://localhost:3030/
- **Presenter mode**: http://localhost:3030/presenter/
- **Overview**: http://localhost:3030/overview/

## Build for Production

```bash
npm run build
```

Output will be in `dist/` directory.

## Export Options

```bash
# Export slides
npm run export

# Export as PDF (requires playwright)
npx slidev export slides.md --format pdf

# Export as PNG images
npx slidev export slides.md --format png
```

## Using Makefile

```bash
make setup      # Install dependencies
make dev        # Run dev server
make build      # Build for production
```

## Keyboard Shortcuts

During presentation:
- **Space / Arrow keys**: Navigate slides
- **O**: Overview mode
- **D**: Dark mode toggle
- **F**: Fullscreen
- **G**: Show slide grid
- **Escape**: Exit modes

## Adding Your Mascot

1. Place your mascot GIF in `public/mascot.gif`
2. Edit `slides.md` on slide 2 to reference it:

```markdown
---
layout: image-right
image: /mascot.gif
backgroundSize: contain
---
```

## Troubleshooting

### Port Already in Use

If you see "Port 3030 is already in use":

```bash
# Kill the process
lsof -ti:3030 | xargs kill -9

# Or use a different port
npx slidev slides.md --port 3333
```

### Build Errors

If you see module resolution errors:

```bash
# Clear cache and reinstall
rm -rf node_modules package-lock.json
npm install
```

### Mascot Image Not Found

The presentation uses a placeholder emoji (🤖) by default. To use your mascot GIF:

1. Add `public/mascot.gif`
2. Update slides.md references (currently commented out)

## Customization

Before presenting, update in `slides.md`:
- GitHub URLs (replace `yourusername/cmdai`)
- Contact information
- Project statistics
- Demo results

See `TALKING_POINTS.md` for detailed speaker notes.

## Questions?

Refer to:
- `README.md` - Detailed documentation
- `TALKING_POINTS.md` - Speaker script
- `DELIVERABLES_SUMMARY.md` - Complete overview
