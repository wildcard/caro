import type { Metadata, Viewport } from "next";

export const metadata: Metadata = {
  title: "Seattle AI Startup Summit 2026 | Demo Day Applications",
  description: "Showcase your AI startup to Fortune 500 leaders at the Seattle AI Startup Summit 2026. Apply for demo day, connect with investors, and get media coverage.",
  keywords: [
    "Seattle AI Summit",
    "AI startup demo day",
    "AI investors",
    "startup pitch",
    "AI conference Seattle",
    "Fortune 500",
    "startup funding",
    "AI demo",
    "tech conference 2026"
  ],
  authors: [{ name: "Seattle AI Startup Summit" }],
  openGraph: {
    title: "Seattle AI Startup Summit 2026 | Demo Day Applications",
    description: "Showcase your AI startup to Fortune 500 leaders. Apply for demo day, connect with investors, and get media coverage.",
    type: "website",
    url: "https://cmdai.dev/summit",
    siteName: "Seattle AI Startup Summit 2026",
    images: [
      {
        url: "/summit/og-image.png",
        width: 1200,
        height: 630,
        alt: "Seattle AI Startup Summit 2026",
      },
    ],
  },
  twitter: {
    card: "summary_large_image",
    title: "Seattle AI Startup Summit 2026 | Demo Day Applications",
    description: "Showcase your AI startup to Fortune 500 leaders. Apply now!",
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

// JSON-LD structured data for the event
const jsonLd = {
  "@context": "https://schema.org",
  "@type": "Event",
  name: "Seattle AI Startup Summit 2026",
  description: "The premier event for AI startups to showcase their innovations to Fortune 500 leaders, investors, and media.",
  startDate: "2026-06-15T09:00:00-07:00",
  endDate: "2026-06-16T18:00:00-07:00",
  eventStatus: "https://schema.org/EventScheduled",
  eventAttendanceMode: "https://schema.org/OfflineEventAttendanceMode",
  location: {
    "@type": "Place",
    name: "Washington State Convention Center",
    address: {
      "@type": "PostalAddress",
      streetAddress: "705 Pike St",
      addressLocality: "Seattle",
      addressRegion: "WA",
      postalCode: "98101",
      addressCountry: "US",
    },
  },
  image: ["/summit/og-image.png"],
  organizer: {
    "@type": "Organization",
    name: "Seattle AI Startup Summit",
    url: "https://cmdai.dev/summit",
  },
  offers: {
    "@type": "Offer",
    name: "Demo Day Application",
    url: "https://cmdai.dev/summit#apply",
    availability: "https://schema.org/InStock",
    validFrom: "2026-01-01T00:00:00-08:00",
    priceCurrency: "USD",
    price: "0",
  },
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
