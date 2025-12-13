# cmdai Website

Simple, clean website for the cmdai project - "Claude for your shell"

## Overview

This website showcases cmdai, a Rust CLI tool that converts natural language into safe POSIX shell commands using local LLMs.

## Features

- **Hero Section**: Clean introduction with call-to-action buttons
- **Terminal Demo**: Interactive terminal preview showing cmdai in action
- **Video Section**: Placeholder for demo video (easily replaceable)
- **Features Grid**: Highlights key capabilities (safety, speed, offline support)
- **Download Section**: Installation instructions and quick start guide
- **Responsive Design**: Mobile-friendly and works across all devices

## Local Development

To view the website locally, simply open the `index.html` file in your browser:

```bash
# macOS
open index.html

# Linux
xdg-open index.html

# Or use a simple HTTP server
python3 -m http.server 8000
# Then visit http://localhost:8000
```

## Customization

### Adding a Video

Replace the video placeholder in the video section:

```html
<!-- Replace this section in index.html -->
<div class="video-container">
    <iframe
        width="100%"
        height="100%"
        src="https://www.youtube.com/embed/YOUR_VIDEO_ID"
        frameborder="0"
        allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
        allowfullscreen>
    </iframe>
</div>
```

### Updating Colors

The main brand colors are defined at the top of the CSS:

- Primary gradient: `#667eea` to `#764ba2` (purple)
- Text: `#2c3e50` (dark blue-gray)
- Background: `#ffffff` (white)
- Accent: `#7f8c8d` (gray)

### Modifying Content

All content is in `index.html` with clear section markers:
- Hero section: Main tagline and CTA
- Terminal demo: Example command/output
- Features: 6 feature cards
- Download: Installation instructions

## Deployment

### GitHub Pages

1. Push the website directory to your repository
2. Go to Settings > Pages
3. Select the branch and `/website` folder
4. Your site will be available at `https://username.github.io/cmdai`

### Netlify/Vercel

1. Drag and drop the `website` folder to Netlify/Vercel
2. Or connect your GitHub repo and set the publish directory to `website`

### Custom Domain

Update the deployment settings in your hosting provider to point to your custom domain.

## File Structure

```
website/
├── index.html       # Main website file (HTML + CSS + JS)
└── README.md        # This file
```

## Design Philosophy

The website follows a minimal, clean aesthetic similar to modern SaaS landing pages:
- Single-page design for simplicity
- Gradient accents for visual interest
- Clear hierarchy and generous whitespace
- Mobile-first responsive design
- Fast loading (no external dependencies except optional video)

## Browser Support

- Chrome/Edge: ✅ Full support
- Firefox: ✅ Full support
- Safari: ✅ Full support
- Mobile browsers: ✅ Responsive design

## License

Same as the cmdai project - MIT License
