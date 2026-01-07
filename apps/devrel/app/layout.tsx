import type { Metadata, Viewport } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "cmdai - Natural Language to Safe Shell Commands",
  description: "Transform natural language into safe, POSIX-compliant shell commands using local LLMs. Built with Rust for blazing-fast performance and safety-first design.",
  keywords: ["cmdai", "CLI", "shell", "commands", "AI", "LLM", "Rust", "terminal", "POSIX", "safety"],
  authors: [{ name: "cmdai Community" }],
  openGraph: {
    title: "cmdai - Natural Language to Safe Shell Commands",
    description: "Transform natural language into safe shell commands using local LLMs",
    type: "website",
    url: "https://cmdai.dev",
  },
  twitter: {
    card: "summary_large_image",
    title: "cmdai - Natural Language to Safe Shell Commands",
    description: "Transform natural language into safe shell commands using local LLMs",
  },
};

export const viewport: Viewport = {
  width: "device-width",
  initialScale: 1,
  themeColor: "#39ff14",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="scroll-smooth">
      <body className="antialiased">
        {children}
      </body>
    </html>
  );
}
