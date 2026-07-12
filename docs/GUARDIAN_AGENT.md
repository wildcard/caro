# Guardian Agents and Caro's Role

> *Caro is the execution safety primitive for guardian agent stacks — the
> deterministic layer that sits between an LLM's intent and a shell's power.*

---

## What are guardian agents?

In March 2026, Gartner introduced **Guardian Agents** as a formal market
category: autonomous oversight layers that monitor, constrain, and govern AI
agent behavior in production environments. The category emerged because
enterprise deployments of agentic AI hit a hard reliability and safety wall —
agents that act autonomously need something watching them that is *not itself
an LLM*.

A guardian agent stack typically has three layers:

1. **Identity and access** — *Who* is this agent? What permissions does it hold?
   Orchid Security and similar products address this layer.
2. **Policy and observability** — *What is the agent doing?* Microsoft's Agent
   Governance Toolkit (OWASP + EU AI Act compliance), LangSmith, and Temporal
   operate here.
3. **Execution safety** — *Should this specific action proceed?* This is where
   Caro lives.

## Why execution safety is a distinct layer

The 2026 Microsoft Security Blog disclosure of CVE-2026-25592 and
CVE-2026-26030 demonstrated that prompt injection can become host-level RCE
in a single step when an agent has shell or tool access. Identity and policy
layers cannot prevent this class of attack — they define *who* can act, not
*whether a specific action is safe*. The gap between an agent having permission
to run commands and a specific command being safe to run requires a
deterministic, sub-millisecond validator that does not depend on the LLM
itself.

Caro fills that gap.

## What Caro provides

| Capability | How it works |
|---|---|
| **Command validation** | 62+ pre-compiled regex patterns + CVE rule pipeline. Every command is validated regardless of source backend. |
| **Risk classification** | CRITICAL / HIGH / MEDIUM / LOW levels map onto the tiered approval pattern (auto-approve / async log / sync human gate) the market is converging on. |
| **Defense in depth** | 5-layer pipeline: allowlist, built-in patterns, CVE rules, custom patterns, user confirmation. No single bypass produces execution. |
| **Backend symmetry** | The `SafetyValidator` is backend-agnostic. An embedded model, Ollama, vLLM, or any future backend all pass through the same code path. |
| **Structured output** | JSON and YAML output modes give guardian orchestrators a machine-readable signal to consume. |
| **CVE rule pipeline** | Weekly automated sync with NVD + CISA KEV + GHSA for shell-invocation CVEs (CVSS >= 7.0). Rules compiled into the binary with zero runtime network calls. |

## Integration patterns

Caro can be wired into a guardian agent stack in three ways:

### 1. CLI invocation (available today)

    caro --output json "delete all files older than 30 days"

The JSON output includes the generated command, risk level, and safety
verdict. A guardian orchestrator can parse this and apply its own approval
policy.

### 2. MCP server (in progress)

    caro mcp serve

Exposes `generate_command`, `validate_command`, and `explain_safety` over the
Model Context Protocol. Any MCP-aware agent framework (LangChain, CrewAI,
Semantic Kernel, Google ADK) can call Caro as a tool.

### 3. OpenAI-compatible endpoint (in progress)

    caro serve --openai

An OpenAI Chat Completions endpoint backed by Caro's safety validator.
Drop-in for any agent that uses OpenAI-style tool calls.

## Where Caro fits in the stack

    +---------------------------------------------+
    |         Guardian Agent Orchestrator           |
    | (policy, identity, observability, approval)  |
    +---------------------------------------------+
    |              CARO (you are here)              |
    | (execution safety: validate before shell)     |
    +---------------------------------------------+
    |              Shell / OS / Tool                |
    | (bash, zsh, fish, PowerShell, etc.)           |
    +---------------------------------------------+

Caro does not replace identity, policy, or observability layers. It completes
the stack by adding deterministic safety at the last mile — the point where
an agent's intent becomes an operating system action.

## What Caro does NOT claim to be

- A general-purpose agent governance framework (use Microsoft AGAT, LangGraph
  governance, or similar).
- An identity or access control layer (use Orchid Security, IAM, or similar).
- A monitoring or observability platform (use LangSmith, Temporal, or
  similar).

Caro is a focused, single-responsibility primitive: **validate shell commands
before they execute, regardless of who or what generated them.** That focus
is its strength in a guardian stack — it does one thing, deterministically,
and it does not depend on the LLM to self-police.

## See also

- [SAFETY_PHILOSOPHY.md](SAFETY_PHILOSOPHY.md) — the engineering doctrine
  behind Caro's safety architecture
- [SECURITY.md](../SECURITY.md) — vulnerability disclosure policy
- [README.md](../README.md) — full feature list and installation instructions
