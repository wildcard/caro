/**
 * AuthorByline Component
 * =======================
 * Displays author information with avatar and monospace date format.
 */

import React from 'react';
import styles from './AuthorByline.module.css';

export interface AuthorBylineProps {
  name: string;
  avatar?: string;
  date: string;
  readingTime?: string;
  href?: string;
  size?: 'sm' | 'md' | 'lg';
}

function formatDate(dateString: string): string {
  const date = new Date(dateString);
  const months = ['JAN', 'FEB', 'MAR', 'APR', 'MAY', 'JUN',
                  'JUL', 'AUG', 'SEP', 'OCT', 'NOV', 'DEC'];
  const month = months[date.getMonth()];
  const day = String(date.getDate()).padStart(2, '0');
  const year = date.getFullYear();
  return `${month} ${day} ${year}`;
}

export function AuthorByline({
  name,
  avatar,
  date,
  readingTime,
  href,
  size = 'md',
}: AuthorBylineProps) {
  const formattedDate = formatDate(date);

  const avatarElement = avatar ? (
    <img src={avatar} alt={name} className={styles.avatar} loading="lazy" />
  ) : (
    <div className={styles.avatarPlaceholder} aria-hidden="true">
      {name.charAt(0).toUpperCase()}
    </div>
  );

  const authorContent = (
    <>
      <div className={styles.avatarWrapper}>{avatarElement}</div>
      <div className={styles.info}>
        <span className={styles.name}>{name.toUpperCase()}</span>
        <div className={styles.meta}>
          <time className={styles.date} dateTime={date}>
            {formattedDate}
          </time>
          {readingTime && (
            <>
              <span className={styles.separator} aria-hidden="true">
                ·
              </span>
              <span className={styles.readingTime}>{readingTime}</span>
            </>
          )}
        </div>
      </div>
    </>
  );

  const classNames = [styles.byline, styles[size]].join(' ');

  if (href) {
    return (
      <a href={href} className={`${classNames} ${styles.interactive}`}>
        {authorContent}
      </a>
    );
  }

  return <div className={classNames}>{authorContent}</div>;
}

export default AuthorByline;
