'use client';

import React from 'react';

const visionPillars = [
  {
    title: 'Layer 2 Agent Architecture',
    description: 'Caro is designed as a specialized sub-agent that works alongside AI orchestrators like Claude and ChatGPT.',
    details: [
      'Big agents handle high-level orchestration',
      'Caro handles specialized terminal operations',
      'Each layer maintains its own safety validation',
      'Modular expertise that compounds over time',
    ],
    icon: (
      <svg className="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
      </svg>
    ),
  },
  {
    title: 'Skills Marketplace',
    description: 'Domain-specific skills that extend Caro\'s capabilities for specialized use cases.',
    details: [
      'SRE & incident response skills',
      'Database administration patterns',
      'Cloud infrastructure management',
      'Security auditing workflows',
    ],
    icon: (
      <svg className="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
      </svg>
    ),
  },
  {
    title: 'Community-Driven Safety',
    description: 'Every safety pattern contributed helps someone else work more confidently with their command line.',
    details: [
      'Crowdsourced danger patterns',
      'Community-reviewed best practices',
      'Collective terminal expertise',
      'Open source knowledge base',
    ],
    icon: (
      <svg className="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
      </svg>
    ),
  },
];

const mcpIntegration = {
  title: 'Model Context Protocol (MCP)',
  description: 'First-class integration with AI assistants through the Model Context Protocol.',
  benefits: [
    {
      title: 'Claude Desktop',
      description: 'Official Caro skill for Claude Desktop integration',
    },
    {
      title: 'VS Code Copilot',
      description: 'Enhanced terminal commands in your IDE',
    },
    {
      title: 'Custom Agents',
      description: 'Build your own AI workflows with Caro as a tool',
    },
  ],
};

const roadmapItems = [
  { phase: 'Now', items: ['Core CLI', 'Safety validation', 'Multi-backend', 'Cross-platform'] },
  { phase: 'Next', items: ['MCP integration', 'Skills framework', 'Web Hub beta', 'Telemetry dashboard'] },
  { phase: 'Future', items: ['Skills marketplace', 'Enterprise features', 'Professional guilds', 'Mobile companion'] },
];

export const PreparationChecklist: React.FC = () => {
  return (
    <section id="vision" className="summit-section py-20 md:py-32 bg-summit-secondary/30">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        {/* Section Header */}
        <div className="text-center max-w-3xl mx-auto mb-16">
          <span className="summit-badge summit-badge-teal mb-4 inline-block">
            The Vision
          </span>
          <h2 className="text-3xl md:text-4xl lg:text-5xl font-bold text-summit-text-primary mb-6">
            More Than a CLI Tool
          </h2>
          <p className="text-lg text-summit-text-secondary">
            Caro is a living system with skills, tools, rules, and most importantly,
            a community who cares about making terminals safer for everyone.
          </p>
        </div>

        {/* Vision Pillars */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-8 mb-20">
          {visionPillars.map((pillar, index) => (
            <div key={index} className="summit-card">
              <div className="w-16 h-16 rounded-xl bg-summit-accent-teal/10 flex items-center justify-center text-summit-accent-teal mb-6">
                {pillar.icon}
              </div>
              <h3 className="text-xl font-semibold text-summit-text-primary mb-3">
                {pillar.title}
              </h3>
              <p className="text-summit-text-secondary mb-4">
                {pillar.description}
              </p>
              <ul className="space-y-2">
                {pillar.details.map((detail, i) => (
                  <li key={i} className="flex items-center gap-2 text-sm text-summit-text-muted">
                    <svg className="w-4 h-4 text-summit-accent-teal flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
                    </svg>
                    {detail}
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>

        {/* MCP Integration */}
        <div className="summit-card mb-20">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-12">
            <div>
              <span className="summit-badge summit-badge-purple mb-4 inline-block">
                Integration
              </span>
              <h3 className="text-2xl font-bold text-summit-text-primary mb-4">
                {mcpIntegration.title}
              </h3>
              <p className="text-summit-text-secondary mb-6">
                {mcpIntegration.description}
              </p>
              <div className="bg-summit-primary/50 rounded-lg p-4 font-mono text-sm">
                <div className="text-summit-text-muted mb-2"># Example: Using Caro as a Claude skill</div>
                <div className="text-summit-accent-teal">@caro &quot;find all Docker containers using more than 1GB memory&quot;</div>
              </div>
            </div>
            <div className="space-y-4">
              {mcpIntegration.benefits.map((benefit, index) => (
                <div key={index} className="bg-summit-primary/30 rounded-xl p-4 border border-summit-tertiary/30">
                  <h4 className="text-lg font-semibold text-summit-text-primary mb-1">
                    {benefit.title}
                  </h4>
                  <p className="text-sm text-summit-text-secondary">
                    {benefit.description}
                  </p>
                </div>
              ))}
            </div>
          </div>
        </div>

        {/* Roadmap */}
        <div>
          <div className="text-center mb-12">
            <h3 className="text-2xl md:text-3xl font-bold text-summit-text-primary mb-4">
              Product Roadmap
            </h3>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            {roadmapItems.map((phase, index) => (
              <div
                key={index}
                className={`rounded-xl p-6 border-2 ${
                  index === 0 ? 'border-summit-accent-teal bg-summit-accent-teal/5' :
                  index === 1 ? 'border-summit-accent-blue/50 bg-summit-accent-blue/5' :
                  'border-summit-tertiary/50 bg-summit-tertiary/10'
                }`}
              >
                <div className={`text-lg font-bold mb-4 ${
                  index === 0 ? 'text-summit-accent-teal' :
                  index === 1 ? 'text-summit-accent-blue' :
                  'text-summit-text-muted'
                }`}>
                  {phase.phase}
                </div>
                <ul className="space-y-2">
                  {phase.items.map((item, i) => (
                    <li key={i} className="flex items-center gap-2 text-sm text-summit-text-secondary">
                      <svg className={`w-4 h-4 flex-shrink-0 ${
                        index === 0 ? 'text-summit-success' : 'text-summit-text-muted'
                      }`} fill="none" stroke="currentColor" viewBox="0 0 24 24">
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
      </div>
    </section>
  );
};
