'use client';

import React from 'react';

const painPoints = [
  {
    icon: (
      <svg className="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
      </svg>
    ),
    title: 'Dangerous Commands at 3 AM',
    description: 'One wrong rm -rf command during an incident can cascade into catastrophe. Fatigue-induced errors during on-call shifts have no safety net.',
    persona: 'SRE / DevOps',
    color: 'teal',
  },
  {
    icon: (
      <svg className="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8.228 9c.549-1.165 2.03-2 3.772-2 2.21 0 4 1.343 4 3 0 1.4-1.278 2.575-3.006 2.907-.542.104-.994.54-.994 1.093m0 3h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
      </svg>
    ),
    title: 'Cloud AI Data Leakage',
    description: 'Every command sent to cloud AI tools exposes your infrastructure, credentials, and operational patterns to third parties.',
    persona: 'Security / CISO',
    color: 'purple',
  },
  {
    icon: (
      <svg className="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
      </svg>
    ),
    title: 'Platform Command Chaos',
    description: 'Commands that work on Linux fail on macOS. BSD vs GNU differences. Every environment switch means re-learning syntax.',
    persona: 'Full-Stack / Platform',
    color: 'blue',
  },
];

const targetCustomers = [
  {
    title: 'The 3AM Firefighter',
    role: 'SRE / DevOps / On-Call',
    quote: 'Your 2 AM safety net',
    needs: ['Incident response', 'Production emergencies', 'Zero-margin-for-error ops'],
  },
  {
    title: 'The Privacy Purist',
    role: 'Security / Compliance / CISO',
    quote: 'What happens on your machine, stays on your machine',
    needs: ['Air-gapped networks', 'Compliance audits', 'Data sovereignty'],
  },
  {
    title: 'The New Hire Navigator',
    role: 'Junior Dev / Bootcamp Graduate',
    quote: 'Learn the command line without the landmines',
    needs: ['Knowledge gaps', 'Fear of breaking things', 'Learning safely'],
  },
  {
    title: 'The Cross-Platform Warrior',
    role: 'Full-Stack / Platform Engineer',
    quote: 'One command language. Every platform.',
    needs: ['Multi-environment workflows', 'BSD vs GNU', 'Shell incompatibilities'],
  },
];

export const WhatWereLookingFor: React.FC = () => {
  return (
    <section id="problem" className="summit-section py-20 md:py-32 bg-summit-secondary/30">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        {/* Section Header */}
        <div className="text-center max-w-3xl mx-auto mb-16">
          <span className="summit-badge summit-badge-teal mb-4 inline-block">
            The Problem
          </span>
          <h2 className="text-3xl md:text-4xl lg:text-5xl font-bold text-summit-text-primary mb-6">
            The Terminal is Powerful—<br />
            <span className="summit-gradient-text">And Dangerous</span>
          </h2>
          <p className="text-lg text-summit-text-secondary">
            Every developer, DevOps engineer, and SRE has felt the terror of a command gone wrong.
            Cloud AI tools promise help, but at what cost to privacy and safety?
          </p>
        </div>

        {/* Pain Points Grid */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-8 mb-20">
          {painPoints.map((point, index) => (
            <div key={index} className="summit-card group">
              <div className={`w-16 h-16 rounded-xl bg-summit-accent-${point.color}/10 flex items-center justify-center text-summit-accent-${point.color} mb-6 group-hover:scale-110 transition-transform`}>
                {point.icon}
              </div>
              <div className="summit-badge summit-badge-purple text-xs mb-4">
                {point.persona}
              </div>
              <h3 className="text-xl font-semibold text-summit-text-primary mb-3">
                {point.title}
              </h3>
              <p className="text-summit-text-secondary leading-relaxed">
                {point.description}
              </p>
            </div>
          ))}
        </div>

        {/* Target Customers */}
        <div className="text-center mb-12">
          <h3 className="text-2xl md:text-3xl font-bold text-summit-text-primary mb-4">
            Built For Those Who Can&apos;t Afford Mistakes
          </h3>
          <p className="text-summit-text-secondary max-w-2xl mx-auto">
            Caro is designed for professionals where a single wrong command can mean
            downtime, data loss, or security breaches.
          </p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          {targetCustomers.map((customer, index) => (
            <div key={index} className="bg-summit-tertiary/30 rounded-xl p-6 border border-summit-tertiary/50 hover:border-summit-accent-teal/30 transition-colors">
              <h4 className="text-lg font-semibold text-summit-text-primary mb-1">
                {customer.title}
              </h4>
              <p className="text-sm text-summit-accent-teal mb-3">{customer.role}</p>
              <p className="text-sm text-summit-text-secondary italic mb-4">
                &ldquo;{customer.quote}&rdquo;
              </p>
              <ul className="space-y-2">
                {customer.needs.map((need, i) => (
                  <li key={i} className="flex items-center gap-2 text-xs text-summit-text-muted">
                    <svg className="w-3 h-3 text-summit-accent-teal flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                    </svg>
                    {need}
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
};
