import type { Metadata, Viewport } from "next";

export const metadata: Metadata = {
  title: "Caro | Natural Language to Safe Shell Commands",
  description: "Transform natural language into safe, POSIX-compliant shell commands using local AI. Built with Rust for blazing-fast performance and safety-first design.",
  keywords: [
    "Caro",
    "CLI",
    "shell commands",
    "AI",
    "LLM",
    "Rust",
    "terminal",
    "POSIX",
    "safety",
    "local AI",
    "natural language",
    "command line",
    "DevOps",
    "SRE"
  ],
  authors: [{ name: "Caro Team" }],
  openGraph: {
    title: "Caro | Natural Language to Safe Shell Commands",
    description: "Transform natural language into safe shell commands using local AI. 100% private, 52+ safety patterns.",
    type: "website",
    url: "https://caro.sh/summit",
    siteName: "Caro",
    images: [
      {
        url: "/summit/og-image.png",
        width: 1200,
        height: 630,
        alt: "Caro - Natural Language to Safe Shell Commands",
      },
    ],
  },
  twitter: {
    card: "summary_large_image",
    title: "Caro | Natural Language to Safe Shell Commands",
    description: "Transform natural language into safe shell commands using local AI.",
    images: ["/summit/og-image.png"],
  },
  robots: {
    index: true,
    follow: true,
    googleBot: {
      index: true,
      follow: true,
      "max-video-preview": -1,
      "max-image-preview": "large",
      "max-snippet": -1,
    },
  },
};

export const viewport: Viewport = {
  width: "device-width",
  initialScale: 1,
  themeColor: "#0f172a",
};

// JSON-LD structured data for the software application
const jsonLd = {
  "@context": "https://schema.org",
  "@type": "SoftwareApplication",
  name: "Caro",
  description: "AI-powered CLI tool that converts natural language into safe, POSIX-compliant shell commands using local LLMs.",
  applicationCategory: "DeveloperApplication",
  operatingSystem: ["macOS", "Linux", "Windows"],
  offers: {
    "@type": "Offer",
    price: "0",
    priceCurrency: "USD",
  },
  author: {
    "@type": "Organization",
    name: "Caro Team",
    url: "https://caro.sh",
  },
  license: "https://www.gnu.org/licenses/agpl-3.0.html",
  programmingLanguage: "Rust",
  softwareVersion: "0.1.0",
  downloadUrl: "https://github.com/anthropics/caro/releases",
  sameAs: [
    "https://github.com/anthropics/caro",
    "https://crates.io/crates/caro"
  ],
  featureList: [
    "Natural language to shell commands",
    "52+ safety patterns for dangerous command detection",
    "100% local inference - no cloud dependencies",
    "Multi-backend support (MLX, Ollama, vLLM)",
    "Cross-platform (macOS, Linux, Windows)",
    "Platform-aware command generation",
    "Sub-2-second inference on Apple Silicon"
  ],
};

export default function SummitLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <>
      <script
        type="application/ld+json"
        dangerouslySetInnerHTML={{ __html: JSON.stringify(jsonLd) }}
      />
      {children}
    </>
  );
}
