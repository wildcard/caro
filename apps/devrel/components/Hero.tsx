'use client';

import React from 'react';
import { PixelButton } from './PixelButton';
import { TerminalWindow } from './TerminalWindow';

export const Hero: React.FC = () => {
  return (
    <section className="relative min-h-screen flex items-center justify-center pixel-grid overflow-hidden">
      {/* Scanlines effect */}
      <div className="scanlines absolute inset-0"></div>

      <div className="container mx-auto px-4 py-16 relative z-10">
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-12 items-center">
          {/* Left side - Text content */}
          <div className="space-y-8">
            {/* Logo/Title */}
            <div className="space-y-4">
              <h1 className="pixel-text text-[24px] md:text-[32px] lg:text-[40px] text-neon-green neon-glow leading-tight">
                CMDAI
              </h1>
              <p className="pixel-text text-[12px] md:text-[14px] text-neon-blue">
                Natural Language → Safe Shell Commands
              </p>
            </div>

            {/* Description */}
            <div className="space-y-4">
              <p className="text-base md:text-lg text-terminal-green font-mono leading-relaxed">
                Transform natural language into safe, POSIX-compliant shell commands using local LLMs.
                Built with Rust for blazing-fast performance and safety-first design.
              </p>
              <div className="flex flex-wrap gap-3 text-xs md:text-sm font-mono">
                <span className="px-3 py-1 bg-pixel-bg-secondary border-2 border-neon-green text-neon-green">
                  🚀 Instant startup
                </span>
                <span className="px-3 py-1 bg-pixel-bg-secondary border-2 border-neon-blue text-neon-blue">
                  🧠 Local LLM
                </span>
                <span className="px-3 py-1 bg-pixel-bg-secondary border-2 border-neon-pink text-neon-pink">
                  🛡️ Safety-first
                </span>
              </div>
            </div>

            {/* CTA Buttons */}
            <div className="flex flex-wrap gap-4">
              <PixelButton
                variant="primary"
                size="lg"
                href="https://github.com/wildcard/cmdai"
              >
                Get Started
              </PixelButton>
              <PixelButton
                variant="secondary"
                size="lg"
                href="#docs"
              >
                View Docs
              </PixelButton>
            </div>

            {/* Quick install */}
            <div className="bg-pixel-bg-secondary border-2 border-terminal-green p-4 font-mono text-sm">
              <div className="text-terminal-green mb-2 text-xs">Quick Install:</div>
              <code className="text-neon-green">
                cargo install cmdai
              </code>
            </div>
          </div>

          {/* Right side - Terminal demo */}
          <div className="space-y-6">
            {/* Mascot placeholder - Alrezky will add Caro illustration here */}
            <div className="flex justify-center mb-6">
              <div className="w-48 h-48 bg-pixel-bg-secondary border-4 border-neon-purple flex items-center justify-center relative sprite-animate">
                <div className="text-center">
                  <div className="pixel-text text-[10px] text-neon-purple mb-2">
                    CARO
                  </div>
                  <div className="text-6xl">🤖</div>
                  <div className="text-[8px] text-gray-500 mt-2 font-mono">
                    [Mascot illustration by Alrezky]
                  </div>
                </div>
              </div>
            </div>

            {/* Terminal demo */}
            <TerminalWindow
              title="cmdai@demo"
              command='cmdai "list all PDF files larger than 10MB"'
              output={`Generated command:
  find . -name "*.pdf" -size +10M -ls

Execute this command? (y/N) y

✓ Command executed successfully`}
              animate={true}
            />
          </div>
        </div>
      </div>

      {/* Decorative pixel elements */}
      <div className="absolute top-10 right-10 w-16 h-16 border-4 border-neon-yellow opacity-30 animate-pulse"></div>
      <div className="absolute bottom-10 left-10 w-12 h-12 border-4 border-neon-pink opacity-30 animate-pulse" style={{ animationDelay: '0.5s' }}></div>
    </section>
  );
};
