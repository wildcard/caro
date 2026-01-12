#!/usr/bin/env python3
"""
Build Context for Idea Synthesis

Extracts relevant context from project files to provide to the LLM agents.

Usage:
    python build_context.py --roadmap ROADMAP.md --anti-goals ANTI_GOALS.md --output context.json
"""

import argparse
import json
import re
from pathlib import Path

from pydantic import BaseModel


class ProjectContext(BaseModel):
    """Project context for LLM agents."""
    name: str
    mission: str
    current_milestone: str
    short_term_goals: list[str]
    long_term_goals: list[str]
    anti_goals: list[str]
    anti_goal_details: dict[str, str]
    recent_features: list[str]
    tech_stack: list[str]


def extract_section(content: str, header: str, level: int = 2) -> str:
    """Extract a section from markdown content."""
    pattern = rf"^{'#' * level}\s+{re.escape(header)}\s*\n(.*?)(?=^{'#' * level}\s+|\Z)"
    match = re.search(pattern, content, re.MULTILINE | re.DOTALL)
    if match:
        return match.group(1).strip()
    return ""


def extract_list_items(content: str) -> list[str]:
    """Extract bullet points from markdown content."""
    items = []
    for line in content.split("\n"):
        line = line.strip()
        if line.startswith("- ") or line.startswith("* "):
            items.append(line[2:].strip())
    return items


def parse_roadmap(roadmap_path: Path) -> dict:
    """Parse roadmap file for context."""
    if not roadmap_path.exists():
        return {
            "current_milestone": "Unknown",
            "short_term_goals": [],
            "long_term_goals": [],
            "recent_features": [],
        }

    content = roadmap_path.read_text()

    # Extract current milestone from title or overview
    current_milestone = "v1.1"
    if "v1.3" in content[:500]:
        current_milestone = "v1.3"
    elif "v1.2" in content[:500]:
        current_milestone = "v1.2"

    # Try to find strategic imperatives or goals
    short_term = []
    long_term = []

    # Look for v1.3 features as short-term
    v13_section = extract_section(content, "v1.3.0 Core Features", 3)
    if v13_section:
        for match in re.finditer(r"Feature \d+:\s*(.+)", v13_section):
            short_term.append(match.group(1).strip())

    # Look for v2.0 or long-term section
    v20_section = extract_section(content, "v2.0.0 Horizon", 3)
    if v20_section:
        long_term = extract_list_items(v20_section)

    # Extract recent features from success criteria
    recent = []
    success_section = extract_section(content, "Success Criteria", 3)
    if success_section:
        recent = [item for item in extract_list_items(success_section) if item.startswith("✅")]

    return {
        "current_milestone": current_milestone,
        "short_term_goals": short_term[:5],  # Limit to 5
        "long_term_goals": long_term[:5],
        "recent_features": recent[:5],
    }


def parse_anti_goals(anti_goals_path: Path) -> tuple[list[str], dict[str, str]]:
    """Parse anti-goals file for constraints."""
    if not anti_goals_path.exists():
        return [], {}

    content = anti_goals_path.read_text()

    anti_goals = []
    anti_goal_details = {}

    # Find all numbered anti-goal sections
    pattern = r"###\s+\d+\.\s+(.+?)\n\n\*\*Statement\*\*:\s*(.+?)(?=\n\n|\n###|\Z)"
    for match in re.finditer(pattern, content, re.DOTALL):
        title = match.group(1).strip()
        statement = match.group(2).strip()
        anti_goals.append(statement)
        anti_goal_details[title] = statement

    return anti_goals, anti_goal_details


def main():
    parser = argparse.ArgumentParser(description="Build context for idea pipeline")
    parser.add_argument("--roadmap", type=Path, required=True, help="Path to roadmap file")
    parser.add_argument("--anti-goals", type=Path, required=True, help="Path to anti-goals file")
    parser.add_argument("--output", "-o", default="context.json", help="Output file")
    args = parser.parse_args()

    # Parse files
    roadmap_data = parse_roadmap(args.roadmap)
    anti_goals, anti_goal_details = parse_anti_goals(args.anti_goals)

    context = ProjectContext(
        name="Caro",
        mission="Privacy-first CLI tool that converts natural language to shell commands using local LLMs",
        current_milestone=roadmap_data["current_milestone"],
        short_term_goals=roadmap_data["short_term_goals"],
        long_term_goals=roadmap_data["long_term_goals"],
        anti_goals=anti_goals,
        anti_goal_details=anti_goal_details,
        recent_features=roadmap_data["recent_features"],
        tech_stack=[
            "Rust",
            "MLX (Apple Silicon)",
            "Local LLM inference",
            "Static pattern matching",
            "POSIX shell",
        ],
    )

    with open(args.output, "w") as f:
        json.dump(context.model_dump(), f, indent=2)

    print(f"Context built with {len(anti_goals)} anti-goals")
    print(f"Output written to: {args.output}")


if __name__ == "__main__":
    main()
