'use client';

import React from 'react';

const timelineItems = [
  {
    date: 'Now - March 31',
    title: 'Applications Open',
    description: 'Submit your application through our online form. Early applications are encouraged as we review on a rolling basis.',
    status: 'active',
    icon: (
      <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
      </svg>
    ),
  },
  {
    date: 'April 1 - April 30',
    title: 'Application Review',
    description: 'Our selection committee reviews all applications. We may reach out for additional information or clarification.',
    status: 'upcoming',
    icon: (
      <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4" />
      </svg>
    ),
  },
  {
    date: 'May 1',
    title: 'Selection Notifications',
    description: 'Selected startups receive confirmation emails with demo slot assignments, guidelines, and preparation materials.',
    status: 'upcoming',
    icon: (
      <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
      </svg>
    ),
  },
  {
    date: 'May 1 - June 14',
    title: 'Demo Preparation',
    description: 'Work with our team to polish your presentation. Includes optional coaching sessions and tech checks.',
    status: 'upcoming',
    icon: (
      <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
      </svg>
    ),
  },
  {
    date: 'June 15-16, 2026',
    title: 'Summit Demo Day',
    description: 'Present your AI startup to 500+ attendees including Fortune 500 executives, investors, and media.',
    status: 'upcoming',
    icon: (
      <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z" />
      </svg>
    ),
  },
];

const demoFormat = [
  { label: 'Demo Duration', value: '10 minutes' },
  { label: 'Q&A Session', value: '5 minutes' },
  { label: 'Audience Size', value: '500+ attendees' },
  { label: 'Live Stream', value: 'Yes (optional)' },
];

export const TimelineProcess: React.FC = () => {
  return (
    <section id="timeline" className="summit-section py-20 md:py-32">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        {/* Section Header */}
        <div className="text-center max-w-3xl mx-auto mb-16">
          <span className="summit-badge summit-badge-purple mb-4 inline-block">
            Selection Process
          </span>
          <h2 className="text-3xl md:text-4xl lg:text-5xl font-bold text-summit-text-primary mb-6">
            Timeline & Process
          </h2>
          <p className="text-lg text-summit-text-secondary">
            From application to demo day, here&apos;s what to expect throughout the selection process.
          </p>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-12 lg:gap-16">
          {/* Timeline */}
          <div className="lg:col-span-2">
            <div className="summit-timeline">
              {timelineItems.map((item, index) => (
                <div
                  key={index}
                  className="summit-timeline-item"
                  style={{
                    opacity: item.status === 'active' ? 1 : 0.7,
                  }}
                >
                  <div className="flex items-start gap-4">
                    <div
                      className={`flex-shrink-0 w-10 h-10 rounded-lg flex items-center justify-center ${
                        item.status === 'active'
                          ? 'bg-summit-accent-teal text-white'
                          : 'bg-summit-tertiary text-summit-text-muted'
                      }`}
                    >
                      {item.icon}
                    </div>
                    <div className="flex-1">
                      <div className="flex flex-col sm:flex-row sm:items-center gap-2 mb-2">
                        <h3 className="text-lg font-semibold text-summit-text-primary">
                          {item.title}
                        </h3>
                        {item.status === 'active' && (
                          <span className="summit-badge summit-badge-teal text-xs">
                            Current Phase
                          </span>
                        )}
                      </div>
                      <p className="text-sm text-summit-accent-teal font-medium mb-2">
                        {item.date}
                      </p>
                      <p className="text-summit-text-secondary leading-relaxed">
                        {item.description}
                      </p>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* Demo Format Sidebar */}
          <div className="lg:col-span-1">
            <div className="summit-card sticky top-24">
              <h3 className="text-xl font-semibold text-summit-text-primary mb-6">
                Demo Day Format
              </h3>

              <div className="space-y-4 mb-8">
                {demoFormat.map((item) => (
                  <div
                    key={item.label}
                    className="flex items-center justify-between py-3 border-b border-summit-tertiary/30 last:border-0"
                  >
                    <span className="text-summit-text-secondary">{item.label}</span>
                    <span className="text-summit-text-primary font-medium">{item.value}</span>
                  </div>
                ))}
              </div>

              <div className="bg-summit-primary/50 rounded-lg p-4">
                <h4 className="text-summit-text-primary font-medium mb-2">
                  What&apos;s Included
                </h4>
                <ul className="text-sm text-summit-text-secondary space-y-2">
                  <li className="flex items-center gap-2">
                    <svg className="w-4 h-4 text-summit-success" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                    </svg>
                    Professional AV setup
                  </li>
                  <li className="flex items-center gap-2">
                    <svg className="w-4 h-4 text-summit-success" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                    </svg>
                    2 presenter badges
                  </li>
                  <li className="flex items-center gap-2">
                    <svg className="w-4 h-4 text-summit-success" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                    </svg>
                    Investor meetup access
                  </li>
                  <li className="flex items-center gap-2">
                    <svg className="w-4 h-4 text-summit-success" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                    </svg>
                    Media kit inclusion
                  </li>
                  <li className="flex items-center gap-2">
                    <svg className="w-4 h-4 text-summit-success" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                    </svg>
                    Recording of your demo
                  </li>
                </ul>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
};
