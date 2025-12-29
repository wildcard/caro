/**
 * ArticlePage Component
 * ======================
 * Base article layout with support for different content types.
 */

import React, { ReactNode } from 'react';
import styles from './ArticlePage.module.css';
import { Layout } from '../layouts/Layout';
import { AuthorByline } from '../components/AuthorByline';
import { CategoryTag } from '../components/CategoryTag';
import { NewsletterSignup } from '../components/NewsletterSignup';

export interface ArticlePageProps {
  title: string;
  subtitle?: string;
  category: string;
  categorySlug?: string;
  author: {
    name: string;
    avatar?: string;
    bio?: string;
  };
  publishedAt: string;
  readingTime?: string;
  coverImage?: string;
  coverImageAlt?: string;
  variant?: 'default' | 'code-heavy' | 'image-heavy' | 'text-heavy';
  children: ReactNode;
}

export function ArticlePage({
  title,
  subtitle,
  category,
  categorySlug,
  author,
  publishedAt,
  readingTime,
  coverImage,
  coverImageAlt,
  variant = 'default',
  children,
}: ArticlePageProps) {
  const classNames = [styles.article, styles[variant]].filter(Boolean).join(' ');

  return (
    <Layout>
      <article className={classNames}>
        {/* Article Header */}
        <header className={styles.header}>
          <div className={styles.headerContainer}>
            <div className={styles.meta}>
              <CategoryTag
                label={category}
                href={categorySlug ? `/category/${categorySlug}` : undefined}
              />
              {readingTime && (
                <span className={styles.readingTime}>{readingTime}</span>
              )}
            </div>

            <h1 className={styles.title}>{title}</h1>

            {subtitle && <p className={styles.subtitle}>{subtitle}</p>}

            <div className={styles.authorSection}>
              <AuthorByline
                name={author.name}
                avatar={author.avatar}
                date={publishedAt}
                size="lg"
              />
            </div>
          </div>
        </header>

        {/* Cover Image */}
        {coverImage && (
          <div className={styles.coverImage}>
            <img src={coverImage} alt={coverImageAlt || ''} loading="lazy" />
          </div>
        )}

        {/* Article Content */}
        <div className={styles.content}>
          <div className={styles.contentContainer}>{children}</div>
        </div>

        {/* Article Footer */}
        <footer className={styles.footer}>
          <div className={styles.footerContainer}>
            {/* Author Bio */}
            <div className={styles.authorBio}>
              {author.avatar && (
                <img
                  src={author.avatar}
                  alt={author.name}
                  className={styles.authorAvatar}
                />
              )}
              <div className={styles.authorInfo}>
                <span className={styles.authorLabel}>Written by</span>
                <span className={styles.authorName}>{author.name}</span>
                {author.bio && <p className={styles.authorBioText}>{author.bio}</p>}
              </div>
            </div>

            {/* Share Links */}
            <div className={styles.shareSection}>
              <span className={styles.shareLabel}>Share this article</span>
              <div className={styles.shareLinks}>
                <a
                  href="#"
                  className={styles.shareLink}
                  aria-label="Share on Twitter"
                >
                  <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M18.244 2.25h3.308l-7.227 8.26 8.502 11.24H16.17l-5.214-6.817L4.99 21.75H1.68l7.73-8.835L1.254 2.25H8.08l4.713 6.231zm-1.161 17.52h1.833L7.084 4.126H5.117z" />
                  </svg>
                </a>
                <a
                  href="#"
                  className={styles.shareLink}
                  aria-label="Share on LinkedIn"
                >
                  <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M20.447 20.452h-3.554v-5.569c0-1.328-.027-3.037-1.852-3.037-1.853 0-2.136 1.445-2.136 2.939v5.667H9.351V9h3.414v1.561h.046c.477-.9 1.637-1.85 3.37-1.85 3.601 0 4.267 2.37 4.267 5.455v6.286zM5.337 7.433a2.062 2.062 0 01-2.063-2.065 2.064 2.064 0 112.063 2.065zm1.782 13.019H3.555V9h3.564v11.452zM22.225 0H1.771C.792 0 0 .774 0 1.729v20.542C0 23.227.792 24 1.771 24h20.451C23.2 24 24 23.227 24 22.271V1.729C24 .774 23.2 0 22.222 0h.003z" />
                  </svg>
                </a>
                <button
                  className={styles.shareLink}
                  aria-label="Copy link"
                  onClick={() => navigator.clipboard.writeText(window.location.href)}
                >
                  <svg
                    width="20"
                    height="20"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                  >
                    <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71" />
                    <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71" />
                  </svg>
                </button>
              </div>
            </div>
          </div>
        </footer>

        {/* Newsletter CTA */}
        <div className={styles.newsletter}>
          <NewsletterSignup
            title="Enjoyed this article?"
            description="Subscribe to get more articles like this delivered to your inbox."
            variant="compact"
          />
        </div>
      </article>
    </Layout>
  );
}

export default ArticlePage;
