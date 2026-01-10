'use client';

import React from 'react';

export const ApplicationCTA: React.FC = () => {
  return (
    <section id="apply" className="summit-section py-20 md:py-32 relative overflow-hidden">
      {/* Background decorations */}
      <div className="summit-glow-teal top-0 left-1/4" />
      <div className="summit-glow-purple bottom-0 right-1/4" />

      <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 relative z-10">
        <div className="summit-card text-center py-12 md:py-16">
          {/* Badge */}
          <span className="summit-badge summit-badge-teal mb-6 inline-block">
            Limited Slots Available
          </span>

          {/* Headline */}
          <h2 className="text-3xl md:text-4xl lg:text-5xl font-bold text-summit-text-primary mb-6">
            Ready to Showcase Your
            <br />
            <span className="summit-gradient-text">AI Innovation?</span>
          </h2>

          {/* Description */}
          <p className="text-lg text-summit-text-secondary max-w-2xl mx-auto mb-8">
            Join 50 carefully selected AI startups presenting to Fortune 500 executives,
            top-tier investors, and industry media. Your 10-minute demo could change
            the trajectory of your company.
          </p>

          {/* Key Benefits */}
          <div className="grid grid-cols-1 sm:grid-cols-3 gap-4 max-w-2xl mx-auto mb-10">
            <div className="flex items-center justify-center gap-2 text-summit-text-secondary">
              <svg className="w-5 h-5 text-summit-success" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
              </svg>
              Free to Apply
            </div>
            <div className="flex items-center justify-center gap-2 text-summit-text-secondary">
              <svg className="w-5 h-5 text-summit-success" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
              </svg>
              10-15 min form
            </div>
            <div className="flex items-center justify-center gap-2 text-summit-text-secondary">
              <svg className="w-5 h-5 text-summit-success" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
              </svg>
              Rolling Review
            </div>
          </div>

          {/* CTA Button */}
          <a
            href="https://forms.google.com/your-form-id"
            target="_blank"
            rel="noopener noreferrer"
            className="summit-btn-primary text-lg inline-flex items-center gap-3 mb-6"
          >
            Apply Now
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 7l5 5m0 0l-5 5m5-5H6" />
            </svg>
          </a>

          {/* Deadline */}
          <p className="text-sm text-summit-text-muted">
            Application Deadline: <span className="text-summit-warning font-medium">March 31, 2026</span>
          </p>

          {/* Support */}
          <div className="mt-8 pt-8 border-t border-summit-tertiary/30">
            <p className="text-summit-text-muted text-sm mb-2">
              Have questions before applying?
            </p>
            <a
              href="mailto:summit@cmdai.dev"
              className="text-summit-accent-teal hover:text-summit-accent-blue transition-colors inline-flex items-center gap-2"
            >
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
              </svg>
              summit@cmdai.dev
            </a>
          </div>
        </div>
      </div>
    </section>
  );
};
