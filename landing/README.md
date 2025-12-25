# CMDAI Enterprise Landing Page

A modern, security-focused landing page for CMDAI Enterprise built with Astro and Tailwind CSS.

## Features

- Modern, responsive design optimized for enterprise SaaS
- Security-first messaging and visual design
- Comprehensive feature showcase
- Interactive terminal demo
- Enterprise pricing tiers
- Cross-platform compatibility information
- ROI and productivity metrics
- Complete audit and compliance section

## Tech Stack

- **Framework**: Astro 4.0
- **Styling**: Tailwind CSS 3.4
- **Fonts**: Inter (UI) + JetBrains Mono (code)
- **Build**: Static site generation

## Getting Started

### Prerequisites

- Node.js 18+ and npm

### Installation

```bash
cd landing
npm install
```

### Development

Start the development server:

```bash
npm run dev
```

The site will be available at `http://localhost:4321`

### Build for Production

```bash
npm run build
```

The static site will be generated in the `dist/` directory.

### Preview Production Build

```bash
npm run preview
```

## Project Structure

```
landing/
├── src/
│   ├── layouts/
│   │   └── Layout.astro          # Base layout with head tags
│   └── pages/
│       └── index.astro            # Main landing page
├── public/                         # Static assets
├── astro.config.mjs               # Astro configuration
├── tailwind.config.cjs            # Tailwind configuration
└── package.json                   # Dependencies
```

## Customization

### Colors

The color scheme is defined in `tailwind.config.cjs`:
- **Primary**: Blue tones for main CTAs and highlights
- **Security**: Purple/magenta tones for security-related features

### Content

Edit `src/pages/index.astro` to modify:
- Hero messaging
- Feature descriptions
- Pricing tiers
- Contact information
- Footer links

### Fonts

The page uses:
- **Inter**: Primary UI font (clean, modern)
- **JetBrains Mono**: Code/terminal font (monospace)

Fonts are loaded from Google Fonts in the Layout component.

## Deployment

This is a static site that can be deployed to:
- Vercel (recommended for Astro)
- Netlify
- GitHub Pages
- Any static hosting provider

### Deploy to Vercel

```bash
npm run build
vercel deploy
```

## Design Inspiration

This landing page is inspired by the Fortify Astro template, a security SaaS template with:
- Clean, modern aesthetics
- Security-focused visual language
- Enterprise-grade presentation
- Comprehensive feature showcases

## License

Proprietary - CMDAI Enterprise
