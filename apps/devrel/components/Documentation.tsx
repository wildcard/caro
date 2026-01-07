import React from 'react';
import { PixelButton } from './PixelButton';
import { TerminalWindow } from './TerminalWindow';

const quickStartSteps = [
  {
    title: 'Install',
    command: 'cargo install cmdai',
    description: 'Install cmdai using Cargo',
  },
  {
    title: 'Configure',
    command: 'cmdai --show-config',
    description: 'View and customize your configuration',
  },
  {
    title: 'Run',
    command: 'cmdai "your natural language command"',
    description: 'Generate your first command',
  },
];

const useCases = [
  {
    title: 'File Management',
    example: 'cmdai "find all log files older than 30 days"',
    output: 'find . -name "*.log" -mtime +30',
  },
  {
    title: 'System Monitoring',
    example: 'cmdai "show top 5 memory-consuming processes"',
    output: 'ps aux --sort=-%mem | head -6',
  },
  {
    title: 'Data Processing',
    example: 'cmdai "count unique IPs in access.log"',
    output: 'awk \'{print $1}\' access.log | sort -u | wc -l',
  },
];

export const Documentation: React.FC = () => {
  return (
    <section id="docs" className="py-20 px-4 bg-pixel-bg-primary">
      <div className="container mx-auto">
        {/* Section Header */}
        <div className="text-center mb-16">
          <h2 className="pixel-text text-[20px] md:text-[28px] text-neon-purple mb-4">
            Documentation
          </h2>
          <p className="font-mono text-terminal-green max-w-2xl mx-auto">
            Get started in minutes. Full documentation coming soon.
          </p>
        </div>

        {/* Quick Start */}
        <div className="max-w-5xl mx-auto mb-16">
          <h3 className="pixel-text text-[14px] text-neon-green mb-8 text-center">
            Quick Start
          </h3>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            {quickStartSteps.map((step, index) => (
              <div key={index} className="space-y-4">
                <div className="flex items-center gap-3">
                  <div className="w-10 h-10 bg-pixel-bg-secondary border-2 border-neon-blue flex items-center justify-center pixel-text text-[12px] text-neon-blue">
                    {index + 1}
                  </div>
                  <h4 className="pixel-text text-[10px] text-neon-blue">
                    {step.title}
                  </h4>
                </div>
                <div className="bg-pixel-bg-secondary border-2 border-terminal-green p-4 font-mono text-sm">
                  <code className="text-neon-green">{step.command}</code>
                </div>
                <p className="text-sm font-mono text-gray-400">
                  {step.description}
                </p>
              </div>
            ))}
          </div>
        </div>

        {/* Use Cases */}
        <div className="max-w-5xl mx-auto mb-12">
          <h3 className="pixel-text text-[14px] text-neon-pink mb-8 text-center">
            Example Use Cases
          </h3>
          <div className="space-y-6">
            {useCases.map((useCase, index) => (
              <div key={index} className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                <div>
                  <h4 className="pixel-text text-[10px] text-neon-blue mb-3">
                    {useCase.title}
                  </h4>
                  <div className="bg-pixel-bg-secondary border-2 border-neon-green p-4 font-mono text-sm">
                    <div className="text-terminal-green mb-2 text-xs">Input:</div>
                    <code className="text-neon-blue">{useCase.example}</code>
                  </div>
                </div>
                <div className="flex items-end">
                  <div className="w-full bg-pixel-bg-secondary border-2 border-neon-pink p-4 font-mono text-sm">
                    <div className="text-terminal-green mb-2 text-xs">Generated Command:</div>
                    <code className="text-neon-green">{useCase.output}</code>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Safety Features */}
        <div className="max-w-4xl mx-auto mb-12">
          <h3 className="pixel-text text-[14px] text-terminal-red mb-6 text-center">
            Safety First
          </h3>
          <TerminalWindow title="cmdai@safety-check">
            <div className="space-y-2 font-mono text-sm">
              <div className="text-terminal-amber">⚠️  Dangerous command detected!</div>
              <div className="text-gray-400 mt-2">Command:</div>
              <div className="text-terminal-red ml-4">rm -rf /</div>
              <div className="text-gray-400 mt-2">Risk Level: CRITICAL</div>
              <div className="text-gray-400">Reason: System destruction pattern detected</div>
              <div className="text-terminal-red mt-4">❌ Command blocked by safety validator</div>
            </div>
          </TerminalWindow>
        </div>

        {/* CTA */}
        <div className="text-center space-y-6">
          <div className="pixel-text text-[12px] text-neon-green mb-4">
            Want to learn more?
          </div>
          <div className="flex flex-wrap gap-4 justify-center">
            <PixelButton
              variant="primary"
              size="lg"
              href="https://github.com/wildcard/cmdai#readme"
            >
              Full Documentation
            </PixelButton>
            <PixelButton
              variant="secondary"
              size="lg"
              href="https://github.com/wildcard/cmdai/tree/main/specs"
            >
              Architecture Specs
            </PixelButton>
          </div>
        </div>
      </div>
    </section>
  );
};
