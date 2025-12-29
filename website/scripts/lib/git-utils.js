/**
 * Git Utilities
 *
 * Handles git operations for the content pipeline:
 * branch creation, commits, and content analysis.
 */

import { execSync } from 'child_process';
import { readFileSync, readdirSync, statSync, existsSync } from 'fs';
import { join, extname } from 'path';
import { parse as parseYaml } from 'yaml';

/**
 * Create a feature branch for content
 */
export function createBranch(contentType) {
  const date = new Date().toISOString().split('T')[0];
  const branchName = `content/${contentType}/${date}`;

  try {
    // Check if branch exists
    try {
      execSync(`git rev-parse --verify ${branchName}`, { stdio: 'pipe' });
      // Branch exists, add timestamp
      const timestamp = Date.now().toString(36);
      return createBranchWithName(`${branchName}-${timestamp}`);
    } catch {
      // Branch doesn't exist, create it
      return createBranchWithName(branchName);
    }
  } catch (error) {
    console.error(`Error creating branch: ${error.message}`);
    throw error;
  }
}

/**
 * Create branch with specific name
 */
function createBranchWithName(branchName) {
  execSync(`git checkout -b ${branchName}`, { stdio: 'inherit' });
  return branchName;
}

/**
 * Commit changes with message
 */
export async function commitChanges(message) {
  try {
    execSync('git add -A', { stdio: 'inherit' });
    execSync(`git commit -m "${message.replace(/"/g, '\\"')}"`, { stdio: 'inherit' });
    return true;
  } catch (error) {
    console.error(`Error committing changes: ${error.message}`);
    throw error;
  }
}

/**
 * Push current branch to remote
 */
export async function pushBranch() {
  try {
    const branchName = execSync('git rev-parse --abbrev-ref HEAD', { encoding: 'utf-8' }).trim();
    execSync(`git push -u origin ${branchName}`, { stdio: 'inherit' });
    return branchName;
  } catch (error) {
    console.error(`Error pushing branch: ${error.message}`);
    throw error;
  }
}

/**
 * Create pull request using GitHub CLI
 */
export async function createPullRequest({ title, body, labels = [], reviewers = [] }) {
  try {
    let command = `gh pr create --title "${title.replace(/"/g, '\\"')}"`;
    command += ` --body "${body.replace(/"/g, '\\"')}"`;

    if (labels.length > 0) {
      command += ` --label "${labels.join(',')}"`;
    }

    if (reviewers.length > 0) {
      command += ` --reviewer "${reviewers.join(',')}"`;
    }

    const result = execSync(command, { encoding: 'utf-8' });
    const prUrl = result.trim();
    return prUrl;
  } catch (error) {
    console.error(`Error creating PR: ${error.message}`);
    throw error;
  }
}

/**
 * Get existing content from directory
 */
export async function getExistingContent(contentDir, contentType) {
  const subdir = getContentSubdir(contentType);
  const dirPath = join(contentDir, subdir);

  if (!existsSync(dirPath)) {
    return [];
  }

  const files = readdirSync(dirPath).filter((f) => extname(f) === '.md' || extname(f) === '.mdx');

  const content = [];
  for (const file of files) {
    try {
      const filePath = join(dirPath, file);
      const fileContent = readFileSync(filePath, 'utf-8');
      const parsed = parseFrontmatter(fileContent);

      content.push({
        filename: file,
        path: filePath,
        ...parsed.frontmatter,
        modifiedAt: statSync(filePath).mtime,
      });
    } catch (error) {
      console.warn(`Warning: Could not parse ${file}: ${error.message}`);
    }
  }

  return content;
}

/**
 * Parse frontmatter from markdown content
 */
function parseFrontmatter(content) {
  const frontmatterMatch = content.match(/^---\n([\s\S]*?)\n---\n([\s\S]*)$/);

  if (!frontmatterMatch) {
    return { frontmatter: {}, body: content };
  }

  try {
    const frontmatter = parseYaml(frontmatterMatch[1]);
    const body = frontmatterMatch[2];
    return { frontmatter, body };
  } catch {
    return { frontmatter: {}, body: content };
  }
}

/**
 * Get content subdirectory based on type
 */
function getContentSubdir(contentType) {
  const map = {
    'daily-pick': 'daily-picks',
    command: 'commands',
    story: 'stories',
  };
  return map[contentType] || contentType;
}

/**
 * Get content gap analysis
 */
export async function analyzeContentGaps(contentDir, config) {
  const analysis = {
    commands: {
      covered: [],
      missing: [],
      byDifficulty: { beginner: 0, intermediate: 0, advanced: 0 },
      byPlatform: {},
    },
    stories: {
      covered: [],
      missing: [],
      byCategory: {},
      byEra: {},
    },
    dailyPicks: {
      total: 0,
      byType: { command: 0, tip: 0, story: 0 },
      lastPublished: null,
    },
  };

  // Analyze commands
  const commands = await getExistingContent(contentDir, 'command');
  analysis.commands.covered = commands.map((c) => c.command);

  for (const cmd of commands) {
    if (cmd.difficulty) {
      analysis.commands.byDifficulty[cmd.difficulty]++;
    }
    if (cmd.platforms) {
      for (const platform of cmd.platforms) {
        analysis.commands.byPlatform[platform] = (analysis.commands.byPlatform[platform] || 0) + 1;
      }
    }
  }

  // Analyze stories
  const stories = await getExistingContent(contentDir, 'story');
  analysis.stories.covered = stories.map((s) => s.title);

  for (const story of stories) {
    if (story.category) {
      analysis.stories.byCategory[story.category] =
        (analysis.stories.byCategory[story.category] || 0) + 1;
    }
    if (story.era) {
      analysis.stories.byEra[story.era] = (analysis.stories.byEra[story.era] || 0) + 1;
    }
  }

  // Analyze daily picks
  const dailyPicks = await getExistingContent(contentDir, 'daily-pick');
  analysis.dailyPicks.total = dailyPicks.length;

  for (const pick of dailyPicks) {
    if (pick.type) {
      analysis.dailyPicks.byType[pick.type] = (analysis.dailyPicks.byType[pick.type] || 0) + 1;
    }
  }

  if (dailyPicks.length > 0) {
    const sorted = dailyPicks.sort((a, b) => new Date(b.publishedAt) - new Date(a.publishedAt));
    analysis.dailyPicks.lastPublished = sorted[0].publishedAt;
  }

  return analysis;
}

/**
 * Check if content already exists
 */
export function contentExists(contentDir, contentType, identifier) {
  const subdir = getContentSubdir(contentType);
  const dirPath = join(contentDir, subdir);

  if (!existsSync(dirPath)) {
    return false;
  }

  const files = readdirSync(dirPath);

  // Check by filename
  const identifierSlug = identifier
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-|-$/g, '');

  return files.some((f) => {
    const fileSlug = f.replace(/\.mdx?$/, '').toLowerCase();
    return fileSlug.includes(identifierSlug);
  });
}
