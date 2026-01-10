'use client';

import React from 'react';

const features = [
  {
    icon: (
      <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
      </svg>
    ),
    title: 'Safety-First Validation',
    description: '52+ pre-compiled patterns detect dangerous commands like rm -rf /, fork bombs, and privilege escalation attempts.',
    details: [
      'Multi-level risk assessment (Safe/Moderate/High/Critical)',
      'Blocks system destruction patterns automatically',
      'Configurable confirmation modes',
      '0% false positive rate on critical patterns',
    ],
  },
  {
    icon: (
      <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
      </svg>
    ),
    title: '100% Local Inference',
    description: 'All AI processing happens on your machine. No cloud APIs, no data exfiltration, no network dependencies.',
    details: [
      'Embedded Qwen2.5-Coder-1.5B model',
      'MLX optimization for Apple Silicon (<2s inference)',
      'CPU fallback for cross-platform support',
      'Works in air-gapped environments',
    ],
  },
  {
    icon: (
      <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
      </svg>
    ),
    title: 'Platform-Aware Generation',
    description: 'Automatically detects your OS, shell, and available tools to generate commands that work the first time.',
    details: [
      'macOS (Intel + Apple Silicon), Linux, Windows',
      'Handles GNU vs BSD command differences',
      'Shell-specific syntax (bash, zsh, fish, PowerShell)',
      '2-iteration refinement for accuracy',
    ],
  },
  {
    icon: (
      <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4m0 5c0 2.21-3.582 4-8 4s-8-1.79-8-4" />
      </svg>
    ),
    title: 'Multi-Backend Architecture',
    description: 'Flexible backend system supports embedded models, Ollama, vLLM, and future integrations.',
    details: [
      'Embedded MLX (default for Apple Silicon)',
      'Ollama integration with auto-fallback',
      'vLLM for enterprise deployments',
      'Extensible trait system for new backends',
    ],
  },
];

const workflowSteps = [
  {
    step: 1,
    title: 'Describe What You Need',
    description: 'Type a natural language description of what you want to accomplish.',
    example: '$ caro "find all log files modified in the last hour"',
  },
  {
    step: 2,
    title: 'AI Generates Command',
    description: 'Caro uses context-aware AI to generate a platform-specific command.',
    example: 'find /var/log -name "*.log" -mmin -60',
  },
  {
    step: 3,
    title: 'Safety Validation',
    description: 'Every command is checked against 52+ safety patterns before execution.',
    example: '✓ SAFE - No dangerous patterns detected',
  },
  {
    step: 4,
    title: 'Review & Execute',
    description: 'You always maintain control. Approve, modify, or reject before running.',
    example: 'Execute this command? (y/N)',
  },
];

export const ApplicationFAQ: React.FC = () => {
  return (
    <section id="solution" className="summit-section py-20 md:py-32">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        {/* Section Header */}
        <div className="text-center max-w-3xl mx-auto mb-16">
          <span className="summit-badge summit-badge-blue mb-4 inline-block">
            The Solution
          </span>
          <h2 className="text-3xl md:text-4xl lg:text-5xl font-bold text-summit-text-primary mb-6">
            How Caro Works
          </h2>
          <p className="text-lg text-summit-text-secondary">
            A 2-iteration agentic loop that combines context awareness with smart refinement,
            wrapped in comprehensive safety validation.
          </p>
        </div>

        {/* Workflow Steps */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-20">
          {workflowSteps.map((step) => (
            <div key={step.step} className="relative">
              <div className="summit-card h-full">
                <div className="w-10 h-10 rounded-full bg-gradient-to-br from-summit-accent-teal to-summit-accent-blue flex items-center justify-center text-white font-bold mb-4">
                  {step.step}
                </div>
                <h3 className="text-lg font-semibold text-summit-text-primary mb-2">
                  {step.title}
                </h3>
                <p className="text-sm text-summit-text-secondary mb-4">
                  {step.description}
                </p>
                <div className="bg-summit-primary/50 rounded-lg p-3 font-mono text-xs text-summit-accent-teal">
                  {step.example}
                </div>
              </div>
              {step.step < 4 && (
                <div className="hidden lg:block absolute top-1/2 -right-3 transform -translate-y-1/2 z-10">
                  <svg className="w-6 h-6 text-summit-tertiary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
                  </svg>
                </div>
              )}
            </div>
          ))}
        </div>

        {/* Core Features Grid */}
        <div className="text-center mb-12">
          <h3 className="text-2xl md:text-3xl font-bold text-summit-text-primary mb-4">
            Core Capabilities
          </h3>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
          {features.map((feature, index) => (
            <div key={index} className="summit-card">
              <div className="flex items-start gap-4">
                <div className="w-12 h-12 rounded-xl bg-summit-accent-teal/10 flex items-center justify-center text-summit-accent-teal flex-shrink-0">
                  {feature.icon}
                </div>
                <div className="flex-1">
                  <h4 className="text-lg font-semibold text-summit-text-primary mb-2">
                    {feature.title}
                  </h4>
                  <p className="text-summit-text-secondary mb-4">
                    {feature.description}
                  </p>
                  <ul className="space-y-2">
                    {feature.details.map((detail, i) => (
                      <li key={i} className="flex items-center gap-2 text-sm text-summit-text-muted">
                        <svg className="w-4 h-4 text-summit-success flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                        </svg>
                        {detail}
                      </li>
                    ))}
                  </ul>
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
};
