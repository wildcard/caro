'use client';

import React from 'react';

const criteriaCategories = [
  {
    title: 'Company Fundamentals',
    icon: (
      <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4" />
      </svg>
    ),
    badge: 'Foundation',
    badgeColor: 'summit-badge-teal',
    items: [
      {
        label: 'Year Founded',
        description: 'Typically 2020 or later. We prioritize early-stage startups with recent momentum.',
      },
      {
        label: 'Team Size',
        description: '3-50 employees. We look for lean teams with strong technical capabilities.',
      },
      {
        label: 'ARR Expectations',
        description: '$100K-$5M ARR preferred. Pre-revenue companies with strong traction also considered.',
      },
      {
        label: 'Funding Stage',
        description: 'Seed to Series A. We welcome bootstrapped companies with demonstrated growth.',
      },
    ],
  },
  {
    title: 'Strategic Clarity',
    icon: (
      <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
      </svg>
    ),
    badge: 'Strategy',
    badgeColor: 'summit-badge-blue',
    items: [
      {
        label: 'Problem Statement',
        description: 'Clear articulation of the specific pain point you solve. No vague "AI for everything" pitches.',
      },
      {
        label: 'Target Customer Profile',
        description: 'Well-defined ICP with specific industries, company sizes, and buyer personas.',
      },
      {
        label: 'Competitive Differentiation',
        description: 'What makes your approach unique? Technology, data, team, or go-to-market strategy.',
      },
      {
        label: 'Business Model',
        description: 'Clear monetization strategy with pricing rationale and unit economics understanding.',
      },
    ],
  },
  {
    title: 'Proof Points',
    icon: (
      <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
      </svg>
    ),
    badge: 'Traction',
    badgeColor: 'summit-badge-purple',
    items: [
      {
        label: 'Revenue Milestones',
        description: 'MRR growth rate, customer expansion, or significant POC conversions.',
      },
      {
        label: 'Customer Validation',
        description: 'Logos, testimonials, case studies, or NPS scores from existing customers.',
      },
      {
        label: 'Partnerships',
        description: 'Strategic integrations, channel partnerships, or enterprise pilot programs.',
      },
      {
        label: 'Investor Credibility',
        description: 'Notable angel investors, VCs, or advisors who have backed your vision.',
      },
    ],
  },
];

export const WhatWereLookingFor: React.FC = () => {
  return (
    <section id="criteria" className="summit-section py-20 md:py-32">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        {/* Section Header */}
        <div className="text-center max-w-3xl mx-auto mb-16">
          <span className="summit-badge summit-badge-teal mb-4 inline-block">
            Evaluation Criteria
          </span>
          <h2 className="text-3xl md:text-4xl lg:text-5xl font-bold text-summit-text-primary mb-6">
            What We&apos;re Looking For
          </h2>
          <p className="text-lg text-summit-text-secondary">
            We evaluate applications across three core dimensions. Understanding our criteria
            helps you prepare a stronger application and increases your chances of selection.
          </p>
        </div>

        {/* Criteria Cards */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
          {criteriaCategories.map((category, index) => (
            <div
              key={category.title}
              className="summit-card group"
              style={{ animationDelay: `${index * 100}ms` }}
            >
              {/* Card Header */}
              <div className="flex items-center gap-4 mb-6">
                <div className="w-12 h-12 rounded-xl bg-gradient-to-br from-summit-accent-teal/20 to-summit-accent-blue/20 flex items-center justify-center text-summit-accent-teal group-hover:scale-110 transition-transform">
                  {category.icon}
                </div>
                <div>
                  <span className={`summit-badge ${category.badgeColor} text-xs mb-1`}>
                    {category.badge}
                  </span>
                  <h3 className="text-xl font-semibold text-summit-text-primary">
                    {category.title}
                  </h3>
                </div>
              </div>

              {/* Criteria Items */}
              <div className="space-y-4">
                {category.items.map((item) => (
                  <div
                    key={item.label}
                    className="border-l-2 border-summit-tertiary pl-4 py-1 hover:border-summit-accent-teal transition-colors"
                  >
                    <h4 className="text-summit-text-primary font-medium mb-1">
                      {item.label}
                    </h4>
                    <p className="text-sm text-summit-text-muted leading-relaxed">
                      {item.description}
                    </p>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>

        {/* Pro Tip */}
        <div className="mt-12 p-6 rounded-xl bg-gradient-to-r from-summit-accent-teal/10 to-summit-accent-blue/10 border border-summit-accent-teal/20">
          <div className="flex items-start gap-4">
            <div className="flex-shrink-0 w-10 h-10 rounded-lg bg-summit-accent-teal/20 flex items-center justify-center">
              <svg className="w-5 h-5 text-summit-accent-teal" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
            </div>
            <div>
              <h4 className="text-summit-text-primary font-semibold mb-1">Pro Tip</h4>
              <p className="text-summit-text-secondary text-sm leading-relaxed">
                Don&apos;t have all these criteria covered? That&apos;s okay! We evaluate holistically
                and understand that early-stage startups are works in progress. Focus on your
                strongest proof points and be honest about areas you&apos;re still developing.
              </p>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
};
