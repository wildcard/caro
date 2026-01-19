/**
 * Cultural Metro Contexts for Translation
 *
 * Defines the cultural context and tone for each locale.
 * This guides translators (human and AI) to produce culturally appropriate translations.
 *
 * Each locale is associated with a major tech hub that represents its cultural voice.
 */

import type { Locale } from './config';

export interface CulturalContext {
  /** Primary tech metro representing this locale's cultural voice */
  metro: string;

  /** Cultural tone and communication style */
  tone: string;

  /** Common slang or technical jargon in local tech communities */
  slang?: string;

  /** Special cultural considerations for translation */
  notes?: string;
}

export const METRO_CONTEXTS: Record<Locale, CulturalContext> = {
  en: {
    metro: 'San Francisco',
    tone: 'Direct, optimistic, startup culture, Portal 2 references',
    slang: 'crash (system failure), wrapper (abstraction layer), nuke (delete everything)',
    notes: 'Default voice. Warm, approachable, technical but friendly. "Computer says yes!"'
  },

  es: {
    metro: 'Madrid',
    tone: 'Passionate, direct, confident',
    slang: 'tumbar (crash), de primera (first time right), envoltorio (wrapper)',
    notes: 'Use "tú" for familiarity. Tech terms often stay in English but with Spanish articles.'
  },

  fr: {
    metro: 'Paris',
    tone: 'Sophisticated, elegant, precise',
    slang: 'planter (crash), enveloppe (wrapper), binaire (executable)',
    notes: 'Avoid anglicisms where possible. Prefer French tech terminology. Formal but not stuffy.'
  },

  pt: {
    metro: 'São Paulo',
    tone: 'Warm, expressive, welcoming',
    slang: 'derrubar (bring down), invólucro (wrapper), binário (binary)',
    notes: 'Brazilian Portuguese (not European). Friendly "você" form. Tech community is vibrant and casual.'
  },

  de: {
    metro: 'Berlin',
    tone: 'Precise, efficient, straightforward',
    slang: 'abstürzen (crash), Wrapper (unchanged), Binärdatei (binary file)',
    notes: 'Compound words are natural. Technical accuracy valued. Direct communication style.'
  },

  he: {
    metro: 'Tel Aviv',
    tone: 'Direct, informal (dugri), startup culture',
    slang: 'קרס (crash), עטיפה (wrapper), בינארי (binary)',
    notes: 'Hebrew tech slang mixes English terms. RTL layout. Use informal "אתה" (you).'
  },

  ar: {
    metro: 'Dubai',
    tone: 'Respectful, hospitable, elegant',
    slang: 'تعطل (crash), غلاف (wrapper), ثنائي (binary)',
    notes: 'Modern Standard Arabic with tech terms. RTL layout. Balance formality with approachability.'
  },

  ja: {
    metro: 'Tokyo',
    tone: 'Polite, precise, harmonious (wa)',
    slang: 'クラッシュ (crash), ラッパー (wrapper), バイナリ (binary)',
    notes: 'Use です/ます form. Tech terms often katakana. Value clarity and respect.'
  },

  ko: {
    metro: 'Seoul',
    tone: 'Community-focused, innovative, rapid iteration',
    slang: '크래시 (crash), 래퍼 (wrapper), 바이너리 (binary)',
    notes: 'Tech culture is fast-paced. Mix Korean and English terms naturally. Polite but not overly formal.'
  },

  ru: {
    metro: 'Moscow',
    tone: 'Pragmatic, technically rigorous',
    slang: 'крашнуться (crash), обёртка (wrapper), бинарник (binary)',
    notes: 'Strong technical vocabulary. Direct communication. Tech community values depth over polish.'
  },

  uk: {
    metro: 'Kyiv',
    tone: 'Resilient, community-oriented, technically skilled',
    slang: 'крашнутися (crash), обгортка (wrapper), бінарний файл (binary)',
    notes: 'Ukrainian (not Russian). Strong tech identity. Balance technical precision with warmth.'
  },

  hi: {
    metro: 'Bangalore',
    tone: 'Multilingual, adaptive, collaborative',
    slang: 'क्रैश (crash), रैपर (wrapper), बाइनरी (binary)',
    notes: 'Hindi-English code-switching is natural. Many tech terms stay in English. Respectful but approachable.'
  },

  ur: {
    metro: 'Karachi',
    tone: 'Eloquent, poetic, technically curious',
    slang: 'کریش (crash), ریپر (wrapper), بائنری (binary)',
    notes: 'Urdu with English tech terms. RTL layout. Formal register with technical content.'
  },

  fil: {
    metro: 'Manila',
    tone: 'Friendly, humorous, resilient',
    slang: 'bumagsak (crash), wrapper (unchanged), binary (unchanged)',
    notes: 'Filipino/Tagalog with heavy English influence. Code-switching is natural. Warm and informal.'
  },

  id: {
    metro: 'Jakarta',
    tone: 'Friendly, collaborative, pragmatic',
    slang: 'crash (unchanged), pembungkus (wrapper), biner (binary)',
    notes: 'Indonesian (not Malay). Many tech terms borrowed from English. Casual but respectful.'
  }
};

/**
 * Get cultural context for a locale
 */
export function getCulturalContext(locale: Locale): CulturalContext {
  return METRO_CONTEXTS[locale];
}

/**
 * Format cultural context as a prompt for AI translation
 */
export function formatCulturalPrompt(locale: Locale): string {
  const context = METRO_CONTEXTS[locale];

  return `
Translate to ${locale} with the following cultural context:

**Metro**: ${context.metro}
**Tone**: ${context.tone}
${context.slang ? `**Local Slang**: ${context.slang}` : ''}
${context.notes ? `**Notes**: ${context.notes}` : ''}

Remember Caro's brand voice: "We love humans! Terminals are a gift for humankind. Caro ushers a new era of agents and human collaboration. We love computers - computer says yes!"
`.trim();
}

/**
 * Check if locale uses Right-to-Left text direction
 */
export function isRtlLocale(locale: Locale): boolean {
  return ['he', 'ar', 'ur'].includes(locale);
}

/**
 * Get tier level for a locale (for prioritization)
 */
export function getLocaleTier(locale: Locale): 1 | 2 | 3 {
  const tier1 = ['es', 'fr', 'de', 'pt', 'ja'];
  const tier2 = ['ko', 'he', 'ar', 'hi'];
  // tier3 = ['ru', 'uk', 'ur', 'fil', 'id'];

  if (tier1.includes(locale)) return 1;
  if (tier2.includes(locale)) return 2;
  return 3;
}
