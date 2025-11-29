/** @type {import('next').NextConfig} */
const nextConfig = {
  output: 'export',
  images: {
    unoptimized: true,
  },
  trailingSlash: true,

  // SEO and performance optimizations
  poweredByHeader: false,
  compress: true,

  // Experimental features
  experimental: {
    optimizePackageImports: ['lucide-react'],
  },

  // Environment variables
  env: {
    SITE_URL: process.env.SITE_URL || 'https://cmdai.dev',
    SITE_NAME: 'cmdai Community Hub',
    SITE_DESCRIPTION: 'Discover safety guardrails and community guides for cmdai - the AI-powered command-line assistant',
  },
}

module.exports = nextConfig
