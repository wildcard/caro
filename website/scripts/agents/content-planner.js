/**
 * Content Planner Agent
 *
 * Analyzes existing content to identify gaps and generates
 * content briefs for new items.
 */

import Anthropic from '@anthropic-ai/sdk';
import { UNIX_COMMANDS, STORY_TOPICS, DAILY_PICK_TYPES } from '../lib/content-database.js';

const anthropic = new Anthropic();

/**
 * Generate content briefs based on gaps analysis
 */
export async function planContent({ contentType, existingContent, count, config }) {
  const existingTitles = existingContent.map(c => c.title?.toLowerCase() || '');
  const existingCommands = existingContent.map(c => c.command?.toLowerCase() || '');

  // Build context about what already exists
  const existingContext = buildExistingContext(contentType, existingContent);

  // Get candidates based on content type
  const candidates = getCandidates(contentType, existingContent);

  // Use Claude to select and brief the best candidates
  const prompt = buildPlannerPrompt(contentType, existingContext, candidates, count, config);

  try {
    const response = await anthropic.messages.create({
      model: 'claude-sonnet-4-20250514',
      max_tokens: 4096,
      messages: [
        {
          role: 'user',
          content: prompt,
        },
      ],
    });

    const content = response.content[0].text;

    // Parse JSON response
    const jsonMatch = content.match(/```json\n?([\s\S]*?)\n?```/) ||
                      content.match(/\[[\s\S]*\]/);

    if (!jsonMatch) {
      console.error('Failed to parse planner response:', content);
      return [];
    }

    const briefs = JSON.parse(jsonMatch[1] || jsonMatch[0]);

    // Validate and enrich briefs
    return briefs.map(brief => enrichBrief(brief, contentType));

  } catch (error) {
    console.error('Content planner error:', error.message);
    throw error;
  }
}

/**
 * Build context string about existing content
 */
function buildExistingContext(contentType, existingContent) {
  if (existingContent.length === 0) {
    return 'No existing content yet.';
  }

  switch (contentType) {
    case 'command':
      const commands = existingContent.map(c => c.command).filter(Boolean);
      const difficulties = existingContent.reduce((acc, c) => {
        acc[c.difficulty] = (acc[c.difficulty] || 0) + 1;
        return acc;
      }, {});
      return `Existing commands (${commands.length}): ${commands.join(', ')}
Difficulty distribution: ${JSON.stringify(difficulties)}`;

    case 'story':
      const categories = existingContent.reduce((acc, c) => {
        acc[c.category] = (acc[c.category] || 0) + 1;
        return acc;
      }, {});
      const eras = existingContent.map(c => c.era).filter(Boolean);
      return `Existing stories (${existingContent.length})
Categories: ${JSON.stringify(categories)}
Eras covered: ${[...new Set(eras)].join(', ')}`;

    case 'daily-pick':
      const types = existingContent.reduce((acc, c) => {
        acc[c.type] = (acc[c.type] || 0) + 1;
        return acc;
      }, {});
      const recentTitles = existingContent.slice(-10).map(c => c.title);
      return `Existing picks (${existingContent.length})
Type distribution: ${JSON.stringify(types)}
Recent titles: ${recentTitles.join(', ')}`;

    default:
      return `Existing items: ${existingContent.length}`;
  }
}

/**
 * Get candidate topics based on content type
 */
function getCandidates(contentType, existingContent) {
  switch (contentType) {
    case 'command':
      const existingCommands = new Set(existingContent.map(c => c.command?.toLowerCase()));
      return UNIX_COMMANDS.filter(cmd => !existingCommands.has(cmd.name.toLowerCase()));

    case 'story':
      const existingTopics = new Set(existingContent.map(c => c.title?.toLowerCase()));
      return STORY_TOPICS.filter(topic =>
        !existingTopics.has(topic.title.toLowerCase())
      );

    case 'daily-pick':
      return DAILY_PICK_TYPES;

    default:
      return [];
  }
}

/**
 * Build the prompt for the planner
 */
function buildPlannerPrompt(contentType, existingContext, candidates, count, config) {
  const basePrompt = `You are a content planner for Caro Learn, a Unix/shell educational platform.
Your job is to select the best topics for new content and create detailed briefs.

## Current Content State
${existingContext}

## Content Type: ${contentType}

## Available Candidates
${JSON.stringify(candidates.slice(0, 50), null, 2)}

## Requirements
- Generate exactly ${count} content brief(s)
- Ensure variety in difficulty levels and themes
- Prioritize topics that fill gaps in existing coverage
- Consider seasonal relevance and trending topics

`;

  switch (contentType) {
    case 'command':
      return basePrompt + `
## Command Brief Format
Each brief should include:
- command: The Unix command name
- title: Full title (e.g., "awk: The Pattern Scanning Powerhouse")
- description: 1-2 sentence SEO description
- difficulty: beginner | intermediate | advanced
- platforms: Array of [linux, macos, bsd, posix]
- tags: Array of relevant tags (pipeline, text-processing, automation, etc.)
- learningObjective: What users will learn
- keyExamples: 3-5 practical examples to include
- relatedCommands: Array of related command names

Respond with a JSON array of briefs:
\`\`\`json
[
  {
    "command": "...",
    "title": "...",
    ...
  }
]
\`\`\``;

    case 'story':
      return basePrompt + `
## Story Brief Format
Each brief should include:
- title: Compelling story title
- subtitle: Optional tagline
- category: history | people | technology | culture | platform
- era: Time period (e.g., "1970s", "1990s-2000s", "Modern")
- synopsis: 2-3 sentence story summary
- keyPoints: 3-5 main points to cover
- sources: Suggested source URLs to reference
- tags: Array of relevant tags

Respond with a JSON array of briefs:
\`\`\`json
[
  {
    "title": "...",
    "category": "...",
    ...
  }
]
\`\`\``;

    case 'daily-pick':
      return basePrompt + `
## Daily Pick Brief Format
Each brief should include:
- title: Catchy, short title
- type: command | tip | trivia | quote | error
- hook: The compelling opening line
- command: (if type=command) The command to feature
- keyTakeaway: Main learning point
- hashtags: Array of 3-5 hashtags (without #)

Respond with a JSON array of briefs:
\`\`\`json
[
  {
    "title": "...",
    "type": "...",
    ...
  }
]
\`\`\``;

    default:
      return basePrompt + '\nRespond with a JSON array of content briefs.';
  }
}

/**
 * Enrich and validate a brief
 */
function enrichBrief(brief, contentType) {
  // Add timestamp
  brief.generatedAt = new Date().toISOString();

  // Add default values based on content type
  switch (contentType) {
    case 'command':
      brief.platforms = brief.platforms || ['linux', 'macos', 'bsd', 'posix'];
      brief.difficulty = brief.difficulty || 'intermediate';
      break;

    case 'story':
      brief.readingTime = brief.readingTime || 5;
      brief.author = brief.author || 'Caro Team';
      break;

    case 'daily-pick':
      brief.hashtags = brief.hashtags || ['unix', 'cli', 'caro'];
      break;
  }

  return brief;
}
