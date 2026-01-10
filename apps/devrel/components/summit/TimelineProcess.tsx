'use client';

import React from 'react';

const advantages = [
  {
    title: 'vs Cloud AI Tools',
    competitor: 'ChatGPT, Claude, GitHub Copilot',
    caroAdvantage: [
      '100% local - no data leaves your machine',
      'Works in air-gapped networks',
      'No API costs or rate limits',
      'No internet dependency',
    ],
    icon: (
      <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 15a4 4 0 004 4h9a5 5 0 10-.1-9.999 5.002 5.002 0 10-9.78 2.096A4.001 4.001 0 003 15z" />
      </svg>
    ),
  },
  {
    title: 'vs Shell Wrappers',
    competitor: 'thefuck, tldr, explainshell',
    caroAdvantage: [
      'AI-powered natural language understanding',
      'Generates commands, not just corrections',
      'Platform-aware generation',
      'Comprehensive safety validation',
    ],
    icon: (
      <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
      </svg>
    ),
  },
  {
    title: 'vs Other AI CLIs',
    competitor: 'aichat, shell-gpt, ask-cli',
    caroAdvantage: [
      '52+ safety patterns (most have zero)',
      'Embedded model - no setup required',
      'Single binary distribution',
      'Multi-backend architecture',
    ],
    icon: (
      <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z" />
      </svg>
    ),
  },
];

const techStack = [
  { name: 'Rust', description: 'Systems language for performance & safety', color: 'orange' },
  { name: 'MLX', description: 'Apple Silicon optimization', color: 'blue' },
  { name: 'GGUF', description: 'Quantized model format', color: 'purple' },
  { name: 'AGPL-3.0', description: 'Open source license', color: 'teal' },
];

const safetyLevels = [
  { level: 'Safe', color: 'green', description: 'Normal operations, no confirmation needed', example: 'ls, pwd, echo' },
  { level: 'Moderate', color: 'yellow', description: 'Requires confirmation in strict mode', example: 'cp, mv, chmod' },
  { level: 'High', color: 'orange', description: 'Requires confirmation in moderate mode', example: 'rm, kill, sudo' },
  { level: 'Critical', color: 'red', description: 'Blocked in strict mode', example: 'rm -rf /, :(){ :|:& };:' },
];

export const TimelineProcess: React.FC = () => {
  return (
    <section id="advantages" className="summit-section py-20 md:py-32 bg-summit-secondary/30">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        {/* Section Header */}
        <div className="text-center max-w-3xl mx-auto mb-16">
          <span className="summit-badge summit-badge-purple mb-4 inline-block">
            Competitive Edge
          </span>
          <h2 className="text-3xl md:text-4xl lg:text-5xl font-bold text-summit-text-primary mb-6">
            Why Caro Wins
          </h2>
          <p className="text-lg text-summit-text-secondary">
            The only AI CLI tool that combines local inference, comprehensive safety,
            and multi-platform support in a single binary.
          </p>
        </div>

        {/* Competitive Comparison */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-8 mb-20">
          {advantages.map((adv, index) => (
            <div key={index} className="summit-card">
              <div className="w-12 h-12 rounded-xl bg-summit-accent-purple/10 flex items-center justify-center text-summit-accent-purple mb-4">
                {adv.icon}
              </div>
              <h3 className="text-lg font-semibold text-summit-text-primary mb-2">
                {adv.title}
              </h3>
              <p className="text-sm text-summit-text-muted mb-4">
                Compared to: {adv.competitor}
              </p>
              <ul className="space-y-2">
                {adv.caroAdvantage.map((point, i) => (
                  <li key={i} className="flex items-start gap-2 text-sm text-summit-text-secondary">
                    <svg className="w-4 h-4 text-summit-success flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                    </svg>
                    {point}
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>

        {/* Safety Levels Visualization */}
        <div className="mb-20">
          <div className="text-center mb-12">
            <h3 className="text-2xl md:text-3xl font-bold text-summit-text-primary mb-4">
              Multi-Level Safety System
            </h3>
            <p className="text-summit-text-secondary max-w-2xl mx-auto">
              Every command is classified and handled according to its risk level.
            </p>
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
            {safetyLevels.map((level) => (
              <div
                key={level.level}
                className={`p-6 rounded-xl border-2 ${
                  level.color === 'green' ? 'border-green-500/30 bg-green-500/5' :
                  level.color === 'yellow' ? 'border-yellow-500/30 bg-yellow-500/5' :
                  level.color === 'orange' ? 'border-orange-500/30 bg-orange-500/5' :
                  'border-red-500/30 bg-red-500/5'
                }`}
              >
                <div className={`text-lg font-bold mb-2 ${
                  level.color === 'green' ? 'text-green-400' :
                  level.color === 'yellow' ? 'text-yellow-400' :
                  level.color === 'orange' ? 'text-orange-400' :
                  'text-red-400'
                }`}>
                  {level.level}
                </div>
                <p className="text-sm text-summit-text-secondary mb-3">
                  {level.description}
                </p>
                <div className="font-mono text-xs text-summit-text-muted">
                  {level.example}
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Tech Stack */}
        <div>
          <div className="text-center mb-12">
            <h3 className="text-2xl md:text-3xl font-bold text-summit-text-primary mb-4">
              Built With
            </h3>
          </div>

          <div className="flex flex-wrap justify-center gap-4">
            {techStack.map((tech) => (
              <div
                key={tech.name}
                className={`summit-badge summit-badge-${tech.color} text-base px-6 py-3`}
              >
                <span className="font-semibold">{tech.name}</span>
                <span className="mx-2">•</span>
                <span className="opacity-80">{tech.description}</span>
              </div>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
};
