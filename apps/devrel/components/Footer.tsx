import React from 'react';

export const Footer: React.FC = () => {
  const currentYear = new Date().getFullYear();

  return (
    <footer className="bg-pixel-bg-secondary border-t-4 border-neon-green py-12 px-4">
      <div className="container mx-auto">
        <div className="grid grid-cols-1 md:grid-cols-4 gap-8 mb-8">
          {/* Brand */}
          <div className="space-y-4">
            <div className="pixel-text text-[14px] text-neon-green">CMDAI</div>
            <p className="font-mono text-xs text-gray-400 leading-relaxed">
              Natural language to safe shell commands using local LLMs.
            </p>
            <div className="flex gap-3">
              <a
                href="https://github.com/wildcard/cmdai"
                className="w-8 h-8 border-2 border-neon-blue flex items-center justify-center text-neon-blue hover:bg-neon-blue hover:text-pixel-bg-primary transition-colors"
                target="_blank"
                rel="noopener noreferrer"
                aria-label="GitHub"
              >
                <span className="text-sm">GH</span>
              </a>
            </div>
          </div>

          {/* Product */}
          <div>
            <h3 className="pixel-text text-[10px] text-neon-blue mb-4">Product</h3>
            <ul className="space-y-2 font-mono text-xs">
              <li>
                <a href="#features" className="text-gray-400 hover:text-neon-green transition-colors">
                  Features
                </a>
              </li>
              <li>
                <a href="#docs" className="text-gray-400 hover:text-neon-green transition-colors">
                  Documentation
                </a>
              </li>
              <li>
                <a
                  href="https://github.com/wildcard/cmdai#readme"
                  className="text-gray-400 hover:text-neon-green transition-colors"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  Getting Started
                </a>
              </li>
              <li>
                <a
                  href="https://github.com/wildcard/cmdai/releases"
                  className="text-gray-400 hover:text-neon-green transition-colors"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  Releases
                </a>
              </li>
            </ul>
          </div>

          {/* Community */}
          <div>
            <h3 className="pixel-text text-[10px] text-neon-pink mb-4">Community</h3>
            <ul className="space-y-2 font-mono text-xs">
              <li>
                <a
                  href="https://github.com/wildcard/cmdai/blob/main/CONTRIBUTING.md"
                  className="text-gray-400 hover:text-neon-green transition-colors"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  Contributing
                </a>
              </li>
              <li>
                <a
                  href="https://github.com/wildcard/cmdai/blob/main/CODE_OF_CONDUCT.md"
                  className="text-gray-400 hover:text-neon-green transition-colors"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  Code of Conduct
                </a>
              </li>
              <li>
                <a
                  href="https://github.com/wildcard/cmdai/issues"
                  className="text-gray-400 hover:text-neon-green transition-colors"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  Issues
                </a>
              </li>
              <li>
                <a
                  href="https://github.com/wildcard/cmdai/discussions"
                  className="text-gray-400 hover:text-neon-green transition-colors"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  Discussions
                </a>
              </li>
            </ul>
          </div>

          {/* Resources */}
          <div>
            <h3 className="pixel-text text-[10px] text-neon-purple mb-4">Resources</h3>
            <ul className="space-y-2 font-mono text-xs">
              <li>
                <a
                  href="https://github.com/wildcard/cmdai/tree/main/specs"
                  className="text-gray-400 hover:text-neon-green transition-colors"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  Architecture
                </a>
              </li>
              <li>
                <a
                  href="https://github.com/wildcard/cmdai/blob/main/SECURITY.md"
                  className="text-gray-400 hover:text-neon-green transition-colors"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  Security
                </a>
              </li>
              <li>
                <a
                  href="https://github.com/wildcard/cmdai/blob/main/LICENSE"
                  className="text-gray-400 hover:text-neon-green transition-colors"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  License (AGPL-3.0)
                </a>
              </li>
              <li>
                <a
                  href="https://github.com/wildcard/cmdai/blob/main/CHANGELOG.md"
                  className="text-gray-400 hover:text-neon-green transition-colors"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  Changelog
                </a>
              </li>
            </ul>
          </div>
        </div>

        {/* Bottom Bar */}
        <div className="border-t-2 border-pixel-bg-tertiary pt-8">
          <div className="flex flex-col md:flex-row justify-between items-center gap-4">
            <div className="font-mono text-xs text-gray-500">
              © {currentYear} cmdai. Licensed under AGPL-3.0.
            </div>
            <div className="font-mono text-xs text-gray-500">
              Built with Rust 🦀 | Powered by Local LLMs 🧠
            </div>
          </div>
        </div>

        {/* Easter Egg */}
        <div className="mt-8 text-center">
          <div className="pixel-text text-[8px] text-pixel-bg-tertiary opacity-30 hover:opacity-100 hover:text-neon-purple transition-all cursor-default">
            Made with ❤️ by the Open Source Community
          </div>
        </div>
      </div>
    </footer>
  );
};
