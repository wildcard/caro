const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

// ============================================
// Configuration
// ============================================

const targetLocale = process.env.TARGET_LOCALE;
const forceRetranslate = process.env.FORCE_RETRANSLATE === 'true';
// 'auto' (the default) resolves the model from translation-config.json's chain.
// Any explicit value (openai, gemini, claude, libretranslate, skill) pins that
// backend and bypasses model selection — kept for manual workflow_dispatch runs.
const translationBackend = process.env.TRANSLATION_BACKEND || 'auto';

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
  constructor(model) {
    super();
    const OpenAI = require('openai');
    this.openai = new OpenAI({ apiKey: process.env.OPENAI_API_KEY });
    this.model = model || process.env.OPENAI_MODEL || 'gpt-4o';
  }

  async initialize() {
    if (!process.env.OPENAI_API_KEY) {
      throw new Error('OPENAI_API_KEY environment variable is required for OpenAI backend');
    }
  }

  getName() {
    return `OpenAI (${this.model})`;
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
        model: this.model,
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
  constructor(model) {
    super();
    this.apiKey = process.env.ANTHROPIC_API_KEY;
    this.model = model || process.env.CLAUDE_MODEL || 'claude-sonnet-4-5-20250929';
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
// Gemini Backend
// ============================================

// Raw fetch against the Generative Language API rather than @google/genai:
// the translate workflow only runs `npm install openai`, so adding an SDK
// would mean a workflow change too. ClaudeBackend already sets the precedent.
class GeminiBackend extends TranslationBackend {
  constructor(model) {
    super();
    this.apiKey = process.env.GEMINI_API_KEY || process.env.GOOGLE_API_KEY;
    this.model = model || process.env.GEMINI_MODEL || 'gemini-2.5-flash';
  }

  async initialize() {
    if (!this.apiKey) {
      throw new Error('GEMINI_API_KEY (or GOOGLE_API_KEY) is required for Gemini backend');
    }
  }

  getName() {
    return `Gemini (${this.model})`;
  }

  async translate(enContent, locale, fileName) {
    const metadata = languageMetadata[locale];

    const systemPrompt = `You are a professional technical writer and translator specializing in ${metadata.name}.

**Cultural Context:**
- Target metro: ${metadata.metro}
- Culture: ${metadata.culture}
- Pop culture references: ${metadata.popCulture}

**Critical Rules:**
1. Translate ONLY string values, NEVER JSON keys
2. PRESERVE placeholders: {count}, {name}, {var}, etc.
3. PRESERVE brand names: "Caro", "Claude", "GitHub"
4. PRESERVE technical terms: POSIX, shell, CLI, MLX, vLLM, Ollama, JSON, API, HTTP
5. PRESERVE code blocks and commands
6. PRESERVE emoji and special characters
7. Return ONLY valid JSON, no explanations
${metadata.rtl ? '8. For RTL: translate text naturally, keep technical terms LTR' : ''}

**File:** ${fileName}`;

    const userPrompt = `Translate this JSON to natural ${metadata.name} for developers in ${metadata.metro}:\n\n${JSON.stringify(enContent, null, 2)}`;

    try {
      console.log(`[${locale}] [Gemini] Translating ${fileName}...`);

      const url = `https://generativelanguage.googleapis.com/v1beta/models/${this.model}:generateContent`;
      const response = await fetch(url, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'x-goog-api-key': this.apiKey
        },
        body: JSON.stringify({
          systemInstruction: { parts: [{ text: systemPrompt }] },
          contents: [{ role: 'user', parts: [{ text: userPrompt }] }],
          generationConfig: {
            temperature: 0.3,
            maxOutputTokens: 8192,
            responseMimeType: 'application/json'
          }
        })
      });

      if (!response.ok) {
        const body = await response.text();
        throw new Error(`Gemini API error: ${response.status} ${response.statusText} — ${body.slice(0, 200)}`);
      }

      const result = await response.json();
      const translatedText = result.candidates?.[0]?.content?.parts?.[0]?.text?.trim();
      if (!translatedText) {
        throw new Error(`Gemini returned no text (finishReason: ${result.candidates?.[0]?.finishReason ?? 'unknown'})`);
      }

      let jsonText = translatedText;
      if (translatedText.startsWith('```')) {
        const match = translatedText.match(/```(?:json)?\n?([\s\S]*?)\n?```/);
        if (match) jsonText = match[1];
      }

      const translated = JSON.parse(jsonText);
      console.log(`[${locale}] [Gemini] ✓ Successfully translated ${fileName}`);
      return translated;

    } catch (error) {
      console.error(`[${locale}] [Gemini] ✗ Error: ${error.message}`);
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

function createBackend(backendName, model) {
  switch (backendName.toLowerCase()) {
    case 'openai':
      return new OpenAIBackend(model);
    case 'libretranslate':
    case 'libre':
      return new LibreTranslateBackend();
    case 'claude':
    case 'anthropic':
      return new ClaudeBackend(model);
    case 'gemini':
    case 'google':
      return new GeminiBackend(model);
    case 'skill':
      return new SkillBackend();
    default:
      throw new Error(`Unknown backend: ${backendName}. Use: openai, gemini, libretranslate, claude, or skill`);
  }
}

// ============================================
// Model Selection (evergreen — see translation-config.json)
// ============================================

const MODEL_CONFIG_PATH = path.join(__dirname, 'translation-config.json');

function loadModelConfig(configPath = MODEL_CONFIG_PATH) {
  const raw = JSON.parse(fs.readFileSync(configPath, 'utf8'));

  if (!Array.isArray(raw.chain) || raw.chain.length === 0) {
    throw new Error(`${configPath}: "chain" must be a non-empty array`);
  }
  for (const id of raw.chain) {
    const entry = raw.models?.[id];
    if (!entry) {
      throw new Error(`${configPath}: chain references unknown model "${id}"`);
    }
    for (const field of ['backend', 'api_model', 'api_key_env', 'last_verified']) {
      if (!entry[field]) {
        throw new Error(`${configPath}: model "${id}" is missing required field "${field}"`);
      }
    }
  }
  return raw;
}

function daysSince(isoDate) {
  const then = Date.parse(isoDate);
  if (Number.isNaN(then)) return Infinity;
  return Math.floor((Date.now() - then) / 86400000);
}

// Staleness is a WARNING, never a hard failure: a model that hasn't been
// re-verified in 31 days is a maintenance smell, but blocking the pipeline on
// a calendar date would replace one silent outage with a louder one.
function warnIfStale(modelId, entry, staleAfterDays) {
  const age = daysSince(entry.last_verified);
  if (age > staleAfterDays) {
    console.warn(
      `⚠️  [model-config] "${modelId}" was last verified ${age} days ago ` +
      `(threshold ${staleAfterDays}d). Confirm it is still live and bump ` +
      `"last_verified" in translation-config.json.`
    );
  }
  return age;
}

// Extract the HTTP status from an error, whichever backend raised it.
// The OpenAI SDK sets `error.status`; the Claude and Gemini backends format
// their own message as "<Provider> API error: <status> <text>". We anchor on
// that "API error:" prefix rather than scanning for any 3-digit run, because
// the Gemini message also carries a slice of the response body.
function extractStatus(error) {
  if (typeof error?.status === 'number') return error.status;
  if (typeof error?.statusCode === 'number') return error.statusCode;
  const match = /API error:\s*(\d{3})\b/.exec(error?.message ?? '');
  return match ? Number(match[1]) : null;
}

const MODEL_GONE_PATTERN = /model[^.]*not\s*found|not\s*found[^.]*model|unsupported\s*model|invalid\s*model|does not exist|deprecated|decommissioned/i;

// Classify a failure into the three cases the fallback policy cares about:
//   'retry' — the model is fine, we're throttled or the server hiccuped.
//             Backing off and retrying is cheaper (and better quality) than
//             demoting to a weaker model for the whole run.
//   'gone'  — the model is retired. Retrying cannot help; move down the chain
//             immediately. This is the caro-z1rp failure mode.
//   'other' — unknown (auth, malformed response, parse failure). Fall back
//             rather than hang the pipeline, but don't waste retries on it.
function classifyError(error) {
  const status = extractStatus(error);

  if (status === 429 || (status !== null && status >= 500 && status < 600)) {
    return 'retry';
  }
  if (status === 404) return 'gone';
  if (status === 400 && MODEL_GONE_PATTERN.test(error?.message ?? '')) return 'gone';

  return 'other';
}

// Should this error advance to the next model in the chain *right now*?
// Retryable errors say no — the caller backs off first, and only falls back
// once retries are exhausted.
function shouldFallback(error) {
  return classifyError(error) !== 'retry';
}

const sleep = ms => new Promise(resolve => setTimeout(resolve, ms));

// Retry with exponential backoff, but ONLY for errors that a retry can fix.
// A 404 rethrows immediately so the caller can fall back without burning
// ~14s of backoff on a model that no longer exists.
async function withRetry(fn, { label, maxRetries = 3, baseDelayMs = 2000 } = {}) {
  let lastError;

  for (let attempt = 1; attempt <= maxRetries + 1; attempt++) {
    try {
      return await fn();
    } catch (error) {
      lastError = error;

      if (classifyError(error) !== 'retry') throw error;
      if (attempt > maxRetries) break;

      const status = extractStatus(error);
      const delay = baseDelayMs * Math.pow(2, attempt - 1);
      console.warn(
        `[retry] Retrying ${label} (attempt ${attempt}/${maxRetries}, got ${status ?? 'error'}) — waiting ${delay}ms`
      );
      await sleep(delay);
    }
  }

  console.warn(`[retry] ${label}: retries exhausted after ${maxRetries} attempts`);
  throw lastError;
}

// Probe the model with a real (tiny) translation. This exercises the exact path
// a production call takes — auth, model name, JSON mode, fence stripping, parse
// — instead of just proving the endpoint is reachable.
async function probeModel(backend, probeCfg) {
  const translated = await backend.translate(
    probeCfg.payload,
    probeCfg.locale,
    'health-probe.json'
  );
  if (!translated || typeof translated !== 'object') {
    throw new Error('probe returned a non-object response');
  }
  const keys = Object.keys(probeCfg.payload);
  const missing = keys.filter(k => !(k in translated));
  if (missing.length > 0) {
    throw new Error(`probe response dropped key(s): ${missing.join(', ')}`);
  }
  return translated;
}

// Walk the chain and return the first model that has a key AND passes its probe.
async function resolveBackend(config) {
  const staleAfterDays = config.policy?.stale_after_days ?? 30;
  const probeCfg = config.policy?.probe ?? { locale: 'es', payload: { greeting: 'Hello' } };
  const retryCfg = config.policy?.retry ?? {};
  const retryOpts = {
    maxRetries: retryCfg.max_retries ?? 3,
    baseDelayMs: retryCfg.base_delay_ms ?? 2000
  };
  const attempts = [];

  for (const [index, modelId] of config.chain.entries()) {
    const entry = config.models[modelId];
    const nextModel = config.chain[index + 1] ?? 'none (chain exhausted)';

    if (!process.env[entry.api_key_env]) {
      console.log(`[model-config] ⊘ ${modelId}: ${entry.api_key_env} not set, skipping`);
      attempts.push(`${modelId} (no ${entry.api_key_env})`);
      continue;
    }

    warnIfStale(modelId, entry, staleAfterDays);

    let backend;
    try {
      backend = createBackend(entry.backend, entry.api_model);
      await backend.initialize();
      console.log(`[model-config] … probing ${modelId} (${backend.getName()})`);
      await withRetry(() => probeModel(backend, probeCfg), {
        label: `probe ${modelId}`,
        ...retryOpts
      });
    } catch (error) {
      const status = extractStatus(error);
      // Two distinct paths land here: a 'gone' error (immediate, no retries
      // burned) or a retryable error whose retries withRetry already exhausted.
      // Either way the next model is the right move — but say which happened.
      const reason = shouldFallback(error)
        ? `got ${status ?? 'error'}`
        : `got ${status ?? 'error'}, retries exhausted`;
      console.warn(
        `[model-config] ✗ Falling back from ${modelId} to ${nextModel} (${reason}): ${error.message}`
      );
      attempts.push(`${modelId} (${reason})`);
      continue;
    }

    console.log(`[model-config] ✓ Using ${modelId} via ${backend.getName()}`);
    return { backend, modelId, entry };
  }

  throw new Error(
    `[model-config] No usable translation model. Tried:\n  - ${attempts.join('\n  - ')}`
  );
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

  // Resolve the backend: either the configured chain (health-checked) or an
  // explicitly pinned backend. `cacheTag` keys the cache so that switching
  // models re-translates instead of serving another model's output.
  let backend;
  let cacheTag;
  let retryOpts = { maxRetries: 3, baseDelayMs: 2000 };

  if (translationBackend === 'auto') {
    const config = loadModelConfig();
    retryOpts = {
      maxRetries: config.policy?.retry?.max_retries ?? retryOpts.maxRetries,
      baseDelayMs: config.policy?.retry?.base_delay_ms ?? retryOpts.baseDelayMs
    };
    const resolved = await resolveBackend(config);
    backend = resolved.backend;
    cacheTag = resolved.modelId;
  } else {
    backend = createBackend(translationBackend);
    await backend.initialize();
    cacheTag = translationBackend;
    console.log(`[model-config] Model selection bypassed: TRANSLATION_BACKEND=${translationBackend}`);
  }

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

      if (!needsRetranslation(cache, targetLocale, file, sourceHash, forceRetranslate, cacheTag)) {
        console.log(`[${targetLocale}] ⊘ Skipping ${file} (unchanged, cached)`);
        skippedCount++;
        continue;
      }

      const enContent = JSON.parse(fs.readFileSync(enFilePath, 'utf8'));
      // Retry transient throttling here too — a 429 midway through a locale
      // would otherwise drop that file and count it as a failure.
      const translated = await withRetry(
        () => backend.translate(enContent, targetLocale, file),
        { label: `${targetLocale}/${file}`, ...retryOpts }
      );

      fs.writeFileSync(targetFilePath, JSON.stringify(translated, null, 2) + '\n', 'utf8');
      console.log(`[${targetLocale}] ✓ Wrote ${file}`);

      // Update cache with backend-specific key
      const cacheKey = `${cacheTag}-${targetLocale}`;
      if (!cache[cacheKey]) {
        cache[cacheKey] = {};
      }
      cache[cacheKey][file] = {
        sourceHash: sourceHash,
        timestamp: new Date().toISOString(),
        backend: cacheTag
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

  // Fail loud on total failure. Per-file errors are caught and counted above so
  // one bad file doesn't sink the whole locale — but if EVERY attempted file
  // failed (e.g. a dead model name 404ing on every call), the run must go red.
  // Without this gate the workflow reports success while translating nothing,
  // and the only artifact is package-lock drift in the auto-PR (see caro-z1rp).
  if (translatedCount === 0 && failedCount > 0) {
    throw new Error(
      `[${targetLocale}] All ${failedCount} attempted file(s) failed — translated 0. ` +
      `Treating as a fatal error so the workflow surfaces it instead of opening a no-op PR.`
    );
  }
}

// ============================================
// Validation & Execution
// ============================================

// Only run as a CLI. Exported below so the selection/retry logic is testable.
if (require.main === module) {
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
}

module.exports = {
  classifyError,
  shouldFallback,
  extractStatus,
  withRetry,
  loadModelConfig,
  daysSince,
  createBackend
};
