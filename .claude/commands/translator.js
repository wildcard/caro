#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

// Parse arguments
const args = process.argv.slice(2);
const locale = args[0];
const specificFile = args.find(arg => arg.endsWith('.json'));
const translateAll = args.includes('--all');
const forceRetranslate = args.includes('--force');

// Language metadata with cultural context
const languageMetadata = {
  es: {
    name: 'Spanish',
    englishName: 'Spanish (Spain)',
    metro: 'Madrid',
    culture: 'Spanish culture with influences from Latin America; known for passionate expression, late dining culture, and vibrant street life',
    popCulture: 'Football (Real Madrid, FC Barcelona), flamenco, tapas culture, siesta tradition',
    tone: 'passionate, direct, confident',
    rtl: false
  },
  fr: {
    name: 'French',
    englishName: 'French (France)',
    metro: 'Paris',
    culture: 'French culture emphasizes elegance, sophistication, and intellectual discourse; strong tradition of art, cuisine, and philosophy',
    popCulture: 'Cinema (Nouvelle Vague), fashion, café culture, wine appreciation, Tour de France',
    tone: 'sophisticated, elegant, precise',
    rtl: false
  },
  pt: {
    name: 'Portuguese',
    englishName: 'Portuguese (Brazil)',
    metro: 'São Paulo',
    culture: 'Brazilian culture is warm, expressive, and diverse; emphasis on community, celebration, and natural beauty',
    popCulture: 'Carnival, football (soccer), samba, bossa nova, beach culture, Pelé',
    tone: 'warm, expressive, welcoming',
    rtl: false
  },
  de: {
    name: 'German',
    englishName: 'German (Germany)',
    metro: 'Berlin',
    culture: 'German culture values precision, efficiency, and engineering excellence; strong tradition of philosophy and classical music',
    popCulture: 'Beer culture, Oktoberfest, automobiles (BMW, Mercedes), electronic music, Christmas markets',
    tone: 'precise, efficient, straightforward',
    rtl: false
  },
  he: {
    name: 'Hebrew',
    englishName: 'Hebrew (Israel)',
    metro: 'Tel Aviv',
    culture: 'Israeli culture is entrepreneurial, direct, and informal (dugri culture); strong emphasis on innovation and debate',
    popCulture: 'Startup Nation, hummus culture, beach lifestyle, Eurovision, kibbutz tradition',
    tone: 'direct, informal, entrepreneurial',
    rtl: true
  },
  ar: {
    name: 'Arabic',
    englishName: 'Arabic (Modern Standard Arabic)',
    metro: 'Dubai',
    culture: 'Arab culture emphasizes hospitality, family values, and rich literary tradition; blend of ancient heritage and modern innovation',
    popCulture: 'Coffee culture, poetry, Arabic calligraphy, traditional music, modern architecture',
    tone: 'respectful, hospitable, elegant',
    rtl: true
  },
  uk: {
    name: 'Ukrainian',
    englishName: 'Ukrainian',
    metro: 'Kyiv',
    culture: 'Ukrainian culture is resilient, artistic, and deeply connected to land and traditions; strong emphasis on independence and identity',
    popCulture: 'Vyshyvanka (embroidered shirts), borscht, Cossack history, contemporary art scene',
    tone: 'resilient, proud, artistic',
    rtl: false
  },
  ru: {
    name: 'Russian',
    englishName: 'Russian',
    metro: 'Moscow',
    culture: 'Russian culture is profound, literary, and expansive; strong tradition of arts, ballet, and philosophical depth',
    popCulture: 'Ballet, literature (Dostoevsky, Tolstoy), chess, matryoshka dolls, tea culture',
    tone: 'profound, literary, philosophical',
    rtl: false
  },
  ja: {
    name: 'Japanese',
    englishName: 'Japanese',
    metro: 'Tokyo',
    culture: 'Japanese culture emphasizes harmony (wa), respect, and precision; balance of tradition and cutting-edge innovation',
    popCulture: 'Anime/manga, cherry blossoms (sakura), sushi, bullet trains (shinkansen), karaoke',
    tone: 'polite, precise, harmonious',
    rtl: false
  },
  ko: {
    name: 'Korean',
    englishName: 'Korean',
    metro: 'Seoul',
    culture: 'Korean culture values community, education, and rapid innovation; blend of Confucian values and modern technology',
    popCulture: 'K-pop, K-dramas, kimchi, PC bang gaming culture, hanbok fashion',
    tone: 'community-focused, innovative, respectful',
    rtl: false
  },
  hi: {
    name: 'Hindi',
    englishName: 'Hindi',
    metro: 'Mumbai',
    culture: 'Indian culture is diverse, vibrant, and family-oriented; strong emphasis on spirituality, festivals, and cuisine',
    popCulture: 'Bollywood, cricket, chai culture, Diwali, yoga, spices and curry',
    tone: 'warm, expressive, colorful',
    rtl: false
  },
  ur: {
    name: 'Urdu',
    englishName: 'Urdu',
    metro: 'Karachi',
    culture: 'Urdu culture is poetic, expressive, and rich in literary tradition; emphasis on hospitality and family bonds',
    popCulture: 'Poetry (ghazals, nazms), qawwali music, cricket, chai culture, kebabs',
    tone: 'poetic, expressive, warm',
    rtl: true
  },
  fil: {
    name: 'Filipino',
    englishName: 'Filipino (Tagalog)',
    metro: 'Manila',
    culture: 'Filipino culture is warm, hospitable, and family-centered; strong Catholic influence mixed with indigenous traditions',
    popCulture: 'Karaoke, basketball, jeepneys, adobo cuisine, fiestas, overseas workers (OFWs)',
    tone: 'warm, hospitable, friendly',
    rtl: false
  },
  id: {
    name: 'Indonesian',
    englishName: 'Indonesian',
    metro: 'Jakarta',
    culture: 'Indonesian culture is diverse, multicultural, and community-oriented; emphasis on harmony (gotong royong) and respect',
    popCulture: 'Gamelan music, batik, satay, motorcycle culture, island tourism (Bali)',
    tone: 'respectful, harmonious, welcoming',
    rtl: false
  }
};

// Validation
if (!locale) {
  console.error('❌ Error: No locale specified');
  console.error('Usage: /translator <locale> [file.json] [--all] [--force]');
  console.error(`Supported locales: ${Object.keys(languageMetadata).join(', ')}`);
  process.exit(1);
}

if (!languageMetadata[locale]) {
  console.error(`❌ Error: Unknown locale: ${locale}`);
  console.error(`Supported locales: ${Object.keys(languageMetadata).join(', ')}`);
  process.exit(1);
}

// Generate sub-agent prompt
function generateSubAgentPrompt(locale, metadata, fileName) {
  const rtlNote = metadata.rtl ? `
**RTL Language Note:**
${metadata.name} is written right-to-left. Your translation should:
- Flow naturally in RTL direction
- Keep technical terms and code in LTR (left-to-right)
- Use appropriate RTL punctuation and formatting` : '';

  return `You are a professional technical writer and translator specializing in ${metadata.name}.

**Your Identity:**
- Native ${metadata.name} speaker from ${metadata.metro}
- Deep understanding of ${metadata.metro} tech culture and developer community
- Expert in software localization and UI/UX writing for developer tools
- Writing tone: ${metadata.tone}

**Cultural Context:**
- **Metro:** ${metadata.metro}
- **Culture:** ${metadata.culture}
- **Pop Culture:** ${metadata.popCulture}
${rtlNote}

**Translation Philosophy:**
1. **Natural Language:** Write for developers in ${metadata.metro} who understand tech culture. Use natural, idiomatic ${metadata.name} that feels native, not translated.

2. **Cultural Adaptation:** Adapt English idioms and expressions to resonate with ${metadata.metro} culture. For example:
   - English metaphors → ${metadata.metro} cultural equivalents
   - American references → ${metadata.metro} references
   - Generic examples → ${metadata.metro}-specific examples when appropriate

3. **Technical Terms:** Keep widely-used English technical terms when they're standard in ${metadata.metro} tech scene (e.g., "API", "CLI", "shell" are often kept in tech docs).

4. **Tone Balance:** Match the casual, friendly tone of developer tools while maintaining professionalism appropriate for ${metadata.metro} culture.

**Critical Rules (NEVER VIOLATE):**
1. ✅ Translate ONLY string values - NEVER translate JSON keys
2. ✅ PRESERVE ALL placeholders exactly: \`{count}\`, \`{name}\`, \`{var}\`, etc.
3. ✅ PRESERVE brand names: "Caro", "Claude", "GitHub"
4. ✅ PRESERVE technical terms when standard in ${metadata.metro} (use your judgment)
5. ✅ PRESERVE code blocks, commands, and technical examples
6. ✅ PRESERVE emoji and special characters
7. ✅ Maintain the EXACT same JSON structure
8. ✅ Return ONLY valid JSON - no explanations, no markdown formatting

**Your Task:**
Translate the file \`${fileName}\` from English to ${metadata.name}.

The content is for "Caro" - a shell command generation tool with AI safety validation.

Target audience: Developers in ${metadata.metro} who want AI-generated shell commands they can trust.

**Process:**
1. I'll provide the English JSON content
2. You translate it with cultural adaptation for ${metadata.metro}
3. Return ONLY the translated JSON (no explanations, no markdown blocks)
4. If you're uncertain about a cultural adaptation, lean toward natural ${metadata.metro} usage

Ready to begin?`;
}

// Main execution
const metadata = languageMetadata[locale];

console.log('');
console.log('========================================');
console.log(`🌍 Translator Skill - ${metadata.englishName}`);
console.log('========================================');
console.log(`Metro:  ${metadata.metro}`);
console.log(`Tone:   ${metadata.tone}`);
console.log(`RTL:    ${metadata.rtl ? 'Yes' : 'No'}`);
console.log('========================================');
console.log('');

// Determine files to translate
const enDir = path.join(process.cwd(), 'website/src/i18n/locales/en');
let filesToTranslate = [];

if (specificFile) {
  filesToTranslate = [specificFile];
} else if (translateAll) {
  filesToTranslate = fs.readdirSync(enDir).filter(f => f.endsWith('.json'));
} else {
  console.error('❌ Error: Please specify a file or use --all flag');
  console.error('Examples:');
  console.error(`  /translator ${locale} common.json`);
  console.error(`  /translator ${locale} --all`);
  process.exit(1);
}

console.log(`Files to translate: ${filesToTranslate.join(', ')}`);
console.log('');

// Generate and display sub-agent prompt
const exampleFile = filesToTranslate[0];
const subAgentPrompt = generateSubAgentPrompt(locale, metadata, exampleFile);

console.log('========================================');
console.log('📋 Sub-Agent Prompt Generated');
console.log('========================================');
console.log('');
console.log(subAgentPrompt);
console.log('');
console.log('========================================');
console.log('🚀 Next Steps');
console.log('========================================');
console.log('');
console.log('To complete the translation:');
console.log('');
console.log('1. **Launch Sub-Agent:**');
console.log('   Use the Task tool with subagent_type="general-purpose"');
console.log('   Copy the prompt above into the task description');
console.log('');
console.log('2. **Provide English Content:**');
console.log(`   Read: website/src/i18n/locales/en/${exampleFile}`);
console.log('   Paste the JSON content to the sub-agent');
console.log('');
console.log('3. **Get Translation:**');
console.log('   Sub-agent returns translated JSON');
console.log(`   Write to: website/src/i18n/locales/${locale}/${exampleFile}`);
console.log('');
console.log('4. **Repeat for Remaining Files:**');
if (filesToTranslate.length > 1) {
  console.log(`   ${filesToTranslate.length - 1} more files to translate`);
  filesToTranslate.slice(1).forEach(f => {
    console.log(`   - ${f}`);
  });
}
console.log('');
console.log('========================================');
console.log('💡 Cultural Translation Tips');
console.log('========================================');
console.log('');
console.log(`Culture: ${metadata.culture}`);
console.log(`Pop Culture: ${metadata.popCulture}`);
console.log('');
console.log(`When translating, think about how a developer in ${metadata.metro} would naturally express these ideas.`);
console.log(`Use ${metadata.tone} tone that matches ${metadata.metro} tech culture.`);
console.log('');

// Export prompt for easy copying
fs.writeFileSync(
  `/tmp/translator-${locale}-prompt.txt`,
  subAgentPrompt,
  'utf8'
);

console.log(`✅ Prompt saved to: /tmp/translator-${locale}-prompt.txt`);
console.log('');
