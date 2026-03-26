# Research Summary: 2025 AI Agents & Guardian Capabilities

## Overview
The AI coding landscape in 2025 has undergone a definitive shift from passive "assistants" (autocomplete and chat) to autonomous **agents**. These agents manage multi-file orchestrations, maintain context persistence across sessions, and execute tasks via iterative Plan-Act-Verify loops. As agents gain more autonomy, robust safety and sandboxing policies have become critical requirements.

## Competitive Landscape
- **Cursor & Windsurf**: Leading the market by embedding AI at the core of the IDE. Features like Cursor's Composer Mode and Windsurf's Cascade allow for orchestrating complex, repo-wide changes without constant micro-prompting.
- **Claude Code & GitHub Copilot Agent Mode**: Focusing heavily on terminal integration and automated issue resolution within GitHub PRs, relying heavily on Model Context Protocol (MCP) to dynamically access external resources.
- **Context Windows**: Top-tier models (Gemini 2.0/3.0, Claude) now support 1M+ token context windows, enabling agents to ingest entire codebases, documentation, and CI logs at once.

## Code Sandboxing & Execution Isolation
To execute code safely, agents rely on multi-tiered sandboxing:
1. **OS-Level Primitives**: 
   - macOS: **Seatbelt** sandbox profiles (used for restricted local execution).
   - Linux: **Landlock** or **Seccomp** to prevent agents from accessing sensitive host files or directories outside the working path.
2. **Runtime Environments**: Cloud-hosted execution relies on **Firecracker MicroVMs** or **gVisor** containers. These provide ephemeral, hardware-level isolation ensuring cross-tenant data leakage is structurally impossible.

## Policies & Guardian-like Features
With the increased autonomy, mechanisms similar to the open-source "Codex Guardian" are now industry standards:
- **Policy-as-Code**: Governance rules defined in YAML or Markdown (e.g., prohibiting writes to `/dist` or blocking read access to `.env` files).
- **Capability Gating**: Granular control over an agent's abilities, such as disabling web searches, shell command execution, or destructive git operations per project.
- **Human-in-the-Loop (HITL)**: Mandatory user review and explicit approval (e.g., via interactive prompt or diff-first execution) before irreversible operations or network requests are executed.

## Relevance to Caro
Caro's roadmap and architecture directly intersect with these trends. Our resolution of tech debt (specifically addressing bugs #150 and #161) is essential to provide a stable foundation for introducing MCP-based tool discovery. Moreover, Caro's current utilization of Bubblewrap for execution isolation should be continually evaluated against emerging MicroVM patterns and strict Seatbelt profiles to maintain top-tier security for agentic tasks.