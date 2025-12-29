const OpenAI = require('openai');
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

const openai = new OpenAI({ apiKey: process.env.OPENAI_API_KEY });
const targetLocale = process.env.TARGET_LOCALE;
const forceRetranslate = process.env.FORCE_RETRANSLATE === 'true';

// Language names for better translation context
const localeNames = {
  es: 'Spanish (Spain)',
  fr: 'French (France)',
  pt: 'Portuguese (Brazil)',
  de: 'German (Germany)',
  he: 'Hebrew (Israel)',
  ar: 'Arabic (Modern Standard Arabic)',
  uk: 'Ukrainian',
  ru: 'Russian',
  ja: 'Japanese',
  ko: 'Korean',
  hi: 'Hindi',
  ur: 'Urdu',
  fil: 'Filipino (Tagalog)',
  id: 'Indonesian'
};

// RTL languages
const rtlLocales = ['he', 'ar', 'ur'];

// ============================================
// Caching Functions
// ============================================

/**
 * Compute MD5 hash of file content for change detection
 */
function computeFileHash(filePath) {
  const content = fs.readFileSync(filePath, 'utf8');
  return crypto.createHash('md5').update(content).digest('hex');
}

/**
 * Load translation cache from disk
 * Cache format: { locale: { filename: { sourceHash: 'abc123', timestamp: '2025-12-29T...' } } }
 */
function loadCache(cacheDir) {
  const cachePath = path.join(cacheDir, '.translation-cache.json');
  if (!fs.existsSync(cachePath)) {
    return {};
  }
  try {
    return JSON.parse(fs.readFileSync(cachePath, 'utf8'));
  } catch (error) {
    console.warn(`Warning: Failed to load cache file, starting fresh: ${error.message}`);
    return {};
  }
}

/**
 * Save translation cache to disk
 */
function saveCache(cacheDir, cache) {
  const cachePath = path.join(cacheDir, '.translation-cache.json');
  fs.writeFileSync(cachePath, JSON.stringify(cache, null, 2) + '\n', 'utf8');
}

/**
 * Check if file needs retranslation
 */
function needsRetranslation(cache, locale, fileName, sourceHash, forceRetranslate) {
  if (forceRetranslate) {
    return true; // Force flag overrides cache
  }

  if (!cache[locale] || !cache[locale][fileName]) {
    return true; // No cache entry, needs translation
  }

  const cached = cache[locale][fileName];
  if (cached.sourceHash !== sourceHash) {
    return true; // Source content changed, needs retranslation
  }

  return false; // Cache is valid, skip translation
}

async function translateJSON(enContent, targetLocale, fileName) {
  const localeName = localeNames[targetLocale] || targetLocale;
  const isRTL = rtlLocales.includes(targetLocale);

  const systemPrompt = `You are a professional translator specializing in software localization.

**Target Language:** ${localeName}${isRTL ? ' (Right-to-Left language)' : ''}

**Critical Rules:**
1. Translate ONLY the string values, NEVER the JSON keys
2. PRESERVE ALL placeholders exactly as they appear: {count}, {name}, {var}, etc.
3. PRESERVE brand names: "Caro", "Claude", "GitHub", etc.
4. PRESERVE technical terms: POSIX, shell, CLI, MLX, vLLM, Ollama, JSON, API, HTTP
5. PRESERVE code blocks and command examples unchanged
6. PRESERVE emoji and special characters
7. Maintain the same JSON structure
8. Return ONLY valid JSON, no explanations or comments
9. For RTL languages: translate text but keep technical terms in LTR
10. Cultural adaptation: adapt idioms and expressions to sound natural in ${localeName}

**File context:** ${fileName}`;

  const userPrompt = `Translate this JSON to ${localeName}:\n\n${JSON.stringify(enContent, null, 2)}`;

  try {
    console.log(`[${targetLocale}] Translating ${fileName}...`);

    const response = await openai.chat.completions.create({
      model: 'gpt-4-turbo-preview',
      messages: [
        { role: 'system', content: systemPrompt },
        { role: 'user', content: userPrompt }
      ],
      temperature: 0.3, // Lower temperature for more consistent translations
      max_tokens: 4000
    });

    const translatedText = response.choices[0].message.content.trim();

    // Extract JSON from potential markdown code blocks
    let jsonText = translatedText;
    if (translatedText.startsWith('```')) {
      const match = translatedText.match(/```(?:json)?\n?([\s\S]*?)\n?```/);
      if (match) {
        jsonText = match[1];
      }
    }

    const translated = JSON.parse(jsonText);
    console.log(`[${targetLocale}] ✓ Successfully translated ${fileName}`);
    return translated;

  } catch (error) {
    console.error(`[${targetLocale}] ✗ Error translating ${fileName}:`, error.message);

    // If parsing fails, try to extract JSON more aggressively
    if (error instanceof SyntaxError) {
      try {
        const response = await openai.chat.completions.create({
          model: 'gpt-4-turbo-preview',
          messages: [
            { role: 'system', content: systemPrompt + '\n\nIMPORTANT: Return ONLY the JSON object, with no markdown formatting or explanations.' },
            { role: 'user', content: userPrompt }
          ],
          temperature: 0.3,
          max_tokens: 4000
        });

        const retryText = response.choices[0].message.content.trim();
        const retryTranslated = JSON.parse(retryText);
        console.log(`[${targetLocale}] ✓ Successfully translated ${fileName} (retry)`);
        return retryTranslated;
      } catch (retryError) {
        console.error(`[${targetLocale}] ✗ Retry failed for ${fileName}:`, retryError.message);
        throw retryError;
      }
    }
    throw error;
  }
}

async function translateAllFiles() {
  const enDir = path.join(process.cwd(), 'website/src/i18n/locales/en');
  const targetDir = path.join(process.cwd(), `website/src/i18n/locales/${targetLocale}`);
  const cacheDir = path.join(process.cwd(), 'website/src/i18n/locales');

  // Ensure target directory exists
  if (!fs.existsSync(targetDir)) {
    fs.mkdirSync(targetDir, { recursive: true });
    console.log(`[${targetLocale}] Created directory: ${targetDir}`);
  }

  // Load translation cache
  const cache = loadCache(cacheDir);

  // Read all JSON files from English directory
  const files = fs.readdirSync(enDir).filter(file => file.endsWith('.json'));

  if (files.length === 0) {
    console.log(`[${targetLocale}] No JSON files found in ${enDir}`);
    return;
  }

  console.log(`[${targetLocale}] Found ${files.length} files to translate`);

  let translatedCount = 0;
  let skippedCount = 0;
  let failedCount = 0;

  for (const file of files) {
    const enFilePath = path.join(enDir, file);
    const targetFilePath = path.join(targetDir, file);

    try {
      // Compute hash of English source file
      const sourceHash = computeFileHash(enFilePath);

      // Check if translation is needed
      if (!needsRetranslation(cache, targetLocale, file, sourceHash, forceRetranslate)) {
        console.log(`[${targetLocale}] ⊘ Skipping ${file} (unchanged, cached)`);
        skippedCount++;
        continue;
      }

      // Read English JSON
      const enContent = JSON.parse(fs.readFileSync(enFilePath, 'utf8'));

      // Translate
      const translated = await translateJSON(enContent, targetLocale, file);

      // Write to target locale directory
      fs.writeFileSync(targetFilePath, JSON.stringify(translated, null, 2) + '\n', 'utf8');
      console.log(`[${targetLocale}] ✓ Wrote ${file}`);

      // Update cache
      if (!cache[targetLocale]) {
        cache[targetLocale] = {};
      }
      cache[targetLocale][file] = {
        sourceHash: sourceHash,
        timestamp: new Date().toISOString()
      };

      translatedCount++;

      // Add delay to avoid rate limiting (OpenAI has rate limits)
      await new Promise(resolve => setTimeout(resolve, 1000));

    } catch (error) {
      console.error(`[${targetLocale}] ✗ Failed to process ${file}:`, error.message);
      failedCount++;
      // Continue with next file instead of crashing
      continue;
    }
  }

  // Save updated cache
  saveCache(cacheDir, cache);

  console.log(`[${targetLocale}] Translation completed!`);
  console.log(`[${targetLocale}] Summary: ${translatedCount} translated, ${skippedCount} skipped (cached), ${failedCount} failed`);
}

// Validate environment variables
if (!process.env.OPENAI_API_KEY) {
  console.error('ERROR: OPENAI_API_KEY environment variable is not set');
  process.exit(1);
}

if (!targetLocale) {
  console.error('ERROR: TARGET_LOCALE environment variable is not set');
  process.exit(1);
}

if (!localeNames[targetLocale]) {
  console.error(`ERROR: Unknown locale: ${targetLocale}`);
  console.error(`Supported locales: ${Object.keys(localeNames).join(', ')}`);
  process.exit(1);
}

// Run translation
translateAllFiles().catch(error => {
  console.error('Fatal error:', error);
  process.exit(1);
});
