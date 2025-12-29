/**
 * Base Layout Component
 * ======================
 * Wraps all pages with common header and footer.
 */

import React, { ReactNode, useState, useEffect } from 'react';
import styles from './Layout.module.css';

interface LayoutProps {
  children: ReactNode;
}

export function Layout({ children }: LayoutProps) {
  const [theme, setTheme] = useState<'dark' | 'light'>('dark');
  const [menuOpen, setMenuOpen] = useState(false);

  useEffect(() => {
    // Check for saved theme preference
    const saved = localStorage.getItem('theme') as 'dark' | 'light' | null;
    if (saved) {
      setTheme(saved);
      document.documentElement.setAttribute('data-theme', saved);
    }
  }, []);

  const toggleTheme = () => {
    const newTheme = theme === 'dark' ? 'light' : 'dark';
    setTheme(newTheme);
    localStorage.setItem('theme', newTheme);
    document.documentElement.setAttribute('data-theme', newTheme);
  };

  return (
    <div className={styles.layout}>
      {/* Navigation */}
      <header className={styles.header}>
        <div className={styles.headerContainer}>
          <a href="/" className={styles.logo}>
            <svg
              width="32"
              height="32"
              viewBox="0 0 32 32"
              fill="none"
              className={styles.logoIcon}
            >
              <rect width="32" height="32" rx="6" fill="var(--bg-surface)" />
              <path
                d="M10 11 L16 16 L10 21"
                stroke="var(--green-500)"
                strokeWidth="2.5"
                strokeLinecap="round"
                strokeLinejoin="round"
              />
              <rect
                x="18"
                y="14"
                width="5"
                height="4"
                rx="1"
                fill="var(--green-500)"
              />
            </svg>
            <span className={styles.logoText}>caro</span>
          </a>

          <nav className={`${styles.nav} ${menuOpen ? styles.navOpen : ''}`}>
            <a href="/articles" className={styles.navLink}>
              Articles
            </a>
            <a href="/guides" className={styles.navLink}>
              Guides
            </a>
            <a href="/changelog" className={styles.navLink}>
              Changelog
            </a>
            <a href="/about" className={styles.navLink}>
              About
            </a>
          </nav>

          <div className={styles.headerActions}>
            <button
              type="button"
              className={styles.themeToggle}
              onClick={toggleTheme}
              aria-label={`Switch to ${theme === 'dark' ? 'light' : 'dark'} mode`}
            >
              {theme === 'dark' ? (
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none">
                  <circle
                    cx="12"
                    cy="12"
                    r="5"
                    stroke="currentColor"
                    strokeWidth="2"
                  />
                  <path
                    d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"
                    stroke="currentColor"
                    strokeWidth="2"
                    strokeLinecap="round"
                  />
                </svg>
              ) : (
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none">
                  <path
                    d="M21 12.79A9 9 0 1111.21 3 7 7 0 0021 12.79z"
                    stroke="currentColor"
                    strokeWidth="2"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                  />
                </svg>
              )}
            </button>

            <a
              href="https://github.com/wildcard/caro"
              className={styles.githubLink}
              target="_blank"
              rel="noopener noreferrer"
              aria-label="View on GitHub"
            >
              <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 2C6.477 2 2 6.477 2 12c0 4.42 2.865 8.17 6.839 9.49.5.092.682-.217.682-.482 0-.237-.008-.866-.013-1.7-2.782.604-3.369-1.34-3.369-1.34-.454-1.156-1.11-1.464-1.11-1.464-.908-.62.069-.608.069-.608 1.003.07 1.531 1.03 1.531 1.03.892 1.529 2.341 1.087 2.91.831.092-.646.35-1.086.636-1.336-2.22-.253-4.555-1.11-4.555-4.943 0-1.091.39-1.984 1.029-2.683-.103-.253-.446-1.27.098-2.647 0 0 .84-.269 2.75 1.025A9.578 9.578 0 0112 6.836c.85.004 1.705.114 2.504.336 1.909-1.294 2.747-1.025 2.747-1.025.546 1.377.203 2.394.1 2.647.64.699 1.028 1.592 1.028 2.683 0 3.842-2.339 4.687-4.566 4.935.359.309.678.919.678 1.852 0 1.336-.012 2.415-.012 2.743 0 .267.18.578.688.48C19.138 20.167 22 16.418 22 12c0-5.523-4.477-10-10-10z" />
              </svg>
            </a>

            <button
              type="button"
              className={styles.menuButton}
              onClick={() => setMenuOpen(!menuOpen)}
              aria-label="Toggle menu"
              aria-expanded={menuOpen}
            >
              <span className={styles.menuIcon}>
                <span />
                <span />
                <span />
              </span>
            </button>
          </div>
        </div>
      </header>

      {/* Main content */}
      <main className={styles.main}>{children}</main>

      {/* Footer */}
      <footer className={styles.footer}>
        <div className={styles.footerContainer}>
          <div className={styles.footerTop}>
            <div className={styles.footerBrand}>
              <a href="/" className={styles.footerLogo}>
                <svg width="24" height="24" viewBox="0 0 32 32" fill="none">
                  <rect width="32" height="32" rx="6" fill="var(--bg-surface)" />
                  <path
                    d="M10 11 L16 16 L10 21"
                    stroke="var(--green-500)"
                    strokeWidth="2.5"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                  />
                  <rect
                    x="18"
                    y="14"
                    width="5"
                    height="4"
                    rx="1"
                    fill="var(--green-500)"
                  />
                </svg>
                <span>caro</span>
              </a>
              <p className={styles.footerTagline}>
                Natural language to shell commands.
                <br />
                Powered by local LLMs.
              </p>
            </div>

            <div className={styles.footerLinks}>
              <div className={styles.footerColumn}>
                <h4 className={styles.footerHeading}>Product</h4>
                <ul className={styles.footerList}>
                  <li>
                    <a href="/docs">Documentation</a>
                  </li>
                  <li>
                    <a href="/changelog">Changelog</a>
                  </li>
                  <li>
                    <a href="/roadmap">Roadmap</a>
                  </li>
                </ul>
              </div>

              <div className={styles.footerColumn}>
                <h4 className={styles.footerHeading}>Resources</h4>
                <ul className={styles.footerList}>
                  <li>
                    <a href="/articles">Articles</a>
                  </li>
                  <li>
                    <a href="/guides">Guides</a>
                  </li>
                  <li>
                    <a href="/faq">FAQ</a>
                  </li>
                </ul>
              </div>

              <div className={styles.footerColumn}>
                <h4 className={styles.footerHeading}>Community</h4>
                <ul className={styles.footerList}>
                  <li>
                    <a
                      href="https://github.com/wildcard/caro"
                      target="_blank"
                      rel="noopener noreferrer"
                    >
                      GitHub
                    </a>
                  </li>
                  <li>
                    <a
                      href="https://discord.gg/caro"
                      target="_blank"
                      rel="noopener noreferrer"
                    >
                      Discord
                    </a>
                  </li>
                  <li>
                    <a
                      href="https://twitter.com/caro_cli"
                      target="_blank"
                      rel="noopener noreferrer"
                    >
                      Twitter
                    </a>
                  </li>
                </ul>
              </div>
            </div>
          </div>

          <div className={styles.footerBottom}>
            <p className={styles.copyright}>
              &copy; {new Date().getFullYear()} Caro. Open source under MIT license.
            </p>
            <div className={styles.footerMeta}>
              <a href="/rss.xml" className={styles.rssLink}>
                <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                  <circle cx="6.5" cy="17.5" r="2.5" />
                  <path d="M4 4v3c7.18 0 13 5.82 13 13h3c0-8.837-7.163-16-16-16z" />
                  <path d="M4 10v3c3.866 0 7 3.134 7 7h3c0-5.523-4.477-10-10-10z" />
                </svg>
                RSS Feed
              </a>
            </div>
          </div>
        </div>
      </footer>
    </div>
  );
}

export default Layout;
