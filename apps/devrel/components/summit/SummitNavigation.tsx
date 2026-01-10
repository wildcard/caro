'use client';

import React, { useState, useEffect } from 'react';

const navLinks = [
  { href: '#criteria', label: 'Criteria' },
  { href: '#faq', label: 'FAQ' },
  { href: '#timeline', label: 'Timeline' },
  { href: '#success-stories', label: 'Success Stories' },
  { href: '#checklist', label: 'Checklist' },
];

export const SummitNavigation: React.FC = () => {
  const [isScrolled, setIsScrolled] = useState(false);
  const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false);

  useEffect(() => {
    const handleScroll = () => {
      setIsScrolled(window.scrollY > 50);
    };

    window.addEventListener('scroll', handleScroll);
    return () => window.removeEventListener('scroll', handleScroll);
  }, []);

  return (
    <nav className={`summit-nav ${isScrolled ? 'summit-nav-scrolled' : ''}`}>
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex items-center justify-between h-16">
          {/* Logo */}
          <a href="#" className="flex items-center gap-3">
            <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-summit-accent-teal to-summit-accent-blue flex items-center justify-center">
              <span className="text-white font-bold text-sm">AI</span>
            </div>
            <span className="text-summit-text-primary font-semibold hidden sm:block">
              Seattle AI Summit 2026
            </span>
          </a>

          {/* Desktop Navigation */}
          <div className="hidden md:flex items-center gap-8">
            {navLinks.map((link) => (
              <a
                key={link.href}
                href={link.href}
                className="text-summit-text-secondary hover:text-summit-accent-teal transition-colors text-sm font-medium"
              >
                {link.label}
              </a>
            ))}
          </div>

          {/* CTA Button */}
          <div className="flex items-center gap-4">
            <a
              href="#apply"
              className="summit-btn-primary text-sm py-2 px-4 hidden sm:inline-block"
            >
              Apply Now
            </a>

            {/* Mobile Menu Button */}
            <button
              className="md:hidden p-2 text-summit-text-secondary hover:text-summit-text-primary"
              onClick={() => setIsMobileMenuOpen(!isMobileMenuOpen)}
              aria-label="Toggle menu"
            >
              <svg
                className="w-6 h-6"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                {isMobileMenuOpen ? (
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M6 18L18 6M6 6l12 12"
                  />
                ) : (
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M4 6h16M4 12h16M4 18h16"
                  />
                )}
              </svg>
            </button>
          </div>
        </div>

        {/* Mobile Menu */}
        {isMobileMenuOpen && (
          <div className="md:hidden py-4 border-t border-summit-tertiary/30">
            <div className="flex flex-col gap-4">
              {navLinks.map((link) => (
                <a
                  key={link.href}
                  href={link.href}
                  className="text-summit-text-secondary hover:text-summit-accent-teal transition-colors text-sm font-medium py-2"
                  onClick={() => setIsMobileMenuOpen(false)}
                >
                  {link.label}
                </a>
              ))}
              <a
                href="#apply"
                className="summit-btn-primary text-sm py-3 px-4 text-center mt-2"
                onClick={() => setIsMobileMenuOpen(false)}
              >
                Apply Now
              </a>
            </div>
          </div>
        )}
      </div>
    </nav>
  );
};
