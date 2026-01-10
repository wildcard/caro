'use client';

import React from 'react';

const footerLinks = [
  {
    title: 'Product',
    links: [
      { label: 'Features', href: '#solution' },
      { label: 'Safety', href: '#advantages' },
      { label: 'Roadmap', href: '#vision' },
      { label: 'Changelog', href: 'https://github.com/anthropics/caro/releases' },
    ],
  },
  {
    title: 'Resources',
    links: [
      { label: 'Documentation', href: 'https://caro.sh/docs' },
      { label: 'Getting Started', href: '#get-started' },
      { label: 'API Reference', href: 'https://caro.sh/docs/api' },
      { label: 'Examples', href: 'https://github.com/anthropics/caro/tree/main/examples' },
    ],
  },
  {
    title: 'Community',
    links: [
      { label: 'GitHub', href: 'https://github.com/anthropics/caro' },
      { label: 'Discussions', href: 'https://github.com/anthropics/caro/discussions' },
      { label: 'Twitter', href: 'https://twitter.com/caro_cli' },
      { label: 'Discord', href: 'https://discord.gg/caro' },
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
                <span className="text-white font-bold">C</span>
              </div>
              <div>
                <div className="text-summit-text-primary font-semibold">
                  Caro
                </div>
                <div className="text-xs text-summit-text-muted">Your loyal shell companion</div>
              </div>
            </div>
            <p className="text-sm text-summit-text-secondary leading-relaxed mb-4">
              Transform natural language into safe, POSIX-compliant shell commands
              using local AI. Built with Rust for safety and performance.
            </p>
            <div className="text-sm text-summit-text-muted">
              <div className="flex items-center gap-2 mb-1">
                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
                </svg>
                52+ safety patterns
              </div>
              <div className="flex items-center gap-2">
                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                </svg>
                100% local inference
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
            &copy; {currentYear} Caro. AGPL-3.0 License. Made with Rust.
          </div>
          <div className="flex items-center gap-6">
            <a href="https://github.com/anthropics/caro/blob/main/LICENSE" className="text-sm text-summit-text-secondary hover:text-summit-accent-teal transition-colors">
              License
            </a>
            <a href="https://github.com/anthropics/caro/blob/main/CONTRIBUTING.md" className="text-sm text-summit-text-secondary hover:text-summit-accent-teal transition-colors">
              Contributing
            </a>
            <a href="https://github.com/anthropics/caro/blob/main/CODE_OF_CONDUCT.md" className="text-sm text-summit-text-secondary hover:text-summit-accent-teal transition-colors">
              Code of Conduct
            </a>
          </div>
        </div>
      </div>
    </footer>
  );
};
