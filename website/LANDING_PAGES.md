# Landing Pages Strategy

This document outlines the landing page strategy for Caro, including existing pages, planned pages, and guidelines for creating new ones.

## Directory Structure

```
website/src/
├── pages/
│   ├── index.astro                    # Homepage (general audience)
│   ├── safe-shell-commands.astro      # Landing page 1 (SRE/DevOps)
│   ├── ai-agent-safety.astro          # Landing page 2 (Enterprise AI)
│   └── ...
├── layouts/
│   └── LandingPage.astro              # Shared layout for all landing pages
└── components/
    └── landing/                       # Landing page components
        ├── LPNavigation.astro         # Shared navigation
        ├── LPHero.astro               # Safe shell commands hero
        ├── LPDemo.astro               # Terminal demos
        ├── LPBestPractices.astro      # Defense in depth section
        ├── LPScenarios.astro          # Role-based scenarios
        ├── LPCommunityVoices.astro    # Community quotes
        ├── LPTrust.astro              # Trust badges
        ├── LPFeatures.astro           # Feature cards
        ├── LPDifferentiators.astro    # Comparison section
        ├── LPFAQ.astro                # Common concerns
        ├── LPDownload.astro           # Installation CTA
        ├── LPFooter.astro             # Footer
        ├── AIHero.astro               # AI safety hero
        ├── AIWhyFlagsFail.astro       # Flag vs pattern comparison
        ├── AIRiskCalculator.astro     # Interactive risk calculator
        ├── AIIncidents.astro          # Real incidents
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

### 2. `/ai-agent-safety`

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

## Planned Landing Pages

### 3. `/air-gapped-ai-terminal` (Priority: High)

**Target Persona:** Security-Conscious Platform Engineer
**Pain Point:** Need AI assistance but can't send data to cloud
**Main Message:** "AI shell commands for air-gapped environments"

**Search Queries:**
- "air gapped ai tools"
- "offline ai terminal"
- "local llm shell"
- "no cloud ai cli"

**Key Differentiators to Highlight:**
- Zero telemetry
- 100% local inference
- Compliance-friendly
- Self-hostable

**Components to Customize:**
- Hero: Focus on compliance/security
- Use Cases: Regulated industries (healthcare, finance, gov)
- Social Proof: Compliance-focused quotes

---

### 3. `/cross-platform-shell-commands` (Priority: Medium)

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

### 4. `/natural-language-terminal` (Priority: Medium)

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

### 5. `/junior-dev-safety-rails` (Priority: Low)

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

### 6. `/claude-mcp-shell-agent` (Priority: Future)

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
