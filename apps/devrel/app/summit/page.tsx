import {
  SummitNavigation,
  SummitHero,
  WhatWereLookingFor,
  ApplicationFAQ,
  TimelineProcess,
  SuccessStories,
  PreparationChecklist,
  ApplicationCTA,
  SummitFooter,
} from '@/components/summit';

export default function SummitPage() {
  return (
    <div className="summit-page min-h-screen">
      {/* Navigation */}
      <SummitNavigation />

      {/* Main Content */}
      <main>
        {/* Hero Section */}
        <SummitHero />

        {/* What We're Looking For */}
        <WhatWereLookingFor />

        {/* Application FAQ */}
        <ApplicationFAQ />

        {/* Timeline & Process */}
        <TimelineProcess />

        {/* Success Stories */}
        <SuccessStories />

        {/* Preparation Checklist */}
        <PreparationChecklist />

        {/* Application CTA */}
        <ApplicationCTA />
      </main>

      {/* Footer */}
      <SummitFooter />
    </div>
  );
}
