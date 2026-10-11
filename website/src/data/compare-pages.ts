// Shared data for the /compare hub pages (English and localized).
// One source, so the two hubs cannot drift apart.

export interface CompareCard {
  slug: string;
  name: string;
  tagline: string;
  description: string;
  icon: string;
  keyDiff: string;
}

export const vsPages: CompareCard[] = [
  {
    slug: 'github-copilot-cli',
    name: 'GitHub Copilot CLI',
    tagline: 'Proprietary agent vs open specialist',
    description: 'Copilot CLI can run local models if you configure it. Caro is open source, offline by default, and ships a built-in safety library.',
    icon: '🐙',
    keyDiff: 'Built-in Safety'
  },
  {
    slug: 'warp',
    name: 'Warp',
    tagline: 'Open client, cloud AI vs fully local',
    description: 'Warp open-sourced its client, but every AI request goes through Warp\'s servers. Caro runs in your terminal with local models and no account.',
    icon: '🚀',
    keyDiff: 'Local AI'
  },
  {
    slug: 'kiro-cli',
    name: 'Kiro CLI',
    tagline: 'Hosted agent vs local control',
    description: 'Kiro CLI (formerly Amazon Q CLI) runs closed source on Amazon Bedrock and needs a sign-in. Caro is open, local and account-free.',
    icon: '👻',
    keyDiff: 'Self-Contained'
  },
  {
    slug: 'opencode',
    name: 'OpenCode',
    tagline: 'General coding vs shell specialist',
    description: 'Both are open source and can run local models. Caro specializes in shell commands with safety checks on by default.',
    icon: '💻',
    keyDiff: 'Safety by Default'
  },
];

// Competitor facts last checked against vendor docs on this date.
// Research notes: docs/research/2026-10-11-competitive-landscape-refresh.md
export const lastVerified = '2026-10-11';
