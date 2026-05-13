/**
 * Caro integrations matrix — surfaces caro exposes to other coder agents,
 * agentic IDEs, and routing layers, plus the inbound backends caro can drive.
 *
 * Maintained by the **caro-integrator** nightly agent (cron `0 23 * * *`).
 * The agent's source-of-truth is `.claude/memory/integrations-status.md`;
 * this file mirrors the user-facing rows of that matrix and adds copy-paste
 * snippets the caro-integrator validates each night.
 *
 * Why all snippets live here (not inline in .astro): Astro 5.x esbuild treats
 * `{` in template content as a JSX expression. Shell snippets like `{:|:&};:`
 * crash the build. Keeping snippets in a `.ts` data file avoids the parser
 * entirely. See `.claude/rules/astro-esbuild-shell-syntax.md`.
 */

export type IntegrationStatus =
  | 'working'    // ✅ Validated end-to-end against published caro binary
  | 'partial'    // ⚠️ Works with caveats
  | 'in-progress' // 🚧 PR or epic in flight
  | 'not-yet'    // ⏳ Tracked target, no implementation yet
  | 'broken';    // ❌ Was working, now fails

export type IntegrationSurface =
  | 'claude-code-skill'
  | 'mcp-server'
  | 'openai-compat'
  | 'native-backend'
  | 'cli-shell-out'
  | 'docs-only';

export interface Integration {
  id: string;
  name: string;
  category: 'agent' | 'ide' | 'cli' | 'backend' | 'router' | 'platform';
  status: IntegrationStatus;
  surface: IntegrationSurface;
  /** YYYY-MM-DD when the caro-integrator last verified this end-to-end. */
  lastValidated: string | null;
  description: string;
  /** Copy-paste snippet for end users — kept here, never inline in .astro. */
  snippet?: string;
  /** Optional homepage / docs link. */
  homepage?: string;
  /** Linked GH issue/PR if work is in flight or planned. */
  tracking?: string;
}

/**
 * Caro-as-a-tool: surfaces other agents/IDEs use to call caro.
 */
export const outboundIntegrations: Integration[] = [
  {
    id: 'claude-code-skill',
    name: 'Claude Code (skill)',
    category: 'agent',
    status: 'working',
    surface: 'claude-code-skill',
    lastValidated: '2026-04-26',
    description:
      'Bundled skill at `.claude/skills/caro-shell/SKILL.md`. Triggers when the user asks for shell-command synthesis; shells out to the published `caro` binary with `--dry-run` and presents the safety-validated suggestion for approval.',
    snippet: [
      '# Install the caro CLI first',
      'cargo install caro',
      '',
      '# Then point Claude Code at the skill bundled in this repo:',
      '#   .claude/skills/caro-shell/SKILL.md',
      '# Claude Code auto-discovers skills from .claude/skills/ in any project.',
    ].join('\n'),
    homepage: 'https://www.anthropic.com/claude-code',
  },
  {
    id: 'claude-code-mcp',
    name: 'Claude Code (MCP server)',
    category: 'agent',
    status: 'in-progress',
    surface: 'mcp-server',
    lastValidated: null,
    description:
      'Planned `caro mcp serve` subcommand exposing `generate_command`, `validate_command`, `explain_safety`, `show_decision_tree` over the Model Context Protocol. Spec drafted in `.github/first-time-issues/06-mcp-claude-code-integration.md`.',
    tracking: 'see GitHub issues labeled `integration` + `mcp`',
  },
  {
    id: 'openai-compat-shim',
    name: 'OpenAI-compat HTTP shim',
    category: 'agent',
    status: 'in-progress',
    surface: 'openai-compat',
    lastValidated: null,
    description:
      'Planned `caro serve --openai` mode. Exposes an OpenAI Chat Completions endpoint backed by caro\'s safety-validated generation. Single integration unlocks Codex, Cursor, Continue, Aider, Tabby, and most long-tail tools that already speak OpenAI\'s schema.',
    tracking: 'see GitHub issues labeled `integration` + `openai-compat`',
  },
  {
    id: 'codex',
    name: 'OpenAI Codex',
    category: 'agent',
    status: 'not-yet',
    surface: 'openai-compat',
    lastValidated: null,
    description:
      'Will work via the OpenAI-compat shim once that ships — point Codex at `http://localhost:PORT/v1` instead of `api.openai.com`.',
    homepage: 'https://openai.com/codex',
  },
  {
    id: 'cursor',
    name: 'Cursor',
    category: 'ide',
    status: 'not-yet',
    surface: 'openai-compat',
    lastValidated: null,
    description:
      'Cursor accepts a custom OpenAI base URL in settings. Once the OpenAI-compat shim ships, set base URL to caro and enjoy validated shell-command tool calls.',
    homepage: 'https://cursor.com',
  },
  {
    id: 'continue-dev',
    name: 'Continue.dev',
    category: 'ide',
    status: 'not-yet',
    surface: 'openai-compat',
    lastValidated: null,
    description:
      'Continue (VS Code / JetBrains) supports OpenAI-compatible providers via `config.json`. OpenAI-compat shim covers it.',
    homepage: 'https://continue.dev',
  },
  {
    id: 'aider',
    name: 'Aider',
    category: 'cli',
    status: 'not-yet',
    surface: 'openai-compat',
    lastValidated: null,
    description: 'Aider speaks OpenAI; OpenAI-compat shim covers it.',
    homepage: 'https://aider.chat',
  },
  {
    id: 'tabby',
    name: 'Tabby (self-hosted)',
    category: 'platform',
    status: 'not-yet',
    surface: 'openai-compat',
    lastValidated: null,
    description: 'Self-hosted coding assistant. Point its model endpoint at the caro shim.',
    homepage: 'https://tabby.tabbyml.com',
  },
  {
    id: 'opencode',
    name: 'opencode',
    category: 'cli',
    status: 'not-yet',
    surface: 'mcp-server',
    lastValidated: null,
    description: 'Charm.sh terminal coding agent. Will integrate via MCP server or OpenAI shim.',
    homepage: 'https://opencode.ai',
  },
  {
    id: 'crush',
    name: 'crush',
    category: 'cli',
    status: 'in-progress',
    surface: 'mcp-server',
    lastValidated: null,
    description: 'Charm.sh terminal coding agent. Codex MCP server config pattern in flight.',
    tracking: 'PR #789 (Crush MCP config)',
    homepage: 'https://github.com/charmbracelet/crush',
  },
  {
    id: 'droid',
    name: 'droid',
    category: 'cli',
    status: 'not-yet',
    surface: 'mcp-server',
    lastValidated: null,
    description: 'Mobile-first agent. Plan: MCP server or OpenAI shim.',
  },
  {
    id: 'sourcegraph-amp',
    name: 'Sourcegraph Amp',
    category: 'agent',
    status: 'not-yet',
    surface: 'mcp-server',
    lastValidated: null,
    description: 'Enterprise coding agent. MCP-based integration once `caro mcp serve` ships.',
    homepage: 'https://sourcegraph.com/amp',
  },
  {
    id: 'letta',
    name: 'Letta (MemGPT)',
    category: 'agent',
    status: 'not-yet',
    surface: 'mcp-server',
    lastValidated: null,
    description: 'Persistent-memory agent runtime. Tool registration via MCP.',
    homepage: 'https://letta.com',
  },
  {
    id: 'qwen-code',
    name: 'Qwen Code',
    category: 'cli',
    status: 'not-yet',
    surface: 'openai-compat',
    lastValidated: null,
    description: 'Qwen-based coding agent; OpenAI-compat shim or native backend.',
  },
  {
    id: 'gemini-cli',
    name: 'Gemini CLI',
    category: 'cli',
    status: 'in-progress',
    surface: 'native-backend',
    lastValidated: null,
    description: 'Google\'s agentic CLI. Caro-side Gemini backend in flight.',
    tracking: 'PR #782 (Gemini integration workflows)',
    homepage: 'https://github.com/google-gemini/gemini-cli',
  },
  {
    id: 'jules',
    name: 'Jules (Google)',
    category: 'agent',
    status: 'not-yet',
    surface: 'native-backend',
    lastValidated: null,
    description: 'Google\'s async coding agent. Coordinate with the Gemini backend work.',
    tracking: 'PR #782',
  },
];

/**
 * Inbound backends: providers caro can drive for inference.
 */
export const inboundBackends: Integration[] = [
  {
    id: 'claude-api',
    name: 'Anthropic Claude API',
    category: 'backend',
    status: 'in-progress',
    surface: 'native-backend',
    lastValidated: null,
    description: 'Backend implementation exists at `src/backends/remote/claude.rs` (`ClaudeBackend`, `BackendType::Claude`), but the CLI does not yet route `--backend claude` to it — the published binary returns `Unknown backend \'claude\'`. Wiring is the missing piece.',
  },
  {
    id: 'ollama',
    name: 'Ollama',
    category: 'backend',
    status: 'partial',
    surface: 'native-backend',
    lastValidated: '2026-05-11',
    description: 'Local or remote Ollama server. Implementation lives in `src/backends/remote/ollama.rs` behind the non-default `remote-backends` Cargo feature. The default `cargo install caro` build and the published release binaries ship without this feature, so `--backend ollama` logs a `WARN` ("Remote backends not compiled in") and silently falls back to the embedded backend. To get the real Ollama backend, build from source with `cargo install caro --features remote-backends`.',
    snippet: [
      '# Default install does NOT include remote backends:',
      'cargo install caro --features remote-backends',
      '',
      'ollama pull qwen2.5-coder:7b',
      'caro --backend ollama --model qwen2.5-coder:7b "tar this dir excluding .git"',
    ].join('\n'),
  },
  {
    id: 'vllm',
    name: 'vLLM',
    category: 'backend',
    status: 'partial',
    surface: 'native-backend',
    lastValidated: '2026-05-11',
    description: 'Self-hosted vLLM server (OpenAI-compatible). `src/backends/remote/vllm.rs`. Same caveat as Ollama: only available when caro is built with `--features remote-backends`; default install silently falls back to embedded.',
  },
  {
    id: 'exo',
    name: 'Exo',
    category: 'backend',
    status: 'partial',
    surface: 'native-backend',
    lastValidated: '2026-05-11',
    description: 'Distributed local inference across your devices. `src/backends/remote/exo.rs`. Same caveat as Ollama: only available when caro is built with `--features remote-backends`; default install silently falls back to embedded.',
    homepage: 'https://github.com/exo-explore/exo',
  },
  {
    id: 'mlx-embedded',
    name: 'MLX (embedded, Apple Silicon)',
    category: 'backend',
    status: 'working',
    surface: 'native-backend',
    lastValidated: '2026-05-11',
    description: 'On-device inference via MLX + llama.cpp. Default on macOS arm64. No network. Ships in the default `cargo install caro` build.',
    snippet: 'caro --backend embedded "kill the process on port 3000"',
  },
  {
    id: 'candle-cpu',
    name: 'Candle CPU (embedded)',
    category: 'backend',
    status: 'working',
    surface: 'native-backend',
    lastValidated: '2026-05-11',
    description: 'Cross-platform CPU inference via Candle. Slower but works everywhere. Ships in the default `cargo install caro` build.',
  },
  {
    id: 'openrouter',
    name: 'OpenRouter (incl. `auto`)',
    category: 'router',
    status: 'not-yet',
    surface: 'native-backend',
    lastValidated: null,
    description: 'Single backend → 100+ models including OpenRouter\'s `auto` routing. Planned: clones the vLLM backend shape.',
  },
  {
    id: 'claude-code-session',
    name: 'Claude Code session token',
    category: 'backend',
    status: 'not-yet',
    surface: 'cli-shell-out',
    lastValidated: null,
    description: 'Reuse the user\'s active Claude Code authentication (subscription or API). Default to Haiku for cheap fast inference.',
  },
];

export const allIntegrations: Integration[] = [
  ...outboundIntegrations,
  ...inboundBackends,
];

export const statusLabel: Record<IntegrationStatus, string> = {
  working: '✅ Working',
  partial: '⚠️ Partial',
  'in-progress': '🚧 In progress',
  'not-yet': '⏳ Planned',
  broken: '❌ Broken',
};

export const surfaceLabel: Record<IntegrationSurface, string> = {
  'claude-code-skill': 'Claude Code skill',
  'mcp-server': 'MCP server',
  'openai-compat': 'OpenAI-compat HTTP',
  'native-backend': 'Native backend',
  'cli-shell-out': 'CLI shell-out',
  'docs-only': 'Docs only',
};
