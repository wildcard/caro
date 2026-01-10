'use client';

import React from 'react';

const installMethods = [
  {
    name: 'Homebrew',
    platform: 'macOS',
    command: 'brew install caro',
    icon: '🍺',
  },
  {
    name: 'Cargo',
    platform: 'All platforms',
    command: 'cargo install caro',
    icon: '📦',
  },
  {
    name: 'Download',
    platform: 'Binary',
    command: 'curl -fsSL https://caro.sh/install.sh | sh',
    icon: '⬇️',
  },
];

export const ApplicationCTA: React.FC = () => {
  return (
    <section id="get-started" className="summit-section py-20 md:py-32 relative overflow-hidden">
      {/* Background decorations */}
      <div className="summit-glow-teal top-0 left-1/4" />
      <div className="summit-glow-purple bottom-0 right-1/4" />

      <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 relative z-10">
        <div className="text-center">
          {/* Badge */}
          <span className="summit-badge summit-badge-teal mb-6 inline-block">
            Open Source
          </span>

          {/* Headline */}
          <h2 className="text-3xl md:text-4xl lg:text-5xl font-bold text-summit-text-primary mb-6">
            Stop Googling.<br />
            <span className="summit-gradient-text">Start Shipping.</span>
          </h2>

          {/* Subheadline */}
          <p className="text-lg text-summit-text-secondary max-w-2xl mx-auto mb-12">
            Install Caro in seconds and transform how you work with the terminal.
            100% local, 100% private, 100% safe.
          </p>

          {/* Install Methods */}
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-12">
            {installMethods.map((method) => (
              <div
                key={method.name}
                className="summit-card text-left"
              >
                <div className="flex items-center gap-3 mb-3">
                  <span className="text-2xl">{method.icon}</span>
                  <div>
                    <div className="font-semibold text-summit-text-primary">{method.name}</div>
                    <div className="text-xs text-summit-text-muted">{method.platform}</div>
                  </div>
                </div>
                <div className="bg-summit-primary rounded-lg p-3 font-mono text-sm text-summit-accent-teal overflow-x-auto">
                  {method.command}
                </div>
              </div>
            ))}
          </div>

          {/* Primary CTAs */}
          <div className="flex flex-col sm:flex-row justify-center gap-4 mb-12">
            <a
              href="https://github.com/anthropics/caro"
              target="_blank"
              rel="noopener noreferrer"
              className="summit-btn-primary text-lg inline-flex items-center justify-center gap-2"
            >
              <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
                <path fillRule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z" clipRule="evenodd" />
              </svg>
              View on GitHub
            </a>
            <a
              href="https://caro.sh/docs"
              className="summit-btn-secondary text-lg inline-flex items-center justify-center gap-2"
            >
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253" />
              </svg>
              Read Documentation
            </a>
          </div>

          {/* Quick Stats */}
          <div className="flex flex-wrap justify-center gap-8 text-sm text-summit-text-muted">
            <div className="flex items-center gap-2">
              <svg className="w-4 h-4 text-summit-success" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
              </svg>
              AGPL-3.0 License
            </div>
            <div className="flex items-center gap-2">
              <svg className="w-4 h-4 text-summit-success" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
              </svg>
              No cloud dependencies
            </div>
            <div className="flex items-center gap-2">
              <svg className="w-4 h-4 text-summit-success" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
              </svg>
              Community-driven
            </div>
          </div>

          {/* Support */}
          <div className="mt-12 pt-8 border-t border-summit-tertiary/30">
            <p className="text-summit-text-muted mb-4">
              Questions? Feedback? We&apos;d love to hear from you.
            </p>
            <div className="flex flex-wrap justify-center gap-4">
              <a
                href="https://github.com/anthropics/caro/issues"
                className="text-summit-accent-teal hover:underline"
              >
                Open an issue
              </a>
              <span className="text-summit-tertiary">•</span>
              <a
                href="https://github.com/anthropics/caro/discussions"
                className="text-summit-accent-teal hover:underline"
              >
                Join discussions
              </a>
              <span className="text-summit-tertiary">•</span>
              <a
                href="mailto:hello@caro.sh"
                className="text-summit-accent-teal hover:underline"
              >
                Contact us
              </a>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
};
