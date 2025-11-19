# cmdai DevRel Website - Design Guidelines

> **For:** Alrezky (Brand Designer & Art Director) and Sulo (Frontend Dev & UI/UX Designer)
>
> **Date:** November 2025
>
> **Status:** Initial Implementation Complete - Ready for Enhancement

---

## 📋 Table of Contents

1. [Project Overview](#project-overview)
2. [Design Philosophy](#design-philosophy)
3. [Color Palette](#color-palette)
4. [Typography](#typography)
5. [Components](#components)
6. [Mascot - Caro](#mascot---caro)
7. [Animation & Interactions](#animation--interactions)
8. [Layout & Spacing](#layout--spacing)
9. [Development Workflow](#development-workflow)
10. [Next Steps](#next-steps)

---

## 🎯 Project Overview

### What We've Built

A DevRel (Developer Relations) landing page for **cmdai** - an open-source Rust CLI tool that converts natural language to safe shell commands using local LLMs.

### Technology Stack

- **Framework:** Next.js 16 (App Router)
- **Styling:** Tailwind CSS 4
- **Language:** TypeScript
- **Font:** Press Start 2P (pixel font from Google Fonts)
- **Design System:** Custom 8-bit/pixel art theme

### Directory Structure

```
apps/devrel/
├── app/
│   ├── globals.css          # Design system & theme
│   ├── layout.tsx           # Root layout & metadata
│   └── page.tsx             # Landing page
├── components/
│   ├── Hero.tsx             # Hero section with mascot placeholder
│   ├── Features.tsx         # Feature showcase grid
│   ├── Documentation.tsx    # Docs preview & examples
│   ├── Contributors.tsx     # Community & contribution section
│   ├── Navigation.tsx       # Top navigation bar
│   ├── Footer.tsx           # Footer with links
│   ├── PixelButton.tsx      # Reusable pixel-style button
│   ├── TerminalWindow.tsx   # Terminal-style display component
│   ├── PixelCard.tsx        # Card component with pixel borders
│   └── index.ts             # Component exports
└── DESIGN_GUIDELINES.md     # This file
```

---

## 🎨 Design Philosophy

### Core Principles

1. **Retro Meets Modern**: Blend 8-bit nostalgia with contemporary web design
2. **Cybersecurity Aesthetic**: Inspired by modern security tools (Trivy, Wiz, Torq)
3. **Game Boy Heritage**: Celebrate pixel art and classic handheld gaming
4. **Terminal UI Excellence**: Honor the best terminal user interfaces
5. **Safety & Trust**: Communicate reliability and security visually

### Design Inspirations

- **Color/Style:** Game Boy color palette, retro pixel art
- **Layout/UX:** Modern cybersecurity OSS projects (trivy.dev, torq.io, wiz.com)
- **Typography:** 8-bit pixel fonts, monospace terminals
- **Animation:** Sprite-based animations, scanlines, CRT effects
- **References:**
  - https://dribbble.com/shots/15482912-Is-Retro-Back
  - https://easy-peasy.ai/ai-image-generator/images/8-bit-pixel-art-e9400d9f-5aa7-4e34-9340-54626148c179
  - https://uxdesign.cc/an-8-bit-introduction-to-ux-design-6f96480432e4

---

## 🌈 Color Palette

### Primary Background Colors

```css
--pixel-bg-primary: #0f0f23      /* Deep dark blue-black */
--pixel-bg-secondary: #1a1a2e    /* Dark navy */
--pixel-bg-tertiary: #16213e     /* Mid dark blue */
```

### Game Boy Greens

```css
--gameboy-dark: #0f380f          /* Deep green */
--gameboy-medium: #306230        /* Mid green */
--gameboy-light: #8bac0f         /* Classic GB green */
--gameboy-lightest: #9bbc0f      /* Bright GB green */
```

### Neon Accent Colors

```css
--neon-green: #39ff14            /* Electric green */
--neon-blue: #00f0ff             /* Cyan */
--neon-pink: #ff10f0             /* Magenta */
--neon-purple: #bf00ff           /* Purple */
--neon-yellow: #ffff00           /* Yellow */
```

### Terminal Colors

```css
--terminal-green: #00ff41        /* Matrix green */
--terminal-amber: #ffb000        /* Warning amber */
--terminal-red: #ff3b3b          /* Error red */
```

### Usage Guidelines

- **Primary text:** `--neon-green` or `--terminal-green`
- **Headings:** `--neon-green`, `--neon-blue`, `--neon-pink` (varies by section)
- **Borders:** Neon colors on dark backgrounds
- **Backgrounds:** Use the pixel-bg palette for depth
- **Call-to-actions:** High-contrast neon colors
- **Status indicators:** Terminal colors (green=success, amber=warning, red=error)

---

## ✍️ Typography

### Fonts

1. **Pixel Font (Headings & Display)**
   - Font: `Press Start 2P` from Google Fonts
   - Usage: Logo, headings, buttons, labels
   - Sizes: 8px - 40px (use pixel-perfect sizes: 8, 10, 12, 14, 16, 20, 24, 28, 32, 40)
   - CSS class: `.pixel-text`

2. **Monospace (Body & Code)**
   - Font: `Geist Mono` (included with Next.js)
   - Usage: Body text, code blocks, terminal output
   - CSS variable: `var(--font-geist-mono)`

### Text Rendering

```css
/* Applied to pixel fonts */
image-rendering: pixelated;
-webkit-font-smoothing: none;
text-rendering: geometricPrecision;
```

### Scale

- **Display (Hero):** 24px - 40px
- **H1:** 20px - 28px
- **H2:** 14px - 16px
- **H3:** 10px - 12px
- **Body:** 12px - 16px
- **Small/Caption:** 8px - 10px

---

## 🧩 Components

### PixelButton

**Purpose:** Primary CTA and navigation buttons

**Variants:**
- `primary`: Neon green border/text
- `secondary`: Neon blue border/text
- `danger`: Terminal red border/text

**Sizes:**
- `sm`: 8px text, compact padding
- `md`: 10px text, standard padding
- `lg`: 12px text, generous padding

**Interaction:**
- Hover: Translates up-left with offset shadow
- Active: Reduces shadow for "pressed" effect

### TerminalWindow

**Purpose:** Display code examples, command demos

**Features:**
- Window controls (red/amber/green dots)
- Terminal title bar
- Typing animation (optional)
- Blinking cursor
- Syntax highlighting via color

**Usage:**
```tsx
<TerminalWindow
  title="cmdai@demo"
  command='cmdai "your command"'
  output="Generated output..."
  animate={true}
/>
```

### PixelCard

**Purpose:** Content containers, feature cards

**Variants:**
- `default`: Neon blue border
- `neon`: Neon pink border
- `gameboy`: Game Boy green theme

**Interaction:**
- Hover: Lifts up-left with colored shadow

### Navigation

**Features:**
- Fixed position, transitions on scroll
- Smooth scrolling to anchors
- Responsive mobile menu (button present, needs implementation)

### Footer

**Sections:**
- Brand & social links
- Product links
- Community links
- Resources links
- Copyright & tech stack

---

## 🤖 Mascot - Caro

### Current State

A **placeholder** exists in the Hero section (`Hero.tsx` around line 66):

```tsx
<div className="w-48 h-48 bg-pixel-bg-secondary border-4 border-neon-purple
     flex items-center justify-center relative sprite-animate">
  <div className="text-center">
    <div className="pixel-text text-[10px] text-neon-purple mb-2">
      CARO
    </div>
    <div className="text-6xl">🤖</div>
    <div className="text-[8px] text-gray-500 mt-2 font-mono">
      [Mascot illustration by Alrezky]
    </div>
  </div>
</div>
```

### For Alrezky: Mascot Design Brief

**Character:** Caro (adapted from Kyaro, similar to GitHub's Octocat)

**Personality:**
- Friendly, approachable, helpful
- Tech-savvy but not intimidating
- Security-conscious
- Playful retro vibe

**Style Requirements:**
- **8-bit pixel art** aesthetic
- Should work at multiple sizes (48x48, 96x96, 192x192, 256x256)
- Limited color palette (use neon accent colors)
- Animated sprite sheets (optional but recommended)
- Transparent background

**Deliverables:**
1. Static mascot (PNG, SVG if vector-friendly)
2. Animated sprite sheet (optional)
3. Multiple poses/expressions (optional):
   - Default/neutral
   - Happy/success
   - Thinking/processing
   - Alert/warning

**Integration:**
Replace the placeholder in `Hero.tsx` with:
```tsx
<Image
  src="/caro-mascot.png"
  alt="Caro - cmdai mascot"
  width={192}
  height={192}
  className="sprite-animate"
/>
```

---

## ✨ Animation & Interactions

### Existing Animations

1. **Neon Glow** (`.neon-glow`)
   - Pulsing text shadow effect
   - Used for logo and important headings

2. **Sprite Bounce** (`.sprite-animate`)
   - 8-pixel vertical bounce
   - 0.6s duration, infinite loop
   - Used for mascot and decorative elements

3. **Scanlines** (`.scanlines::before`)
   - CRT monitor effect
   - Subtle horizontal lines overlay
   - Applied to hero section

4. **CRT Flicker** (`.crt-effect`)
   - Subtle opacity flicker
   - Emulates old monitor
   - Optional overlay

5. **Pixel Loader** (`.pixel-loader`)
   - 8-step spinning loader
   - Square corners (no border-radius)
   - Terminal green color

6. **Button Interactions**
   - Hover: Translate + shadow offset
   - Active: Shadow reduction

### Animation Guidelines

- **Duration:** Keep under 1 second for interactions
- **Easing:** Use `steps()` for pixel-perfect sprite animations
- **Performance:** Use `transform` over position changes
- **Accessibility:** Respect `prefers-reduced-motion`

---

## 📐 Layout & Spacing

### Grid System

Using Tailwind's responsive grid:
- **Mobile:** 1 column
- **Tablet (md):** 2 columns
- **Desktop (lg):** 3-4 columns

### Section Spacing

- **Vertical padding:** `py-20` (80px) for major sections
- **Container max-width:** `max-w-6xl` (1152px)
- **Horizontal padding:** `px-4` (16px) on mobile, maintained in container

### Pixel Grid

Background pattern available via `.pixel-grid` class:
- 8x8 pixel grid
- Subtle white lines at 3% opacity
- Creates authentic retro feel

---

## 🛠️ Development Workflow

### For Sulo: Getting Started

1. **Install Dependencies**
   ```bash
   cd apps/devrel
   npm install
   ```

2. **Run Development Server**
   ```bash
   npm run dev
   ```
   Visit: http://localhost:3000

3. **Build for Production**
   ```bash
   npm run build
   npm run start
   ```

4. **Lint & Format**
   ```bash
   npm run lint
   ```

### File Structure for New Components

```tsx
// components/NewComponent.tsx
import React from 'react';

interface NewComponentProps {
  // Props here
}

export const NewComponent: React.FC<NewComponentProps> = ({
  // Destructure props
}) => {
  return (
    <div className="pixel-border bg-pixel-bg-secondary">
      {/* Component content */}
    </div>
  );
};
```

Don't forget to export in `components/index.ts`:
```ts
export { NewComponent } from './NewComponent';
```

### Creating Figma Prototypes

**Recommended workflow:**
1. Take screenshots of the current implementation
2. Use as base reference in Figma
3. Create component variants for different states
4. Document spacing, colors, and typography
5. Share prototypes for feedback
6. Implement approved designs

**Design Tokens to Document:**
- Color palette (already in CSS variables)
- Typography scale
- Spacing system (4px, 8px, 12px, 16px, 24px, 32px, 48px)
- Component states (default, hover, active, disabled)
- Breakpoints (640px, 768px, 1024px, 1280px)

---

## 🚀 Next Steps

### Immediate Priorities

#### For Alrezky (Art Director):

1. **Mascot Creation**
   - [ ] Design Caro character in 8-bit style
   - [ ] Create multiple sizes (48px, 96px, 192px, 256px)
   - [ ] Optional: Create sprite sheet for animations
   - [ ] Deliver in PNG and/or SVG format

2. **Brand Assets**
   - [ ] Logo variations (full, icon-only, wordmark)
   - [ ] Social media preview images (OG images)
   - [ ] Favicon set (16x16, 32x32, 192x192)

3. **Illustration Assets**
   - [ ] Hero section graphics
   - [ ] Feature icons (if replacing emoji)
   - [ ] Decorative pixel art elements
   - [ ] Terminal window decorations

4. **Style Guide Refinement**
   - [ ] Review and adjust color palette if needed
   - [ ] Suggest additional animation effects
   - [ ] Provide pixel art pattern library

#### For Sulo (Frontend Dev & UI/UX):

1. **Figma Prototype**
   - [ ] Create comprehensive Figma prototype
   - [ ] Document component library
   - [ ] Create responsive mockups
   - [ ] Define interaction states

2. **Mobile Responsiveness**
   - [ ] Implement mobile navigation menu
   - [ ] Optimize hero section for small screens
   - [ ] Test all breakpoints
   - [ ] Ensure touch-friendly interactions

3. **Accessibility**
   - [ ] Add ARIA labels to interactive elements
   - [ ] Implement keyboard navigation
   - [ ] Test screen reader compatibility
   - [ ] Add skip-to-content links
   - [ ] Ensure color contrast meets WCAG AA

4. **Performance Optimization**
   - [ ] Optimize images and assets
   - [ ] Implement lazy loading
   - [ ] Add loading states
   - [ ] Optimize font loading
   - [ ] Analyze and improve Core Web Vitals

5. **Additional Pages**
   - [ ] About page
   - [ ] Blog/News section (optional)
   - [ ] Detailed documentation landing
   - [ ] Community showcase
   - [ ] Download/Installation page

6. **Enhanced Features**
   - [ ] Interactive terminal demo
   - [ ] Command playground
   - [ ] Dark/light mode toggle (or Game Boy mode toggle)
   - [ ] Smooth scroll animations
   - [ ] Parallax effects (subtle)

### Future Enhancements

- [ ] Blog integration
- [ ] Newsletter signup
- [ ] Community contributor showcase
- [ ] Interactive tutorials
- [ ] Video demonstrations
- [ ] Command cheat sheet generator
- [ ] Testimonials section
- [ ] Changelog/release notes page

---

## 💡 Design Tips & Best Practices

### Do's ✅

- **Keep pixel-perfect sizing** (multiples of 4px or 8px)
- **Use limited color palette** (don't introduce new colors without reason)
- **Embrace the retro aesthetic** but maintain usability
- **Test on multiple devices** and screen sizes
- **Optimize for performance** (pixel art can be heavy)
- **Maintain accessibility** (contrast, keyboard nav, screen readers)
- **Document your changes** in this file or component comments

### Don'ts ❌

- **Don't over-animate** - subtle is better
- **Don't compromise readability** for aesthetics
- **Don't use gradients** (unless intentionally pixelated)
- **Don't use rounded corners** (stick to sharp edges)
- **Don't mix modern and retro** inconsistently
- **Don't forget mobile users** (pixel fonts can be hard to read)

---

## 📞 Communication & Collaboration

### Feedback & Questions

- **GitHub Issues:** Use for bugs and feature requests
- **Pull Requests:** For submitting changes
- **Discussions:** For design ideas and questions

### File Organization

- **Design Assets:** Place in `/public` directory
  - `/public/images/` - General images
  - `/public/mascot/` - Caro variations
  - `/public/icons/` - Icon set
  - `/public/patterns/` - Pixel patterns/textures

- **Component Updates:** Always update this guide when adding new components

---

## 🎨 Resources & References

### Pixel Art Tools

- **Aseprite** - Professional pixel art editor
- **Piskel** - Free online pixel art tool
- **Lospec** - Pixel art color palette database
- **Pixlr** - Free online image editor

### Inspiration

- **Game Boy Interface:** Classic Nintendo UI patterns
- **Terminal Emulators:** iTerm2, Hyper, Alacritty
- **Cybersecurity Tools:** Trivy, Wiz, Torq, Snyk
- **Retro Games:** 8-bit and 16-bit game UIs
- **Modern Retro Sites:** Websites combining old and new aesthetics

### Design Systems

- **IBM Carbon** - Terminal UI patterns
- **Microsoft Fluent** - Design tokens approach
- **Material Design** - Accessibility guidelines
- **Tailwind UI** - Component patterns

---

## 📝 Version History

- **v1.0.0** (November 2025) - Initial implementation by Claude
  - Next.js 16 setup
  - Core component library
  - 8-bit pixel art design system
  - Landing page structure
  - This design guidelines document

---

## 🤝 Credits

**Design System:** Claude (AI Assistant)
**Brand Direction:** Alrezky (8-Bit Illustrator & Animator)
**UI/UX Implementation:** Sulo (Frontend Dev & UI/UX Designer)
**Project:** cmdai - Open Source Community

---

**Questions or suggestions?** Update this document as the project evolves!

**Happy designing! 🎮✨**
