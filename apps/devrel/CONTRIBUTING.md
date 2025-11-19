# Contributing to cmdai DevRel Website

> **👋 Welcome Alrezky & Sulo!**
>
> This guide will help you continue developing the cmdai developer relations website. Everything you need to know is here.

---

## 🎯 Quick Start

### For Both Team Members

**1. Clone and Navigate**
```bash
cd cmdai/apps/devrel
```

**2. Install Dependencies**
```bash
npm install
```

**3. Start Development Server**
```bash
npm run dev
```

**4. Open in Browser**
```
http://localhost:3000
```

**5. Make Changes**
- Edit files in `app/` or `components/`
- Browser auto-refreshes with your changes
- See changes instantly!

---

## 🎨 For Alrezky (Brand Designer & Art Director)

### Your Mission

Create the visual identity and pixel art assets that will make cmdai's website unforgettable.

### Priority 1: Caro the Mascot

**What:** The star of our website - a friendly 8-bit character that represents cmdai

**Why:** Every great open-source project has a mascot (think GitHub's Octocat). Caro will be ours!

**Requirements:**
- **Style:** 8-bit pixel art (think Game Boy games)
- **Personality:** Friendly, helpful, tech-savvy, security-conscious
- **Sizes needed:**
  - 48×48px (favicon, small icons)
  - 96×96px (mobile)
  - 192×192px (desktop)
  - 256×256px (hero section)
- **Format:** PNG with transparent background (SVG if you can make it work)
- **Colors:** Use colors from our palette (see below)

**Our Color Palette:**
```
Neon Green:  #39ff14  (primary)
Cyan:        #00f0ff
Magenta:     #ff10f0
Purple:      #bf00ff
Terminal:    #00ff41
```

**Where to Place Files:**
```
apps/devrel/public/mascot/
├── caro-48.png
├── caro-96.png
├── caro-192.png
└── caro-256.png
```

**How to Add to Website:**

Once you've created the mascot, ask Sulo to replace the placeholder in `components/Hero.tsx` around line 66-85.

Or, if you're comfortable with code:
```tsx
// Replace the placeholder div with:
<Image
  src="/mascot/caro-256.png"
  alt="Caro - cmdai mascot"
  width={256}
  height={256}
  className="sprite-animate"
/>
```

### Priority 2: Logo Variations

**What:** Different versions of the cmdai logo

**Needed:**
- **Full logo** (text + icon) - for header
- **Icon only** - for favicon, small spaces
- **Wordmark** - text only for certain contexts

**Format:** PNG + SVG if possible

**Where:** `apps/devrel/public/logo/`

### Priority 3: Social Media Assets

**What:** Images for when people share our website

**Sizes:**
- **OG Image:** 1200×630px (Facebook, LinkedIn)
- **Twitter Card:** 1200×675px
- **Favicon set:** 16×16, 32×32, 192×192

**Content ideas:**
- Caro the mascot
- Terminal window showing a command
- cmdai logo
- Tagline: "Natural Language → Safe Shell Commands"

**Where:** `apps/devrel/public/og/`

### Priority 4: Feature Icons

**What:** Replace emoji in the Features section with pixel art icons

**Current emoji to replace:**
- ⚡ (Blazing Fast)
- 🧠 (Local LLM)
- 🛡️ (Safety First)
- 🎯 (Multiple Backends)
- 📦 (Zero Dependencies)
- 🌐 (Cross-Platform)

**Size:** 64×64px pixel art icons
**Where:** `apps/devrel/public/icons/`

### How to Share Your Work

**Option 1: Through Sulo**
- Send files to Sulo on Slack
- Sulo will integrate them into the website

**Option 2: Direct GitHub (if comfortable)**
```bash
# 1. Create a new branch
git checkout -b feat/mascot-and-brand-assets

# 2. Add your files
# Place them in the correct folders (see above)

# 3. Commit
git add apps/devrel/public/
git commit -m "feat: Add Caro mascot and brand assets"

# 4. Push
git push -u origin feat/mascot-and-brand-assets

# 5. Ask Sulo or team to review
```

### Tools We Recommend

- **Aseprite** - Industry standard for pixel art ($19.99)
- **Piskel** - Free online pixel art tool (https://www.piskelapp.com/)
- **Lospec Palette List** - Color palette inspiration (https://lospec.com/palette-list)
- **Photopea** - Free Photoshop alternative (https://www.photopea.com/)

### Questions?

**Design Direction:**
- Refer to `DESIGN_GUIDELINES.md` for color palette, style guide
- Inspirations: Game Boy games, retro arcade, modern cybersecurity sites (Trivy, Wiz)
- Keep it playful but professional

**Technical Questions:**
- Ask Sulo about implementation
- Ping on Slack: #devrel-website

---

## 💻 For Sulo (Frontend Dev & UI/UX Designer)

### Your Mission

Turn the initial implementation into a polished, production-ready website with excellent UX and accessibility.

### Development Workflow

**1. Branch Strategy**
```bash
# Create feature branch for each task
git checkout main
git pull origin main
git checkout -b feat/your-feature-name

# Make your changes...

# Commit with clear message
git commit -m "feat: Add mobile navigation menu"

# Push and create PR
git push -u origin feat/your-feature-name
```

**2. Available Scripts**
```bash
npm run dev      # Development server (hot reload)
npm run build    # Production build
npm run start    # Run production build locally
npm run lint     # Check for issues
```

**3. File Structure**
```
apps/devrel/
├── app/
│   ├── globals.css       # Edit styles here
│   ├── layout.tsx        # Root layout, SEO metadata
│   ├── page.tsx          # Landing page
│   └── [new-page]/       # Add new pages here
├── components/
│   ├── Hero.tsx          # Each section is a component
│   ├── Features.tsx
│   ├── ...
│   └── index.ts          # Export components here
└── public/               # Static assets (images, fonts, etc.)
```

### Priority 1: Mobile Responsiveness

**Current State:**
- Desktop layout is complete
- Mobile needs refinement

**Tasks:**

**1.1 Hamburger Menu (Navigation.tsx)**
```tsx
// The button exists but doesn't do anything yet
// You need to:
// 1. Add state for menu open/close
// 2. Create mobile menu panel
// 3. Add smooth animations

const [mobileMenuOpen, setMobileMenuOpen] = useState(false);

// Toggle function
const toggleMenu = () => setMobileMenuOpen(!mobileMenuOpen);

// Mobile menu panel (add after nav)
{mobileMenuOpen && (
  <div className="md:hidden bg-pixel-bg-secondary border-t-4 border-neon-green">
    {/* Menu items */}
  </div>
)}
```

**1.2 Hero Section (Hero.tsx)**
- Test on mobile devices
- Adjust font sizes for readability
- Ensure mascot doesn't overflow
- Stack terminal demo below text on small screens

**1.3 Features Grid (Features.tsx)**
- Currently 3 columns on desktop
- Should be 1 column on mobile
- Already has `grid-cols-1 md:grid-cols-2 lg:grid-cols-3`
- Test and adjust spacing

**1.4 Pixel Font Sizes**
- Press Start 2P can be hard to read on mobile
- Consider:
  - Larger sizes for mobile
  - Alternative font for body text on small screens
  - Testing on real devices

**Testing:**
```bash
# Test on different sizes
npm run dev

# Open DevTools (F12)
# Toggle device toolbar
# Test: iPhone SE, iPhone 12, iPad, Desktop
```

### Priority 2: Accessibility

**2.1 ARIA Labels**

Add to all interactive elements:
```tsx
// Buttons
<button
  aria-label="Open navigation menu"
  onClick={toggleMenu}
>

// Links
<a
  href="#features"
  aria-label="Jump to features section"
>

// Navigation
<nav aria-label="Main navigation">
```

**2.2 Keyboard Navigation**

Test:
- Tab through all links and buttons
- Enter/Space to activate
- Escape to close modals/menus
- No keyboard traps

**2.3 Screen Readers**

Test with:
- **macOS:** VoiceOver (Cmd+F5)
- **Windows:** NVDA (free)
- Ensure all content is readable
- Ensure all images have alt text

**2.4 Color Contrast**

Check WCAG AA compliance:
- Tool: https://webaim.org/resources/contrastchecker/
- Our neon green (#39ff14) on dark (#0f0f23) should pass
- Test all color combinations

**2.5 Focus Indicators**

Ensure visible focus:
```css
/* Add to globals.css */
*:focus-visible {
  outline: 2px solid var(--neon-green);
  outline-offset: 2px;
}
```

### Priority 3: Figma Prototype

**Why:** Design system documentation for consistency

**Steps:**

**3.1 Set Up Figma File**
- Create new Figma project: "cmdai DevRel Design System"
- Import screenshots of current website
- Create pages: Components, Colors, Typography, Layouts

**3.2 Document Colors**
```
Create color palette in Figma:
- Each color as a style
- Name them matching CSS variables
- Document usage (primary, accent, etc.)
```

**3.3 Document Components**

For each component (Button, Card, Terminal, etc.):
- Create Figma component
- Add variants (primary/secondary, sizes)
- Document states (default, hover, active, disabled)
- Add specs (padding, margins, borders)

**3.4 Create Responsive Mockups**
- Mobile (375px)
- Tablet (768px)
- Desktop (1440px)

**3.5 Export Design Tokens**
```
Consider using Figma plugins:
- Figma Tokens (for design system)
- Style Dictionary (for export to CSS)
```

### Priority 4: Performance Optimization

**4.1 Image Optimization**

When Alrezky provides images:
```bash
# Install sharp for image processing
npm install sharp

# Convert to WebP
npx @squoosh/cli --webp auto apps/devrel/public/mascot/*.png

# Or use Next.js Image component (already using in some places)
import Image from 'next/image';
```

**4.2 Lazy Loading**
```tsx
// For images below the fold
<Image
  src="/mascot/caro-256.png"
  loading="lazy"
  // ...other props
/>

// For components
import dynamic from 'next/dynamic';

const Contributors = dynamic(() =>
  import('@/components/Contributors').then(mod => mod.Contributors)
);
```

**4.3 Font Loading**

Already using system fonts, but for Press Start 2P:
```tsx
// Consider preloading in layout.tsx
<link
  rel="preload"
  href="https://fonts.googleapis.com/css2?family=Press+Start+2P&display=swap"
  as="style"
/>
```

**4.4 Analytics**

Test performance:
```bash
# Build for production
npm run build

# Run Lighthouse audit in Chrome DevTools
# Target: 90+ for all metrics
```

### Priority 5: Enhanced Features

**5.1 Interactive Terminal Demo**

Make the TerminalWindow truly interactive:
```tsx
// New component: components/InteractiveTerminal.tsx
// Features:
// - User can type
// - Simulated command execution
// - Copy button for commands
// - Multiple example commands

'use client';

export const InteractiveTerminal = () => {
  const [input, setInput] = useState('');
  const [history, setHistory] = useState<string[]>([]);

  // Implementation here
};
```

**5.2 Command Playground**

Create a tryout page:
```
apps/devrel/app/playground/page.tsx

Features:
- Input field for natural language
- "Generate" button
- Shows generated command
- Safety warnings if dangerous
- Copy to clipboard
- Note: This would need backend API
```

**5.3 Dark/Light Mode Toggle**

Actually a "Game Boy Mode" toggle:
```tsx
// Add to Navigation.tsx
const [theme, setTheme] = useState<'dark' | 'gameboy'>('dark');

// Toggle between dark theme and Game Boy green theme
// Already have gameboy colors in CSS variables
```

**5.4 Smooth Scroll Animations**

```bash
npm install framer-motion

# Add to components for entry animations
import { motion } from 'framer-motion';

<motion.div
  initial={{ opacity: 0, y: 20 }}
  whileInView={{ opacity: 1, y: 0 }}
  transition={{ duration: 0.5 }}
>
  {/* Component content */}
</motion.div>
```

### Priority 6: Additional Pages

**6.1 About Page**
```tsx
// apps/devrel/app/about/page.tsx

export default function AboutPage() {
  return (
    <div>
      <h1>About cmdai</h1>
      {/* Story, mission, team */}
    </div>
  );
}
```

**6.2 Blog/News**
```tsx
// apps/devrel/app/blog/page.tsx
// Could use MDX for blog posts
npm install @next/mdx @mdx-js/loader @mdx-js/react
```

**6.3 Community Page**
```tsx
// apps/devrel/app/community/page.tsx
// Showcase contributors, testimonials, projects
```

### How to Integrate Alrezky's Assets

When Alrezky provides assets:

**1. Add to Public Folder**
```bash
# Place in appropriate folders
apps/devrel/public/
├── mascot/
├── logo/
├── icons/
└── og/
```

**2. Update Components**

```tsx
// Replace placeholder in Hero.tsx
import Image from 'next/image';

<Image
  src="/mascot/caro-256.png"
  alt="Caro - cmdai mascot"
  width={256}
  height={256}
  className="sprite-animate"
  priority
/>
```

**3. Update Metadata**

```tsx
// In app/layout.tsx
export const metadata: Metadata = {
  // ... existing metadata
  icons: {
    icon: '/logo/favicon.png',
    apple: '/logo/apple-touch-icon.png',
  },
  openGraph: {
    // ... existing openGraph
    images: ['/og/cmdai-og-image.png'],
  },
};
```

### Testing Checklist

Before marking any task complete:

- [ ] Works on mobile (375px width)
- [ ] Works on tablet (768px width)
- [ ] Works on desktop (1440px width)
- [ ] All interactive elements have focus states
- [ ] Tab navigation works correctly
- [ ] Screen reader announces content correctly
- [ ] Color contrast passes WCAG AA
- [ ] Images have alt text
- [ ] Links have descriptive text (not "click here")
- [ ] Build succeeds (`npm run build`)
- [ ] No console errors
- [ ] Lighthouse score > 90

### Code Style

**Follow Existing Patterns:**
```tsx
// Component structure
export const ComponentName: React.FC<ComponentProps> = ({
  prop1,
  prop2,
}) => {
  return (
    <div className="pixel-card bg-pixel-bg-secondary">
      {/* Content */}
    </div>
  );
};

// Use existing CSS classes from globals.css
// pixel-text, pixel-button, terminal-window, etc.

// Keep retro aesthetic
// - No rounded corners (border-radius: 0)
// - Use pixel borders (4px, 8px)
// - Use neon colors for accents
```

### Getting Help

**Design System:**
- `DESIGN_GUIDELINES.md` - Comprehensive guide
- `app/globals.css` - All available classes

**Component Examples:**
- Look at existing components in `components/`
- Copy patterns for consistency

**Questions:**
- Slack: #devrel-website
- Tag: @alrezky (design) or @team (technical)

---

## 📝 Pull Request Process

### For Both Team Members

**1. Create Descriptive PR**

Use this template:
```markdown
## What

Brief description of changes

## Why

Why this change is needed

## Screenshots

Before/after images (for visual changes)

## Testing

- [ ] Tested on mobile
- [ ] Tested on desktop
- [ ] Tested in Chrome
- [ ] Tested in Safari
- [ ] Tested in Firefox
- [ ] Accessibility checked

## Checklist

- [ ] Follows design guidelines
- [ ] No console errors
- [ ] Build succeeds
- [ ] Responsive on all sizes
```

**2. Request Review**

Tag appropriate team members:
- Design changes: @alrezky
- Code changes: @sulo
- All changes: @team

**3. Address Feedback**

Make requested changes, commit, push to same branch

**4. Merge**

Once approved, merge to main branch

---

## 🎯 Current Priorities (Sprint 1)

### Week 1-2

**Alrezky:**
- [ ] Caro mascot (all sizes)
- [ ] Logo variations
- [ ] Favicon set

**Sulo:**
- [ ] Mobile hamburger menu
- [ ] Responsive testing and fixes
- [ ] Accessibility ARIA labels

### Week 3-4

**Alrezky:**
- [ ] Social media OG images
- [ ] Feature icons (replace emoji)
- [ ] Additional brand assets

**Sulo:**
- [ ] Figma prototype
- [ ] Performance optimization
- [ ] Integrate Alrezky's first batch of assets

### Week 5-6

**Both:**
- [ ] Enhanced features (interactive terminal)
- [ ] Additional pages (About, Community)
- [ ] Final polish and launch prep

---

## 🚀 Launch Checklist

Before going live:

### Design (Alrezky)
- [ ] All mascot sizes created
- [ ] Logo in header
- [ ] Favicon updated
- [ ] OG images for social sharing
- [ ] All emoji replaced with icons (optional)

### Development (Sulo)
- [ ] Mobile fully responsive
- [ ] Accessibility audit passed
- [ ] Performance: Lighthouse score > 90
- [ ] All interactive elements work
- [ ] Cross-browser tested (Chrome, Firefox, Safari)
- [ ] SEO metadata complete
- [ ] Analytics integrated (if desired)

### Content (Team)
- [ ] Copy reviewed and finalized
- [ ] Links tested and working
- [ ] Documentation complete
- [ ] Social media ready

---

## 📞 Communication

### Daily Async Updates

Post in Slack #devrel-website:
```
Daily Update [Your Name] - [Date]

✅ Completed:
- Thing 1
- Thing 2

🏗️ In Progress:
- Thing 3

🚧 Blocked:
- Thing 4 (need help with X)

📅 Tomorrow:
- Thing 5
```

### Weekly Sync (Optional)

30-minute video call to:
- Review progress
- Show demos
- Discuss blockers
- Plan next week

---

## 🎉 Success Metrics

We'll know we're successful when:

✅ **Visual Identity**
- Caro mascot is memorable and fits the brand
- Website has cohesive 8-bit aesthetic
- Brand assets ready for marketing

✅ **User Experience**
- Mobile navigation is intuitive
- Site loads fast (<3s)
- Accessible to all users
- Interactive elements are engaging

✅ **Development Quality**
- Code is maintainable
- Components are reusable
- Design system is documented
- Easy for future contributors

✅ **Community Impact**
- Attracts contributors
- Generates excitement
- Clear call-to-actions work
- Social sharing looks great

---

## 💡 Tips for Success

### For Alrezky
- **Start small:** Caro mascot first, then expand
- **Iterate:** Show drafts early, get feedback
- **Pixel perfect:** Use tools that keep sharp edges
- **Consistency:** Stick to the color palette

### For Sulo
- **Mobile first:** Test on small screens constantly
- **Component thinking:** Keep pieces reusable
- **User testing:** Show to others, watch them use it
- **Document:** Comment complex code, update README

### For Both
- **Over-communicate:** Better to ask than assume
- **Show progress:** Post screenshots, share wins
- **Stay organized:** Use GitHub Projects or Trello
- **Have fun:** This is a creative project! 🎮

---

**Welcome to the team! Let's build something amazing together! 🚀**

Questions? Reach out on Slack: #devrel-website
