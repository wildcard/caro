import React from 'react';
import { PixelButton } from './PixelButton';
import { PixelCard } from './PixelCard';

const contributionAreas = [
  {
    icon: '🔌',
    title: 'Backend Implementations',
    description: 'Add support for new LLM backends and inference engines',
  },
  {
    icon: '🛡️',
    title: 'Safety Patterns',
    description: 'Define and implement new dangerous command detection patterns',
  },
  {
    icon: '🧪',
    title: 'Test Coverage',
    description: 'Expand test suite with edge cases and integration tests',
  },
  {
    icon: '📚',
    title: 'Documentation',
    description: 'Improve docs, add tutorials, and create usage examples',
  },
  {
    icon: '🎨',
    title: 'UI/UX',
    description: 'Enhance terminal output, colors, and user experience',
  },
  {
    icon: '⚡',
    title: 'Performance',
    description: 'Optimize startup time, memory usage, and inference speed',
  },
];

export const Contributors: React.FC = () => {
  return (
    <section id="contribute" className="py-20 px-4 bg-pixel-bg-secondary relative overflow-hidden">
      <div className="container mx-auto">
        {/* Section Header */}
        <div className="text-center mb-16">
          <h2 className="pixel-text text-[20px] md:text-[28px] text-neon-pink mb-4">
            Join the Community
          </h2>
          <p className="font-mono text-terminal-green max-w-2xl mx-auto text-lg">
            We&apos;re building the future of safe, AI-powered shell commands.
            <br />
            Help us make cmdai better for everyone.
          </p>
        </div>

        {/* Why Contribute */}
        <div className="max-w-4xl mx-auto mb-12">
          <div className="terminal-window">
            <div className="flex items-center justify-between bg-pixel-bg-tertiary px-4 py-2 border-b-4 border-terminal-green">
              <div className="flex gap-2">
                <div className="w-3 h-3 bg-terminal-red"></div>
                <div className="w-3 h-3 bg-terminal-amber"></div>
                <div className="w-3 h-3 bg-terminal-green"></div>
              </div>
              <span className="pixel-text text-[8px] text-terminal-green">Why Contribute?</span>
              <div className="w-12"></div>
            </div>
            <div className="p-6 font-mono text-sm space-y-2">
              <div className="text-neon-green">✓ Shape the future of AI-powered CLI tools</div>
              <div className="text-neon-blue">✓ Learn Rust, LLMs, and systems programming</div>
              <div className="text-neon-pink">✓ Join a passionate open-source community</div>
              <div className="text-neon-purple">✓ Build something that developers love</div>
            </div>
          </div>
        </div>

        {/* Contribution Areas */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 max-w-6xl mx-auto mb-12">
          {contributionAreas.map((area, index) => (
            <PixelCard key={index} variant="neon">
              <div className="text-3xl mb-3">{area.icon}</div>
              <h3 className="pixel-text text-[10px] text-neon-blue mb-2">
                {area.title}
              </h3>
              <p className="text-sm font-mono text-gray-300">
                {area.description}
              </p>
            </PixelCard>
          ))}
        </div>

        {/* Getting Started */}
        <div className="max-w-3xl mx-auto text-center space-y-8">
          <div className="space-y-4">
            <h3 className="pixel-text text-[16px] text-neon-green">
              Ready to Contribute?
            </h3>
            <p className="font-mono text-gray-300">
              Fork the repo, pick an issue, and submit a PR. We&apos;ll guide you through the process!
            </p>
          </div>

          <div className="flex flex-wrap gap-4 justify-center">
            <PixelButton
              variant="primary"
              size="lg"
              href="https://github.com/wildcard/cmdai"
            >
              View on GitHub
            </PixelButton>
            <PixelButton
              variant="secondary"
              size="lg"
              href="https://github.com/wildcard/cmdai/issues"
            >
              Browse Issues
            </PixelButton>
            <PixelButton
              variant="secondary"
              size="lg"
              href="https://github.com/wildcard/cmdai/blob/main/CONTRIBUTING.md"
            >
              Contributing Guide
            </PixelButton>
          </div>

          {/* Community Stats */}
          <div className="grid grid-cols-3 gap-4 mt-12">
            <div className="bg-pixel-bg-primary border-2 border-neon-green p-4">
              <div className="pixel-text text-[16px] text-neon-green mb-1">AGPL-3.0</div>
              <div className="text-xs font-mono text-gray-400">Open Source</div>
            </div>
            <div className="bg-pixel-bg-primary border-2 border-neon-blue p-4">
              <div className="pixel-text text-[16px] text-neon-blue mb-1">Rust</div>
              <div className="text-xs font-mono text-gray-400">Language</div>
            </div>
            <div className="bg-pixel-bg-primary border-2 border-neon-pink p-4">
              <div className="pixel-text text-[16px] text-neon-pink mb-1">Active</div>
              <div className="text-xs font-mono text-gray-400">Development</div>
            </div>
          </div>
        </div>
      </div>

      {/* Decorative elements */}
      <div className="absolute top-20 left-10 w-8 h-8 border-4 border-neon-green opacity-20"></div>
      <div className="absolute bottom-20 right-10 w-12 h-12 border-4 border-neon-pink opacity-20"></div>
    </section>
  );
};
