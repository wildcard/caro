'use client';

import React, { useState, useEffect } from 'react';
import { PixelButton } from './PixelButton';

export const Navigation: React.FC = () => {
  const [isScrolled, setIsScrolled] = useState(false);

  useEffect(() => {
    const handleScroll = () => {
      setIsScrolled(window.scrollY > 50);
    };

    window.addEventListener('scroll', handleScroll);
    return () => window.removeEventListener('scroll', handleScroll);
  }, []);

  return (
    <nav
      className={`fixed top-0 left-0 right-0 z-50 transition-all duration-300 ${
        isScrolled
          ? 'bg-pixel-bg-primary border-b-4 border-neon-green'
          : 'bg-transparent'
      }`}
    >
      <div className="container mx-auto px-4">
        <div className="flex items-center justify-between h-20">
          {/* Logo */}
          <a href="#" className="flex items-center gap-3">
            <div className="pixel-text text-[16px] text-neon-green neon-glow">
              CMDAI
            </div>
          </a>

          {/* Navigation Links - Desktop */}
          <div className="hidden md:flex items-center gap-8 font-mono text-sm">
            <a
              href="#features"
              className="text-terminal-green hover:text-neon-green transition-colors"
            >
              Features
            </a>
            <a
              href="#docs"
              className="text-terminal-green hover:text-neon-blue transition-colors"
            >
              Docs
            </a>
            <a
              href="#contribute"
              className="text-terminal-green hover:text-neon-pink transition-colors"
            >
              Contribute
            </a>
            <a
              href="https://github.com/wildcard/cmdai"
              className="text-terminal-green hover:text-neon-purple transition-colors"
              target="_blank"
              rel="noopener noreferrer"
            >
              GitHub
            </a>
          </div>

          {/* CTA Button */}
          <div className="hidden md:block">
            <PixelButton
              variant="primary"
              size="sm"
              href="https://github.com/wildcard/cmdai"
            >
              Star on GitHub
            </PixelButton>
          </div>

          {/* Mobile Menu Button */}
          <button
            className="md:hidden w-10 h-10 border-2 border-neon-green flex flex-col items-center justify-center gap-1.5 hover:bg-neon-green hover:bg-opacity-10 transition-colors"
            aria-label="Menu"
          >
            <div className="w-5 h-0.5 bg-neon-green"></div>
            <div className="w-5 h-0.5 bg-neon-green"></div>
            <div className="w-5 h-0.5 bg-neon-green"></div>
          </button>
        </div>
      </div>
    </nav>
  );
};
