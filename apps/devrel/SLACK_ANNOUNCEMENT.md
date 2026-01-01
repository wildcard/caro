# Slack Announcement - cmdai DevRel Website Launch

> **Copy and paste the text below into your Slack #general or #devrel-website channel**

---

## 🎮 Announcing: cmdai DevRel Website - 8-Bit Pixel Art Edition!

Hey team! 👋

Excited to share that we've just launched a **new sub-project** within cmdai: our official **Developer Relations website** with a unique 8-bit pixel art design!

### 🎯 What is this?

A landing page for cmdai that:
- **Markets** our project to potential users
- **Recruits** contributors to our community
- **Showcases** our safety-first approach and features
- **Celebrates** retro gaming, terminal UI, and cybersecurity aesthetics

Think of it as the friendly face of cmdai - the first thing people see when they discover our project!

### 🎨 Design Direction

We're going **full retro** with:
- 🎮 **8-bit pixel art** (Game Boy vibes!)
- 💚 **Neon accents** on dark backgrounds (cyberpunk meets terminal)
- 🖥️ **Terminal UI aesthetics** (authentic command-line feel)
- 🕹️ **Smooth animations** (sprite bounce, scanlines, neon glow)

**Inspiration:** Game Boy games + modern cybersecurity sites (Trivy, Wiz, Torq) + best-in-class terminal UIs

### 🏗️ What's Built So Far

✅ **Complete Next.js 16 project** in `apps/devrel/`
✅ **Design system** with retro color palette and pixel fonts
✅ **Core components:** Hero, Features, Documentation, Contributors, Footer
✅ **Comprehensive documentation:** Design guidelines + contribution guide
✅ **Monorepo structure:** Clean separation from main Rust project

**Branch:** `claude/devrel-website-pixel-design-014Ku7d3Yf4a6FwX1TBskyHy`

**Preview:**
```bash
cd apps/devrel
npm install
npm run dev
# Visit: http://localhost:3000
```

### 👥 Team Leads

This project is now in the capable hands of:

**🎨 Alrezky** (Brand Designer & Art Director)
- Create "Caro" our 8-bit mascot (like GitHub's Octocat!)
- Design logos, icons, and brand assets
- Define visual identity

**💻 Sulo** (Frontend Dev & UI/UX Designer)
- Mobile responsiveness and interactions
- Accessibility improvements
- Figma prototype and design system
- Performance optimization
- New features and pages

### 🚀 How to Contribute

**For Alrezky & Sulo:**
📖 See `apps/devrel/CONTRIBUTING.md` for detailed guides

**For everyone else:**
- Check out the website locally and give feedback!
- Suggest features or improvements
- Help with content and copy
- Test on your devices

### 📂 Project Structure

```
cmdai/
├── src/               # Main Rust CLI (unchanged)
├── apps/
│   └── devrel/        # ⭐ NEW: DevRel website
│       ├── app/           # Next.js pages
│       ├── components/    # React components
│       ├── public/        # Static assets
│       ├── DESIGN_GUIDELINES.md
│       ├── CONTRIBUTING.md
│       └── README.md
```

### 🎯 Why This Matters

**For the project:**
- Professional landing page attracts more users
- Clear contribution path grows our community
- Strong brand identity makes us memorable

**For contributors:**
- Clear place to showcase features
- Easy-to-share URL for social media
- Onboarding hub for new contributors

**For users:**
- Understand cmdai at a glance
- See safety features in action
- Quick installation guide

### 📅 Sprint 1 Goals (Next 2 Weeks)

**Alrezky:**
- [ ] Caro mascot (8-bit pixel art, multiple sizes)
- [ ] Logo variations (full, icon, wordmark)
- [ ] Favicon set

**Sulo:**
- [ ] Mobile hamburger menu
- [ ] Responsive fixes
- [ ] Accessibility (ARIA labels, keyboard nav)

**Everyone:**
- [ ] Review and provide feedback
- [ ] Test on your devices
- [ ] Suggest improvements

### 🔗 Important Links

- **Code:** `apps/devrel/` in main repo
- **Documentation:** `apps/devrel/DESIGN_GUIDELINES.md`
- **Contributing:** `apps/devrel/CONTRIBUTING.md`
- **Design Inspirations:**
  - https://dribbble.com/shots/15482912-Is-Retro-Back
  - https://easy-peasy.ai/ai-image-generator/images/8-bit-pixel-art-e9400d9f-5aa7-4e34-9340-54626148c179
  - https://trivy.dev/ (layout inspiration)

### 💬 Communication

**Slack channel:** #devrel-website (or create one!)

**Daily updates:** Post progress, blockers, questions

**Questions?** Tag @alrezky for design, @sulo for dev, or @team for general

### 🎉 What's Next

**Week 1-2:** Core assets + mobile responsiveness
**Week 3-4:** Figma prototype + asset integration
**Week 5-6:** Enhanced features + polish
**Launch:** Public beta, gather feedback, iterate

### 🙌 Thank You!

Big thanks to:
- **Alrezky** for taking on the brand identity challenge
- **Sulo** for bringing the design to life with code
- **Everyone** for supporting this initiative

This is going to look amazing! Let's build something the open-source community will love! 🚀🎮

---

**Want to see it?**
```bash
git checkout claude/devrel-website-pixel-design-014Ku7d3Yf4a6FwX1TBskyHy
cd apps/devrel
npm install
npm run dev
```

**Want to contribute?** Read `apps/devrel/CONTRIBUTING.md` and jump in!

**Questions?** Reply in thread! 👇

---

*P.S. - Keep an eye out for Caro, our new mascot. She's going to be adorable! 🤖💚*
