# cmdai Web Design & Implementation Guide

Welcome to the complete guide for bringing cmdai's 8-bit terminal aesthetic to the web! 🚀

---

## 📚 Documentation Overview

This directory contains everything frontend developers need to build the cmdai web experience.

### Core Documents

| Document | Purpose | Audience |
|----------|---------|----------|
| **[DESIGN_SYSTEM.md](./DESIGN_SYSTEM.md)** | Complete visual design system | Designers, Frontend Devs |
| **[COMPONENT_ARCHITECTURE.md](./COMPONENT_ARCHITECTURE.md)** | React/Next.js component specs | Frontend Developers |
| **[WEB_SIMULATOR_SPEC.md](./WEB_SIMULATOR_SPEC.md)** | Interactive TUI simulator requirements | Product, Frontend |
| **[MASTER_PROMPTS.md](./MASTER_PROMPTS.md)** | AI-assisted implementation guides | All Developers |

---

## 🎯 Quick Start

### For Designers

1. Read [DESIGN_SYSTEM.md](./DESIGN_SYSTEM.md) to understand the 8-bit aesthetic
2. Review color palette, typography, and component patterns
3. Use provided tokens in your designs
4. Maintain terminal authenticity in all visual work

### For Frontend Developers

1. **Setup**: Use prompts from [MASTER_PROMPTS.md](./MASTER_PROMPTS.md) #1 to initialize project
2. **Build Components**: Follow patterns in [COMPONENT_ARCHITECTURE.md](./COMPONENT_ARCHITECTURE.md)
3. **Reference Design**: Check [DESIGN_SYSTEM.md](./DESIGN_SYSTEM.md) for all visual tokens
4. **Implement Simulator**: Use specs from [WEB_SIMULATOR_SPEC.md](./WEB_SIMULATOR_SPEC.md)

### For Project Managers

1. Review [WEB_SIMULATOR_SPEC.md](./WEB_SIMULATOR_SPEC.md) for feature requirements
2. Use the implementation checklist for sprint planning
3. Track success metrics outlined in the simulator spec
4. Reference [DESIGN_SYSTEM.md](./DESIGN_SYSTEM.md) for brand consistency

---

## 🏗️ Implementation Roadmap

### Phase 1: Foundation (Week 1)
- [ ] Project setup (Next.js 14 + Tailwind + TypeScript)
- [ ] Design tokens implemented in Tailwind config
- [ ] Base components (TerminalWindow, Button, Input)
- [ ] Font loading (JetBrains Mono)

### Phase 2: Core Components (Week 2)
- [ ] StatusBar component
- [ ] CommandOutput component
- [ ] KeyboardShortcut component
- [ ] InputArea with shortcuts
- [ ] Help modal

### Phase 3: Simulator (Week 3)
- [ ] Mock backend with 15+ responses
- [ ] TUISimulator main component
- [ ] History management (localStorage)
- [ ] Examples gallery
- [ ] Loading states & animations

### Phase 4: Polish (Week 4)
- [ ] Responsive design (mobile/tablet)
- [ ] Accessibility audit (WCAG AA)
- [ ] Performance optimization (Lighthouse >95)
- [ ] Error boundaries
- [ ] Analytics integration

### Phase 5: Launch (Week 5)
- [ ] Deploy to production (Vercel)
- [ ] SEO optimization
- [ ] Social sharing (Open Graph)
- [ ] Documentation site integration
- [ ] Collect user feedback

---

## 🎨 Design Philosophy

### The cmdai Aesthetic

cmdai's visual identity is built on three pillars:

1. **Terminal Authenticity**
   - Monospace fonts everywhere
   - ASCII art and box-drawing characters
   - Dark background (#0A0A0A) with vibrant accents
   - Cursor blinking and terminal-style animations

2. **8-Bit Nostalgia**
   - Retro color palette (Cyan, Green, Yellow, Red)
   - Pixel-perfect borders and spacing
   - Simple, geometric shapes
   - No gradients, no shadows (except glows)

3. **Modern Professionalism**
   - Clean, organized layouts
   - Consistent spacing system
   - Accessible (WCAG AA minimum)
   - Responsive and mobile-friendly
   - Fast performance

### What Makes It Special

- **Cyan Glow Effect**: Primary actions have subtle cyan glow on hover
- **Risk Color Coding**: Green (safe) → Yellow (moderate) → Red (danger)
- **Box Drawing**: Uses Unicode characters (─ │ ┌ ┐ └ ┘) for borders
- **Monospace Everything**: All text in JetBrains Mono (except marketing copy)
- **Keyboard-First**: Every action has a keyboard shortcut

---

## 🛠️ Tech Stack

### Recommended

- **Framework**: Next.js 14 (App Router)
- **Language**: TypeScript (strict mode)
- **Styling**: Tailwind CSS 3
- **UI Library**: shadcn/ui (base components)
- **Icons**: lucide-react
- **Animation**: Framer Motion (complex) + Tailwind (simple)
- **Fonts**: JetBrains Mono (Google Fonts)
- **Analytics**: Plausible (privacy-respecting)
- **Deployment**: Vercel

### Why These Choices?

- **Next.js**: SEO, performance, developer experience
- **TypeScript**: Type safety, better tooling, fewer bugs
- **Tailwind**: Matches our design token system perfectly
- **shadcn/ui**: Unstyled components we can theme
- **Framer Motion**: Best animation library for React
- **Vercel**: Zero-config deployment, edge functions

---

## 🎯 Success Metrics

### Performance Targets

- **Lighthouse Score**: > 95 across all metrics
- **First Contentful Paint**: < 1.5s
- **Time to Interactive**: < 3s
- **Total Bundle Size**: < 150KB (gzipped)

### User Engagement

- **Session Duration**: > 2 minutes average
- **Commands Generated**: > 5 per session
- **Return Visitors**: > 30% within 7 days
- **Install Conversion**: > 30% click "Install cmdai"

### Accessibility

- **WCAG Compliance**: AA minimum, AAA preferred
- **Keyboard Navigation**: 100% of features accessible
- **Screen Reader**: Full compatibility
- **Color Contrast**: 4.5:1 minimum (7:1 for AAA)

---

## 🤝 Contributing

### Before You Start

1. Read all four documentation files
2. Review the TUI (terminal version) to understand the feel
3. Test in multiple browsers (Chrome, Firefox, Safari, Edge)
4. Check accessibility with screen readers

### Contribution Guidelines

1. **Follow the Design System**: No arbitrary colors or fonts
2. **TypeScript Required**: Strict mode, no `any` types
3. **Accessibility First**: ARIA labels, keyboard navigation
4. **Test Everything**: Unit tests for logic, integration for UI
5. **Document Your Code**: JSDoc comments for public APIs

### PR Checklist

- [ ] Follows design system (colors, fonts, spacing)
- [ ] TypeScript with no errors
- [ ] Accessible (keyboard, screen reader)
- [ ] Responsive (mobile, tablet, desktop)
- [ ] Performant (Lighthouse checked)
- [ ] Tested (unit + integration)
- [ ] Documented (JSDoc, README updates)

---

## 📖 Reference Materials

### Inspiration

- **Atuin**: Shell history TUI (fuzzy search UX)
- **Claude Code**: Interactive TUI patterns
- **GitHub CLI**: Modern terminal aesthetic
- **Charm.sh**: Beautiful terminal apps

### Technical References

- [Ratatui Docs](https://ratatui.rs/) - Original TUI framework
- [Tailwind Docs](https://tailwindcss.com/docs)
- [Next.js Docs](https://nextjs.org/docs)
- [WCAG Guidelines](https://www.w3.org/WAI/WCAG21/quickref/)

---

## 💬 Getting Help

### Questions?

- **Design**: Review DESIGN_SYSTEM.md or ask in #design
- **Implementation**: Check MASTER_PROMPTS.md or #frontend
- **Features**: See WEB_SIMULATOR_SPEC.md or ask PM
- **Architecture**: Read COMPONENT_ARCHITECTURE.md or #architecture

### Stuck?

1. Check the relevant doc file first
2. Search existing GitHub issues
3. Ask in project Discord/Slack
4. Create a detailed GitHub issue

---

## 🎉 What's Next?

### Immediate Priorities

1. **Get buy-in**: Share design system with stakeholders
2. **Validate design**: Create Figma mockups from specs
3. **Proof of concept**: Build TerminalWindow component
4. **Test with users**: Show mockups to potential users

### Future Enhancements

- **Real-time collaboration**: Multiple users in one terminal
- **Video recording**: Record terminal sessions as GIFs
- **Theme editor**: Let users customize colors
- **Plugin system**: Community extensions
- **Mobile app**: React Native version

---

## 📝 Document Updates

This is a living documentation system. Updates are tracked here:

- **2025-11-19**: Initial release - All four core documents
- Future updates will be listed chronologically

To suggest updates: Create PR with changes + rationale

---

## 🙏 Acknowledgments

This design system was inspired by:

- The cmdai Rust TUI implementation (the source of truth)
- Classic terminal emulators (xterm, iTerm2)
- Modern CLI tools (GitHub CLI, Vercel CLI)
- 1980s computer aesthetics
- The developer community's love of terminals

---

**Ready to build something amazing? Start with [DESIGN_SYSTEM.md](./DESIGN_SYSTEM.md) →**
