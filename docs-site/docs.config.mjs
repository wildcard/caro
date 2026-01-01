/**
 * Documentation Inclusion Configuration
 *
 * This file configures which markdown files from /docs are included
 * in the Starlight documentation site.
 *
 * Usage:
 * - Add paths to `include` to explicitly include files
 * - Add paths to `exclude` to skip specific files
 * - Use glob patterns for bulk include/exclude
 * - Files are relative to the project root /docs directory
 */

export default {
  // Source directory (relative to project root)
  sourceDir: '../docs',

  // Output directory for generated pages (relative to docs-site)
  outputDir: 'src/content/docs/external',

  // Sidebar group configuration
  sidebarGroups: {
    'development': {
      label: 'Development',
      collapsed: false,
    },
    'implementation': {
      label: 'Implementation Status',
      collapsed: true,
    },
    'legal': {
      label: 'Legal',
      collapsed: true,
    },
    'adr': {
      label: 'Architecture Decisions',
      collapsed: true,
    },
    'enterprise': {
      label: 'Enterprise',
      collapsed: true,
    },
  },

  // Explicit include list - files to include from /docs
  // Format: { source: 'path/to/file.md', target: 'output/path.md', title?: 'Override Title' }
  include: [
    // Development docs
    { source: 'development/CLAUDE.md', target: 'development/claude.md', title: 'Claude Integration' },
    { source: 'development/TDD-WORKFLOW.md', target: 'development/tdd-workflow.md', title: 'TDD Workflow' },
    { source: 'development/TECH_DEBT.md', target: 'development/tech-debt.md', title: 'Technical Debt' },
    { source: 'development/AGENTIC_LOOP_ARCHITECTURE.md', target: 'development/agentic-loop.md', title: 'Agentic Loop Architecture' },

    // Implementation status docs
    { source: 'implementation/MLX_WORKING_STATUS.md', target: 'implementation/mlx-status.md', title: 'MLX Backend Status' },
    { source: 'implementation/IMPLEMENTATION_COMPLETE.md', target: 'implementation/complete.md', title: 'Implementation Complete' },
    { source: 'implementation/LAUNCH_READY.md', target: 'implementation/launch-ready.md', title: 'Launch Readiness' },

    // Setup guides
    { source: 'MACOS_SETUP.md', target: 'guides/macos-advanced.md', title: 'Advanced macOS Setup' },
    { source: 'XCODE_SETUP.md', target: 'guides/xcode-setup.md', title: 'Xcode Setup' },
    { source: 'ACT_SETUP.md', target: 'guides/act-setup.md', title: 'Act Setup (Local CI)' },

    // Reference docs
    { source: 'MODEL_CATALOG.md', target: 'reference/model-catalog.md', title: 'Model Catalog' },
    { source: 'SECURITY_SETTINGS.md', target: 'reference/security.md', title: 'Security Settings' },
    { source: 'PERFORMANCE_ANALYSIS.md', target: 'reference/performance.md', title: 'Performance Analysis' },
    { source: 'RELEASE_PROCESS.md', target: 'reference/release-process.md', title: 'Release Process' },

    // ADRs
    { source: 'adr/001-llm-inference-architecture.md', target: 'adr/001-inference.md', title: 'ADR-001: LLM Inference' },
    { source: 'adr/002-karo-system-definition.md', target: 'adr/002-karo.md', title: 'ADR-002: Karo System' },

    // Legal
    { source: 'legal/CLA.md', target: 'legal/cla.md', title: 'Contributor License Agreement' },
    { source: 'legal/DCO.txt', target: 'legal/dco.md', title: 'Developer Certificate of Origin' },
    { source: 'legal/DUAL_LICENSE_COMPLIANCE.md', target: 'legal/dual-license.md', title: 'Dual License Compliance' },

    // Enterprise docs
    { source: 'enterprise/ENTERPRISE-VALUE-PROPOSITION.md', target: 'enterprise/value-prop.md', title: 'Enterprise Value' },
    { source: 'enterprise/MOAT.md', target: 'enterprise/moat.md', title: 'Competitive Moat' },

    // Spec-Kitty workflow
    { source: 'SPEC_KITTY_GUIDE.md', target: 'guides/spec-kitty.md', title: 'Spec-Kitty Guide' },
    { source: 'SPEC_KITTY_QUICKREF.md', target: 'reference/spec-kitty-quickref.md', title: 'Spec-Kitty Quick Reference' },
  ],

  // Exclude patterns - files/directories to never include
  exclude: [
    'README.md',           // Use custom index
    '**/README.md',        // Skip all README files
    'sessions/**',         // Skip session logs
    'brand/**',            // Skip brand assets
    'marketing/**',        // Skip marketing docs
    'research/**',         // Skip research docs
    'strategy/**',         // Skip strategy docs
    'governance/**',       // Skip governance for now
    '*.txt',               // Skip text files (except DCO which is handled specially)
  ],

  // Default frontmatter to add if not present
  defaultFrontmatter: {
    editUrl: false,  // Disable edit link for external docs
  },

  // Transform function for file content (optional)
  // transform: (content, sourcePath) => content,
};
