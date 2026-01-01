# cmdai Brand & Identity — Project Canvas

> **Team-wide alignment document for cmdai DevRel website**
>
> Vision → Identity → Mascot → Terminal Constraints → Web → Deliverables

---

## 🎯 1. Vision & Purpose

### Core Purpose

**cmdai** is a safe, reliable, retro-inspired terminal companion for:
- System engineers
- DevOps & SREs
- Security-conscious teams
- POSIX/Unix power users
- Newcomers intimidated by the terminal

### It Acts As:
- ✅ Command-generation assistant
- ✅ Runbook buddy
- ✅ Safe execution co-pilot
- ✅ Trustworthy agent (starts in terminal, grows to other interfaces)

### Core Philosophy
- **Rooted deeply in Unix principles**
- **Community-driven, enterprise-ready**
- **Friendly + safe + engaging**
- **Takes terminal limitations and turns them into delight**

---

## 👥 2. Audience

### Primary
- Hardcore Unix/POSIX engineers
- DevOps/SRE professionals
- Security-first teams
- Engineers with sensitive infra or production systems

### Secondary
- Developers learning the terminal
- Users wanting confidence with shell commands
- Open-source contributors and tinkerers

---

## 🎨 3. Brand Identity Direction

### Aesthetic
- **Cute, friendly, retro 8-bit/16-bit**
- **Game Boy + early Pokémon era**
- **Pixel art in a modern, intentional way**
- **Feels like a Tamagotchi or Pokémon companion**
- **"Geeky minimalist nostalgia"**

### Brand Personality (Tone)
- Friendly
- Loyal
- Safe
- Retro
- Geeky
- Playful
- Clear
- Slightly sassy / bubblegummy
- Helpful and trustworthy

---

## 🐕 4. Mascot — Kyaro

### Role

**Kyaro** is the emotional and visual anchor of the brand.

She is:
- A **Shiba-inspired Pokémon-like companion**
- Your **loyal terminal buddy**
- Always watching, reacting, helping, protecting
- Expressive, reactive, warm

### Kyaro States (Sprite Sheet Needed)

**Essential states for Aci to design:**

| State | Description | Use Case |
|-------|-------------|----------|
| **Idle** | Resting, waiting | Default state |
| **Waiting** | Anticipating input | User about to type |
| **Listening** | Actively processing input | Command being parsed |
| **Thinking** | Deep processing | LLM inference |
| **Success** | Happy, celebrating | Command succeeded |
| **Warning** | Cautious, alert | Potentially dangerous command |
| **Error** | Sad, concerned | Command failed or blocked |
| **Bored** | Sleepy, unengaged | No activity for a while |
| **Long-inference** | Patient waiting | Model taking time |
| **Greeting** | Welcoming | First launch, hello |
| **Farewell** | Waving goodbye | Exit, shutdown |

### ASCII Fallback

**Required for limited terminals.**

Example ASCII Kyaro:
```
    /\_/\
   ( o.o )
    > ^ <
   /|   |\
  (_|   |_)
```

### Narrative

Kyaro grows with the user over time:
- From terminal helper
- To full system companion
- Long-term: gamification, progression, accessories, badges, lore

---

## 🖥️ 5. Terminal UX Constraints (Critical)

### Must-Haves ✅

- **Monospace only**
- **Retro/8-bit/bitmap/pixel fonts**
- **ANSI-rendered pixel art**
- **Sprite-based animations** (allowed)
- **ASCII fallback** (always)
- **Limited color palettes**
- **Powerline-compatible glyphs**

### No-Go Rules ❌

- ❌ **Gradients** (not terminal-compatible)
- ❌ **Photos** (defeats retro aesthetic)
- ❌ **Unicode emojis in UI** (allowed in content only)
- ❌ **Non-monospace fonts** (breaks terminal)
- ❌ **Shadows / blur / translucency** (not ANSI-compatible)
- ❌ **Complex pixel art** (must be simple, readable)
- ❌ **Diagonal anti-aliased shapes** (jagged in terminal)
- ❌ **Image files** (everything must be rendered pixel-by-pixel)

### Allowed With Care ⚠️

- ✅ **Simple dithering** (for texture)
- ✅ **Fancy TUI components** (Bubble Tea style, if terminal-native)
- ✅ **Longer animations** (as long as ANSI-based)

---

## 🏷️ 6. Logo Direction

### Options to Explore (Aci)

**Not necessarily Kyaro's face. Consider:**
- Shiba tail (iconic shape)
- Shiba marking (forehead pattern)
- Pixel shape inspired by Kyaro
- Abstract symbol representing loyalty/safety

### Requirements

- **Terminal-safe version** (ANSI-compatible)
- **Expressive web version** (more detailed)
- Should feel: **safe, loyal, familiar, retro-modern**

### Naming Exploration

- **Current:** cmdai
- **Possible future:** Hiro/Jiro (Japanese Shiba names)
- **Symbolism:** Shiba-inspired loyalty, protection

---

## 🎨 7. Color Palette

### Terminal Palette

**Primary palette:**
```
Background:     #0f0f23  (Deep navy - Game Boy dark)
Foreground:     #9bbc0f  (Game Boy green)
Accent 1:       #8bac0f  (Classic GB light)
Accent 2:       #306230  (GB medium green)
Error:          #ff3b3b  (Terminal red)
Warning:        #ffb000  (Terminal amber)
Success:        #00ff41  (Matrix green)
```

**Game Boy Inspired:**
```
--gameboy-dark:      #0f380f
--gameboy-medium:    #306230
--gameboy-light:     #8bac0f
--gameboy-lightest:  #9bbc0f
```

**Pokémon Era Neons:**
```
--neon-green:   #39ff14  (Electric energy)
--neon-blue:    #00f0ff  (Water type)
--neon-pink:    #ff10f0  (Fairy type)
--neon-purple:  #bf00ff  (Psychic type)
--neon-yellow:  #ffff00  (Electric type)
```

### Web Palette

**More freedom:**
- Retro-inspired but modern execution
- Optional: glass/liquid modern effects
- Should echo pixel aesthetic without being constrained by it
- **Can use gradients, shadows, modern CSS**

### User Configuration

- **Palette rules** for future theme creation
- **Configurable by users** (dark, light, extra variations)
- **Community-contributed themes** (future)

---

## 📦 8. Required V1 Deliverables

### Identity (Aci)

- [ ] Logo system (terminal + web versions)
- [ ] Color palette documentation (terminal + web)
- [ ] Typography system (pixel fonts)
- [ ] Brand guidelines document

### Mascot System (Aci)

- [ ] Kyaro master design (reference sheet)
- [ ] Sprite sheets for key states (11 states minimum)
- [ ] ASCII fallback design
- [ ] Emotion/state mapping
- [ ] Animation rules and timing

### Terminal UI System (Aci + Sulo)

- [ ] Primary theme
- [ ] Alternative palettes (dark, light, high-contrast)
- [ ] Layout examples
- [ ] Interaction flows (request → discovery → follow-ups)
- [ ] Error/success/warning patterns
- [ ] Timestamp/log formatting
- [ ] Fallback modes (ANSI/ASCII)

### Web System (Sulo)

- [ ] Lean component library
- [ ] Hero/header/footer
- [ ] Section templates
- [ ] Blog + changelog design
- [ ] Interactive terminal demo
- [ ] Install/quickstart instructions
- [ ] Newsletter/wishlist block
- [ ] Security page
- [ ] Docs color + typography basics (Mintlify/Docusaurus)

---

## 🌐 9. DevRel Website Purpose

### Primary Objective

A **hybrid DevRel + community hub** that:

- ✅ Explains the product clearly
- ✅ Converts visitors into users
- ✅ Attracts contributors
- ✅ Showcases demos
- ✅ Communicates philosophy
- ✅ Hosts docs + changelog
- ✅ Tells the story of the project and Kyaro
- ✅ Builds credibility + trust

### Success Metrics

- User signups / downloads
- GitHub stars and contributors
- Community engagement (Discord, GitHub Discussions)
- Documentation usage
- Newsletter subscribers

---

## 🗺️ 10. Website Page Map (V1)

### Core Pages (Must Have)

- [ ] **Home / Landing Page** (flagship)
- [ ] **About / Story** (why cmdai exists)
- [ ] **Philosophy** (Unix principles, safety-first)
- [ ] **Install / Quickstart** (get up and running)
- [ ] **Docs** (comprehensive guides)
- [ ] **Changelog** (release notes)
- [ ] **Blog** (updates, stories)
- [ ] **Security** (audit logs, safety features)
- [ ] **Contribution** (GitHub, how to help)
- [ ] **Interactive Demo** (try it in browser)

### Optional (v1.5–2.0)

- [ ] **Theme Gallery** (community themes)
- [ ] **Kyaro's World** (ASCII story / retro text-adventure game)
- [ ] **Community** (Discord, forums)
- [ ] **Governance** (project structure, decision-making)

---

## 🔮 11. Long-Term Brand Evolution (1–3 Years)

### Direction

- **Enterprise-trustworthy** (for production use)
- **Community-driven** (open governance)
- **Character-driven** (Kyaro as central figure)
- **Increasingly gamified:**
  - XP points
  - Badges and achievements
  - Items and accessories
  - Customization options
  - User profiles
  - Leaderboards (optional)

### Vision

**The world should feel like a retro game universe built for modern infra engineers.**

---

## ⚖️ 12. Working Principles

### Design Principles

- **Embrace constraints** → "make the terminal delightful"
- **Always clear + readable**
- **Cute but reliable**
- **Retro but modern**
- **Terminal-first, not terminal-limited**
- **Personality-driven** (Kyaro is the emotional anchor)

### Product Principles

- **Safe by default** (blocks dangerous commands)
- **Transparent** (shows what it's doing)
- **Auditable** (logs everything)
- **Trustworthy** (open source, community-reviewed)
- **Configurable** (user preferences respected)
- **Local-first mindset** (no cloud dependency)

---

## 📝 13. Summary for Designers

**For Aci (Art Director):**

| Aspect | Direction |
|--------|-----------|
| **Terminal aesthetics** | Strict, pixel, ANSI, retro |
| **Web aesthetics** | Retro-inspired, modern execution |
| **Mascot** | Kyaro, Pokémon-like Shiba companion |
| **Identity** | Cute, safe, friendly, nostalgic, geeky |
| **Purpose** | Empower engineers + make terminal delightful |
| **Logo** | Simple, pixel-friendly, Shiba-inspired |
| **Deliverables** | Logo, palettes, fonts, Kyaro system, terminal themes, web components |
| **Website** | DevRel hub + interactive demo + community gateway |

---

## 🎯 14. Next Steps for Aci (Art Director)

### Phase 1: Visual Exploration (Week 1)

1. **Review full brand direction** ✅
2. **Start early visual exploration:**
   - Pixel motifs
   - Kyaro silhouette tests
   - 8-bit palettes
   - Terminal theme sketches

### Phase 2: Logo & Identity (Week 2)

3. **Explore 2–3 logo directions:**
   - Shiba tail option
   - Abstract symbol
   - Simplified Kyaro face
4. **Draft visual moodboards:**
   - Game Boy references
   - Pokémon color inspiration
   - Terminal UI examples
   - Modern retro websites

### Phase 3: Mascot Development (Week 3-4)

5. **Create Kyaro reference sheet:**
   - Front, side, back views
   - Color palette
   - Personality notes
6. **Begin Kyaro sprite sheet concepts:**
   - 11 essential states
   - Frame-by-frame animations
   - Pixel dimensions (32x32, 64x64, 128x128)
7. **Design ASCII fallback:**
   - Simple, recognizable
   - Works in any terminal

### Phase 4: Web Styleframes (Week 4-5)

8. **Create early web styleframes:**
   - Hero section with Kyaro
   - Feature grid
   - Terminal demo mockup
   - Footer with personality

### Phase 5: Terminal UI (Week 5-6)

9. **Prepare early terminal UI mockups:**
   - ANSI color blocks
   - Box drawing characters
   - Layout examples
   - State transitions

---

## 🔗 Integration with Current Work

### What We've Already Built

✅ **Next.js 16 website** with 8-bit theme
✅ **Component library** (Hero, Features, Docs, Contributors)
✅ **Design system** (colors, fonts, animations)
✅ **CI/CD pipeline** (GitHub Actions)
✅ **Deployment ready** (Vercel configured)

### What Needs Updating

🔄 **Mascot name:** Change "Caro" → "Kyaro" everywhere
🔄 **Color palette:** Emphasize Game Boy + Pokémon inspiration
🔄 **Terminal constraints:** Add explicit documentation
🔄 **Sprite sheet placeholder:** Prepare for 11 states
🔄 **ASCII fallback:** Add examples

### Files to Create

📄 **BRAND_IDENTITY.md** (this file)
📄 **TERMINAL_CONSTRAINTS.md** (detailed technical guide)
📄 **KYARO_SPRITE_GUIDE.md** (for Aci's sprite work)
📄 **COLOR_PALETTE.md** (expanded color documentation)
📄 **PAGE_MAP.md** (website structure)

---

## 📚 Resources for Aci

### Visual Inspiration

- **Game Boy Interface:** https://www.reddit.com/r/Gameboy/
- **Pokémon UI:** https://bulbapedia.bulbagarden.net/wiki/User_interface
- **Pixel Art:** https://lospec.com/palette-list
- **8-bit Fonts:** https://fonts.google.com/?query=pixel
- **Terminal UI:** https://github.com/charmbracelet/bubbletea

### Tools

- **Aseprite** - Industry standard pixel art editor
- **Piskel** - Free online pixel art tool
- **Lospec** - Palette library
- **GIMP** - Free image editor (pixel-perfect mode)

---

## ✅ Acceptance Criteria

### Identity Ready When:
- [ ] Logo works in terminal (ANSI) and web
- [ ] Color palette documented with usage rules
- [ ] Typography system defined
- [ ] Brand guidelines complete

### Kyaro Ready When:
- [ ] 11 sprite states designed
- [ ] ASCII fallback created
- [ ] Animation timing defined
- [ ] Emotion mapping documented

### Website Ready When:
- [ ] All core pages complete
- [ ] Interactive demo functional
- [ ] Kyaro integrated throughout
- [ ] Terminal constraints respected
- [ ] Performance optimized
- [ ] Accessibility verified

---

**This document is the north star for all design and development work on cmdai.** 🌟

**Version:** 1.0 (December 2025)
**Last Updated:** Based on Aci's comprehensive feedback
**Team:** Aci (Art Director) + Sulo (Frontend Dev) + Community
