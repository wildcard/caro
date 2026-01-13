/**
 * Type definitions for Competitor Alternative Landing Pages
 *
 * These types support conversion-optimized landing pages that intercept
 * high-intent "competitor alternative" search traffic.
 */

// ============================================
// Core Content Types
// ============================================

export interface CompetitorInfo {
  /** Competitor name (e.g., "Confluent", "Warp") */
  name: string;
  /** Emoji or icon identifier */
  icon: string;
  /** Short tagline about the competitor */
  tagline?: string;
  /** Longer description if needed */
  description?: string;
}

export interface MarketEvent {
  /** Brief description of the event (e.g., "IBM Acquisition") */
  title: string;
  /** Longer description of what happened */
  description: string;
  /** Date of the event (for freshness indicators) */
  date?: string;
  /** External source link for credibility */
  sourceUrl?: string;
  /** Source name (e.g., "TechCrunch", "Reuters") */
  sourceName?: string;
}

// ============================================
// Hero Section
// ============================================

export interface HeroSection {
  /** Main H1 headline - should reference market event + pain point */
  headline: string;
  /** Subheading - single-sentence alternative positioning */
  subheadline: string;
  /** Primary CTA button */
  primaryCta: CTAButton;
  /** Secondary CTA button */
  secondaryCta: CTAButton;
  /** Optional thematic imagery description */
  imagery?: string;
}

export interface CTAButton {
  /** Button text */
  text: string;
  /** Link destination */
  href: string;
  /** Optional: track event name for analytics */
  trackingEvent?: string;
  /** Optional: position identifier for analytics */
  position?: 'hero' | 'mid' | 'bottom';
}

// ============================================
// Social Proof
// ============================================

export interface CustomerLogo {
  /** Company name */
  name: string;
  /** Path to logo image or SVG */
  logo: string;
  /** Alt text for accessibility */
  alt: string;
  /** Optional link to case study */
  caseStudyUrl?: string;
}

export interface SocialProofSection {
  /** Optional title above logos */
  title?: string;
  /** Customer logos (6-8 recommended) */
  logos: CustomerLogo[];
  /** Optional micro-stat (e.g., "Trusted by 500+ companies") */
  microStat?: string;
}

// ============================================
// Problem Framing ("The Dark Side")
// ============================================

export interface PainPoint {
  /** Icon or emoji */
  icon: string;
  /** Bold heading describing the risk */
  title: string;
  /** 2-3 sentences explaining why it matters now */
  description: string;
}

export interface ProblemSection {
  /** Section title (e.g., "The Risk of Staying") */
  title: string;
  /** Optional subtitle */
  subtitle?: string;
  /** 3-4 pain points */
  painPoints: PainPoint[];
}

// ============================================
// Comparison Table (Enhanced)
// ============================================

export type MigrationDifficulty = 'easy' | 'medium' | 'complex';
export type ComparisonValue = boolean | string | 'partial';

export interface ComparisonRow {
  /** Feature or capability name */
  feature: string;
  /** Your product's value */
  caro: ComparisonValue;
  /** Competitor's value */
  competitor: ComparisonValue;
  /** Migration difficulty indicator */
  migrationDifficulty?: MigrationDifficulty;
  /** Who wins this comparison */
  winner: 'caro' | 'competitor' | 'tie';
  /** Optional tooltip with more details */
  tooltip?: string;
  /** Optional "Learn More" link */
  learnMoreUrl?: string;
  /** Category for grouping (e.g., "Pricing", "Deployment", "Security") */
  category?: string;
}

export interface ComparisonTableSection {
  /** Section title */
  title: string;
  /** Optional subtitle */
  subtitle?: string;
  /** Comparison rows */
  rows: ComparisonRow[];
  /** Optional footnotes with sources/dates */
  footnotes?: Footnote[];
}

export interface Footnote {
  /** Footnote marker (e.g., "1", "*") */
  marker: string;
  /** Footnote text */
  text: string;
  /** Optional source URL */
  sourceUrl?: string;
  /** Date of benchmark/source */
  date?: string;
}

// ============================================
// Solution Positioning
// ============================================

export interface Capability {
  /** Icon or emoji */
  icon: string;
  /** Capability headline */
  title: string;
  /** 2-3 concrete, measurable bullet points */
  bullets: string[];
  /** Optional screenshot or diagram path */
  image?: string;
  /** "Learn More" link to deep-dive documentation */
  learnMoreUrl?: string;
}

export interface SolutionSection {
  /** Section title */
  title: string;
  /** Optional subtitle */
  subtitle?: string;
  /** 3-5 capability blocks */
  capabilities: Capability[];
}

// ============================================
// Migration Quickstart
// ============================================

export interface MigrationStep {
  /** Step number or identifier */
  step: number;
  /** Step title */
  title: string;
  /** Step description (supports HTML) */
  content: string;
  /** Optional code snippet */
  code?: string;
  /** Optional list of sub-items */
  items?: string[];
  /** Optional link to detailed docs */
  docsUrl?: string;
}

export interface MigrationGotcha {
  /** Issue title */
  title: string;
  /** Issue description */
  description: string;
  /** Solution or workaround */
  solution: string;
}

export interface MigrationSection {
  /** Section title (e.g., "Migration in 30 Minutes") */
  title: string;
  /** Time estimate */
  timeEstimate: string;
  /** Migration steps */
  steps: MigrationStep[];
  /** Common gotchas/issues */
  gotchas?: MigrationGotcha[];
  /** CTA for gated content (e.g., "Get Migration Checklist") */
  checklistCta?: {
    text: string;
    href: string;
  };
  /** Link to full migration guide */
  fullGuideUrl?: string;
}

// ============================================
// Case Studies
// ============================================

export interface CaseStudyMetric {
  /** Metric value (e.g., "60%", "$500K") */
  value: string;
  /** Metric label (e.g., "Cost Reduction", "Latency Improvement") */
  label: string;
}

export interface CaseStudy {
  /** Customer name or anonymous descriptor (e.g., "Leading FinTech Company") */
  customer: string;
  /** Customer logo path (optional if anonymous) */
  logo?: string;
  /** Industry/sector */
  industry: string;
  /** The challenge - why they left competitor (1 sentence) */
  challenge: string;
  /** Measurable outcomes (at least 2) */
  metrics: CaseStudyMetric[];
  /** Pull quote from technical decision-maker */
  quote?: {
    text: string;
    author: string;
    title: string;
  };
  /** Link to full case study */
  fullStudyUrl?: string;
}

export interface CaseStudiesSection {
  /** Section title */
  title: string;
  /** Optional subtitle */
  subtitle?: string;
  /** 2-3 customer stories */
  studies: CaseStudy[];
}

// ============================================
// Dual CTA Section
// ============================================

export interface FormField {
  /** Field name */
  name: string;
  /** Field label */
  label: string;
  /** Field type */
  type: 'text' | 'email' | 'select' | 'textarea';
  /** Whether field is required */
  required: boolean;
  /** Placeholder text */
  placeholder?: string;
  /** Options for select type */
  options?: string[];
}

export interface DualCTASection {
  /** High-commitment side */
  highCommitment: {
    title: string;
    subtitle?: string;
    buttonText: string;
    href: string;
    /** Optional form fields for inline form */
    formFields?: FormField[];
    /** Whether to show calendar picker */
    showCalendar?: boolean;
  };
  /** Low-friction side */
  lowFriction: {
    title: string;
    subtitle?: string;
    buttonText: string;
    href: string;
    /** Whether to show OAuth options */
    showOAuth?: boolean;
    /** OAuth providers */
    oauthProviders?: ('github' | 'google' | 'microsoft')[];
  };
  /** Tertiary CTAs below both */
  tertiary?: {
    liveChat?: {
      text: string;
      widgetId?: string;
    };
    community?: {
      text: string;
      href: string;
      platform: 'slack' | 'discord' | 'other';
    };
  };
}

// ============================================
// FAQ Section
// ============================================

export interface FAQItem {
  /** Question text */
  question: string;
  /** Answer (supports HTML) */
  answer: string;
  /** Optional link to more details */
  learnMoreUrl?: string;
}

export interface FAQSection {
  /** Section title */
  title: string;
  /** 5-7 FAQ items */
  items: FAQItem[];
}

// ============================================
// SEO & Analytics
// ============================================

export interface SEOConfig {
  /** Page title - should include "[COMPETITOR] Alternative" */
  title: string;
  /** Meta description (150-160 chars) */
  description: string;
  /** Open Graph title */
  ogTitle?: string;
  /** Open Graph description */
  ogDescription?: string;
  /** Open Graph image URL */
  ogImage?: string;
  /** Canonical URL */
  canonical?: string;
  /** Keywords (optional, limited SEO value) */
  keywords?: string[];
}

export interface AnalyticsConfig {
  /** Page identifier for analytics */
  pageId: string;
  /** Events to track */
  events: {
    ctaClick: string;
    comparisonInteraction: string;
    faqExpand: string;
    migrationDownload: string;
    scrollDepth: string;
    exitIntent?: string;
  };
}

// ============================================
// Complete Page Configuration
// ============================================

export interface CompetitorAlternativePageConfig {
  /** Competitor information */
  competitor: CompetitorInfo;
  /** Market event that triggered the opportunity */
  marketEvent: MarketEvent;
  /** Hero section content */
  hero: HeroSection;
  /** Social proof section */
  socialProof?: SocialProofSection;
  /** Problem framing section */
  problemSection: ProblemSection;
  /** Comparison table */
  comparisonTable: ComparisonTableSection;
  /** Solution positioning */
  solutionSection: SolutionSection;
  /** Migration quickstart */
  migrationSection?: MigrationSection;
  /** Case studies */
  caseStudies?: CaseStudiesSection;
  /** Final dual CTA */
  dualCta: DualCTASection;
  /** FAQ section */
  faq: FAQSection;
  /** SEO configuration */
  seo: SEOConfig;
  /** Analytics configuration */
  analytics?: AnalyticsConfig;
}

// ============================================
// Utility Types
// ============================================

/** Helper type for section visibility */
export type SectionVisibility = {
  socialProof: boolean;
  problemSection: boolean;
  comparisonTable: boolean;
  solutionSection: boolean;
  migrationSection: boolean;
  caseStudies: boolean;
  dualCta: boolean;
  faq: boolean;
};

/** Default section visibility */
export const defaultSectionVisibility: SectionVisibility = {
  socialProof: true,
  problemSection: true,
  comparisonTable: true,
  solutionSection: true,
  migrationSection: true,
  caseStudies: true,
  dualCta: true,
  faq: true,
};
