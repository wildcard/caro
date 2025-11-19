# cmdai DevRel Website

> 🎮 **8-bit Pixel Art Developer Relations Landing Page**

A retro-themed, pixel art developer relations website for **cmdai** - the open-source Rust CLI tool that converts natural language to safe shell commands using local LLMs.

---

## 🎨 Design Direction

This website celebrates:
- **8-bit pixel art** and retro gaming aesthetics
- **Game Boy** inspired color palette and UI patterns
- **Modern cybersecurity** design trends (inspired by Trivy, Wiz, Torq)
- **Terminal UI excellence** with authentic command-line aesthetics
- **Safety and trust** through visual design language

## 🚀 Tech Stack

- **Framework:** [Next.js 16](https://nextjs.org/) (App Router, Turbopack)
- **Styling:** [Tailwind CSS 4](https://tailwindcss.com/)
- **Language:** TypeScript
- **Fonts:**
  - [Press Start 2P](https://fonts.google.com/specimen/Press+Start+2P) (pixel font)
  - System monospace fonts
- **Deployment Ready:** Static export compatible

## 📁 Project Structure

```
apps/devrel/
├── app/
│   ├── globals.css          # 8-bit design system & theme
│   ├── layout.tsx           # Root layout with metadata
│   ├── page.tsx             # Landing page
│   └── favicon.ico          # Site favicon
├── components/
│   ├── Hero.tsx             # Hero section with mascot
│   ├── Features.tsx         # Feature showcase grid
│   ├── Documentation.tsx    # Docs preview & examples
│   ├── Contributors.tsx     # Community section
│   ├── Navigation.tsx       # Top navigation
│   ├── Footer.tsx           # Site footer
│   ├── PixelButton.tsx      # Pixel-style button
│   ├── TerminalWindow.tsx   # Terminal display
│   ├── PixelCard.tsx        # Card with pixel borders
│   └── index.ts             # Component exports
├── public/                  # Static assets
├── DESIGN_GUIDELINES.md     # Design system documentation
└── README.md                # This file
```

## 🛠️ Development

### Prerequisites

- **Node.js** 18+ (v22.21.1 recommended)
- **npm** 10+

### Getting Started

```bash
# Navigate to the devrel directory
cd apps/devrel

# Install dependencies
npm install

# Start development server
npm run dev

# Open in browser
# Visit: http://localhost:3000
```

### Available Scripts

```bash
# Development server with hot reload
npm run dev

# Production build
npm run build

# Start production server
npm run start

# Lint code
npm run lint
```

## 🎨 Design System

### Color Palette

The website uses a carefully crafted retro color palette:

**Backgrounds:**
- `#0f0f23` - Deep dark blue-black (primary)
- `#1a1a2e` - Dark navy (secondary)
- `#16213e` - Mid dark blue (tertiary)

**Neon Accents:**
- `#39ff14` - Electric green (primary)
- `#00f0ff` - Cyan
- `#ff10f0` - Magenta
- `#bf00ff` - Purple

**Terminal Colors:**
- `#00ff41` - Matrix green
- `#ffb000` - Warning amber
- `#ff3b3b` - Error red

**Game Boy Greens:**
- `#0f380f` to `#9bbc0f` - Classic Game Boy palette

### Typography

- **Headings:** Press Start 2P (8-40px)
- **Body:** System monospace fonts
- **Code:** Monospace with pixelated rendering

### Components

All components follow the pixel art aesthetic with:
- Sharp corners (no border-radius)
- 4px/8px pixel borders
- Retro animations (sprite bounce, scanlines, neon glow)
- Terminal-style interactions

## 📋 For Design Team

### Alrezky (Brand Designer & Art Director)

**🎨 Mascot - Caro**

The mascot placeholder in `components/Hero.tsx` (line ~66) is ready for your pixel art:

**Requirements:**
- 8-bit pixel art style
- Multiple sizes: 48×48, 96×96, 192×192, 256×256
- Transparent background
- Use neon accent colors from the palette
- Deliverables: PNG (and SVG if suitable)

**Placement:**
```
/public/mascot/
├── caro-192.png
├── caro-256.png
└── caro-sprite.png (optional animation)
```

**Brand Assets Needed:**
- [ ] Caro mascot (multiple poses/expressions)
- [ ] Logo variations (full, icon, wordmark)
- [ ] Social media preview images (1200×630)
- [ ] Favicon set (16×16, 32×32, 192×192)
- [ ] Feature icons (if replacing emoji)
- [ ] Decorative pixel art elements

### Sulo (Frontend Dev & UI/UX Designer)

**🎯 Next Steps:**

1. **Figma Prototype**
   - Create comprehensive design system
   - Document component states and variants
   - Design responsive breakpoints
   - Define interaction patterns

2. **Mobile Optimization**
   - Implement hamburger menu functionality
   - Optimize hero for small screens
   - Test touch interactions
   - Ensure readable font sizes

3. **Accessibility**
   - Add ARIA labels
   - Implement keyboard navigation
   - Test with screen readers
   - Verify WCAG AA contrast ratios

4. **Performance**
   - Optimize images (WebP format)
   - Implement lazy loading
   - Add loading states
   - Analyze Core Web Vitals

5. **Enhanced Features**
   - Interactive terminal demo
   - Command playground
   - Game Boy mode toggle
   - Smooth scroll animations
   - Parallax effects (subtle)

6. **Additional Pages**
   - About page
   - Blog/News section
   - Detailed documentation portal
   - Community showcase
   - Download/Installation guide

**See:** [DESIGN_GUIDELINES.md](./DESIGN_GUIDELINES.md) for comprehensive design documentation.

## 🔧 Customization

### Adding New Components

```tsx
// components/NewComponent.tsx
import React from 'react';

export const NewComponent: React.FC = () => {
  return (
    <div className="pixel-border bg-pixel-bg-secondary p-6">
      <h3 className="pixel-text text-[12px] text-neon-green mb-4">
        Component Title
      </h3>
      {/* Component content */}
    </div>
  );
};
```

Don't forget to export in `components/index.ts`:
```ts
export { NewComponent } from './NewComponent';
```

### Custom CSS Utilities

Available custom classes in `globals.css`:
- `.pixel-text` - Pixel font styling
- `.pixel-border` - 4px solid border
- `.pixel-button` - Retro button style
- `.terminal-window` - Terminal container
- `.pixel-card` - Card with hover effect
- `.neon-glow` - Pulsing text shadow
- `.scanlines` - CRT scanline effect
- `.pixel-grid` - 8×8 pixel grid background

## 🚀 Deployment

### Static Export

```bash
# Build for static hosting
npm run build

# Output will be in .next/ directory
# Deploy .next/ to your hosting provider
```

### Recommended Platforms

- **Vercel** (optimized for Next.js)
- **Netlify**
- **Cloudflare Pages**
- **GitHub Pages** (with static export)
- **AWS S3 + CloudFront**

### Environment Variables

Create `.env.local` for local development:
```env
# Add any environment variables here
NEXT_PUBLIC_SITE_URL=http://localhost:3000
```

## 📝 Contributing

This website is part of the cmdai open-source project. Contributions are welcome!

1. Review [DESIGN_GUIDELINES.md](./DESIGN_GUIDELINES.md)
2. Follow the established design system
3. Test on multiple screen sizes
4. Ensure accessibility compliance
5. Submit PR with clear description

## 🎮 Design Inspirations

- **Retro Gaming:** Game Boy, NES, SNES UI patterns
- **Cybersecurity:** Trivy, Wiz, Torq, Snyk websites
- **Terminal Emulators:** iTerm2, Hyper, Alacritty
- **Pixel Art:** Dribbble retro designs, 8-bit galleries
- **Modern OSS:** Rust, Deno, Bun landing pages

## 📚 Resources

- [Next.js Documentation](https://nextjs.org/docs)
- [Tailwind CSS Documentation](https://tailwindcss.com/docs)
- [Press Start 2P Font](https://fonts.google.com/specimen/Press+Start+2P)
- [cmdai Repository](https://github.com/wildcard/cmdai)
- [DESIGN_GUIDELINES.md](./DESIGN_GUIDELINES.md)

## 🤝 Team

- **Brand & Art Direction:** Alrezky (8-bit Illustrator & Animator)
- **UI/UX Development:** Sulo (Frontend Dev & UI/UX Designer)
- **Initial Implementation:** Claude AI Assistant
- **Project:** cmdai Open Source Community

## 📄 License

This website is part of the cmdai project, licensed under **AGPL-3.0**.

---

**Questions or feedback?** Open an issue in the main [cmdai repository](https://github.com/wildcard/cmdai).

**Happy coding! 🎮✨**
