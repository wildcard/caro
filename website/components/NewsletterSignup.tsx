/**
 * NewsletterSignup Component
 * ===========================
 * Email subscription form with validation and feedback states.
 */

import React, { useState, FormEvent } from 'react';
import styles from './NewsletterSignup.module.css';
import { Button } from './Button';

export interface NewsletterSignupProps {
  title?: string;
  description?: string;
  placeholder?: string;
  buttonText?: string;
  variant?: 'default' | 'compact' | 'inline';
  onSubmit?: (email: string) => Promise<void>;
  className?: string;
}

type SubmitState = 'idle' | 'loading' | 'success' | 'error';

export function NewsletterSignup({
  title = 'Stay in the loop',
  description = 'Weekly insights on developer tools, CLI design, and building software that developers love.',
  placeholder = 'you@company.com',
  buttonText = 'Subscribe',
  variant = 'default',
  onSubmit,
  className = '',
}: NewsletterSignupProps) {
  const [email, setEmail] = useState('');
  const [state, setState] = useState<SubmitState>('idle');
  const [errorMessage, setErrorMessage] = useState('');

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();

    if (!email || !email.includes('@')) {
      setState('error');
      setErrorMessage('Please enter a valid email address');
      return;
    }

    setState('loading');
    setErrorMessage('');

    try {
      if (onSubmit) {
        await onSubmit(email);
      } else {
        // Simulate API call
        await new Promise((resolve) => setTimeout(resolve, 1000));
      }
      setState('success');
      setEmail('');
    } catch (err) {
      setState('error');
      setErrorMessage('Something went wrong. Please try again.');
    }
  };

  const classNames = [
    styles.wrapper,
    styles[variant],
    className,
  ]
    .filter(Boolean)
    .join(' ');

  if (state === 'success') {
    return (
      <div className={classNames}>
        <div className={styles.successState}>
          <div className={styles.successIcon} aria-hidden="true">
            <svg
              width="24"
              height="24"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <polyline points="20 6 9 17 4 12" />
            </svg>
          </div>
          <h3 className={styles.successTitle}>You're subscribed!</h3>
          <p className={styles.successMessage}>
            Thanks for signing up. Check your inbox to confirm your subscription.
          </p>
          <button
            type="button"
            className={styles.resetButton}
            onClick={() => setState('idle')}
          >
            Subscribe another email
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className={classNames}>
      {variant !== 'inline' && (
        <div className={styles.content}>
          <h2 className={styles.title}>{title}</h2>
          <p className={styles.description}>{description}</p>
        </div>
      )}

      <form className={styles.form} onSubmit={handleSubmit}>
        <div className={styles.inputWrapper}>
          <input
            type="email"
            className={`${styles.input} ${state === 'error' ? styles.inputError : ''}`}
            placeholder={placeholder}
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            disabled={state === 'loading'}
            aria-label="Email address"
            aria-describedby={state === 'error' ? 'newsletter-error' : undefined}
          />
          {state === 'error' && (
            <p id="newsletter-error" className={styles.errorMessage}>
              {errorMessage}
            </p>
          )}
        </div>

        <Button
          type="submit"
          variant="primary"
          size={variant === 'compact' ? 'sm' : 'md'}
          loading={state === 'loading'}
        >
          {buttonText}
        </Button>
      </form>

      <p className={styles.privacy}>
        No spam. Unsubscribe anytime.
      </p>
    </div>
  );
}

export default NewsletterSignup;
