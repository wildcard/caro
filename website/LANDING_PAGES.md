# Landing Pages Strategy

This document outlines the landing page strategy for Caro, including existing pages, planned pages, and guidelines for creating new ones.

## Directory Structure

```
website/src/
├── pages/
│   ├── index.astro                    # Homepage (general audience)
│   ├── safe-shell-commands.astro      # Landing page 1 (SRE/DevOps - original)
│   ├── ai-command-safety.astro        # Landing page 2 (AI hallucination safety)
│   ├── ai-agent-safety.astro          # Landing page 3 (Enterprise AI scale)
│   ├── try-caro.astro                 # Landing page 4 (Interactive Ratzilla demo)
│   └── ...
├── layouts/
│   └── LandingPage.astro              # Shared layout for all landing pages
└── components/
    └── landing/                       # Landing page components
        # Shared components (used by multiple pages)
        ├── LPNavigation.astro         # Shared navigation
        ├── LPBestPractices.astro      # Defense in depth section
        ├── LPScenarios.astro          # Role-based scenarios
        ├── LPTrust.astro              # Trust badges
        ├── LPDownload.astro           # Installation CTA
        ├── LPFooter.astro             # Footer
        #
        # /safe-shell-commands components (original SRE/DevOps focus)
        ├── LPHero.astro               # Safe shell commands hero
        ├── LPDemo.astro               # Terminal demos (2 scenarios)
        ├── LPCommunityVoices.astro    # Community quotes (general)
        ├── LPFeatures.astro           # Feature cards (4 features)
        ├── LPDifferentiators.astro    # Comparison section (3 points)
        ├── LPFAQ.astro                # Common concerns (6 questions)
        #
        # /ai-command-safety components (AI hallucination focus)
        ├── AICommandHero.astro        # Hero with incident warnings
        ├── AICommandDemo.astro        # Terminal demos (3 scenarios incl. AI disaster)
        ├── AICommandCommunityVoices.astro  # HN incident quotes
        ├── AICommandFeatures.astro    # Features (6 incl. hallucination resistant)
        ├── AICommandDifferentiators.astro  # Comparison (4 incl. flag vs pattern)
        ├── AICommandFAQ.astro         # AI-focused FAQ (10 questions)
        #
        # /ai-agent-safety components (Enterprise scale focus)
        ├── AIHero.astro               # Enterprise AI hero
        ├── AIWhyFlagsFail.astro       # Flag vs pattern comparison
        ├── AIRiskCalculator.astro     # Interactive risk calculator
        ├── AIIncidents.astro          # Real incidents with HN links
        └── AITestimonials.astro       # Enterprise testimonials
```

## URL Strategy

Landing pages use **clean, search-query-friendly slugs**:
- NO "landing", "lp", or "page" in the URL
- Slug should match what users search for
- Slug should be 2-4 words, hyphenated
- Slug should describe the pain point or solution

Examples:
- `/safe-shell-commands` (what they want)
- `/prevent-terminal-mistakes` (action they want)
- `/ai-shell-safety` (technology + benefit)

---

## Existing Landing Pages

### 1. `/safe-shell-commands`

**Target Persona:** Production-Paranoid SRE/DevOps Engineer
**Pain Point:** Fear of making catastrophic shell command mistakes
**Main Message:** "The last line of defense between you and rm -rf /"

**Search Queries This Page Targets:**
- "safe shell commands"
- "prevent rm rf"
- "ai terminal safety"
- "shell command validation"
- "block dangerous commands"

**Status:** ✅ Live

---

### 2. `/ai-command-safety`

**Target Persona:** Engineers concerned about AI CLI tool safety
**Pain Point:** AI hallucinations and permission flag failures in CLI tools
**Main Message:** "AI tools will fail. Caro catches what flags can't."

**Search Queries This Page Targets:**
- "ai cli safety"
- "llm shell command safety"
- "ai hallucination prevention"
- "claude code safety"
- "gemini cli safety"
- "ai terminal mistakes"

**Key Differentiators:**
- Real incident documentation (Claude Code, Gemini CLI deleting files)
- Hallucination-resistant (pattern-based, not permission-based)
- "The AI Disaster" demo scenario
- Flags vs pattern matching comparison
- Enhanced FAQ addressing AI-specific concerns

**Components:**
- AICommandHero (incident warning banner, hallucination focus)
- AICommandDemo (3 scenarios including "The AI Disaster")
- LPBestPractices (defense in depth)
- LPScenarios (role-based)
- AICommandCommunityVoices (HN incident quotes)
- LPTrust (trust badges)
- AICommandFeatures (6 features incl. hallucination resistant)
- AICommandDifferentiators (4 points incl. flag vs pattern)
- AICommandFAQ (10 AI-focused questions)
- LPDownload, LPFooter

**Status:** ✅ Live

---

### 3. `/ai-agent-safety`

**Target Persona:** Enterprise teams deploying AI agents at scale
**Pain Point:** AI hallucinations, flag failures, stochastic system risks
**Main Message:** "AI Agents Run Dangerous Commands. Caro Catches Them."

**Search Queries This Page Targets:**
- "ai agent safety"
- "llm hallucination prevention"
- "ai coding tool safety"
- "claude code safety"
- "gemini cli safety"
- "enterprise ai risk"

**Key Differentiators:**
- Hallucination-resistant (pattern-based, not permission-based)
- Deterministic validation (not probabilistic)
- Enterprise scale math (risk calculator)
- Real incident documentation (Claude, Gemini)

**Components:**
- AIHero (incidents banner, hallucination focus)
- AIWhyFlagsFail (flag vs pattern comparison)
- AIRiskCalculator (interactive enterprise risk demo)
- AIIncidents (documented incidents with HN links)
- AITestimonials (enterprise-focused quotes)
- LPBestPractices (defense in depth)
- LPFAQ (AI-focused questions)

**Status:** ✅ Live

---

### 4. `/try-caro`

**Target Persona:** Developers curious about Caro's capabilities
**Pain Point:** Want to experience Caro before installing
**Main Message:** "Experience Caro's safety-first shell companion in your browser"

**Search Queries This Page Targets:**
- "try caro online"
- "caro demo"
- "shell command generator demo"
- "ai terminal demo"
- "caro playground"

**Key Differentiators:**
- Interactive WebAssembly-powered terminal demo
- Ratzilla framework (Rust + WASM + Ratatui)
- Real-time safety validation demonstration
- No installation required

**Components:**
- RatzillaDemo (interactive WASM terminal UI)
- Technology showcase (Rust, WASM, Ratzilla, Safety Patterns)
- Quick install CTAs
- Ratzilla attribution section

**Technology:**
- Powered by [Ratzilla](https://github.com/orhun/ratzilla) by @orhun
- Compiled Rust to WebAssembly using Trunk
- WebGL2 rendering for authentic terminal aesthetics

**Status:** ✅ Live

---

## Planned Landing Pages

### 5. `/air-gapped-ai-terminal` (Priority: High)

**Target Persona:** Security-Conscious Platform Engineer
**Pain Point:** Need AI assistance but can't send data to cloud
**Main Message:** "AI shell commands for air-gapped environments"

**Search Queries:**
- "air gapped ai tools"
- "offline ai terminal"
- "local llm shell"
- "no cloud ai cli"

**Key Differentiators to Highlight:**
- Privacy-first design (see /telemetry)
- 100% local inference
- Compliance-friendly
- Self-hostable

**Components to Customize:**
- Hero: Focus on compliance/security
- Use Cases: Regulated industries (healthcare, finance, gov)
- Social Proof: Compliance-focused quotes

---

### 6. `/cross-platform-shell-commands` (Priority: Medium)

**Target Persona:** Developer who works across Mac/Linux/BSD
**Pain Point:** Commands work on Mac but fail on Linux server
**Main Message:** "Shell commands that work on every platform, first time"

**Search Queries:**
- "cross platform shell commands"
- "mac linux command differences"
- "bsd vs gnu commands"
- "portable shell scripts"

**Key Differentiators:**
- Platform detection
- BSD vs GNU awareness
- POSIX compliance
- Works everywhere

**Components to Customize:**
- Hero: Focus on frustration of platform differences
- Demo: Show same command, different platforms
- Use Cases: CI/CD, multi-platform dev

---

### 7. `/natural-language-terminal` (Priority: Medium)

**Target Persona:** Developer who struggles to remember command syntax
**Pain Point:** Constantly looking up command syntax on Stack Overflow
**Main Message:** "Describe what you want. Get the command that works."

**Search Queries:**
- "natural language shell commands"
- "ai terminal commands"
- "describe command get code"
- "english to bash"

**Key Differentiators:**
- Natural language understanding
- Context-aware suggestions
- No memorization needed
- Faster than Stack Overflow

---

### 8. `/junior-dev-safety-rails` (Priority: Low)

**Target Persona:** Team Lead / Senior Engineer
**Pain Point:** Junior devs running dangerous commands in production
**Main Message:** "Safety rails for your whole team"

**Search Queries:**
- "prevent junior dev mistakes"
- "production safety tools"
- "team shell command safety"
- "devops guardrails"

**Key Differentiators:**
- Team-wide protection
- No micromanagement
- Learning tool
- Reduces on-call incidents

---

### 9. `/claude-mcp-shell-agent` (Priority: Future)

**Target Persona:** Claude user who wants shell integration
**Pain Point:** Claude can't safely execute shell commands
**Main Message:** "Give Claude safe shell superpowers"

**Search Queries:**
- "claude mcp shell"
- "claude terminal integration"
- "ai agent shell commands"
- "claude devops"

**Dependencies:** MCP integration must be complete first

---

## Landing Page Creation Guide

### Step 1: Define the Page

Before creating a landing page, document:

```yaml
slug: [url-friendly-slug]
persona: [specific target user]
pain_point: [the specific problem they have]
main_message: [the headline/hook]
search_queries:
  - query 1
  - query 2
  - query 3
differentiators:
  - key point 1
  - key point 2
  - key point 3
```

### Step 2: Create the Page File

```astro
---
/**
 * Landing Page: [Title]
 *
 * Target Persona: [who]
 * Pain Point: [what problem]
 * Search Queries: [list of queries]
 *
 * Main Message: "[headline]"
 */

import LandingPage from '../layouts/LandingPage.astro';
import LPNavigation from '../components/landing/LPNavigation.astro';
// ... other imports
---

<LandingPage
  title="[SEO Title] | Caro"
  description="[Meta description targeting search queries]"
  slug="[slug]"
  persona="[persona]"
  painPoint="[pain point]"
>
  <LPNavigation />
  <!-- Customized components -->
</LandingPage>
```

### Step 3: Customize Components

For each landing page, customize:

1. **Hero Section**
   - Headline (speaks to specific pain)
   - Subtitle (specific solution)
   - Social proof quote (relevant to persona)
   - Trust badges (most relevant for persona)

2. **Demo Section**
   - Scenarios relevant to persona
   - Commands relevant to their workflow

3. **Use Cases**
   - 4 scenarios that resonate with persona
   - Problem/solution format

4. **Features**
   - Order by relevance to persona
   - Highlight 2 most important

5. **Differentiators**
   - Focus on what matters to this persona
   - Competitor comparison relevant to them

### Step 4: Test & Iterate

1. Build and verify: `npm run build`
2. Check SEO: title, description, meta tags
3. Test conversion flow: hero → demo → download
4. A/B test headlines if possible

---

## Shared Components vs Custom

**Always Shared:**
- LPNavigation (consistent brand)
- LPDownload (same install process)
- LPFooter (consistent links)

**Usually Customized:**
- LPHero (headline, subtitle, quote)
- LPDemo (scenarios, commands)
- LPUseCases (specific to persona)

**Sometimes Customized:**
- LPFeatures (reorder, highlight different ones)
- LPDifferentiators (emphasize different points)

---

## Metrics to Track

For each landing page, track:

1. **Traffic**
   - Organic search visits
   - Search queries leading to page
   - Referral sources

2. **Engagement**
   - Time on page
   - Scroll depth
   - Demo interaction

3. **Conversion**
   - Install command copies
   - GitHub clicks
   - Bounce rate

---

## Future Enhancements

1. **Dynamic Personalization**
   - Detect OS and customize demo
   - Show relevant use cases by referrer

2. **A/B Testing**
   - Test different headlines
   - Test different social proof

3. **Landing Page Generator**
   - CLI tool to scaffold new pages
   - Template with prompts for customization

4. **Analytics Dashboard**
   - Per-page conversion tracking
   - Search query → conversion mapping
