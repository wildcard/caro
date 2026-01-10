# Competitor Alternative Landing Page System

A conversion-optimized landing page system for intercepting high-intent "competitor alternative" search traffic. Inspired by Redpanda's acquisition-triggered competitor page strategy.

## Overview

This system provides a complete toolkit for creating landing pages that capture users actively searching for alternatives to competitors after significant market events (acquisitions, pricing changes, product sunsets, etc.).

## Quick Start

### 1. Create a new page

```astro
---
// src/pages/alternative/[competitor].astro
import CompetitorAlternativeLayout from '../../layouts/CompetitorAlternativeLayout.astro';
import type { CompetitorAlternativePageConfig } from '../../types/competitor-alternative';

const pageConfig: CompetitorAlternativePageConfig = {
  competitor: { name: 'Competitor', icon: '🎯' },
  marketEvent: { title: 'Event Title', description: '...' },
  hero: { headline: '...', subheadline: '...', primaryCta: {...}, secondaryCta: {...} },
  // ... rest of config
};
---

<CompetitorAlternativeLayout config={pageConfig} />
```

### 2. Customize sections

Control which sections appear:

```astro
<CompetitorAlternativeLayout
  config={pageConfig}
  sectionVisibility={{
    socialProof: true,
    problemSection: true,
    comparisonTable: true,
    solutionSection: true,
    migrationSection: false, // Hide this section
    caseStudies: true,
    dualCta: true,
    faq: true,
  }}
/>
```

## Components

### HeroSection
Above-fold hero with market event badge, dual CTAs, and visual competitor → Caro transition.

```astro
import HeroSection from '../components/competitor-alternative/HeroSection.astro';

<HeroSection
  config={{
    headline: "[COMPETITOR] + [EVENT] = Your Pain",
    subheadline: "One-sentence positioning",
    primaryCta: { text: "Book Demo", href: "/demo" },
    secondaryCta: { text: "Try Free", href: "/signup" }
  }}
  competitor={{ name: "Competitor", icon: "🎯" }}
  marketEvent={{ title: "Big News", date: "Jan 2026" }}
/>
```

### SocialProofBand
Trust-building logo carousel with optional micro-stats.

```astro
import SocialProofBand from '../components/competitor-alternative/SocialProofBand.astro';

<SocialProofBand
  title="Trusted by leading companies"
  microStat="500+ companies worldwide"
  logos={[
    { name: "Company", logo: "/logos/company.svg", alt: "Company logo" }
  ]}
/>
```

### ProblemSection
Amplifies visitor concerns with a grid of pain points.

```astro
import ProblemSection from '../components/competitor-alternative/ProblemSection.astro';

<ProblemSection
  title="The Risk of Staying"
  subtitle="Why engineers are switching"
  painPoints={[
    { icon: "🔒", title: "Lock-in Risk", description: "..." },
    { icon: "💰", title: "Cost Trajectory", description: "..." }
  ]}
/>
```

### ComparisonTableEnhanced
Fact-based comparison with migration difficulty indicators.

```astro
import ComparisonTableEnhanced from '../components/competitor-alternative/ComparisonTableEnhanced.astro';

<ComparisonTableEnhanced
  title="Feature Comparison"
  competitorName="Competitor"
  rows={[
    {
      feature: "Works Offline",
      caro: true,
      competitor: false,
      winner: "caro",
      migrationDifficulty: "easy",
      category: "Privacy"
    }
  ]}
  footnotes={[
    { marker: "*", text: "Based on testing Jan 2026" }
  ]}
/>
```

**Features:**
- Desktop: Full table view
- Mobile: Accordion collapse for each row
- Migration difficulty badges (Easy/Medium/Complex)
- Tooltips for additional context
- "Learn more" links per row
- Category grouping support

### SolutionSection
Showcases 3-5 core differentiators with measurable claims.

```astro
import SolutionSection from '../components/competitor-alternative/SolutionSection.astro';

<SolutionSection
  title="Why Engineers Choose Caro"
  capabilities={[
    {
      icon: "🔐",
      title: "Privacy by Design",
      bullets: ["Point 1", "Point 2", "Point 3"],
      learnMoreUrl: "/privacy"
    }
  ]}
/>
```

### MigrationAccordion
Step-by-step migration guide with code snippets and gotchas.

```astro
import MigrationAccordion from '../components/competitor-alternative/MigrationAccordion.astro';

<MigrationAccordion
  title="Migrate in 30 Minutes"
  timeEstimate="30 minutes"
  steps={[
    {
      step: 1,
      title: "Install",
      content: "<p>Description...</p>",
      code: "curl -sSfL https://example.com/install.sh | sh"
    }
  ]}
  gotchas={[
    {
      title: "Known Issue",
      description: "What happens",
      solution: "How to fix"
    }
  ]}
  checklistCta={{ text: "Download Checklist", href: "/checklist.pdf" }}
/>
```

### CaseStudiesSection
Customer success stories with metrics and quotes.

```astro
import CaseStudiesSection from '../components/competitor-alternative/CaseStudiesSection.astro';

<CaseStudiesSection
  title="Success Stories"
  studies={[
    {
      customer: "Leading FinTech Company",
      industry: "Finance",
      challenge: "Why they switched",
      metrics: [
        { value: "60%", label: "Cost Reduction" },
        { value: "3x", label: "Performance" }
      ],
      quote: {
        text: "Quote text...",
        author: "Jane Doe",
        title: "CTO"
      }
    }
  ]}
/>
```

### DualCTA
Dual-path conversion section with high-commitment and low-friction options.

```astro
import DualCTA from '../components/competitor-alternative/DualCTA.astro';

<DualCTA config={{
  highCommitment: {
    title: "Talk to Sales",
    buttonText: "Book Demo",
    href: "/demo"
  },
  lowFriction: {
    title: "Try It Now",
    buttonText: "Start Free",
    href: "/signup",
    showOAuth: true,
    oauthProviders: ["github", "google"]
  },
  tertiary: {
    community: { text: "Join Slack", href: "...", platform: "slack" }
  }
}} />
```

### FAQAccordion
Expandable FAQ with Schema.org FAQPage markup for SEO.

```astro
import FAQAccordion from '../components/competitor-alternative/FAQAccordion.astro';

<FAQAccordion
  title="FAQ"
  items={[
    {
      question: "How does migration work?",
      answer: "<p>Answer with <strong>HTML</strong> support...</p>",
      learnMoreUrl: "/docs/migration"
    }
  ]}
/>
```

## Type Definitions

All types are defined in `src/types/competitor-alternative.ts`:

- `CompetitorAlternativePageConfig` - Complete page configuration
- `CompetitorInfo` - Competitor details
- `MarketEvent` - Triggering event information
- `HeroSection` - Hero configuration
- `CTAButton` - CTA button config
- `CustomerLogo` - Social proof logos
- `PainPoint` - Problem section items
- `ComparisonRow` - Comparison table rows
- `Capability` - Solution section items
- `MigrationStep` - Migration guide steps
- `CaseStudy` - Case study data
- `FAQItem` - FAQ questions/answers
- `SEOConfig` - SEO metadata
- `AnalyticsConfig` - Analytics events

## SEO Best Practices

### Page Title
Include "[COMPETITOR] Alternative" and market event:
```
Warp Alternative After Privacy Concerns | Caro
```

### Meta Description
Address search intent directly (150-160 chars):
```
Looking for a Warp alternative that works offline? Caro offers AI shell commands with 100% local inference. No cloud, no data sharing.
```

### Schema.org Markup
The layout automatically includes:
- FAQPage schema (from FAQAccordion)
- Product schema (from layout)

## Analytics & Tracking

Built-in tracking for:
- CTA clicks (with position: hero/mid/bottom)
- Comparison table interactions
- FAQ expansions
- Migration guide engagement
- Scroll depth (25%, 50%, 75%, 100%)
- Exit intent detection
- Time on page

Requires PostHog integration (configured in Layout.astro).

## A/B Testing Recommendations

### Headlines
Test variations of:
- Direct vs. indirect competitor naming
- Problem-focused vs. solution-focused
- Short vs. long headlines

### CTAs
Test:
- Button text ("Book Demo" vs. "Talk to Expert")
- Button placement (hero, mid-page, bottom)
- Dual CTA vs. single CTA

### Social Proof
Test:
- Logo count (6 vs. 8 vs. 10)
- With/without micro-stats
- Static vs. scrolling logos

### Problem Section
Test:
- 3 vs. 4 pain points
- Icon styles
- Section order (before vs. after comparison)

## Conversion Optimization Checklist

- [ ] Hero headline addresses specific market event
- [ ] Above-fold CTAs are visible on mobile
- [ ] Trust logos include recognizable brands
- [ ] Comparison table has clear Caro advantages
- [ ] Migration section reduces perceived switching cost
- [ ] Case studies include concrete metrics
- [ ] FAQ addresses top objections
- [ ] Page loads in < 2.5s (LCP)
- [ ] Schema.org markup validates
- [ ] Analytics tracking fires correctly

## File Structure

```
src/
├── components/
│   └── competitor-alternative/
│       ├── HeroSection.astro
│       ├── SocialProofBand.astro
│       ├── ProblemSection.astro
│       ├── ComparisonTableEnhanced.astro
│       ├── SolutionSection.astro
│       ├── MigrationAccordion.astro
│       ├── CaseStudyCard.astro
│       ├── CaseStudiesSection.astro
│       ├── DualCTA.astro
│       ├── FAQAccordion.astro
│       ├── index.ts
│       └── README.md
├── layouts/
│   └── CompetitorAlternativeLayout.astro
├── types/
│   └── competitor-alternative.ts
└── pages/
    └── alternative/
        └── warp-cloud.astro (example)
```

## Creating a New Competitor Page

1. **Identify the market event** - What's triggering the search?
2. **Research competitor** - Document features, pricing, limitations
3. **Gather data** - Benchmark sources, migration compatibility
4. **Create content** - Fill in the `CompetitorAlternativePageConfig`
5. **Add page** - Create `src/pages/alternative/[competitor].astro`
6. **Test** - Verify all sections render, CTAs work, analytics fire
7. **Launch** - Submit to Google Search Console, monitor rankings

## Performance Targets

- LCP (Largest Contentful Paint): < 2.5s
- FID (First Input Delay): < 100ms
- CLS (Cumulative Layout Shift): < 0.1
- Mobile PageSpeed Score: > 90

## Support

For issues or feature requests, see the main project repository.
