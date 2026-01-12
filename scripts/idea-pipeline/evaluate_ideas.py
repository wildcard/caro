#!/usr/bin/env python3
"""
Critical Evaluation Agent

Decisively evaluates ideas against project criteria.
Default behavior: REJECT. Only accept ideas that clearly pass all criteria.

Usage:
    python evaluate_ideas.py --ideas ideas.json --context context.json --output vetted.json
"""

import argparse
import json
import os
import re
import sys
from datetime import datetime, timezone
from pathlib import Path

from anthropic import Anthropic
from pydantic import BaseModel
from tenacity import retry, stop_after_attempt, wait_exponential


class EvaluationResult(BaseModel):
    """Evaluation result for a single idea."""
    idea_id: str
    decision: str  # ACCEPT, REJECT, NEEDS-HUMAN-REVIEW
    category: str | None = None  # quick-win, strategic, research (if accepted)
    milestone: str | None = None  # v1.2, v1.3, v2.0, backlog (if accepted)
    confidence: float  # 0.0 - 1.0
    reasoning: str
    anti_goal_check: dict[str, bool]  # Which anti-goals were checked


class VettedIdea(BaseModel):
    """An idea with its evaluation."""
    id: str
    title: str
    description: str
    why_now: str
    relevance_score: float
    implementation_hints: list[str]
    source_signals: list[str]
    original_category: str
    # Evaluation fields
    decision: str
    eval_category: str | None = None
    milestone: str | None = None
    confidence: float
    reasoning: str


class VettedCollection(BaseModel):
    """Collection of vetted ideas."""
    evaluated_at: datetime
    model_used: str
    ideas: list[VettedIdea]
    stats: dict


EVALUATION_PROMPT = """You are a CRITICAL evaluation agent for Caro. Your job is to REJECT most ideas.
Only ideas that clearly benefit the project should pass.

## Your Mindset
- Default to REJECT
- Be skeptical of "nice to have" features
- Protect the project from scope creep
- Only ACCEPT ideas with clear, demonstrable value

## Project Context

**Mission**: {mission}

**Current Milestone**: {current_milestone}

**Anti-Goals (HARD REJECTS if violated)**:
{anti_goals}

## Anti-Goal Violation = Instant REJECT

If an idea requires ANY of these, it MUST be rejected:
{anti_goal_list}

## Evaluation Criteria

For an idea to be ACCEPTED, it must:
1. NOT violate any anti-goals (mandatory)
2. Have relevance score > 0.6 (from synthesis)
3. Align with current roadmap direction
4. Be technically feasible with Rust/MLX
5. Address a real user need (not hypothetical)

## Decision Categories

**ACCEPT (quick-win)**: High fit + addresses immediate need → assign to next minor release
**ACCEPT (strategic)**: High fit + larger scope → assign to v1.3 or v2.0
**ACCEPT (research)**: Interesting but needs exploration → assign to backlog as research
**NEEDS-HUMAN-REVIEW**: Borderline case, unclear fit → flag for human decision
**REJECT**: Doesn't meet criteria or violates anti-goals

## Idea to Evaluate

{idea}

## Output Format

Return a JSON object with:
- decision: "ACCEPT", "REJECT", or "NEEDS-HUMAN-REVIEW"
- category: "quick-win", "strategic", "research", or null (if rejected)
- milestone: "v1.2", "v1.3", "v2.0", "backlog", or null (if rejected)
- confidence: 0.0-1.0
- reasoning: 2-3 sentences explaining the decision
- anti_goal_violations: List of violated anti-goals (empty if none)

Be DECISIVE. Do not hedge. If uncertain, lean toward REJECT.

Return ONLY valid JSON."""


def load_anti_goals(anti_goals_path: Path) -> list[str]:
    """Load anti-goals from file."""
    if not anti_goals_path.exists():
        return []

    content = anti_goals_path.read_text()

    # Extract statements from Quick Reference Card
    anti_goals = []
    in_quick_ref = False

    for line in content.split("\n"):
        if "Quick Reference Card" in line:
            in_quick_ref = True
            continue
        if in_quick_ref:
            if line.startswith("| **"):
                match = re.search(r"\*\*(.+?)\*\*.*\|\s*(.+?)\s*\|", line)
                if match:
                    anti_goals.append(f"{match.group(1)}: {match.group(2)}")
            elif line.startswith("---") or line.startswith("##"):
                break

    # Fallback: extract from context if quick ref not found
    if not anti_goals:
        pattern = r"###\s+\d+\.\s+(.+?)\n\n\*\*Statement\*\*:\s*(.+?)(?=\n\n)"
        for match in re.finditer(pattern, content, re.DOTALL):
            anti_goals.append(f"{match.group(1)}: {match.group(2).strip()}")

    return anti_goals


@retry(stop=stop_after_attempt(3), wait=wait_exponential(multiplier=1, min=2, max=10))
def call_claude(client: Anthropic, prompt: str, model: str = "claude-sonnet-4-20250514") -> str:
    """Call Claude API with retry logic."""
    response = client.messages.create(
        model=model,
        max_tokens=1024,
        messages=[{"role": "user", "content": prompt}],
    )
    return response.content[0].text


def evaluate_idea(idea: dict, context: dict, anti_goals: list[str], client: Anthropic, model: str) -> EvaluationResult:
    """Evaluate a single idea using Claude."""

    idea_text = f"""
ID: {idea.get('id')}
Title: {idea.get('title')}
Description: {idea.get('description')}
Why Now: {idea.get('why_now')}
Relevance Score: {idea.get('relevance_score')}
Category: {idea.get('category')}
Implementation Hints:
{chr(10).join('- ' + h for h in idea.get('implementation_hints', []))}
"""

    prompt = EVALUATION_PROMPT.format(
        mission=context.get("mission", "Unknown"),
        current_milestone=context.get("current_milestone", "Unknown"),
        anti_goals="\n".join(f"- {ag}" for ag in context.get("anti_goals", [])),
        anti_goal_list="\n".join(f"{i+1}. {ag}" for i, ag in enumerate(anti_goals)),
        idea=idea_text,
    )

    try:
        response = call_claude(client, prompt, model)

        # Parse JSON response
        if "```json" in response:
            response = response.split("```json")[1].split("```")[0]
        elif "```" in response:
            response = response.split("```")[1].split("```")[0]

        result = json.loads(response.strip())

        # Build anti-goal check dict
        violations = result.get("anti_goal_violations", [])
        anti_goal_check = {ag: ag not in violations for ag in anti_goals}

        return EvaluationResult(
            idea_id=idea.get("id", "unknown"),
            decision=result.get("decision", "REJECT"),
            category=result.get("category"),
            milestone=result.get("milestone"),
            confidence=min(1.0, max(0.0, float(result.get("confidence", 0.5)))),
            reasoning=result.get("reasoning", "No reasoning provided"),
            anti_goal_check=anti_goal_check,
        )

    except Exception as e:
        print(f"Error evaluating idea {idea.get('id')}: {e}", file=sys.stderr)
        return EvaluationResult(
            idea_id=idea.get("id", "unknown"),
            decision="REJECT",
            category=None,
            milestone=None,
            confidence=0.0,
            reasoning=f"Evaluation error: {e}",
            anti_goal_check={},
        )


def main():
    parser = argparse.ArgumentParser(description="Evaluate ideas against project criteria")
    parser.add_argument("--ideas", type=str, required=True, help="Path to ideas JSON")
    parser.add_argument("--context", type=str, required=True, help="Path to context JSON")
    parser.add_argument("--anti-goals", type=Path, required=True, help="Path to anti-goals file")
    parser.add_argument("--output", "-o", default="vetted.json", help="Output file")
    parser.add_argument("--model", default="claude-sonnet-4-20250514", help="Claude model to use")
    args = parser.parse_args()

    # Check for API key
    api_key = os.environ.get("ANTHROPIC_API_KEY")
    if not api_key:
        print("Error: ANTHROPIC_API_KEY environment variable required", file=sys.stderr)
        sys.exit(1)

    # Load inputs
    with open(args.ideas) as f:
        ideas_data = json.load(f)

    with open(args.context) as f:
        context = json.load(f)

    anti_goals = load_anti_goals(args.anti_goals)
    ideas = ideas_data.get("ideas", [])

    if not ideas:
        print("No ideas to evaluate")
        collection = VettedCollection(
            evaluated_at=datetime.now(timezone.utc),
            model_used=args.model,
            ideas=[],
            stats={"total": 0, "accepted": 0, "rejected": 0, "human_review": 0},
        )
    else:
        client = Anthropic(api_key=api_key)
        vetted_ideas = []

        for i, idea in enumerate(ideas):
            print(f"Evaluating idea {i+1}/{len(ideas)}: {idea.get('title', 'Unknown')}")

            result = evaluate_idea(idea, context, anti_goals, client, args.model)

            vetted = VettedIdea(
                id=idea.get("id", f"unknown-{i}"),
                title=idea.get("title", "Untitled"),
                description=idea.get("description", ""),
                why_now=idea.get("why_now", ""),
                relevance_score=idea.get("relevance_score", 0.0),
                implementation_hints=idea.get("implementation_hints", []),
                source_signals=idea.get("source_signals", []),
                original_category=idea.get("category", "unknown"),
                decision=result.decision,
                eval_category=result.category,
                milestone=result.milestone,
                confidence=result.confidence,
                reasoning=result.reasoning,
            )
            vetted_ideas.append(vetted)

            print(f"  → {result.decision} ({result.confidence:.2f}): {result.reasoning[:80]}...")

        # Calculate stats
        stats = {
            "total": len(vetted_ideas),
            "accepted": len([i for i in vetted_ideas if i.decision == "ACCEPT"]),
            "rejected": len([i for i in vetted_ideas if i.decision == "REJECT"]),
            "human_review": len([i for i in vetted_ideas if i.decision == "NEEDS-HUMAN-REVIEW"]),
        }

        collection = VettedCollection(
            evaluated_at=datetime.now(timezone.utc),
            model_used=args.model,
            ideas=vetted_ideas,
            stats=stats,
        )

    # Write output
    with open(args.output, "w") as f:
        json.dump(collection.model_dump(mode="json"), f, indent=2, default=str)

    print(f"\nEvaluation complete:")
    print(f"  Total: {collection.stats['total']}")
    print(f"  Accepted: {collection.stats['accepted']}")
    print(f"  Rejected: {collection.stats['rejected']}")
    print(f"  Human Review: {collection.stats['human_review']}")
    print(f"Output written to: {args.output}")


if __name__ == "__main__":
    main()
