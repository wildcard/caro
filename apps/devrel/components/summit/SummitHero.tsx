'use client';

import React from 'react';

const stats = [
  { value: '52+', label: 'Safety Patterns' },
  { value: '<2s', label: 'Inference Time' },
  { value: '100%', label: 'Local & Private' },
  { value: '6', label: 'Backends Supported' },
];

export const SummitHero: React.FC = () => {
  return (
    <section className="relative min-h-screen flex items-center pt-16 overflow-hidden">
      {/* Background decorations */}
      <div className="summit-glow-teal -top-40 -right-40" />
      <div className="summit-glow-purple top-1/3 -left-60" />

      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-16 md:py-24 relative z-10">
        <div className="text-center max-w-4xl mx-auto">
          {/* Badge */}
          <div className="inline-flex items-center gap-2 summit-badge summit-badge-teal mb-8">
            <span className="w-2 h-2 bg-summit-accent-teal rounded-full animate-pulse" />
            Seattle AI Startup Summit 2026 - Demo Showcase
          </div>

          {/* Main Headline */}
          <h1 className="text-4xl sm:text-5xl md:text-6xl lg:text-7xl font-bold mb-6 leading-tight">
            <span className="text-summit-text-primary">Natural Language to</span>
            <br />
            <span className="summit-gradient-text">Safe Shell Commands</span>
          </h1>

          {/* Subheadline */}
          <p className="text-lg md:text-xl text-summit-text-secondary max-w-2xl mx-auto mb-8 leading-relaxed">
            Caro is an AI-powered CLI tool that converts plain English into validated,
            platform-aware shell commands. Built with Rust. Runs 100% locally.
            Never sends your data to the cloud.
          </p>

          {/* Value Props */}
          <div className="flex flex-wrap justify-center gap-4 mb-10">
            <span className="summit-badge summit-badge-blue">
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
              </svg>
              Safety-First AI
            </span>
            <span className="summit-badge summit-badge-purple">
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
              </svg>
              100% Local & Private
            </span>
            <span className="summit-badge summit-badge-orange">
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
              </svg>
              Rust-Powered Performance
            </span>
          </div>

          {/* Terminal Demo Preview */}
          <div className="max-w-2xl mx-auto mb-12 text-left">
            <div className="bg-summit-primary border border-summit-tertiary rounded-xl overflow-hidden shadow-2xl">
              <div className="flex items-center gap-2 px-4 py-3 bg-summit-secondary border-b border-summit-tertiary">
                <div className="w-3 h-3 rounded-full bg-red-500" />
                <div className="w-3 h-3 rounded-full bg-yellow-500" />
                <div className="w-3 h-3 rounded-full bg-green-500" />
                <span className="ml-2 text-sm text-summit-text-muted font-mono">Terminal</span>
              </div>
              <div className="p-6 font-mono text-sm">
                <div className="text-summit-text-muted mb-2">$ caro &quot;list all PDF files larger than 10MB&quot;</div>
                <div className="text-summit-text-secondary mb-4">
                  <span className="text-summit-accent-teal">Generated command:</span>
                  <br />
                  <span className="text-summit-text-primary ml-2">find ~/Downloads -name &quot;*.pdf&quot; -size +10M -ls</span>
                </div>
                <div className="flex items-center gap-2">
                  <span className="inline-flex items-center gap-1 px-2 py-1 rounded bg-green-500/20 text-green-400 text-xs">
                    <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                    </svg>
                    SAFE
                  </span>
                  <span className="text-summit-text-muted">Execute this command? (y/N)</span>
                  <span className="text-summit-accent-teal animate-pulse">▌</span>
                </div>
              </div>
            </div>
          </div>

          {/* CTAs */}
          <div className="flex flex-col sm:flex-row justify-center gap-4 mb-16">
            <a href="#problem" className="summit-btn-primary text-lg">
              See the Problem We Solve
            </a>
            <a href="https://github.com/anthropics/caro" target="_blank" rel="noopener noreferrer" className="summit-btn-secondary text-lg">
              View on GitHub
            </a>
          </div>

          {/* Stats */}
          <div className="grid grid-cols-2 md:grid-cols-4 gap-8 max-w-3xl mx-auto">
            {stats.map((stat) => (
              <div key={stat.label} className="summit-stat">
                <div className="summit-stat-value">{stat.value}</div>
                <div className="summit-stat-label">{stat.label}</div>
              </div>
            ))}
          </div>
        </div>

        {/* Scroll indicator */}
        <div className="absolute bottom-8 left-1/2 transform -translate-x-1/2 animate-bounce">
          <svg
            className="w-6 h-6 text-summit-text-muted"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M19 14l-7 7m0 0l-7-7m7 7V3"
            />
          </svg>
        </div>
      </div>
    </section>
  );
};
