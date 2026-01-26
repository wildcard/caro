#!/usr/bin/env python3
"""
Idea Synthesis Agent

Takes collected signals and project context, uses Claude to synthesize
project-relevant ideas.

Usage:
    python synthesize_ideas.py --signals signals.json --context context.json --output ideas.json
"""

import argparse
import json
import os
import sys
from datetime import datetime, timezone

from anthropic import Anthropic
from pydantic import BaseModel
from tenacity import retry, stop_after_attempt, wait_exponential


class CandidateIdea(BaseModel):
    """A synthesized idea from signals."""
    id: str
    title: str
    description: str
    why_now: str
    relevance_score: float  # 0.0 - 1.0
    implementation_hints: list[str]
    source_signals: list[str]  # Signal IDs
    category: str  # feature, improvement, integration, research


class IdeaCollection(BaseModel):
    """Collection of synthesized ideas."""
    synthesized_at: datetime
    model_used: str
    signal_count: int
    ideas: list[CandidateIdea]


SYNTHESIS_PROMPT = """You are an idea synthesis agent for Caro, a privacy-first CLI tool that converts natural language to shell commands using local LLMs.

## Project Context

**Mission**: {mission}

**Current Milestone**: {current_milestone}

**Short-term Goals**:
{short_term_goals}

**Long-term Goals**:
{long_term_goals}

**Tech Stack**: {tech_stack}

**Anti-Goals (ideas must NOT require any of these)**:
{anti_goals}

## Your Task

Analyze the following signals and synthesize 0-5 concrete product ideas for Caro.

**Quality over quantity**: Only generate ideas that:
1. Directly address user needs evident in signals
2. Align with Caro's mission (local-first, privacy, safety)
3. Are technically feasible with our Rust/MLX stack
4. Do NOT violate any anti-goals

**If no good ideas emerge from signals, return an empty list. This is expected and good.**

## Signals

{signals}

## Output Format

Return a JSON array of ideas. Each idea should have:
- id: Unique identifier (AIP-YYYY-MM-NNN format)
- title: Brief, action-oriented title
- description: 2-3 sentences explaining the feature
- why_now: Why this matters given the signals (1-2 sentences)
- relevance_score: 0.0-1.0 (how relevant to Caro)
- implementation_hints: 2-4 bullet points on implementation approach
- source_signals: List of signal IDs that inspired this
- category: One of [feature, improvement, integration, research]

Return ONLY valid JSON, no markdown code blocks or explanations."""


def format_signals(signals: list[dict], max_signals: int = 30) -> str:
    """Format signals for the prompt."""
    # Sort by score, take top N
    sorted_signals = sorted(signals, key=lambda s: -s.get("score", 0))[:max_signals]

    formatted = []
    for s in sorted_signals:
        entry = f"""
Signal ID: {s['id']}
Source: {s['source']}
Title: {s['title']}
Score: {s.get('score', 0)} | Comments: {s.get('comments', 0)}
Content: {(s.get('content') or '')[:300]}
URL: {s.get('url', 'N/A')}
---"""
        formatted.append(entry)

    return "\n".join(formatted)


def format_list(items: list[str]) -> str:
    """Format list items for prompt."""
    if not items:
        return "- None specified"
    return "\n".join(f"- {item}" for item in items)


@retry(stop=stop_after_attempt(3), wait=wait_exponential(multiplier=1, min=2, max=10))
def call_claude(client: Anthropic, prompt: str, model: str = "claude-sonnet-4-20250514") -> str:
    """Call Claude API with retry logic."""
    response = client.messages.create(
        model=model,
        max_tokens=4096,
        messages=[{"role": "user", "content": prompt}],
    )
    return response.content[0].text


def synthesize_ideas(signals: list[dict], context: dict, client: Anthropic) -> list[CandidateIdea]:
    """Use Claude to synthesize ideas from signals."""

    prompt = SYNTHESIS_PROMPT.format(
        mission=context.get("mission", "Unknown"),
        current_milestone=context.get("current_milestone", "Unknown"),
        short_term_goals=format_list(context.get("short_term_goals", [])),
        long_term_goals=format_list(context.get("long_term_goals", [])),
        tech_stack=", ".join(context.get("tech_stack", [])),
        anti_goals=format_list(context.get("anti_goals", [])),
        signals=format_signals(signals),
    )

    print(f"Sending {len(signals)} signals to Claude for synthesis...")

    try:
        response = call_claude(client, prompt)

        # Parse JSON response
        # Handle potential markdown code blocks
        if "```json" in response:
            response = response.split("```json")[1].split("```")[0]
        elif "```" in response:
            response = response.split("```")[1].split("```")[0]

        ideas_data = json.loads(response.strip())

        ideas = []
        for idea_dict in ideas_data:
            idea = CandidateIdea(
                id=idea_dict.get("id", f"AIP-{datetime.now().strftime('%Y-%m')}-{len(ideas):03d}"),
                title=idea_dict.get("title", "Untitled"),
                description=idea_dict.get("description", ""),
                why_now=idea_dict.get("why_now", ""),
                relevance_score=min(1.0, max(0.0, float(idea_dict.get("relevance_score", 0.5)))),
                implementation_hints=idea_dict.get("implementation_hints", []),
                source_signals=idea_dict.get("source_signals", []),
                category=idea_dict.get("category", "feature"),
            )
            ideas.append(idea)

        return ideas

    except json.JSONDecodeError as e:
        print(f"Error parsing Claude response: {e}", file=sys.stderr)
        print(f"Response was: {response[:500]}", file=sys.stderr)
        return []
    except Exception as e:
        print(f"Error calling Claude: {e}", file=sys.stderr)
        return []


def main():
    parser = argparse.ArgumentParser(description="Synthesize ideas from signals")
    parser.add_argument("--signals", type=str, required=True, help="Path to signals JSON")
    parser.add_argument("--context", type=str, required=True, help="Path to context JSON")
    parser.add_argument("--output", "-o", default="ideas.json", help="Output file")
    parser.add_argument("--model", default="claude-sonnet-4-20250514", help="Claude model to use")
    args = parser.parse_args()

    # Check for API key
    api_key = os.environ.get("ANTHROPIC_API_KEY")
    if not api_key:
        print("Error: ANTHROPIC_API_KEY environment variable required", file=sys.stderr)
        sys.exit(1)

    # Load inputs
    with open(args.signals) as f:
        signals_data = json.load(f)

    with open(args.context) as f:
        context = json.load(f)

    signals = signals_data.get("signals", [])
    if not signals:
        print("No signals to process, creating empty output")
        collection = IdeaCollection(
            synthesized_at=datetime.now(timezone.utc),
            model_used=args.model,
            signal_count=0,
            ideas=[],
        )
    else:
        # Initialize client and synthesize
        client = Anthropic(api_key=api_key)
        ideas = synthesize_ideas(signals, context, client)

        collection = IdeaCollection(
            synthesized_at=datetime.now(timezone.utc),
            model_used=args.model,
            signal_count=len(signals),
            ideas=ideas,
        )

    # Write output
    with open(args.output, "w") as f:
        json.dump(collection.model_dump(mode="json"), f, indent=2, default=str)

    print(f"\nSynthesized {len(collection.ideas)} ideas from {len(signals)} signals")
    print(f"Output written to: {args.output}")


if __name__ == "__main__":
    main()
