'use client';

import React, { useState, useEffect } from 'react';

const stats = [
  { value: '500+', label: 'Attendees' },
  { value: '50', label: 'Demo Slots' },
  { value: '$10M+', label: 'Funding Raised' },
  { value: '30+', label: 'Investors' },
];

// Calculate days until June 15, 2026
const calculateTimeLeft = () => {
  const deadline = new Date('2026-06-15T09:00:00-07:00');
  const now = new Date();
  const difference = deadline.getTime() - now.getTime();

  if (difference <= 0) {
    return { days: 0, hours: 0, minutes: 0, seconds: 0 };
  }

  return {
    days: Math.floor(difference / (1000 * 60 * 60 * 24)),
    hours: Math.floor((difference / (1000 * 60 * 60)) % 24),
    minutes: Math.floor((difference / 1000 / 60) % 60),
    seconds: Math.floor((difference / 1000) % 60),
  };
};

export const SummitHero: React.FC = () => {
  const [timeLeft, setTimeLeft] = useState(calculateTimeLeft());

  useEffect(() => {
    const timer = setInterval(() => {
      setTimeLeft(calculateTimeLeft());
    }, 1000);

    return () => clearInterval(timer);
  }, []);

  return (
    <section className="relative min-h-screen flex items-center pt-16 overflow-hidden">
      {/* Background decorations */}
      <div className="summit-glow-teal -top-40 -right-40" />
      <div className="summit-glow-purple top-1/3 -left-60" />

      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-16 md:py-24 relative z-10">
        <div className="text-center max-w-4xl mx-auto">
          {/* Badge */}
          <div className="inline-flex items-center gap-2 summit-badge summit-badge-teal mb-8">
            <span className="w-2 h-2 bg-summit-accent-teal rounded-full animate-pulse" />
            Applications Open - Limited Slots Available
          </div>

          {/* Main Headline */}
          <h1 className="text-4xl sm:text-5xl md:text-6xl lg:text-7xl font-bold mb-6 leading-tight">
            <span className="text-summit-text-primary">Showcase Your AI Startup to</span>
            <br />
            <span className="summit-gradient-text">Fortune 500 Leaders</span>
          </h1>

          {/* Subheadline */}
          <p className="text-lg md:text-xl text-summit-text-secondary max-w-2xl mx-auto mb-8 leading-relaxed">
            Present your AI innovation to enterprise decision-makers, connect with
            top-tier investors, and gain media coverage at Seattle&apos;s premier AI startup event.
          </p>

          {/* Value Props */}
          <div className="flex flex-wrap justify-center gap-4 mb-10">
            <span className="summit-badge summit-badge-blue">
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z" />
              </svg>
              10-Min Demo Opportunity
            </span>
            <span className="summit-badge summit-badge-purple">
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              Investment Connections
            </span>
            <span className="summit-badge summit-badge-orange">
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 20H5a2 2 0 01-2-2V6a2 2 0 012-2h10a2 2 0 012 2v1m2 13a2 2 0 01-2-2V7m2 13a2 2 0 002-2V9a2 2 0 00-2-2h-2m-4-3H9M7 16h6M7 8h6v4H7V8z" />
              </svg>
              Media Coverage
            </span>
          </div>

          {/* CTAs */}
          <div className="flex flex-col sm:flex-row justify-center gap-4 mb-16">
            <a href="#apply" className="summit-btn-primary text-lg">
              Apply for Demo Day
            </a>
            <a href="#criteria" className="summit-btn-secondary text-lg">
              View Requirements
            </a>
          </div>

          {/* Countdown */}
          <div className="mb-16">
            <p className="text-summit-text-muted text-sm mb-4 uppercase tracking-wider">
              Event Date: June 15-16, 2026
            </p>
            <div className="summit-countdown">
              <div className="summit-countdown-unit">
                <div className="summit-countdown-value">{timeLeft.days}</div>
                <div className="summit-countdown-label">Days</div>
              </div>
              <div className="summit-countdown-unit">
                <div className="summit-countdown-value">{timeLeft.hours}</div>
                <div className="summit-countdown-label">Hours</div>
              </div>
              <div className="summit-countdown-unit">
                <div className="summit-countdown-value">{timeLeft.minutes}</div>
                <div className="summit-countdown-label">Minutes</div>
              </div>
              <div className="summit-countdown-unit">
                <div className="summit-countdown-value">{timeLeft.seconds}</div>
                <div className="summit-countdown-label">Seconds</div>
              </div>
            </div>
          </div>

          {/* Stats */}
          <div className="grid grid-cols-2 md:grid-cols-4 gap-8 max-w-3xl mx-auto">
            {stats.map((stat) => (
              <div key={stat.label} className="summit-stat">
                <div className="summit-stat-value">{stat.value}</div>
                <div className="summit-stat-label">{stat.label}</div>
              </div>
            ))}
          </div>
        </div>

        {/* Scroll indicator */}
        <div className="absolute bottom-8 left-1/2 transform -translate-x-1/2 animate-bounce">
          <svg
            className="w-6 h-6 text-summit-text-muted"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M19 14l-7 7m0 0l-7-7m7 7V3"
            />
          </svg>
        </div>
      </div>
    </section>
  );
};
