## Translation Backends - Complete Guide

The Caro i18n translation system supports **four backends** for maximum flexibility:

1. **OpenAI GPT-4** - High quality, automated, API-based
2. **LibreTranslate** - Open source, self-hosted, privacy-focused
3. **Claude API** - Culturally-aware, context-rich, API-based
4. **Translator Skill** - Interactive, expert-driven, local Claude Code

## Backend Comparison

| Feature | OpenAI | LibreTranslate | Claude API | Translator Skill |
|---------|--------|----------------|------------|------------------|
| **Quality** | ★★★★★ | ★★★☆☆ | ★★★★★ | ★★★★★★ |
| **Cultural Adaptation** | ★★★★☆ | ★★☆☆☆ | ★★★★★ | ★★★★★★ |
| **Speed** | Fast | Very Fast | Fast | Slow (interactive) |
| **Cost** | ~$4-7/run | Free (self-hosted) | ~$5-8/run | Free (local) |
| **Privacy** | Cloud API | Self-hosted | Cloud API | 100% Local |
| **Automation** | Yes | Yes | Yes | No (interactive) |
| **Best For** | CI/CD, batch | Privacy, high volume | Marketing content | Landing pages, critical UX |

## Backend Details

### 1. OpenAI GPT-4 (Default)

**Strengths:**
- Excellent translation quality
- Good at preserving technical terms
- Fast API responses
- Reliable JSON formatting

**Configuration:**
```bash
export TRANSLATION_BACKEND="openai"
export OPENAI_API_KEY="sk-proj-..."
```

**Cost:** ~$0.30-0.50 per locale per file (~$4-7 for full 14-locale run)

**Use When:**
- Running automated translations in CI/CD
- Need fast, reliable batch translations
- Quality matters more than cultural nuance

### 2. LibreTranslate (Open Source)

**Strengths:**
- 100% open source and self-hostable
- Privacy-focused (no data sent to third parties)
- Free to use (public instance or self-host)
- Very fast translations

**Weaknesses:**
- Lower quality than LLM-based backends
- Less cultural adaptation
- May struggle with technical jargon
- Limited context understanding

**Configuration:**
```bash
export TRANSLATION_BACKEND="libretranslate"
export LIBRETRANSLATE_URL="https://libretranslate.com"  # or your instance
export LIBRETRANSLATE_API_KEY=""  # optional, for rate limiting
```

**Self-Hosting:**
```bash
# Docker
docker run -d -p 5000:5000 libretranslate/libretranslate

# Then use
export LIBRETRANSLATE_URL="http://localhost:5000"
```

**Cost:** Free (public instance has rate limits; self-host for unlimited)

**Use When:**
- Privacy is critical (medical, financial, government)
- High translation volume (thousands of strings)
- Budget constraints
- Want to self-host everything

### 3. Claude API (Anthropic)

**Strengths:**
- Excellent cultural understanding
- Deep context awareness
- Great at adapting idioms
- Natural-sounding translations
- Understands metro-specific culture

**Cultural Context:**
Each translation includes metro-specific cultural context (Madrid, Paris, Tokyo, Tel Aviv, etc.) to ensure translations resonate with local developers.

**Configuration:**
```bash
export TRANSLATION_BACKEND="claude"
export ANTHROPIC_API_KEY="sk-ant-..."
export CLAUDE_MODEL="claude-sonnet-4-5-20250929"  # optional
```

**Cost:** ~$0.35-0.60 per locale per file (~$5-8 for full 14-locale run)

**Use When:**
- Translating marketing/landing pages
- Cultural adaptation is critical
- Target audience is specific metro (Madrid developers, Tokyo devs, etc.)
- Quality and naturalness matter more than speed

### 4. Translator Skill (Interactive)

**Strengths:**
- **Highest quality** - Human-in-the-loop with AI assistance
- **Deep cultural expertise** - Specialized sub-agents per language
- **Metro-specific** - Each language knows its major metro's culture
- **Pop culture aware** - References local idioms, trends, expressions
- **100% local** - No API calls, uses local Claude Code session
- **Free** - No API costs

**How It Works:**
1. You run `/translator <locale>` in Claude Code
2. Skill launches a specialized sub-agent for that language
3. Sub-agent is configured as a technical writer from the target metro
4. You provide English content, sub-agent returns culturally-adapted translation
5. You review and commit the translation

**Metro-Specific Sub-Agents:**

| Locale | Metro | Cultural Specialization |
|--------|-------|-------------------------|
| **es** | Madrid | Passionate Spanish expression, tapas culture, football references |
| **fr** | Paris | Sophisticated French elegance, café culture, intellectual discourse |
| **pt** | São Paulo | Warm Brazilian expressiveness, carnival spirit, diverse community |
| **de** | Berlin | German precision, engineering excellence, efficiency |
| **he** | Tel Aviv | Israeli directness (dugri culture), startup innovation, beach lifestyle |
| **ar** | Dubai | Arab hospitality, modern architecture, tradition + innovation |
| **uk** | Kyiv | Ukrainian resilience, artistic tradition, independence |
| **ru** | Moscow | Russian literary depth, ballet, philosophical culture |
| **ja** | Tokyo | Japanese harmony (wa), precision, tradition meets tech |
| **ko** | Seoul | Korean community values, K-pop/K-drama, rapid innovation |
| **hi** | Mumbai | Indian Bollywood energy, chai culture, diverse festivals |
| **ur** | Karachi | Urdu poetry, expressive communication, cricket culture |
| **fil** | Manila | Filipino warmth, karaoke culture, family-centered values |
| **id** | Jakarta | Indonesian diversity, gotong royong, island culture |

**Usage:**
```bash
# Single file
/translator es common.json

# All files
/translator ja --all

# Force retranslation
/translator he --force
```

**Cost:** Free (uses your local Claude Code session)

**Use When:**
- Translating landing pages (high cultural sensitivity)
- User-facing error messages (empathy and clarity critical)
- Tutorial/onboarding content (cultural context important)
- Marketing materials (local references and idioms matter)
- You have time for interactive review

## Backend Selection Guide

### Quick Decision Tree

```
Need automation? ───No──> Use Translator Skill (interactive, highest quality)
       │
      Yes
       │
       ├─ Privacy critical? ───Yes──> LibreTranslate (self-host)
       │          │
       │         No
       │
       ├─ Marketing content? ───Yes──> Claude API (cultural adaptation)
       │          │
       │         No
       │
       └─────────> OpenAI (default, good balance)
```

### By Use Case

**Automated CI/CD Pipeline:**
- **First choice:** OpenAI (reliable, fast, good quality)
- **Second choice:** Claude API (if budget allows, better quality)
- **Third choice:** LibreTranslate (if privacy/budget critical)

**Landing Page Translation:**
- **First choice:** Translator Skill (interactive, cultural expertise)
- **Second choice:** Claude API (automated, culturally-aware)
- **Third choice:** OpenAI (still good, less cultural nuance)

**High-Volume Technical Docs:**
- **First choice:** LibreTranslate (free, fast)
- **Second choice:** OpenAI (better quality, reasonable cost)

**Privacy-Sensitive Content:**
- **First choice:** Translator Skill (100% local)
- **Second choice:** LibreTranslate (self-host)

## Testing Each Backend

### Test OpenAI Backend

```bash
export TRANSLATION_BACKEND="openai"
export OPENAI_API_KEY="sk-proj-..."
export TARGET_LOCALE="es"

node .github/scripts/translate-multi-backend.js
```

### Test LibreTranslate Backend

```bash
export TRANSLATION_BACKEND="libretranslate"
export LIBRETRANSLATE_URL="https://libretranslate.com"
export TARGET_LOCALE="fr"

node .github/scripts/translate-multi-backend.js
```

### Test Claude API Backend

```bash
export TRANSLATION_BACKEND="claude"
export ANTHROPIC_API_KEY="sk-ant-..."
export TARGET_LOCALE="ja"

node .github/scripts/translate-multi-backend.js
```

### Test Translator Skill

```bash
# In Claude Code
/translator he common.json

# Or for all files
/translator ko --all
```

## GitHub Actions Backend Selection

### Via Workflow Dispatch UI

1. Go to: **Actions** → **Translate Website Content** → **Run workflow**
2. Select branch: `main`
3. **Select backend:** Choose from dropdown (openai, libretranslate, claude)
4. Check `force_retranslate` if needed
5. Click **Run workflow**

### Via GitHub CLI

```bash
# OpenAI (default)
gh workflow run translate.yml

# LibreTranslate
gh workflow run translate.yml -f backend=libretranslate

# Claude API
gh workflow run translate.yml -f backend=claude -f force_retranslate=true
```

## Configuration Secrets

Add these secrets in **GitHub Repository Settings** → **Secrets and variables** → **Actions**:

### Required for OpenAI Backend
- `OPENAI_API_KEY` - Your OpenAI API key (sk-proj-...)

### Required for Claude API Backend
- `ANTHROPIC_API_KEY` - Your Anthropic API key (sk-ant-...)

### Optional for LibreTranslate Backend
- `LIBRETRANSLATE_URL` - Custom instance URL (defaults to https://libretranslate.com)
- `LIBRETRANSLATE_API_KEY` - API key if using rate-limited instance

## Cost Comparison

**For 14 locales × 8 JSON files = 112 file translations:**

| Backend | First Run | Incremental (1 file changed) |
|---------|-----------|------------------------------|
| OpenAI | ~$4-7 | ~$0.30-0.50 per locale |
| LibreTranslate | Free | Free |
| Claude API | ~$5-8 | ~$0.35-0.60 per locale |
| Translator Skill | Free | Free |

**With Intelligent Caching:**
- Only changed files are retranslated
- Saves 80-90% of API costs during iterative development
- Cache invalidation based on MD5 hash of source files

## Quality Comparison

### Sample Translation: "Never Nuke Production Again"

**English Original:**
> "Never Nuke Production Again"

**OpenAI (Spanish - es):**
> "Nunca Destruyas Producción de Nuevo"
> *Rating: ★★★★☆ - Good, but formal*

**LibreTranslate (Spanish - es):**
> "Nunca Nuke Producción Otra Vez"
> *Rating: ★★☆☆☆ - Literal, kept "Nuke" untranslated*

**Claude API (Spanish - es, Madrid context):**
> "Nunca Vuelvas a Cargarte Producción"
> *Rating: ★★★★★ - Natural, Madrid slang "cargarte" (mess up)*

**Translator Skill (Spanish - es, Madrid context):**
> "No Vuelvas a Liarla en Producción"
> *Rating: ★★★★★★ - Idiomatic Madrid expression "liarla" (screw up), very natural*

### Sample Translation: RTL Example (Hebrew)

**English Original:**
> "The safety net you need"

**OpenAI (Hebrew - he):**
> "רשת הביטחון שאתה צריך"
> *Rating: ★★★★☆ - Correct but could be more natural*

**LibreTranslate (Hebrew - he):**
> "רשת הבטיחות שאתה צריך"
> *Rating: ★★★☆☆ - Uses "betichut" (physical safety) not "bitachon" (security)*

**Claude API (Hebrew - he, Tel Aviv context):**
> "רשת הביטחון שאתה צריך"
> *Rating: ★★★★★ - Natural Tel Aviv Hebrew, informal "you" (אתה)*

**Translator Skill (Hebrew - he, Tel Aviv dugri culture):**
> "הכלי שיציל אותך"
> *Rating: ★★★★★★ - Ultra-direct Tel Aviv style: "The tool that will save you"*

## Recommendations

### For Most Projects
**Use OpenAI** for automated CI/CD with good quality/cost balance.

### For Privacy-Critical Projects
**Use LibreTranslate** self-hosted for complete data control.

### For Marketing-Heavy Sites
**Use Claude API** for automated culturally-aware translations.

### For Landing Pages & Critical UX
**Use Translator Skill** for highest quality, culturally-perfect translations.

### Hybrid Approach (Recommended)
1. **Automated CI/CD:** OpenAI or LibreTranslate for technical content
2. **Manual Review:** Translator Skill for landing page, hero sections, CTAs
3. **Incremental Updates:** Caching ensures only changed content is retranslated

## Troubleshooting

### OpenAI Backend Issues

**Error: `OPENAI_API_KEY environment variable is required`**
- Fix: Set `OPENAI_API_KEY` in environment or GitHub secrets

**Error: `Rate limit exceeded`**
- Fix: Add delays in script (already has 1s delay)
- Workaround: Reduce `max-parallel` in workflow to 1-2

### LibreTranslate Backend Issues

**Error: `LibreTranslate API error: 403`**
- Fix: Public instance may have rate limits - use API key or self-host
- Workaround: Set `LIBRETRANSLATE_API_KEY` environment variable

**Error: `Unknown locale: fil`**
- Fix: LibreTranslate maps Filipino (fil) to Tagalog (tl) automatically
- Check: Script handles locale code mapping

### Claude API Backend Issues

**Error: `ANTHROPIC_API_KEY environment variable is required`**
- Fix: Set `ANTHROPIC_API_KEY` in environment or GitHub secrets

**Error: `Claude API error: 529`**
- Fix: API temporarily overloaded - retry after a few minutes
- Workaround: Use OpenAI or LibreTranslate temporarily

### Translator Skill Issues

**Error: `Skill backend requires interactive Claude Code session`**
- Fix: This backend only works in interactive Claude Code, not in automated workflows
- Workaround: Use OpenAI, Claude API, or LibreTranslate for automation

## Advanced Configuration

### Custom Backend Implementation

You can add your own backend by extending the `TranslationBackend` class:

```javascript
class MyCustomBackend extends TranslationBackend {
  async initialize() {
    // Setup logic
  }

  getName() {
    return 'My Custom Backend';
  }

  async translate(enContent, locale, fileName) {
    // Translation logic
    return translatedJSON;
  }
}
```

Then add to the factory:

```javascript
case 'mycustom':
  return new MyCustomBackend();
```

### Per-Locale Backend Selection

For advanced workflows, you can use different backends for different locales:

```bash
# Spanish with Claude API (marketing-heavy)
TRANSLATION_BACKEND=claude TARGET_LOCALE=es node .github/scripts/translate-multi-backend.js

# French with OpenAI (technical docs)
TRANSLATION_BACKEND=openai TARGET_LOCALE=fr node .github/scripts/translate-multi-backend.js

# All others with LibreTranslate (privacy)
for locale in de he ar uk ru ja ko hi ur fil id; do
  TRANSLATION_BACKEND=libretranslate TARGET_LOCALE=$locale node .github/scripts/translate-multi-backend.js
done
```

## Next Steps

1. **Choose your backend** based on the decision tree above
2. **Set up API keys** in GitHub secrets or local environment
3. **Test locally** with your chosen backend
4. **Run automated translations** via GitHub Actions
5. **Review translations** for quality and cultural appropriateness
6. **Iterate** - use different backends for different content types
