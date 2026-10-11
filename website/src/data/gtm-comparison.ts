// Copilot and shell-gpt columns last checked on 2026-10-11 against GitHub's
// docs and the TheR1D/shell_gpt README. See
// docs/research/2026-10-11-competitive-landscape-refresh.md
export const comparisons = [
  {
    feature: "Privacy",
    caro: "100% local processing",
    chatgpt: "Sends all prompts to cloud",
    copilot: "GitHub-hosted models by default; local models opt-in",
    shellgpt: "OpenAI by default; Ollama via LiteLLM"
  },
  {
    feature: "Safety Validation",
    caro: "52+ dangerous patterns blocked",
    chatgpt: "No validation",
    copilot: "Your own allow/deny rules; no built-in list",
    shellgpt: "Execute/abort prompt only"
  },
  {
    feature: "Platform Awareness",
    caro: "Detects OS, shell, available commands",
    chatgpt: "Generic bash only",
    copilot: "Partial (agent can inspect the system)",
    shellgpt: "Detects OS and shell"
  },
  {
    feature: "API Key Required",
    caro: "No",
    chatgpt: "Yes (OpenAI account)",
    copilot: "GitHub account, or your own model provider",
    shellgpt: "Yes by default (OpenAI API key)"
  },
  {
    feature: "Works Offline",
    caro: "Yes, fully local",
    chatgpt: "No",
    copilot: "Opt-in (local model + offline mode)",
    shellgpt: "Partial (local models not optimized)"
  },
  {
    feature: "Open Source",
    caro: "Yes (AGPL-3.0)",
    chatgpt: "No",
    copilot: "No",
    shellgpt: "Yes (MIT)"
  },
  {
    feature: "Cost",
    caro: "Free forever",
    chatgpt: "$20/month (Plus)",
    copilot: "Free tier; paid plans from $10/month",
    shellgpt: "Free; API usage costs"
  }
];
