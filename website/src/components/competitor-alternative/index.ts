/**
 * Competitor Alternative Landing Page Components
 *
 * Export all components for easy importing.
 */

// Re-export all components from a single entry point
// Usage: import { ComparisonTableEnhanced, ... } from '../components/competitor-alternative';

// Note: In Astro, we typically import .astro components directly
// This file serves as documentation of available components

export const components = {
  HeroSection: './HeroSection.astro',
  SocialProofBand: './SocialProofBand.astro',
  ProblemSection: './ProblemSection.astro',
  ComparisonTableEnhanced: './ComparisonTableEnhanced.astro',
  SolutionSection: './SolutionSection.astro',
  MigrationAccordion: './MigrationAccordion.astro',
  CaseStudyCard: './CaseStudyCard.astro',
  CaseStudiesSection: './CaseStudiesSection.astro',
  DualCTA: './DualCTA.astro',
  FAQAccordion: './FAQAccordion.astro',
} as const;

// Export type definitions for convenience
export type { CompetitorAlternativePageConfig } from '../../types/competitor-alternative';
