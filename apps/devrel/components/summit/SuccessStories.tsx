'use client';

import React from 'react';

const achievements = [
  {
    metric: '52+',
    label: 'Safety Patterns',
    description: 'Pre-compiled dangerous command patterns with 0% false positive rate',
    icon: (
      <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
      </svg>
    ),
  },
  {
    metric: '6',
    label: 'Backend Integrations',
    description: 'MLX, CPU, Ollama, vLLM, OpenAI, and Anthropic backends',
    icon: (
      <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4m0 5c0 2.21-3.582 4-8 4s-8-1.79-8-4" />
      </svg>
    ),
  },
  {
    metric: '3',
    label: 'Platforms',
    description: 'macOS (Intel + Apple Silicon), Linux, and Windows support',
    icon: (
      <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
      </svg>
    ),
  },
  {
    metric: '<2s',
    label: 'Inference Time',
    description: 'Sub-2-second command generation on Apple Silicon',
    icon: (
      <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
      </svg>
    ),
  },
];

const milestones = [
  {
    title: 'Core CLI Complete',
    status: 'completed',
    items: ['Natural language parsing', 'Command generation', 'Safety validation', 'Multi-backend support'],
  },
  {
    title: 'Safety System',
    status: 'completed',
    items: ['52+ danger patterns', 'Risk level classification', 'Confirmation workflows', 'Fork bomb detection'],
  },
  {
    title: 'Platform Support',
    status: 'completed',
    items: ['macOS (Intel + ARM)', 'Linux distributions', 'Windows support', 'Platform detection'],
  },
  {
    title: 'Open Source Launch',
    status: 'in_progress',
    items: ['AGPL-3.0 license', 'Community contribution guide', 'GitHub repository', 'Documentation'],
  },
];

const integrations = [
  { name: 'Claude Desktop', status: 'available', description: 'Official MCP skill' },
  { name: 'VS Code', status: 'available', description: 'Extension integration' },
  { name: 'Terminal.app', status: 'available', description: 'Native macOS' },
  { name: 'iTerm2', status: 'available', description: 'Native macOS' },
  { name: 'Warp', status: 'planned', description: 'Modern terminal' },
  { name: 'Alacritty', status: 'available', description: 'Cross-platform' },
];

export const SuccessStories: React.FC = () => {
  return (
    <section id="traction" className="summit-section py-20 md:py-32">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        {/* Section Header */}
        <div className="text-center max-w-3xl mx-auto mb-16">
          <span className="summit-badge summit-badge-orange mb-4 inline-block">
            Traction
          </span>
          <h2 className="text-3xl md:text-4xl lg:text-5xl font-bold text-summit-text-primary mb-6">
            What We&apos;ve Built
          </h2>
          <p className="text-lg text-summit-text-secondary">
            A production-ready CLI tool with comprehensive safety validation,
            multi-platform support, and extensible architecture.
          </p>
        </div>

        {/* Key Metrics */}
        <div className="grid grid-cols-2 md:grid-cols-4 gap-6 mb-20">
          {achievements.map((achievement, index) => (
            <div key={index} className="summit-card text-center">
              <div className="w-12 h-12 mx-auto rounded-xl bg-summit-accent-orange/10 flex items-center justify-center text-summit-accent-orange mb-4">
                {achievement.icon}
              </div>
              <div className="text-3xl font-bold summit-gradient-text mb-2">
                {achievement.metric}
              </div>
              <div className="text-lg font-semibold text-summit-text-primary mb-2">
                {achievement.label}
              </div>
              <p className="text-sm text-summit-text-muted">
                {achievement.description}
              </p>
            </div>
          ))}
        </div>

        {/* Development Milestones */}
        <div className="mb-20">
          <div className="text-center mb-12">
            <h3 className="text-2xl md:text-3xl font-bold text-summit-text-primary mb-4">
              Development Progress
            </h3>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
            {milestones.map((milestone, index) => (
              <div key={index} className="summit-card">
                <div className="flex items-center gap-3 mb-4">
                  <div className={`w-3 h-3 rounded-full ${
                    milestone.status === 'completed' ? 'bg-summit-success' : 'bg-summit-warning animate-pulse'
                  }`} />
                  <span className={`text-xs font-medium uppercase tracking-wider ${
                    milestone.status === 'completed' ? 'text-summit-success' : 'text-summit-warning'
                  }`}>
                    {milestone.status === 'completed' ? 'Complete' : 'In Progress'}
                  </span>
                </div>
                <h4 className="text-lg font-semibold text-summit-text-primary mb-4">
                  {milestone.title}
                </h4>
                <ul className="space-y-2">
                  {milestone.items.map((item, i) => (
                    <li key={i} className="flex items-center gap-2 text-sm text-summit-text-secondary">
                      <svg className="w-4 h-4 text-summit-success flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                      </svg>
                      {item}
                    </li>
                  ))}
                </ul>
              </div>
            ))}
          </div>
        </div>

        {/* Integration Partners */}
        <div>
          <div className="text-center mb-12">
            <h3 className="text-2xl md:text-3xl font-bold text-summit-text-primary mb-4">
              Integration Ecosystem
            </h3>
            <p className="text-summit-text-secondary">
              Works with your existing tools and workflows
            </p>
          </div>

          <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-4">
            {integrations.map((integration) => (
              <div
                key={integration.name}
                className="bg-summit-tertiary/30 rounded-xl p-4 text-center border border-summit-tertiary/50 hover:border-summit-accent-teal/30 transition-colors"
              >
                <div className="text-sm font-medium text-summit-text-primary mb-1">
                  {integration.name}
                </div>
                <div className={`text-xs ${
                  integration.status === 'available' ? 'text-summit-success' : 'text-summit-text-muted'
                }`}>
                  {integration.status === 'available' ? 'Available' : 'Planned'}
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
};
