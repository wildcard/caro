import React from 'react';
import { PixelCard } from './PixelCard';

const features = [
  {
    icon: '⚡',
    title: 'Blazing Fast',
    description: 'Single binary with <100ms cold start. Optimized for Apple Silicon with MLX framework.',
    variant: 'default' as const,
  },
  {
    icon: '🧠',
    title: 'Local LLM Inference',
    description: 'Run powerful language models locally. No cloud dependencies, complete privacy.',
    variant: 'neon' as const,
  },
  {
    icon: '🛡️',
    title: 'Safety First',
    description: 'Comprehensive command validation. Blocks dangerous operations and requires confirmation.',
    variant: 'default' as const,
  },
  {
    icon: '🎯',
    title: 'Multiple Backends',
    description: 'Extensible backend system supporting MLX, vLLM, and Ollama with automatic fallback.',
    variant: 'neon' as const,
  },
  {
    icon: '📦',
    title: 'Zero Dependencies',
    description: 'Self-contained binary distribution. No runtime dependencies or complex setup.',
    variant: 'default' as const,
  },
  {
    icon: '🌐',
    title: 'Cross-Platform',
    description: 'Works on macOS, Linux, and Windows. POSIX-compliant command generation.',
    variant: 'neon' as const,
  },
];

export const Features: React.FC = () => {
  return (
    <section id="features" className="py-20 px-4 bg-pixel-bg-primary relative">
      <div className="container mx-auto">
        {/* Section Header */}
        <div className="text-center mb-16">
          <h2 className="pixel-text text-[20px] md:text-[28px] text-neon-blue mb-4">
            Features
          </h2>
          <p className="font-mono text-terminal-green max-w-2xl mx-auto">
            Built for developers who value safety, performance, and control
          </p>
        </div>

        {/* Features Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 max-w-6xl mx-auto">
          {features.map((feature, index) => (
            <PixelCard
              key={index}
              variant={feature.variant}
              className="h-full"
            >
              <div className="flex flex-col h-full">
                <div className="text-4xl mb-4">{feature.icon}</div>
                <h3 className="pixel-text text-[12px] text-neon-green mb-3">
                  {feature.title}
                </h3>
                <p className="text-sm font-mono text-gray-300 leading-relaxed">
                  {feature.description}
                </p>
              </div>
            </PixelCard>
          ))}
        </div>

        {/* Stats */}
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mt-16 max-w-4xl mx-auto">
          <div className="bg-pixel-bg-secondary border-2 border-terminal-green p-6 text-center">
            <div className="pixel-text text-[20px] text-terminal-green mb-2">
              &lt;100ms
            </div>
            <div className="text-xs font-mono text-gray-400">Cold Start</div>
          </div>
          <div className="bg-pixel-bg-secondary border-2 border-neon-blue p-6 text-center">
            <div className="pixel-text text-[20px] text-neon-blue mb-2">
              &lt;50MB
            </div>
            <div className="text-xs font-mono text-gray-400">Binary Size</div>
          </div>
          <div className="bg-pixel-bg-secondary border-2 border-neon-pink p-6 text-center">
            <div className="pixel-text text-[20px] text-neon-pink mb-2">
              100%
            </div>
            <div className="text-xs font-mono text-gray-400">Safe</div>
          </div>
          <div className="bg-pixel-bg-secondary border-2 border-neon-purple p-6 text-center">
            <div className="pixel-text text-[20px] text-neon-purple mb-2">
              0
            </div>
            <div className="text-xs font-mono text-gray-400">Cloud Deps</div>
          </div>
        </div>
      </div>
    </section>
  );
};
