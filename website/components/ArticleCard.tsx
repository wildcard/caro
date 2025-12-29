/**
 * ArticleCard Component
 * ======================
 * Card component for displaying article previews.
 * Features subtle border, shadow on hover, and accent highlight.
 */

import React from 'react';
import styles from './ArticleCard.module.css';
import { CategoryTag } from './CategoryTag';
import { AuthorByline } from './AuthorByline';

export interface ArticleCardProps {
  title: string;
  excerpt: string;
  slug: string;
  coverImage?: string;
  coverImageAlt?: string;
  category: string;
  categorySlug?: string;
  author: {
    name: string;
    avatar?: string;
  };
  publishedAt: string;
  readingTime?: string;
  featured?: boolean;
  variant?: 'default' | 'horizontal' | 'featured';
}

export function ArticleCard({
  title,
  excerpt,
  slug,
  coverImage,
  coverImageAlt = '',
  category,
  categorySlug,
  author,
  publishedAt,
  readingTime,
  featured = false,
  variant = 'default',
}: ArticleCardProps) {
  const classNames = [
    styles.card,
    styles[variant],
    featured ? styles.featured : '',
  ]
    .filter(Boolean)
    .join(' ');

  return (
    <article className={classNames}>
      {coverImage && (
        <a href={`/articles/${slug}`} className={styles.imageLink}>
          <div className={styles.imageWrapper}>
            <img
              src={coverImage}
              alt={coverImageAlt}
              className={styles.image}
              loading="lazy"
            />
            <div className={styles.imageOverlay} aria-hidden="true" />
            <div className={styles.accentGlow} aria-hidden="true" />
          </div>
        </a>
      )}

      <div className={styles.content}>
        <div className={styles.meta}>
          <CategoryTag
            label={category}
            href={categorySlug ? `/category/${categorySlug}` : undefined}
            size="sm"
          />
          {featured && (
            <span className={styles.featuredBadge}>Featured</span>
          )}
        </div>

        <h3 className={styles.title}>
          <a href={`/articles/${slug}`} className={styles.titleLink}>
            {title}
          </a>
        </h3>

        <p className={styles.excerpt}>{excerpt}</p>

        <div className={styles.footer}>
          <AuthorByline
            name={author.name}
            avatar={author.avatar}
            date={publishedAt}
            readingTime={readingTime}
          />
        </div>
      </div>
    </article>
  );
}

export default ArticleCard;
