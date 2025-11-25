/**
 * Custom JavaScript for cmdai documentation
 * Features: Copy to clipboard, smooth scrolling, mobile enhancements
 */

(function() {
    'use strict';

    // =====================================================
    // Copy to Clipboard for Code Blocks
    // =====================================================
    function initCopyButtons() {
        // Find all code blocks
        const codeBlocks = document.querySelectorAll('.hljs');

        codeBlocks.forEach((block) => {
            // Skip if button already exists
            if (block.querySelector('.copy-button')) return;

            // Create copy button
            const button = document.createElement('button');
            button.className = 'copy-button';
            button.type = 'button';
            button.textContent = 'Copy';
            button.setAttribute('aria-label', 'Copy code to clipboard');

            // Add click handler
            button.addEventListener('click', function() {
                const code = block.textContent;

                // Use modern Clipboard API when available
                if (navigator.clipboard && navigator.clipboard.writeText) {
                    navigator.clipboard.writeText(code).then(() => {
                        showCopiedFeedback(button);
                    }).catch(() => {
                        // Fallback for older browsers
                        fallbackCopy(code, button);
                    });
                } else {
                    // Fallback for older browsers
                    fallbackCopy(code, button);
                }
            });

            // Add button to code block
            block.style.position = 'relative';
            block.appendChild(button);
        });
    }

    function fallbackCopy(text, button) {
        const textarea = document.createElement('textarea');
        textarea.value = text;
        document.body.appendChild(textarea);
        textarea.select();

        try {
            document.execCommand('copy');
            showCopiedFeedback(button);
        } catch (err) {
            console.error('Failed to copy:', err);
        } finally {
            document.body.removeChild(textarea);
        }
    }

    function showCopiedFeedback(button) {
        const originalText = button.textContent;
        button.textContent = 'Copied!';
        button.classList.add('copied');

        setTimeout(() => {
            button.textContent = originalText;
            button.classList.remove('copied');
        }, 2000);
    }

    // =====================================================
    // Smooth Scroll for Anchor Links
    // =====================================================
    function initSmoothScroll() {
        document.querySelectorAll('a[href^="#"]').forEach((anchor) => {
            anchor.addEventListener('click', function(e) {
                const href = this.getAttribute('href');

                // Skip if href is just '#'
                if (href === '#') return;

                const targetElement = document.querySelector(href);
                if (targetElement) {
                    e.preventDefault();
                    targetElement.scrollIntoView({
                        behavior: 'smooth',
                        block: 'start'
                    });
                }
            });
        });
    }

    // =====================================================
    // Table Responsiveness Enhancement
    // =====================================================
    function initTableResponsiveness() {
        const tables = document.querySelectorAll('table');

        tables.forEach((table) => {
            // Skip if already wrapped
            if (table.parentElement.classList.contains('table-wrapper')) return;

            // Create wrapper
            const wrapper = document.createElement('div');
            wrapper.className = 'table-wrapper';

            // Wrap table
            table.parentNode.insertBefore(wrapper, table);
            wrapper.appendChild(table);

            // Add role for accessibility
            table.setAttribute('role', 'table');
        });
    }

    // =====================================================
    // Heading Link Anchors
    // =====================================================
    function initHeadingAnchors() {
        const headings = document.querySelectorAll('h1[id], h2[id], h3[id], h4[id], h5[id], h6[id]');

        headings.forEach((heading) => {
            // Skip if already has anchor link
            if (heading.querySelector('a.header')) return;

            const anchor = document.createElement('a');
            anchor.className = 'header';
            anchor.href = '#' + heading.id;
            anchor.setAttribute('aria-label', 'Link to ' + heading.textContent);
            anchor.innerHTML = heading.innerHTML;

            heading.innerHTML = '';
            heading.appendChild(anchor);
        });
    }

    // =====================================================
    // Progress Indicator for Long Pages
    // =====================================================
    function initProgressIndicator() {
        // Only add progress indicator on desktop (min-width: 900px)
        if (window.innerWidth < 900) return;

        // Create progress element
        const progress = document.createElement('div');
        progress.id = 'scroll-progress';
        progress.style.cssText = `
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            height: 3px;
            background: linear-gradient(to right, var(--cmdai-primary), var(--cmdai-success));
            z-index: 1000;
            transform: scaleX(0);
            transform-origin: left;
            transition: transform 0.1s ease;
        `;

        document.body.prepend(progress);

        // Update progress on scroll
        window.addEventListener('scroll', updateProgress, { passive: true });

        function updateProgress() {
            const scrollTop = window.scrollY;
            const docHeight = document.documentElement.scrollHeight - window.innerHeight;
            const scrollPercent = docHeight > 0 ? scrollTop / docHeight : 0;

            progress.style.transform = `scaleX(${scrollPercent})`;
        }
    }

    // =====================================================
    // Mobile Navigation Toggle
    // =====================================================
    function initMobileNav() {
        // Create toggle button if sidebar exists
        const sidebar = document.querySelector('.sidebar');
        if (!sidebar) return;

        // Only on mobile
        if (window.innerWidth > 768) return;

        const toggle = document.createElement('button');
        toggle.className = 'mobile-nav-toggle';
        toggle.textContent = '☰ Menu';
        toggle.setAttribute('aria-label', 'Toggle navigation menu');
        toggle.style.cssText = `
            display: none;
            padding: 0.5em 1em;
            margin: 1em;
            background-color: var(--cmdai-primary);
            color: white;
            border: none;
            border-radius: 4px;
            cursor: pointer;
            font-weight: 600;
        `;

        // Show toggle on mobile
        const updateToggleVisibility = () => {
            toggle.style.display = window.innerWidth <= 768 ? 'block' : 'none';
        };

        updateToggleVisibility();
        window.addEventListener('resize', updateToggleVisibility);

        // Add toggle functionality
        toggle.addEventListener('click', () => {
            sidebar.classList.toggle('mobile-nav-visible');
            toggle.textContent = sidebar.classList.contains('mobile-nav-visible') ? 'Hide Menu' : 'Show Menu';
        });

        // Close sidebar when clicking a link
        sidebar.querySelectorAll('a').forEach((link) => {
            link.addEventListener('click', () => {
                sidebar.classList.remove('mobile-nav-visible');
                toggle.textContent = '☰ Menu';
            });
        });

        // Insert toggle before sidebar
        sidebar.parentElement.insertBefore(toggle, sidebar);
    }

    // =====================================================
    // Code Block Language Labels
    // =====================================================
    function initCodeLanguageLabels() {
        const codeBlocks = document.querySelectorAll('pre > code');

        codeBlocks.forEach((code) => {
            const classes = code.getAttribute('class') || '';
            const languageMatch = classes.match(/language-(\w+)/);

            if (languageMatch) {
                const language = languageMatch[1].toUpperCase();
                const label = document.createElement('span');
                label.className = 'code-language';
                label.textContent = language;
                label.style.cssText = `
                    position: absolute;
                    top: 0.5em;
                    left: 0.5em;
                    font-size: 0.75em;
                    opacity: 0.5;
                    text-transform: uppercase;
                    letter-spacing: 1px;
                `;

                const parent = code.parentElement;
                if (parent.style.position !== 'relative') {
                    parent.style.position = 'relative';
                }
                parent.appendChild(label);
            }
        });
    }

    // =====================================================
    // Dark Mode Preference Detection
    // =====================================================
    function initDarkModeDetection() {
        // Check for system preference
        if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {
            // Auto-switch to dark theme if available
            const darkThemeOption = document.querySelector('[data-theme="navy"], [data-theme="ayu"], [data-theme="coal"]');
            if (darkThemeOption) {
                darkThemeOption.click?.();
            }
        }

        // Listen for changes
        if (window.matchMedia) {
            window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (e) => {
                if (e.matches) {
                    // Switch to dark theme
                    const darkTheme = document.querySelector('[data-theme="navy"]');
                    if (darkTheme) darkTheme.click?.();
                } else {
                    // Switch to light theme
                    const lightTheme = document.querySelector('[data-theme="rust"]');
                    if (lightTheme) lightTheme.click?.();
                }
            });
        }
    }

    // =====================================================
    // Initialization
    // =====================================================
    function init() {
        // Wait for DOM to be fully loaded
        if (document.readyState === 'loading') {
            document.addEventListener('DOMContentLoaded', initAll);
        } else {
            initAll();
        }
    }

    function initAll() {
        initCopyButtons();
        initSmoothScroll();
        initTableResponsiveness();
        initHeadingAnchors();
        initProgressIndicator();
        initMobileNav();
        initCodeLanguageLabels();
        initDarkModeDetection();

        // Re-initialize copy buttons after mdBook updates content
        const observer = new MutationObserver((mutations) => {
            mutations.forEach((mutation) => {
                if (mutation.addedNodes.length) {
                    // Check if code blocks were added
                    const hasCodeBlocks = Array.from(mutation.addedNodes).some(
                        node => node.querySelector?.('.hljs') || node.classList?.contains('hljs')
                    );
                    if (hasCodeBlocks) {
                        initCopyButtons();
                    }
                }
            });
        });

        // Watch for dynamic content changes
        const contentArea = document.querySelector('main') || document.querySelector('.content');
        if (contentArea) {
            observer.observe(contentArea, {
                childList: true,
                subtree: true
            });
        }
    }

    // Start initialization
    init();
})();
