/**
 * Translation Rules and Guidelines
 *
 * Defines what should NEVER be translated and what should ALWAYS be translated.
 * These rules are enforced by validation scripts and guide translators.
 */

import type { Locale } from './config';

/**
 * Terms that should NEVER be translated (preserve exactly)
 */
export const NEVER_TRANSLATE = {
  /** Brand names */
  brands: [
    'Caro',
    'Claude',
    'GitHub',
    'Anthropic',
    'Aperture Science',
    'GLaDOS',
    'Portal'
  ],

  /** Mascot and character names */
  characters: [
    'Kyaro',
    'Kyarorain',
    'Kadosh',
    'Kyarorain Kadosh',
    'Caroline'
  ],

  /** Technical terms and standards */
  technical: [
    'POSIX',
    'shell',
    'CLI',
    'API',
    'JSON',
    'YAML',
    'BSD',
    'GNU',
    'HTTP',
    'HTTPS',
    'URL',
    'MLX',
    'LLM'
  ],

  /** Programming concepts */
  programming: [
    'async',
    'await',
    'const',
    'let',
    'var',
    'function',
    'class',
    'interface',
    'type',
    'enum'
  ],

  /** Placeholders and variables */
  placeholders: [
    /\{[^}]+\}/g,        // {count}, {name}, {variable}
    /\{\{[^}]+\}\}/g,    // {{variable}}
    /\$\{[^}]+\}/g,      // ${variable}
  ],

  /** Code and commands (in backticks or code blocks) */
  code: [
    /`[^`]+`/g,          // `code`
    /```[\s\S]*?```/g,   // ```code block```
  ],

  /** Version numbers */
  versions: [
    /v?\d+\.\d+\.\d+/g,  // v1.2.3, 1.2.3
    /\d+\+/g,            // 52+
  ],

  /** File paths */
  paths: [
    /~\/[^\s]+/g,                 // ~/.config/caro/
    /\/[a-zA-Z0-9/_.-]+/g,        // /usr/local/bin/
    /[a-zA-Z]:\\[^\s]+/g,         // C:\Program Files\
  ],

  /** Environment variables */
  envVars: [
    /\$[A-Z_]+/g,                 // $HOME, $PATH
    /\$\{[A-Z_]+\}/g,             // ${HOME}
  ]
};

/**
 * Content categories that should ALWAYS be translated
 */
export const ALWAYS_TRANSLATE = {
  /** User-facing text */
  userFacing: [
    'Headlines and titles',
    'Descriptions and body text',
    'Call-to-action buttons',
    'Error messages',
    'Success messages',
    'Form labels and placeholders',
    'Navigation items',
    'Footer content'
  ],

  /** Cultural adaptation needed */
  cultural: [
    'Idioms and expressions',
    'Humor and wordplay',
    'Cultural references',
    'Date and time formats',
    'Number formats',
    'Currency symbols'
  ],

  /** User guidance */
  guidance: [
    'Help text',
    'Tooltips',
    'Instructions',
    'Examples (except code)',
    'Warnings and cautions'
  ]
};

/**
 * Special handling rules for specific content types
 */
export const SPECIAL_RULES = {
  /** For RTL languages (Hebrew, Arabic, Urdu) */
  rtl: {
    locales: ['he', 'ar', 'ur'] as Locale[],
    rules: [
      'Reverse visual order of UI elements',
      'Keep English technical terms in LTR',
      'Punctuation follows RTL rules',
      'Numbers stay in Western format (123) unless locale requires Eastern Arabic (١٢٣)'
    ]
  },

  /** For East Asian languages (Japanese, Korean) */
  eastAsian: {
    locales: ['ja', 'ko'] as Locale[],
    rules: [
      'Technical terms often transliterated to katakana/hangul',
      'Keep some English terms for clarity',
      'Use appropriate honorifics/politeness level',
      'Date format: YYYY年MM月DD日 or YYYY.MM.DD'
    ]
  },

  /** For South Asian languages (Hindi, Urdu) */
  southAsian: {
    locales: ['hi', 'ur'] as Locale[],
    rules: [
      'Heavy code-switching with English is natural',
      'Many technical terms stay in English',
      'Use appropriate formality level',
      'Urdu uses Persian-Arabic script (RTL)'
    ]
  },

  /** For Southeast Asian languages (Filipino, Indonesian) */
  southeastAsian: {
    locales: ['fil', 'id'] as Locale[],
    rules: [
      'English borrowings are common and acceptable',
      'Informal tone is default',
      'Technical terms often unchanged',
      'Use local slang sparingly but naturally'
    ]
  }
};

/**
 * Caro brand voice guidelines
 */
export const BRAND_VOICE = {
  values: [
    'We love humans!',
    'Terminals are a gift for humankind',
    'Caro is a loyal companion',
    'Safety first - never nuke production',
    'Computer says yes!'
  ],

  tone: [
    'Warm and approachable',
    'Technically accurate',
    'Helpful and encouraging',
    'Occasionally playful (Portal 2 references)',
    'Never condescending'
  ],

  avoid: [
    'Cold or robotic language',
    'Overly formal corporate speak',
    'Confusing jargon without explanation',
    'Negative or discouraging tone',
    'Cultural insensitivity'
  ]
};

/**
 * Check if a term should never be translated
 */
export function shouldNotTranslate(text: string): boolean {
  // Check brand names
  if (NEVER_TRANSLATE.brands.some(brand => text.includes(brand))) {
    return true;
  }

  // Check technical terms
  if (NEVER_TRANSLATE.technical.some(term => text === term)) {
    return true;
  }

  // Check patterns (placeholders, code, paths)
  const patterns = [
    ...NEVER_TRANSLATE.placeholders,
    ...NEVER_TRANSLATE.code,
    ...NEVER_TRANSLATE.versions,
    ...NEVER_TRANSLATE.paths,
    ...NEVER_TRANSLATE.envVars
  ];

  for (const pattern of patterns) {
    if (typeof pattern === 'object' && 'test' in pattern) {
      if (pattern.test(text)) {
        return true;
      }
    }
  }

  return false;
}

/**
 * Get special rules for a locale
 */
export function getSpecialRules(locale: Locale): string[] {
  // Check RTL rules
  if (SPECIAL_RULES.rtl.locales.includes(locale)) {
    return SPECIAL_RULES.rtl.rules;
  }

  // Check East Asian rules
  if (SPECIAL_RULES.eastAsian.locales.includes(locale)) {
    return SPECIAL_RULES.eastAsian.rules;
  }

  // Check South Asian rules
  if (SPECIAL_RULES.southAsian.locales.includes(locale)) {
    return SPECIAL_RULES.southAsian.rules;
  }

  // Check Southeast Asian rules
  if (SPECIAL_RULES.southeastAsian.locales.includes(locale)) {
    return SPECIAL_RULES.southeastAsian.rules;
  }

  return [];
}

/**
 * Format translation guidelines as a prompt
 */
export function formatTranslationGuidelines(locale: Locale): string {
  const specialRules = getSpecialRules(locale);

  return `
# Translation Guidelines for ${locale}

## NEVER Translate
- Brand names: ${NEVER_TRANSLATE.brands.join(', ')}
- Technical terms: ${NEVER_TRANSLATE.technical.join(', ')}
- Code, commands, and file paths
- Placeholders like {count}, {name}, {{variable}}
- Version numbers and environment variables

## ALWAYS Translate
- Headlines, descriptions, and all user-facing text
- Idioms and cultural expressions (adapt, don't translate literally)
- Error messages and user guidance
- Navigation and UI labels

## Brand Voice
${BRAND_VOICE.values.map(v => `- ${v}`).join('\n')}

## Tone
${BRAND_VOICE.tone.map(t => `- ${t}`).join('\n')}

${specialRules.length > 0 ? `\n## Special Rules for ${locale}\n${specialRules.map(r => `- ${r}`).join('\n')}` : ''}
`.trim();
}
