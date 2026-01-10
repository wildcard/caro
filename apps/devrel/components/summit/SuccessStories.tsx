'use client';

import React from 'react';

const testimonials = [
  {
    quote: "Presenting at the Seattle AI Summit was a turning point for us. We closed our Series A within 60 days of the event. The quality of investors and enterprise buyers in the room was exceptional.",
    author: "Sarah Chen",
    role: "CEO & Co-founder",
    company: "NeuralFlow AI",
    outcome: "$8M Series A",
    avatar: "SC",
  },
  {
    quote: "We came to the summit with a working prototype and left with three enterprise pilot contracts. The 10-minute demo format forced us to crystallize our value proposition perfectly.",
    author: "Marcus Williams",
    role: "Founder",
    company: "DataMesh Labs",
    outcome: "3 Enterprise Pilots",
    avatar: "MW",
  },
  {
    quote: "The connections we made at demo day led to a strategic partnership with a Fortune 100 company. That partnership became our primary go-to-market channel.",
    author: "Priya Sharma",
    role: "CTO",
    company: "AI Catalyst",
    outcome: "Fortune 100 Partner",
    avatar: "PS",
  },
];

const outcomes = [
  {
    value: '$50M+',
    label: 'Total Funding Raised',
    description: 'By demo day participants in the year following the event',
  },
  {
    value: '40+',
    label: 'Enterprise Deals Closed',
    description: 'Direct connections made during the summit',
  },
  {
    value: '85%',
    label: 'Still Operating',
    description: 'Of past demo companies still active after 3 years',
  },
  {
    value: '12',
    label: 'Acquisitions',
    description: 'Demo day alumni acquired by larger companies',
  },
];

const companyLogos = [
  'TechVentures AI',
  'CloudMind',
  'DataPilot',
  'AIFirst Labs',
  'NeuralScale',
  'Synapse.io',
  'Cortex Systems',
  'DeepQuery',
];

export const SuccessStories: React.FC = () => {
  return (
    <section id="success-stories" className="summit-section py-20 md:py-32 bg-summit-secondary/30">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        {/* Section Header */}
        <div className="text-center max-w-3xl mx-auto mb-16">
          <span className="summit-badge summit-badge-orange mb-4 inline-block">
            Social Proof
          </span>
          <h2 className="text-3xl md:text-4xl lg:text-5xl font-bold text-summit-text-primary mb-6">
            Past Success Stories
          </h2>
          <p className="text-lg text-summit-text-secondary">
            Don&apos;t just take our word for it. Here&apos;s what previous demo day participants
            achieved after presenting at the Seattle AI Summit.
          </p>
        </div>

        {/* Outcomes Grid */}
        <div className="grid grid-cols-2 md:grid-cols-4 gap-6 mb-16">
          {outcomes.map((outcome) => (
            <div key={outcome.label} className="summit-card text-center">
              <div className="summit-stat-value text-3xl md:text-4xl mb-2">
                {outcome.value}
              </div>
              <div className="text-summit-text-primary font-medium mb-1">
                {outcome.label}
              </div>
              <div className="text-xs text-summit-text-muted">
                {outcome.description}
              </div>
            </div>
          ))}
        </div>

        {/* Testimonials */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-8 mb-16">
          {testimonials.map((testimonial, index) => (
            <div key={index} className="summit-card flex flex-col">
              {/* Quote */}
              <div className="flex-1">
                <svg
                  className="w-8 h-8 text-summit-accent-teal/30 mb-4"
                  fill="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path d="M14.017 21v-7.391c0-5.704 3.731-9.57 8.983-10.609l.995 2.151c-2.432.917-3.995 3.638-3.995 5.849h4v10h-9.983zm-14.017 0v-7.391c0-5.704 3.748-9.57 9-10.609l.996 2.151c-2.433.917-3.996 3.638-3.996 5.849h3.983v10h-9.983z" />
                </svg>
                <p className="text-summit-text-secondary leading-relaxed mb-6">
                  {testimonial.quote}
                </p>
              </div>

              {/* Author */}
              <div className="flex items-center gap-4 pt-4 border-t border-summit-tertiary/30">
                <div className="w-12 h-12 rounded-full bg-gradient-to-br from-summit-accent-teal to-summit-accent-blue flex items-center justify-center text-white font-semibold">
                  {testimonial.avatar}
                </div>
                <div className="flex-1">
                  <div className="text-summit-text-primary font-medium">
                    {testimonial.author}
                  </div>
                  <div className="text-sm text-summit-text-muted">
                    {testimonial.role}, {testimonial.company}
                  </div>
                </div>
                <div className="summit-badge summit-badge-teal text-xs">
                  {testimonial.outcome}
                </div>
              </div>
            </div>
          ))}
        </div>

        {/* Company Logos */}
        <div className="text-center">
          <p className="text-summit-text-muted text-sm mb-8 uppercase tracking-wider">
            Previous Demo Day Participants
          </p>
          <div className="flex flex-wrap justify-center gap-6 md:gap-10">
            {companyLogos.map((company) => (
              <div
                key={company}
                className="px-4 py-2 text-summit-text-muted text-sm font-medium opacity-60 hover:opacity-100 transition-opacity"
              >
                {company}
              </div>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
};
