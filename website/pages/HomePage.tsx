/**
 * HomePage Component
 * ===================
 * Main landing page with hero, featured articles, article grid, and newsletter.
 */

import React from 'react';
import styles from './HomePage.module.css';
import { Layout } from '../layouts/Layout';
import { ArticleCard } from '../components/ArticleCard';
import { CategoryTag } from '../components/CategoryTag';
import { NewsletterSignup } from '../components/NewsletterSignup';
import { Button } from '../components/Button';

// Sample data - in a real app, this would come from a CMS or API
const featuredArticle = {
  title: 'Building CLI Tools with Rust: A Complete Guide',
  excerpt:
    'Learn how to create powerful command-line applications with Rust, from basic argument parsing to advanced features like async operations and cross-platform support.',
  slug: 'building-cli-tools-with-rust',
  coverImage: '/images/featured-rust-cli.jpg',
  category: 'Rust',
  categorySlug: 'rust',
  author: {
    name: 'Alex Chen',
    avatar: '/images/authors/alex.jpg',
  },
  publishedAt: '2025-12-15',
  readingTime: '12 min read',
};

const articles = [
  {
    title: 'Understanding Local LLM Inference',
    excerpt:
      'A deep dive into running large language models locally on your machine, with a focus on performance optimization and memory management.',
    slug: 'understanding-local-llm-inference',
    coverImage: '/images/articles/llm-inference.jpg',
    category: 'AI',
    categorySlug: 'ai',
    author: { name: 'Sarah Park' },
    publishedAt: '2025-12-10',
    readingTime: '8 min read',
  },
  {
    title: 'Shell Command Safety Patterns',
    excerpt:
      'Learn the best practices for validating and sanitizing shell commands to prevent dangerous operations in your CLI applications.',
    slug: 'shell-command-safety-patterns',
    coverImage: '/images/articles/shell-safety.jpg',
    category: 'Security',
    categorySlug: 'security',
    author: { name: 'Marcus Webb' },
    publishedAt: '2025-12-08',
    readingTime: '6 min read',
  },
  {
    title: 'MLX Framework on Apple Silicon',
    excerpt:
      'Exploring the MLX framework for machine learning on Apple Silicon, including setup, performance benchmarks, and integration tips.',
    slug: 'mlx-framework-apple-silicon',
    coverImage: '/images/articles/mlx-apple.jpg',
    category: 'Apple',
    categorySlug: 'apple',
    author: { name: 'Jamie Liu' },
    publishedAt: '2025-12-05',
    readingTime: '10 min read',
  },
  {
    title: 'Developer Experience Design Principles',
    excerpt:
      'What makes a great developer tool? We explore the principles of DX design and how to apply them to your projects.',
    slug: 'dx-design-principles',
    coverImage: '/images/articles/dx-design.jpg',
    category: 'DX',
    categorySlug: 'dx',
    author: { name: 'Taylor Kim' },
    publishedAt: '2025-12-01',
    readingTime: '7 min read',
  },
  {
    title: 'Async Rust: Patterns and Pitfalls',
    excerpt:
      'Master async programming in Rust with practical patterns for handling concurrent operations and avoiding common mistakes.',
    slug: 'async-rust-patterns',
    coverImage: '/images/articles/async-rust.jpg',
    category: 'Rust',
    categorySlug: 'rust',
    author: { name: 'Jordan Martinez' },
    publishedAt: '2025-11-28',
    readingTime: '9 min read',
  },
  {
    title: 'POSIX Compliance in Modern CLIs',
    excerpt:
      'Why POSIX compliance still matters in 2025 and how to ensure your CLI tools work across different Unix-like systems.',
    slug: 'posix-compliance-modern-cli',
    coverImage: '/images/articles/posix.jpg',
    category: 'CLI',
    categorySlug: 'cli',
    author: { name: 'Chris Nguyen' },
    publishedAt: '2025-11-25',
    readingTime: '5 min read',
  },
];

const categories = [
  { label: 'All', slug: 'all', active: true },
  { label: 'Rust', slug: 'rust' },
  { label: 'AI', slug: 'ai' },
  { label: 'CLI', slug: 'cli' },
  { label: 'Security', slug: 'security' },
  { label: 'DX', slug: 'dx' },
  { label: 'Apple', slug: 'apple' },
];

export function HomePage() {
  return (
    <Layout>
      {/* Hero Section */}
      <section className={styles.hero}>
        <div className={styles.heroContainer}>
          <div className={styles.heroContent}>
            <h1 className={styles.heroTitle}>
              Developer
              <br />
              <span className={styles.heroAccent}>Insights</span>
            </h1>
            <p className={styles.heroSubtitle}>
              Deep dives into CLI development, local LLMs, and building tools
              that developers love. Weekly articles from the Caro team.
            </p>
            <div className={styles.heroActions}>
              <Button variant="primary" size="lg">
                Subscribe to Newsletter
              </Button>
              <Button variant="secondary" size="lg">
                Browse Articles
              </Button>
            </div>
          </div>

          <div className={styles.heroVisual}>
            <div className={styles.terminalWindow}>
              <div className={styles.terminalHeader}>
                <span className={styles.dot} data-color="red" />
                <span className={styles.dot} data-color="yellow" />
                <span className={styles.dot} data-color="green" />
              </div>
              <div className={styles.terminalBody}>
                <div className={styles.terminalLine}>
                  <span className={styles.prompt}>$</span>
                  <span className={styles.command}>caro "list all rust files"</span>
                </div>
                <div className={styles.terminalLine}>
                  <span className={styles.output}>
                    find . -name "*.rs" -type f
                  </span>
                </div>
                <div className={styles.terminalLine}>
                  <span className={styles.prompt}>$</span>
                  <span className={styles.cursor} />
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Featured Article */}
      <section className={styles.featured}>
        <div className={styles.container}>
          <div className={styles.sectionHeader}>
            <h2 className={styles.sectionTitle}>Featured</h2>
            <a href="/articles/featured" className={styles.sectionLink}>
              View all featured <span aria-hidden="true">→</span>
            </a>
          </div>

          <ArticleCard {...featuredArticle} featured variant="featured" />
        </div>
      </section>

      {/* Article Grid */}
      <section className={styles.articles}>
        <div className={styles.container}>
          <div className={styles.sectionHeader}>
            <h2 className={styles.sectionTitle}>Latest Articles</h2>
            <a href="/articles" className={styles.sectionLink}>
              View all <span aria-hidden="true">→</span>
            </a>
          </div>

          {/* Category Filter */}
          <div className={styles.categoryFilter}>
            {categories.map((cat) => (
              <CategoryTag
                key={cat.slug}
                label={cat.label}
                href={cat.slug === 'all' ? '/articles' : `/category/${cat.slug}`}
                variant={cat.active ? 'primary' : 'default'}
              />
            ))}
          </div>

          {/* Article Grid */}
          <div className={styles.articleGrid}>
            {articles.map((article) => (
              <ArticleCard key={article.slug} {...article} />
            ))}
          </div>

          <div className={styles.loadMore}>
            <Button variant="secondary">Load More Articles</Button>
          </div>
        </div>
      </section>

      {/* Newsletter Section */}
      <section className={styles.newsletter}>
        <div className={styles.container}>
          <NewsletterSignup
            title="Stay ahead of the curve"
            description="Get weekly insights on CLI development, local AI, and developer tools. Join 5,000+ developers already subscribed."
            variant="inline"
          />
        </div>
      </section>
    </Layout>
  );
}

export default HomePage;
