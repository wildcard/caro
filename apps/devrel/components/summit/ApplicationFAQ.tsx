'use client';

import React, { useState } from 'react';

const faqItems = [
  {
    question: 'What company information do I need ready?',
    answer: 'You\'ll need your company name, website URL, LinkedIn company profile, year founded, current employee count, and a concise 100-word company description. Make sure your website is polished and your LinkedIn profile is up-to-date before applying.',
    category: 'Company Info',
  },
  {
    question: 'What financial metrics will be requested?',
    answer: 'We\'ll ask for your current Annual Recurring Revenue (ARR) estimate, total amount raised to date, and your top investors (if applicable). Be prepared to share your funding stage (pre-seed, seed, Series A, etc.) and any notable revenue milestones.',
    category: 'Financials',
  },
  {
    question: 'What strategic questions should I prepare for?',
    answer: 'The application covers: the specific problem you solve, your target customer profile (industry, company size, buyer persona), your largest customers to date, top 3 competitors, your unique competitive advantage, and current challenges you\'re facing. Think through these before starting.',
    category: 'Strategy',
  },
  {
    question: 'Do I need a video demo?',
    answer: 'While optional, we highly recommend including a 2-3 minute demo video showcasing your product. Videos significantly increase your chances of selection. Use Loom, YouTube (unlisted), or any video platform. Focus on the core value proposition and real product functionality.',
    category: 'Demo',
  },
  {
    question: 'Who should submit the application?',
    answer: 'The CEO or Co-Founder should ideally submit the application. However, team members can submit on their behalf - we\'ll ask for both the submitter\'s email and the CEO/Founder\'s email. The founder should be available for follow-up questions.',
    category: 'Process',
  },
  {
    question: 'Is there an exhibition opportunity?',
    answer: 'Yes! Beyond the demo stage, you can indicate interest in purchasing an exhibit table during the summit. Exhibit tables ($2,500) include a 6ft table, 2 chairs, power, WiFi, and 2 attendee badges. This is separate from demo selection.',
    category: 'Exhibit',
  },
  {
    question: 'What happens after I apply?',
    answer: 'We review applications on a rolling basis. You\'ll receive an acknowledgment email within 24 hours. Selected companies are notified 6-8 weeks before the event. If selected, you\'ll receive demo guidelines, time slot options, and preparation materials.',
    category: 'Process',
  },
  {
    question: 'Can I apply if we\'re pre-revenue?',
    answer: 'Yes! We welcome pre-revenue companies with strong traction signals like user growth, waitlist size, pilot programs, or significant technical milestones. Focus your application on your validation story and why your solution matters.',
    category: 'Eligibility',
  },
  {
    question: 'What makes a strong application?',
    answer: 'Strong applications have: a clear problem statement, specific customer examples, quantifiable traction metrics, a compelling demo video, and honest self-assessment. Avoid buzzwords and focus on concrete details that demonstrate product-market fit progress.',
    category: 'Tips',
  },
  {
    question: 'Is there an application fee?',
    answer: 'No, the demo day application is completely free. There are no fees for applying or being selected to present. The only optional cost is the exhibit table if you choose to purchase one.',
    category: 'Fees',
  },
];

const ChevronIcon: React.FC<{ isOpen: boolean }> = ({ isOpen }) => (
  <svg
    className={`w-5 h-5 text-summit-text-muted transition-transform duration-300 ${
      isOpen ? 'rotate-180' : ''
    }`}
    fill="none"
    stroke="currentColor"
    viewBox="0 0 24 24"
  >
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
  </svg>
);

export const ApplicationFAQ: React.FC = () => {
  const [openIndex, setOpenIndex] = useState<number | null>(0);

  const toggleItem = (index: number) => {
    setOpenIndex(openIndex === index ? null : index);
  };

  return (
    <section id="faq" className="summit-section py-20 md:py-32 bg-summit-secondary/30">
      <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8">
        {/* Section Header */}
        <div className="text-center mb-16">
          <span className="summit-badge summit-badge-blue mb-4 inline-block">
            Application Guide
          </span>
          <h2 className="text-3xl md:text-4xl lg:text-5xl font-bold text-summit-text-primary mb-6">
            Application Requirements FAQ
          </h2>
          <p className="text-lg text-summit-text-secondary max-w-2xl mx-auto">
            Everything you need to know before you start your application.
            Review these questions to prepare your information in advance.
          </p>
        </div>

        {/* FAQ Accordion */}
        <div className="space-y-3">
          {faqItems.map((item, index) => (
            <div
              key={index}
              className="summit-accordion-item"
            >
              <button
                className="summit-accordion-trigger"
                onClick={() => toggleItem(index)}
                aria-expanded={openIndex === index}
              >
                <div className="flex items-center gap-3">
                  <span className="text-xs text-summit-accent-teal font-medium uppercase tracking-wider min-w-[80px] text-left">
                    {item.category}
                  </span>
                  <span className="text-summit-text-primary">{item.question}</span>
                </div>
                <ChevronIcon isOpen={openIndex === index} />
              </button>
              <div
                className={`summit-accordion-content ${openIndex === index ? 'open' : ''}`}
              >
                <div className="summit-accordion-content-inner">
                  {item.answer}
                </div>
              </div>
            </div>
          ))}
        </div>

        {/* Additional Help */}
        <div className="mt-12 text-center">
          <p className="text-summit-text-muted mb-4">
            Still have questions about the application?
          </p>
          <a
            href="mailto:summit@cmdai.dev"
            className="inline-flex items-center gap-2 text-summit-accent-teal hover:text-summit-accent-blue transition-colors"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
            </svg>
            Contact us at summit@cmdai.dev
          </a>
        </div>
      </div>
    </section>
  );
};
