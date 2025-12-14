# Caro Website

Official website for **Caro** - Your loyal shell companion. Visit at **caro.sh**

## Overview

Caro is a companion agent that specializes in POSIX shell commands. She's available as an MCP for Claude and as a dedicated Skill, helping keep you safe while Claude gets the work done.

## The Story

Caro is the digitalization of Kyaro (Kyarorain Kadosh), the maintainer's beloved dog. Inspired by Portal's Caroline/GLaDOS—loyalty transformed into eternal companionship.

## Website Features

- **Hero Section**: Warm, inviting introduction with companion badge
- **Terminal Demo**: Interactive preview showing Caro generating safe shell commands
- **Story Section**: The origin story connecting Kyaro, Portal's Caroline, and GLaDOS
- **Features Grid**: 6 key capabilities emphasizing safety, cross-platform support, and Claude integration
- **Download Section**: Three ways to use Caro (CLI, MCP, Skill) with installation instructions
- **Responsive Design**: Mobile-friendly across all devices

## Design Philosophy

The website uses warm orange/amber gradient colors (#ff8c42 → #ff6b35) to evoke:
- Warmth and companionship (like a loyal dog)
- Safety and guardianship
- Energy and reliability

The design emphasizes:
- Companion-focused messaging
- Safety as a core value
- Cross-platform expertise
- Integration with Claude

## Local Development

To view the website locally:

```bash
# Simple way - just open in browser
open index.html

# Or use a local server
python3 -m http.server 8000
# Visit http://localhost:8000
```

## Customization

### Adding the Demo Video

Replace the video placeholder in the video section around line 437:

```html
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

Main brand colors (warm orange/amber for loyalty):
- Primary gradient: `#ff8c42` to `#ff6b35`
- Background warmth: `#fff8f0`
- Text: `#2c3e50` (dark blue-gray)
- Accent: `#7f8c8d` (gray)

### Modifying Content

All content is in `index.html` with clear section markers:
- Hero: Companion badge, tagline, and CTAs
- Terminal demo: Example Caro command
- Story: The Kyaro/Caroline/GLaDOS narrative
- Features: 6 companion-focused capabilities
- Download: Three usage modes (CLI, MCP, Skill)

## Deployment

### GitHub Pages

1. Push to repository
2. Settings > Pages
3. Select branch and `/website` folder
4. Site available at your GitHub Pages URL

### Custom Domain (caro.sh)

Configure your DNS:

```
# DNS Records for caro.sh
A     @     <your-server-ip>
CNAME www   <your-hosting-provider>
```

For GitHub Pages with custom domain:
1. Add `CNAME` file with `caro.sh`
2. Configure DNS A records to GitHub's IPs
3. Enable HTTPS in repository settings

### Netlify/Vercel

1. Connect GitHub repo
2. Set publish directory to `website`
3. Configure custom domain to `caro.sh`

## File Structure

```
website/
├── index.html       # Single-file website (HTML + CSS + JS)
└── README.md        # This file
```

## Key Messaging

- **Tagline**: "Your loyal shell companion"
- **Mission**: Specialized POSIX shell command agent with empathy and agency
- **Safety**: Comprehensive validation like a loyal companion
- **Platform**: Works across macOS, Linux, Windows, GNU, BSD
- **Integration**: MCP for Claude, dedicated Skill, standalone CLI
- **Story**: Kyaro → Caro, inspired by Portal's Caroline → GLaDOS

## Browser Support

- Chrome/Edge: ✅ Full support
- Firefox: ✅ Full support
- Safari: ✅ Full support
- Mobile browsers: ✅ Responsive design

## License

MIT License - Same as the Caro project
