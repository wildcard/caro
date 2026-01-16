const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

// ============================================
// Configuration
// ============================================

const targetLocale = process.env.TARGET_LOCALE;
const forceRetranslate = process.env.FORCE_RETRANSLATE === 'true';
const translationBackend = process.env.TRANSLATION_BACKEND || 'openai'; // openai, libretranslate, claude, skill

// Language metadata with cultural context
const languageMetadata = {
  es: {
    name: 'Spanish (Spain)',
    rtl: false,
    metro: 'Madrid',
    culture: 'Spanish culture with influences from Latin America; known for passionate expression, late dining culture, and vibrant street life',
    popCulture: 'Football (Real Madrid, FC Barcelona), flamenco, tapas culture, siesta tradition'
  },
  fr: {
    name: 'French (France)',
    rtl: false,
    metro: 'Paris',
    culture: 'French culture emphasizes elegance, sophistication, and intellectual discourse; strong tradition of art, cuisine, and philosophy',
    popCulture: 'Cinema (Nouvelle Vague), fashion, café culture, wine appreciation, Tour de France'
  },
  pt: {
    name: 'Portuguese (Brazil)',
    rtl: false,
    metro: 'São Paulo',
    culture: 'Brazilian culture is warm, expressive, and diverse; emphasis on community, celebration, and natural beauty',
    popCulture: 'Carnival, football (soccer), samba, bossa nova, beach culture, Pelé'
  },
  de: {
    name: 'German (Germany)',
    rtl: false,
    metro: 'Berlin',
    culture: 'German culture values precision, efficiency, and engineering excellence; strong tradition of philosophy and classical music',
    popCulture: 'Beer culture, Oktoberfest, automobiles (BMW, Mercedes), electronic music, Christmas markets'
  },
  he: {
    name: 'Hebrew (Israel)',
    rtl: true,
    metro: 'Tel Aviv',
    culture: 'Israeli culture is entrepreneurial, direct, and informal (dugri culture); strong emphasis on innovation and debate',
    popCulture: 'Startup Nation, hummus culture, beach lifestyle, Eurovision, kibbutz tradition'
  },
  ar: {
    name: 'Arabic (Modern Standard Arabic)',
    rtl: true,
    metro: 'Dubai',
    culture: 'Arab culture emphasizes hospitality, family values, and rich literary tradition; blend of ancient heritage and modern innovation',
    popCulture: 'Coffee culture, poetry, Arabic calligraphy, traditional music, modern architecture'
  },
  uk: {
    name: 'Ukrainian',
    rtl: false,
    metro: 'Kyiv',
    culture: 'Ukrainian culture is resilient, artistic, and deeply connected to land and traditions; strong emphasis on independence and identity',
    popCulture: 'Vyshyvanka (embroidered shirts), borscht, Cossack history, contemporary art scene'
  },
  ru: {
    name: 'Russian',
    rtl: false,
    metro: 'Moscow',
    culture: 'Russian culture is profound, literary, and expansive; strong tradition of arts, ballet, and philosophical depth',
    popCulture: 'Ballet, literature (Dostoevsky, Tolstoy), chess, matryoshka dolls, tea culture'
  },
  ja: {
    name: 'Japanese',
    rtl: false,
    metro: 'Tokyo',
    culture: 'Japanese culture emphasizes harmony (wa), respect, and precision; balance of tradition and cutting-edge innovation',
    popCulture: 'Anime/manga, cherry blossoms (sakura), sushi, bullet trains (shinkansen), karaoke'
  },
  ko: {
    name: 'Korean',
    rtl: false,
    metro: 'Seoul',
    culture: 'Korean culture values community, education, and rapid innovation; blend of Confucian values and modern technology',
    popCulture: 'K-pop, K-dramas, kimchi, PC bang gaming culture, hanbok fashion'
  },
  hi: {
    name: 'Hindi',
    rtl: false,
    metro: 'Mumbai',
    culture: 'Indian culture is diverse, vibrant, and family-oriented; strong emphasis on spirituality, festivals, and cuisine',
    popCulture: 'Bollywood, cricket, chai culture, Diwali, yoga, spices and curry'
  },
  ur: {
    name: 'Urdu',
    rtl: true,
    metro: 'Karachi',
    culture: 'Urdu culture is poetic, expressive, and rich in literary tradition; emphasis on hospitality and family bonds',
    popCulture: 'Poetry (ghazals, nazms), qawwali music, cricket, chai culture, kebabs'
  },
  fil: {
    name: 'Filipino (Tagalog)',
    rtl: false,
    metro: 'Manila',
    culture: 'Filipino culture is warm, hospitable, and family-centered; strong Catholic influence mixed with indigenous traditions',
    popCulture: 'Karaoke, basketball, jeepneys, adobo cuisine, fiestas, overseas workers (OFWs)'
  },
  id: {
    name: 'Indonesian',
    rtl: false,
    metro: 'Jakarta',
    culture: 'Indonesian culture is diverse, multicultural, and community-oriented; emphasis on harmony (gotong royong) and respect',
    popCulture: 'Gamelan music, batik, satay, motorcycle culture, island tourism (Bali)'
  }
};

// ============================================
// Translation Backend Interface
// ============================================

class TranslationBackend {
  async translate(enContent, locale, fileName) {
    throw new Error('translate() must be implemented by subclass');
  }

  async initialize() {
    // Optional: setup/validation before translation
  }

  getName() {
    return 'Unknown Backend';
  }
}

// ============================================
// OpenAI Backend
// ============================================

class OpenAIBackend extends TranslationBackend {
  constructor() {
    super();
    const OpenAI = require('openai');
    this.openai = new OpenAI({ apiKey: process.env.OPENAI_API_KEY });
  }

  async initialize() {
    if (!process.env.OPENAI_API_KEY) {
      throw new Error('OPENAI_API_KEY environment variable is required for OpenAI backend');
    }
  }

  getName() {
    return 'OpenAI GPT-4';
  }

  async translate(enContent, locale, fileName) {
    const metadata = languageMetadata[locale];
    const systemPrompt = `You are a professional translator specializing in software localization.

**Target Language:** ${metadata.name}${metadata.rtl ? ' (Right-to-Left language)' : ''}

**Critical Rules:**
1. Translate ONLY the string values, NEVER the JSON keys
2. PRESERVE ALL placeholders exactly: {count}, {name}, {var}, etc.
3. PRESERVE brand names: "Caro", "Claude", "GitHub", etc.
4. PRESERVE technical terms: POSIX, shell, CLI, MLX, vLLM, Ollama, JSON, API, HTTP
5. PRESERVE code blocks and command examples unchanged
6. PRESERVE emoji and special characters
7. Maintain the same JSON structure
8. Return ONLY valid JSON, no explanations or comments
9. For RTL languages: translate text but keep technical terms in LTR
10. Cultural adaptation: adapt idioms to sound natural in ${metadata.name}

**File context:** ${fileName}`;

    const userPrompt = `Translate this JSON to ${metadata.name}:\n\n${JSON.stringify(enContent, null, 2)}`;

    try {
      console.log(`[${locale}] [OpenAI] Translating ${fileName}...`);

      const response = await this.openai.chat.completions.create({
        model: 'gpt-4-turbo-preview',
        messages: [
          { role: 'system', content: systemPrompt },
          { role: 'user', content: userPrompt }
        ],
        temperature: 0.3,
        max_tokens: 4000
      });

      const translatedText = response.choices[0].message.content.trim();
      let jsonText = translatedText;

      if (translatedText.startsWith('```')) {
        const match = translatedText.match(/```(?:json)?\n?([\s\S]*?)\n?```/);
        if (match) jsonText = match[1];
      }

      const translated = JSON.parse(jsonText);
      console.log(`[${locale}] [OpenAI] ✓ Successfully translated ${fileName}`);
      return translated;

    } catch (error) {
      console.error(`[${locale}] [OpenAI] ✗ Error: ${error.message}`);
      throw error;
    }
  }
}

// ============================================
// LibreTranslate Backend
// ============================================

class LibreTranslateBackend extends TranslationBackend {
  constructor() {
    super();
    this.apiUrl = process.env.LIBRETRANSLATE_URL || 'https://libretranslate.com';
    this.apiKey = process.env.LIBRETRANSLATE_API_KEY || null;
  }

  async initialize() {
    console.log(`[LibreTranslate] Using endpoint: ${this.apiUrl}`);
  }

  getName() {
    return 'LibreTranslate (Open Source)';
  }

  // Map our locale codes to LibreTranslate codes
  getLibreTranslateCode(locale) {
    const mapping = {
      fil: 'tl', // Filipino → Tagalog
      he: 'iw',  // Hebrew (some APIs use 'iw')
    };
    return mapping[locale] || locale;
  }

  async translateText(text, targetLocale) {
    const targetCode = this.getLibreTranslateCode(targetLocale);

    const payload = {
      q: text,
      source: 'en',
      target: targetCode,
      format: 'text'
    };

    if (this.apiKey) {
      payload.api_key = this.apiKey;
    }

    const response = await fetch(`${this.apiUrl}/translate`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload)
    });

    if (!response.ok) {
      throw new Error(`LibreTranslate API error: ${response.status} ${response.statusText}`);
    }

    const result = await response.json();
    return result.translatedText;
  }

  async translateJSONRecursive(obj, locale) {
    if (typeof obj === 'string') {
      // Don't translate placeholders, brand names, or technical terms
      if (obj.match(/\{[^}]+\}/) ||
          obj.match(/Caro|Claude|GitHub|POSIX|CLI|MLX|vLLM|Ollama/i)) {
        return obj;
      }
      return await this.translateText(obj, locale);
    } else if (Array.isArray(obj)) {
      return Promise.all(obj.map(item => this.translateJSONRecursive(item, locale)));
    } else if (typeof obj === 'object' && obj !== null) {
      const translated = {};
      for (const [key, value] of Object.entries(obj)) {
        // Never translate keys
        translated[key] = await this.translateJSONRecursive(value, locale);
      }
      return translated;
    }
    return obj;
  }

  async translate(enContent, locale, fileName) {
    console.log(`[${locale}] [LibreTranslate] Translating ${fileName}...`);

    try {
      const translated = await this.translateJSONRecursive(enContent, locale);
      console.log(`[${locale}] [LibreTranslate] ✓ Successfully translated ${fileName}`);
      return translated;
    } catch (error) {
      console.error(`[${locale}] [LibreTranslate] ✗ Error: ${error.message}`);
      throw error;
    }
  }
}

// ============================================
// Claude API Backend
// ============================================

class ClaudeBackend extends TranslationBackend {
  constructor() {
    super();
    this.apiKey = process.env.ANTHROPIC_API_KEY;
    this.model = process.env.CLAUDE_MODEL || 'claude-sonnet-4-5-20250929';
  }

  async initialize() {
    if (!this.apiKey) {
      throw new Error('ANTHROPIC_API_KEY environment variable is required for Claude backend');
    }
  }

  getName() {
    return `Claude API (${this.model})`;
  }

  async translate(enContent, locale, fileName) {
    const metadata = languageMetadata[locale];

    const systemPrompt = `You are a professional technical writer and translator specializing in ${metadata.name}.

**Cultural Context:**
- Target metro: ${metadata.metro}
- Culture: ${metadata.culture}
- Pop culture references: ${metadata.popCulture}

**Translation Philosophy:**
- Write for developers in ${metadata.metro} who understand tech culture
- Use natural, idiomatic ${metadata.name} that feels native, not translated
- Adapt idioms and expressions to resonate with ${metadata.metro} culture
- When technical English terms are widely used in ${metadata.metro} tech scene, keep them
- Balance professionalism with the casual, direct tone common in developer tools

**Critical Rules:**
1. Translate ONLY string values, NEVER JSON keys
2. PRESERVE placeholders: {count}, {name}, {var}, etc.
3. PRESERVE brand names: "Caro", "Claude", "GitHub"
4. PRESERVE technical terms when they're standard in ${metadata.metro} tech culture
5. PRESERVE code blocks and commands
6. PRESERVE emoji and special characters
7. Return ONLY valid JSON
${metadata.rtl ? '8. For RTL: translate text naturally, keep technical terms LTR' : ''}

**File:** ${fileName}`;

    const userPrompt = `Translate this JSON to natural ${metadata.name} for developers in ${metadata.metro}:\n\n${JSON.stringify(enContent, null, 2)}`;

    try {
      console.log(`[${locale}] [Claude] Translating ${fileName}...`);

      const response = await fetch('https://api.anthropic.com/v1/messages', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'x-api-key': this.apiKey,
          'anthropic-version': '2023-06-01'
        },
        body: JSON.stringify({
          model: this.model,
          max_tokens: 4000,
          temperature: 0.3,
          system: systemPrompt,
          messages: [
            { role: 'user', content: userPrompt }
          ]
        })
      });

      if (!response.ok) {
        throw new Error(`Claude API error: ${response.status} ${response.statusText}`);
      }

      const result = await response.json();
      const translatedText = result.content[0].text.trim();

      let jsonText = translatedText;
      if (translatedText.startsWith('```')) {
        const match = translatedText.match(/```(?:json)?\n?([\s\S]*?)\n?```/);
        if (match) jsonText = match[1];
      }

      const translated = JSON.parse(jsonText);
      console.log(`[${locale}] [Claude] ✓ Successfully translated ${fileName}`);
      return translated;

    } catch (error) {
      console.error(`[${locale}] [Claude] ✗ Error: ${error.message}`);
      throw error;
    }
  }
}

// ============================================
// Skill Backend (uses /translator skill with sub-agents)
// ============================================

class SkillBackend extends TranslationBackend {
  getName() {
    return 'Claude Code Translator Skill (Local)';
  }

  async translate(enContent, locale, fileName) {
    console.log(`[${locale}] [Skill] Translation via /translator skill not yet implemented`);
    console.log(`[${locale}] [Skill] This backend requires running Claude Code with the translator skill`);
    throw new Error('Skill backend requires interactive Claude Code session - use OpenAI, LibreTranslate, or Claude API instead');
  }
}

// ============================================
// Backend Factory
// ============================================

function createBackend(backendName) {
  switch (backendName.toLowerCase()) {
    case 'openai':
      return new OpenAIBackend();
    case 'libretranslate':
    case 'libre':
      return new LibreTranslateBackend();
    case 'claude':
    case 'anthropic':
      return new ClaudeBackend();
    case 'skill':
      return new SkillBackend();
    default:
      throw new Error(`Unknown backend: ${backendName}. Use: openai, libretranslate, claude, or skill`);
  }
}

// ============================================
// Caching Functions
// ============================================

function computeFileHash(filePath) {
  const content = fs.readFileSync(filePath, 'utf8');
  return crypto.createHash('md5').update(content).digest('hex');
}

function loadCache(cacheDir) {
  const cachePath = path.join(cacheDir, '.translation-cache.json');
  if (!fs.existsSync(cachePath)) {
    return {};
  }
  try {
    return JSON.parse(fs.readFileSync(cachePath, 'utf8'));
  } catch (error) {
    console.warn(`Warning: Failed to load cache, starting fresh: ${error.message}`);
    return {};
  }
}

function saveCache(cacheDir, cache) {
  const cachePath = path.join(cacheDir, '.translation-cache.json');
  fs.writeFileSync(cachePath, JSON.stringify(cache, null, 2) + '\n', 'utf8');
}

function needsRetranslation(cache, locale, fileName, sourceHash, forceRetranslate, backend) {
  if (forceRetranslate) return true;

  const cacheKey = `${backend}-${locale}`;
  if (!cache[cacheKey] || !cache[cacheKey][fileName]) {
    return true;
  }

  const cached = cache[cacheKey][fileName];
  return cached.sourceHash !== sourceHash;
}

// ============================================
// Main Translation Function
// ============================================

async function translateAllFiles() {
  const enDir = path.join(process.cwd(), 'website/src/i18n/locales/en');
  const targetDir = path.join(process.cwd(), `website/src/i18n/locales/${targetLocale}`);
  const cacheDir = path.join(process.cwd(), 'website/src/i18n/locales');

  // Create backend
  const backend = createBackend(translationBackend);
  await backend.initialize();

  console.log(`========================================`);
  console.log(`Translation Backend: ${backend.getName()}`);
  console.log(`Target Locale: ${targetLocale} (${languageMetadata[targetLocale]?.name})`);
  console.log(`Metro Context: ${languageMetadata[targetLocale]?.metro}`);
  console.log(`========================================`);
  console.log('');

  // Ensure target directory exists
  if (!fs.existsSync(targetDir)) {
    fs.mkdirSync(targetDir, { recursive: true });
    console.log(`[${targetLocale}] Created directory: ${targetDir}`);
  }

  // Load cache
  const cache = loadCache(cacheDir);

  // Read all JSON files
  const files = fs.readdirSync(enDir).filter(file => file.endsWith('.json'));

  if (files.length === 0) {
    console.log(`[${targetLocale}] No JSON files found in ${enDir}`);
    return;
  }

  console.log(`[${targetLocale}] Found ${files.length} files to translate`);
  console.log('');

  let translatedCount = 0;
  let skippedCount = 0;
  let failedCount = 0;

  for (const file of files) {
    const enFilePath = path.join(enDir, file);
    const targetFilePath = path.join(targetDir, file);

    try {
      const sourceHash = computeFileHash(enFilePath);

      if (!needsRetranslation(cache, targetLocale, file, sourceHash, forceRetranslate, translationBackend)) {
        console.log(`[${targetLocale}] ⊘ Skipping ${file} (unchanged, cached)`);
        skippedCount++;
        continue;
      }

      const enContent = JSON.parse(fs.readFileSync(enFilePath, 'utf8'));
      const translated = await backend.translate(enContent, targetLocale, file);

      fs.writeFileSync(targetFilePath, JSON.stringify(translated, null, 2) + '\n', 'utf8');
      console.log(`[${targetLocale}] ✓ Wrote ${file}`);

      // Update cache with backend-specific key
      const cacheKey = `${translationBackend}-${targetLocale}`;
      if (!cache[cacheKey]) {
        cache[cacheKey] = {};
      }
      cache[cacheKey][file] = {
        sourceHash: sourceHash,
        timestamp: new Date().toISOString(),
        backend: translationBackend
      };

      translatedCount++;

      // Rate limiting
      await new Promise(resolve => setTimeout(resolve, 1000));

    } catch (error) {
      console.error(`[${targetLocale}] ✗ Failed to process ${file}: ${error.message}`);
      failedCount++;
      continue;
    }
  }

  saveCache(cacheDir, cache);

  console.log('');
  console.log(`========================================`);
  console.log(`[${targetLocale}] Translation completed!`);
  console.log(`[${targetLocale}] Summary: ${translatedCount} translated, ${skippedCount} skipped (cached), ${failedCount} failed`);
  console.log(`========================================`);
}

// ============================================
// Validation & Execution
// ============================================

if (!targetLocale) {
  console.error('ERROR: TARGET_LOCALE environment variable is not set');
  process.exit(1);
}

if (!languageMetadata[targetLocale]) {
  console.error(`ERROR: Unknown locale: ${targetLocale}`);
  console.error(`Supported locales: ${Object.keys(languageMetadata).join(', ')}`);
  process.exit(1);
}

translateAllFiles().catch(error => {
  console.error('Fatal error:', error);
  process.exit(1);
});
