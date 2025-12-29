/**
 * Content Writer Agent
 *
 * Generates full markdown content from a brief using Claude.
 * Outputs properly formatted content with YAML frontmatter.
 */

import Anthropic from '@anthropic-ai/sdk';
import slugify from 'slugify';
import readingTime from 'reading-time';

const anthropic = new Anthropic();

/**
 * Generate full content from a brief
 */
export async function writeContent({ brief, contentType, config }) {
  const prompt = buildWriterPrompt(brief, contentType, config);

  try {
    const response = await anthropic.messages.create({
      model: 'claude-sonnet-4-20250514',
      max_tokens: 8192,
      messages: [
        {
          role: 'user',
          content: prompt,
        },
      ],
    });

    const rawContent = response.content[0].text;

    // Extract markdown from response
    const markdown = extractMarkdown(rawContent);

    // Generate filename
    const filename = generateFilename(brief, contentType);

    // Calculate reading time
    const stats = readingTime(markdown);

    return {
      markdown,
      filename,
      readingTime: Math.ceil(stats.minutes),
      wordCount: stats.words,
      brief,
    };

  } catch (error) {
    console.error('Content writer error:', error.message);
    throw error;
  }
}

/**
 * Build the prompt for the content writer
 */
function buildWriterPrompt(brief, contentType, config) {
  const today = new Date().toISOString().split('T')[0];

  const basePrompt = `You are an expert technical writer for Caro Learn, creating educational content about Unix/shell commands and history.

## Writing Guidelines
- Write in a clear, engaging, conversational tone
- Use practical examples that readers can try immediately
- Include historical context where relevant
- Keep explanations accessible to the target difficulty level
- Use proper markdown formatting
- Include code blocks with syntax highlighting (\`\`\`bash)

## Brief
${JSON.stringify(brief, null, 2)}

## Today's Date
${today}

`;

  switch (contentType) {
    case 'command':
      return basePrompt + buildCommandPrompt(brief);
    case 'story':
      return basePrompt + buildStoryPrompt(brief);
    case 'daily-pick':
      return basePrompt + buildDailyPickPrompt(brief);
    default:
      throw new Error(`Unknown content type: ${contentType}`);
  }
}

/**
 * Build command tutorial prompt
 */
function buildCommandPrompt(brief) {
  return `
## Output Format

Generate a complete command tutorial with YAML frontmatter. Follow this exact structure:

\`\`\`markdown
---
title: "${brief.title}"
command: "${brief.command}"
description: "${brief.description}"
difficulty: "${brief.difficulty}"
platforms: ${JSON.stringify(brief.platforms)}
tags: ${JSON.stringify(brief.tags || ['cli'])}
publishedAt: YYYY-MM-DD
featured: false
relatedCommands: ${JSON.stringify(brief.relatedCommands || [])}
caroPrompt: "A natural language prompt users could try with Caro"
---

# ${brief.command}: [Catchy Subtitle]

## Quick Summary

[1-2 sentences explaining what this command does]

## The ${brief.keyExamples?.length || 3} Commands You'll Actually Use

### 1. [First practical example]
\`\`\`bash
[command example]
\`\`\`
[Brief explanation]

### 2. [Second practical example]
\`\`\`bash
[command example]
\`\`\`
[Brief explanation]

[Continue for each key example...]

## Deep Dive

[Detailed explanation of how the command works, key options, etc.]

## Real-World Examples

[3-5 practical scenarios with full command examples]

## Caro Connection

Ask Caro:
> "[Natural language prompt]"

Caro suggests:
\`\`\`bash
[Generated command]
\`\`\`

## Common Pitfalls

[2-3 common mistakes and how to avoid them]

## Platform Notes

[Any differences between Linux, macOS, BSD]

---

*[Closing line with call to action]*
\`\`\`

Generate the complete tutorial now. Make it practical, engaging, and educational.
Replace YYYY-MM-DD with today's date in the frontmatter.`;
}

/**
 * Build story prompt
 */
function buildStoryPrompt(brief) {
  return `
## Output Format

Generate a complete Unix history story with YAML frontmatter. Follow this structure:

\`\`\`markdown
---
title: "${brief.title}"
subtitle: "${brief.subtitle || ''}"
category: "${brief.category}"
era: "${brief.era}"
publishedAt: YYYY-MM-DD
featured: false
readingTime: [calculated]
author: "Caro Team"
sources:
  - title: "[Source Title]"
    url: "[Source URL]"
tags: ${JSON.stringify(brief.tags || [])}
---

# ${brief.title}

[Compelling opening hook - 2-3 sentences that draw the reader in]

## [First Major Section]

[Content with historical context, quotes, and narrative flow]

\`\`\`c
// Code example if relevant
[historical code snippet]
\`\`\`

## [Second Major Section]

[Continue the narrative]

## [Third Major Section]

[Build to the key insight or conclusion]

## Modern Relevance

[How this connects to today's development practices]

## Try It With Caro

Ask Caro:
> "[Related prompt]"

Caro suggests:
\`\`\`bash
[Related command]
\`\`\`

---

*[Memorable closing quote or thought]*
\`\`\`

Generate the complete story now. Make it engaging, historically accurate, and insightful.
The reading time should be approximately ${brief.readingTime || 5} minutes.
Replace YYYY-MM-DD with today's date.`;
}

/**
 * Build daily pick prompt
 */
function buildDailyPickPrompt(brief) {
  const today = new Date().toISOString().split('T')[0];
  const slug = `${today}-${slugify(brief.title, { lower: true, strict: true })}`;

  return `
## Output Format

Generate a complete daily pick with YAML frontmatter. Follow this structure:

\`\`\`markdown
---
title: "${brief.title}"
type: "${brief.type}"
publishedAt: ${today}
socialText: |
  [Twitter-ready text - max 280 characters including hashtags]

  ${brief.hashtags?.map(h => '#' + h).join(' ') || '#unix #cli #caro'}
hashtags: ${JSON.stringify(brief.hashtags || ['unix', 'cli', 'caro'])}
source: "${brief.source || 'Unix essentials'}"
---

# ${brief.title}

${brief.hook || '[Opening hook]'}

## ${brief.type === 'command' ? 'The Command' : brief.type === 'tip' ? 'The Tip' : 'The Story'}

${brief.type === 'command' ? `
\`\`\`bash
${brief.command || '[command]'}
\`\`\`
` : ''}

[Main content - 100-200 words explaining the concept]

${brief.type === 'command' ? `
## Quick Examples

\`\`\`bash
# Example 1
[command example]

# Example 2
[command example]
\`\`\`
` : ''}

---

*Ask Caro: "${brief.caroPrompt || 'Help me with this'}"*
\`\`\`

Generate the complete daily pick now. Keep it concise but valuable.
The social text MUST be under 280 characters total.`;
}

/**
 * Extract markdown from response
 */
function extractMarkdown(rawContent) {
  // Try to extract from code block first
  const codeBlockMatch = rawContent.match(/```markdown\n?([\s\S]*?)\n?```/);
  if (codeBlockMatch) {
    return codeBlockMatch[1].trim();
  }

  // Check if the content starts with frontmatter
  if (rawContent.trim().startsWith('---')) {
    return rawContent.trim();
  }

  // Return as-is if no clear format
  return rawContent.trim();
}

/**
 * Generate filename for content
 */
function generateFilename(brief, contentType) {
  const today = new Date().toISOString().split('T')[0];

  switch (contentType) {
    case 'command':
      return `${slugify(brief.command, { lower: true, strict: true })}.md`;

    case 'story':
      return `${slugify(brief.title, { lower: true, strict: true })}.md`;

    case 'daily-pick':
      const shortTitle = brief.title.split(':')[0].trim();
      return `${today}-${slugify(shortTitle, { lower: true, strict: true })}.md`;

    default:
      return `${today}-${slugify(brief.title, { lower: true, strict: true })}.md`;
  }
}
