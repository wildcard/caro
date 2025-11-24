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

✅ **Caro (the cmdai mascot) is already included!**

Caro is a friendly Shiba Inu who serves as cmdai's mascot. The GIF is already in `public/mascot.gif` and configured in the presentation.

If you want to use a different mascot:

1. Replace `public/mascot.gif` with your image
2. Update slide 2 references if needed
3. Update the final slide if desired

See `MASCOT.md` for more about Caro!

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
- GitHub URLs (replace `wildcard/cmdai`)
- Contact information
- Project statistics
- Demo results

See `TALKING_POINTS.md` for detailed speaker notes.

## Questions?

Refer to:
- `README.md` - Detailed documentation
- `TALKING_POINTS.md` - Speaker script
- `DELIVERABLES_SUMMARY.md` - Complete overview
