'use client';

import React, { useState } from 'react';

const checklistItems = [
  {
    id: 'website',
    label: 'Company website is live and polished',
    description: 'Ensure your website clearly communicates your value proposition and product.',
    tip: 'Check mobile responsiveness and page load times.',
  },
  {
    id: 'linkedin',
    label: 'Founder LinkedIn profile is complete and professional',
    description: 'Update your photo, headline, and recent company updates.',
    tip: 'Add recent posts about your company progress.',
  },
  {
    id: 'description',
    label: '100-word company description drafted',
    description: 'Concise, clear description of what you do and for whom.',
    tip: 'Focus on the problem you solve, not technical features.',
  },
  {
    id: 'financials',
    label: 'Financial metrics compiled (ARR, funding)',
    description: 'Have your current ARR, total raised, and funding stage ready.',
    tip: 'Include growth rate if impressive (e.g., "3x YoY").',
  },
  {
    id: 'customers',
    label: 'Customer list and achievements documented',
    description: 'Prepare a list of notable customers and key metrics.',
    tip: 'Get logo usage permission from customers in advance.',
  },
  {
    id: 'competitive',
    label: 'Competitive analysis prepared',
    description: 'Know your top 3 competitors and your differentiation.',
    tip: 'Be honest about competitors; reviewers know the market.',
  },
  {
    id: 'video',
    label: 'Video demo recorded (optional but recommended)',
    description: '2-3 minute product demo showcasing core functionality.',
    tip: 'Use Loom or YouTube (unlisted). Focus on real use cases.',
  },
  {
    id: 'advisors',
    label: 'Advisor/investor list ready',
    description: 'Names and affiliations of notable backers.',
    tip: 'Include angel investors and strategic advisors.',
  },
];

const CheckIcon: React.FC = () => (
  <svg className="w-4 h-4 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={3} d="M5 13l4 4L19 7" />
  </svg>
);

export const PreparationChecklist: React.FC = () => {
  const [checkedItems, setCheckedItems] = useState<Set<string>>(new Set());

  const toggleItem = (id: string) => {
    const newChecked = new Set(checkedItems);
    if (newChecked.has(id)) {
      newChecked.delete(id);
    } else {
      newChecked.add(id);
    }
    setCheckedItems(newChecked);
  };

  const progress = (checkedItems.size / checklistItems.length) * 100;

  return (
    <section id="checklist" className="summit-section py-20 md:py-32">
      <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8">
        {/* Section Header */}
        <div className="text-center mb-12">
          <span className="summit-badge summit-badge-teal mb-4 inline-block">
            Get Ready
          </span>
          <h2 className="text-3xl md:text-4xl lg:text-5xl font-bold text-summit-text-primary mb-6">
            Preparation Checklist
          </h2>
          <p className="text-lg text-summit-text-secondary max-w-2xl mx-auto">
            Use this interactive checklist to prepare everything you need before
            starting your application. Being prepared will help you submit a stronger application.
          </p>
        </div>

        {/* Progress Bar */}
        <div className="mb-10">
          <div className="flex items-center justify-between mb-2">
            <span className="text-sm text-summit-text-secondary">Preparation Progress</span>
            <span className="text-sm font-medium text-summit-accent-teal">
              {checkedItems.size} of {checklistItems.length} complete
            </span>
          </div>
          <div className="h-3 bg-summit-tertiary rounded-full overflow-hidden">
            <div
              className="h-full bg-gradient-to-r from-summit-accent-teal to-summit-accent-blue rounded-full transition-all duration-500"
              style={{ width: `${progress}%` }}
            />
          </div>
          {progress === 100 && (
            <p className="text-center text-summit-success text-sm mt-3 flex items-center justify-center gap-2">
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              You&apos;re ready to apply!
            </p>
          )}
        </div>

        {/* Checklist Items */}
        <div className="space-y-3">
          {checklistItems.map((item) => {
            const isChecked = checkedItems.has(item.id);
            return (
              <div
                key={item.id}
                className={`summit-checklist-item ${isChecked ? 'checked' : ''}`}
                onClick={() => toggleItem(item.id)}
                role="checkbox"
                aria-checked={isChecked}
                tabIndex={0}
                onKeyDown={(e) => {
                  if (e.key === 'Enter' || e.key === ' ') {
                    e.preventDefault();
                    toggleItem(item.id);
                  }
                }}
              >
                <div className="summit-checklist-checkbox">
                  {isChecked && <CheckIcon />}
                </div>
                <div className="flex-1">
                  <div className={`font-medium ${isChecked ? 'text-summit-success' : 'text-summit-text-primary'} mb-1`}>
                    {item.label}
                  </div>
                  <div className="text-sm text-summit-text-muted">
                    {item.description}
                  </div>
                  {!isChecked && (
                    <div className="text-xs text-summit-accent-teal mt-2 flex items-center gap-1">
                      <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                      </svg>
                      Tip: {item.tip}
                    </div>
                  )}
                </div>
              </div>
            );
          })}
        </div>

        {/* Time Estimate */}
        <div className="mt-10 p-6 rounded-xl bg-summit-tertiary/30 border border-summit-tertiary/50 text-center">
          <div className="flex items-center justify-center gap-3 mb-2">
            <svg className="w-6 h-6 text-summit-accent-teal" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <span className="text-summit-text-primary font-semibold">
              Estimated Application Time: 10-15 minutes
            </span>
          </div>
          <p className="text-sm text-summit-text-muted">
            If you have all the items above prepared, the application form should take about 10-15 minutes to complete.
          </p>
        </div>
      </div>
    </section>
  );
};
