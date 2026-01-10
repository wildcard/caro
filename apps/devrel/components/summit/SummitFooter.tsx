'use client';

import React from 'react';

const footerLinks = [
  {
    title: 'Event',
    links: [
      { label: 'About', href: '#criteria' },
      { label: 'Timeline', href: '#timeline' },
      { label: 'FAQ', href: '#faq' },
      { label: 'Apply', href: '#apply' },
    ],
  },
  {
    title: 'Resources',
    links: [
      { label: 'Success Stories', href: '#success-stories' },
      { label: 'Checklist', href: '#checklist' },
      { label: 'Past Events', href: '#' },
      { label: 'Media Kit', href: '#' },
    ],
  },
  {
    title: 'Connect',
    links: [
      { label: 'Twitter', href: 'https://twitter.com/seattleaisummit' },
      { label: 'LinkedIn', href: 'https://linkedin.com/company/seattleaisummit' },
      { label: 'Contact', href: 'mailto:summit@cmdai.dev' },
      { label: 'Newsletter', href: '#' },
    ],
  },
];

export const SummitFooter: React.FC = () => {
  const currentYear = new Date().getFullYear();

  return (
    <footer className="bg-summit-primary border-t border-summit-tertiary/30">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-16">
        {/* Main Footer Content */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-12 mb-12">
          {/* Brand Column */}
          <div className="lg:col-span-1">
            <div className="flex items-center gap-3 mb-4">
              <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-summit-accent-teal to-summit-accent-blue flex items-center justify-center">
                <span className="text-white font-bold">AI</span>
              </div>
              <div>
                <div className="text-summit-text-primary font-semibold">
                  Seattle AI Summit
                </div>
                <div className="text-xs text-summit-text-muted">2026</div>
              </div>
            </div>
            <p className="text-sm text-summit-text-secondary leading-relaxed mb-4">
              The premier event connecting AI startups with Fortune 500 leaders,
              investors, and media in the Pacific Northwest.
            </p>
            <div className="text-sm text-summit-text-muted">
              <div className="flex items-center gap-2 mb-1">
                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z" />
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 11a3 3 0 11-6 0 3 3 0 016 0z" />
                </svg>
                Seattle, WA
              </div>
              <div className="flex items-center gap-2">
                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                </svg>
                June 15-16, 2026
              </div>
            </div>
          </div>

          {/* Link Columns */}
          {footerLinks.map((column) => (
            <div key={column.title}>
              <h4 className="text-summit-text-primary font-semibold mb-4">
                {column.title}
              </h4>
              <ul className="space-y-3">
                {column.links.map((link) => (
                  <li key={link.label}>
                    <a
                      href={link.href}
                      className="text-sm text-summit-text-secondary hover:text-summit-accent-teal transition-colors"
                      target={link.href.startsWith('http') ? '_blank' : undefined}
                      rel={link.href.startsWith('http') ? 'noopener noreferrer' : undefined}
                    >
                      {link.label}
                    </a>
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>

        {/* Divider */}
        <div className="h-px bg-summit-tertiary/30 mb-8" />

        {/* Bottom Bar */}
        <div className="flex flex-col md:flex-row items-center justify-between gap-4">
          <div className="text-sm text-summit-text-muted">
            &copy; {currentYear} Seattle AI Startup Summit. All rights reserved.
          </div>
          <div className="flex items-center gap-6">
            <a href="#" className="text-sm text-summit-text-secondary hover:text-summit-accent-teal transition-colors">
              Privacy Policy
            </a>
            <a href="#" className="text-sm text-summit-text-secondary hover:text-summit-accent-teal transition-colors">
              Terms of Service
            </a>
            <a href="#" className="text-sm text-summit-text-secondary hover:text-summit-accent-teal transition-colors">
              Code of Conduct
            </a>
          </div>
        </div>
      </div>
    </footer>
  );
};
