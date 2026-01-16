---
name: translator
description: Culturally-aware i18n translation with specialized sub-agents for each language
user_invocable: true
---

# Translator Skill - Cultural i18n Expert

This skill provides professional, culturally-adapted translations for the Caro website using specialized sub-agents for each target language.

## Usage

```bash
# Translate specific locale
/translator es

# Translate specific file for a locale
/translator ja common.json

# Translate all files for a locale
/translator he --all

# Force retranslation (ignore cache)
/translator fr --force
```

## Arguments

- `$LOCALE` - Target locale code (es, fr, pt, de, he, ar, uk, ru, ja, ko, hi, ur, fil, id)
- `$FILE` (optional) - Specific JSON file to translate (e.g., common.json)
- `--all` - Translate all files for the locale
- `--force` - Force retranslation, ignore cache

## How It Works

This skill uses the Task tool to launch specialized sub-agents for each language. Each sub-agent is configured as a **technical writer** with deep cultural expertise in their target language and market.

### Sub-Agent Specialization

Each language has a dedicated sub-agent with:

1. **Native Cultural Context**: Deep understanding of the target metro's tech culture
2. **Pop Culture Awareness**: Knowledge of local references, idioms, and expressions
3. **Technical Writing Skills**: Ability to balance professionalism with developer-friendly tone
4. **Localization Expertise**: Knows when to translate vs. keep English technical terms

### Metro-Specific Contexts

| Locale | Metro | Cultural Focus |
|--------|-------|----------------|
| **es** | Madrid | Spanish passion, tapas culture, late-night energy |
| **fr** | Paris | French sophistication, café culture, intellectual discourse |
| **pt** | São Paulo | Brazilian warmth, carnival spirit, diverse community |
| **de** | Berlin | German precision, engineering excellence, efficiency |
| **he** | Tel Aviv | Israeli directness (dugri), startup culture, innovation |
| **ar** | Dubai | Arab hospitality, modern architecture, blend of tradition and innovation |
| **uk** | Kyiv | Ukrainian resilience, artistic tradition, independence |
| **ru** | Moscow | Russian depth, literary tradition, philosophical culture |
| **ja** | Tokyo | Japanese harmony (wa), precision, tradition meets innovation |
| **ko** | Seoul | Korean community values, rapid innovation, K-culture influence |
| **hi** | Mumbai | Indian diversity, Bollywood energy, chai culture |
| **ur** | Karachi | Urdu poetry, expressive communication, hospitality |
| **fil** | Manila | Filipino warmth, karaoke culture, family-centered values |
| **id** | Jakarta | Indonesian diversity, gotong royong (cooperation), island culture |

## Implementation

When you invoke this skill with a locale:

1. **Validation**: Check locale is supported and English source files exist
2. **Load Context**: Read cultural metadata for target locale
3. **Launch Sub-Agent**: Use Task tool with locale-specific cultural prompt
4. **Translation**: Sub-agent translates with cultural adaptation
5. **Quality Check**: Verify placeholders, brand names, technical terms preserved
6. **Cache Update**: Record successful translation to avoid retranslation

## Sub-Agent Prompt Template

Each sub-agent receives a comprehensive prompt with:

```markdown
You are a professional technical writer and translator specializing in {LANGUAGE}.

**Your Identity:**
- Native {LANGUAGE} speaker from {METRO}
- Deep understanding of {METRO} tech culture and developer community
- Expert in software localization and UI/UX writing

**Cultural Context:**
- Metro: {METRO}
- Culture: {CULTURE_DESCRIPTION}
- Pop Culture: {POP_CULTURE_REFERENCES}

**Translation Philosophy:**
- Write for developers in {METRO} who understand tech culture
- Use natural, idiomatic {LANGUAGE} that feels native, not translated
- Adapt idioms to resonate with {METRO} culture
- Keep widely-used English technical terms when standard in {METRO}
- Balance professionalism with casual developer tool tone

**Rules:**
1. Translate ONLY string values, NEVER JSON keys
2. PRESERVE placeholders: {count}, {name}, {var}
3. PRESERVE brand names: "Caro", "Claude", "GitHub"
4. PRESERVE technical terms when standard in {METRO} tech scene
5. PRESERVE code blocks and commands
6. Return valid JSON only

**Your Task:**
Translate the English JSON file to {LANGUAGE} with cultural adaptation for developers in {METRO}.
```

## Quality Standards

All translations must:

- ✅ Sound natural to native speakers from the target metro
- ✅ Resonate with local tech culture and developer community
- ✅ Use culturally-appropriate idioms and expressions
- ✅ Balance technical accuracy with local flavor
- ✅ Preserve all placeholders, brand names, technical terms
- ✅ Maintain consistent tone across all files

## Examples

### Spanish (Madrid) - Passionate & Direct

**English:**
> "Generate shell commands that work the first time"

**Translation (Madrid tech culture):**
> "Genera comandos shell que funcionan a la primera"

*Why:* Direct, confident tone matches Madrid tech culture's passionate, no-nonsense approach.

### Japanese (Tokyo) - Polite & Precise

**English:**
> "Never Nuke Production Again"

**Translation (Tokyo tech culture):**
> "本番環境を二度と破壊しない"
> (Hon-ban kankyō wo nidoto hakai shinai)

*Why:* Precise, professional Japanese that maintains the serious tone while being less casual than English.

### Hebrew (Tel Aviv) - Direct & Informal

**English:**
> "The safety net you need"

**Translation (Tel Aviv dugri culture):**
> "רשת הביטחון שאתה צריך"
> (Reshet habitachon she-ata tzarich)

*Why:* Direct, informal "you" (אתה) matches Tel Aviv's dugri (straight-talking) culture.

## Workflow Integration

This skill integrates with the existing i18n infrastructure:

1. **English content changes** → Detected by workflow or manual trigger
2. **Run /translator** → Launch culturally-aware translation
3. **Review output** → Check cultural appropriateness
4. **Commit translations** → Update locale JSON files
5. **Deploy** → Translated content goes live

## Cost & Performance

- **Cost**: Free (uses local Claude Code session)
- **Speed**: Depends on session availability
- **Quality**: Highest (cultural expertise + human review)

Best for:
- Marketing/landing page content (high cultural sensitivity)
- User-facing error messages (empathy and clarity)
- Tutorial/onboarding content (cultural context important)

Use API backends (OpenAI, Claude API) for:
- Automated CI/CD workflows
- Batch translations of many files
- Quick iterations during development
