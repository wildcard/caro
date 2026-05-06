#!/usr/bin/env python3
"""
convert_0din_probes.py — Convert Mozilla 0din probe data to caro YAML rule format.

Input sources:
  1. 0din-ai/ai-scanner probe YAML files (garak-compatible probe specs)
  2. 0din-jef JSONL scan output (results of running probes against a model)

Output:
  data/cve_rules/ODIN-<ID>.yaml files in caro's dogma compiler schema

Usage:
  # From garak probe YAML directory:
  python3 scripts/convert_0din_probes.py --source probes/ --out data/cve_rules/

  # From 0din-jef JSONL scan output (extract blocked commands from output):
  python3 scripts/convert_0din_probes.py --source scan_results.jsonl --out data/cve_rules/

  # Dry-run (print to stdout):
  python3 scripts/convert_0din_probes.py --source probes/ --dry-run

Schema mapping:
  0din severity       → caro risk_level
  ──────────────────────────────────────
  JEF 7–10 / severe  → critical
  JEF 4–6 / high     → high
  JEF 1–3 / medium   → moderate
  JEF 0   / low      → safe (skip — not blocked)

  0din family         → description prefix tag
  ───────────────────────────────────────────
  prompt_injection    → [0din:prompt-injection]
  data_exfiltration   → [0din:data-exfiltration]
  encoding_bypass     → [0din:encoding-bypass]
  jailbreak           → [0din:jailbreak]
  alignment_bypass    → [0din:jailbreak]

NOTE: 0din probes are primarily LLM-prompt-level attacks. The shell commands
that result from a successful attack (not the adversarial prompts themselves)
should be placed in `pattern`. This script attempts to extract shell command
patterns from probe outputs; adjust the COMMAND_EXTRACTORS below for your
specific probe format.

Apache 2.0 compliance: generated YAML files include source attribution to
0din.ai and the Apache 2.0 license notice.
"""

import argparse
import json
import re
import sys
from pathlib import Path
from datetime import date

# ─── Risk level mapping ──────────────────────────────────────────────────────

def jef_score_to_risk(jef_score: float) -> str | None:
    """Map JEF 0–10 severity to caro risk_level. Returns None to skip low-severity."""
    if jef_score >= 7:
        return "critical"
    elif jef_score >= 4:
        return "high"
    elif jef_score >= 1:
        return "moderate"
    return None  # Skip: too low to block


def severity_str_to_risk(severity: str) -> str | None:
    """Map 0din severity string to caro risk_level."""
    mapping = {
        "severe": "critical",
        "high": "high",
        "medium": "moderate",
        "moderate": "moderate",
        "low": None,  # Skip
    }
    return mapping.get(severity.lower())


# ─── Family tag mapping ───────────────────────────────────────────────────────

FAMILY_TAGS = {
    "prompt_injection": "[0din:prompt-injection]",
    "direct_prompt_injection": "[0din:prompt-injection]",
    "indirect_prompt_injection": "[0din:prompt-injection]",
    "data_exfiltration": "[0din:data-exfiltration]",
    "exfiltration": "[0din:data-exfiltration]",
    "encoding_bypass": "[0din:encoding-bypass]",
    "linguistic_bypass": "[0din:encoding-bypass]",
    "jailbreak": "[0din:jailbreak]",
    "alignment_bypass": "[0din:jailbreak]",
    "persona": "[0din:jailbreak]",
    "harmful_content": "[0din:harmful-content]",
    "cbrn": "[0din:harmful-content]",
}


def family_tag(family: str) -> str:
    return FAMILY_TAGS.get(family.lower(), f"[0din:{family.lower()}]")


# ─── Shell-command pattern extractor ─────────────────────────────────────────
# These heuristics extract the shell command regex from 0din probe data.
# The probe YAML's "payload" or "suffix" field contains the adversarial text;
# we extract the shell command portion and escape it as a Rust regex.

SHELL_COMMAND_RE = re.compile(
    r"(?:"
    r"(?:bash|sh|zsh|fish)\s+-[iIc]|"       # Shell invocations
    r"(?:python|perl|ruby)[23]?\s+-[ec]|"    # Interpreter one-liners
    r"(?:curl|wget)\s+|"                      # Network tools
    r"(?:tar|zip|gzip)\s+|"                  # Archive tools
    r"nc\s+-|"                                # Netcat
    r"echo\s+[A-Za-z0-9+/]{20,}|"           # Base64-like strings
    r"PROMPT_COMMAND\s*="                     # Shell hooks
    r")"
    r"[^\n]{5,100}",  # Capture up to 100 chars of the command
    re.MULTILINE
)


def extract_shell_pattern(probe_text: str) -> str | None:
    """Extract a shell command pattern from probe payload text."""
    m = SHELL_COMMAND_RE.search(probe_text)
    if not m:
        return None
    raw = m.group(0).strip()
    # Escape for Rust regex: escape special chars, keep structure
    escaped = re.escape(raw)
    # Un-escape spaces and pipes for readability
    escaped = escaped.replace(r"\ ", r"\s+").replace(r"\|", r"\|")
    return escaped


# ─── YAML output generator ────────────────────────────────────────────────────

YAML_TEMPLATE = """\
# Mozilla 0din probe — {family} category
# Source: {source_url}
# Apache 2.0 — attribution: Mozilla 0din (0din.ai)
#
# {description}

id: {rule_id}
source: {source_url}
disclosed: {disclosed}
risk_level: {risk_level}
shell_specific: {shell_specific}
pattern: "{pattern}"
description: "{tag} {short_description}"

test_cases:
{test_cases}
"""


def format_test_cases(block_inputs: list[str], allow_inputs: list[str]) -> str:
    lines = []
    for inp in block_inputs:
        safe = inp.replace('"', '\\"')
        lines.append(f'  - input: "{safe}"')
        lines.append(f'    expected_behavior: Block')
    for inp in allow_inputs:
        safe = inp.replace('"', '\\"')
        lines.append(f'  - input: "{safe}"')
        lines.append(f'    expected_behavior: Allow')
    return "\n".join(lines)


def generate_odin_yaml(
    rule_id: str,
    family: str,
    source_url: str,
    disclosed: str,
    risk_level: str,
    shell_specific: str | None,
    pattern: str,
    short_description: str,
    block_examples: list[str],
    allow_examples: list[str],
    long_description: str = "",
) -> str:
    tag = family_tag(family)
    shell_spec = shell_specific if shell_specific else "null"
    test_cases_str = format_test_cases(block_examples, allow_examples)

    return YAML_TEMPLATE.format(
        family=family.replace("_", " "),
        source_url=source_url,
        description=long_description or short_description,
        rule_id=rule_id,
        disclosed=disclosed,
        risk_level=risk_level,
        shell_specific=shell_spec,
        pattern=pattern,
        tag=tag,
        short_description=short_description,
        test_cases=test_cases_str,
    )


# ─── Input parsers ────────────────────────────────────────────────────────────

def parse_garak_probe_yaml(path: Path) -> list[dict]:
    """Parse a garak-format probe YAML file into probe dicts."""
    try:
        import yaml
    except ImportError:
        print("ERROR: PyYAML not installed. Run: pip install pyyaml", file=sys.stderr)
        sys.exit(1)

    with open(path) as f:
        data = yaml.safe_load(f)

    if not data:
        return []

    probes = []
    # Garak probe YAML may have a top-level 'probes' list or be a single probe
    items = data if isinstance(data, list) else [data]
    for item in items:
        probes.append(item)
    return probes


def parse_jef_jsonl(path: Path) -> list[dict]:
    """Parse 0din-jef JSONL scan output."""
    results = []
    with open(path) as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                results.append(json.loads(line))
            except json.JSONDecodeError:
                continue
    return results


def convert_garak_probe(probe: dict, index: int) -> dict | None:
    """Convert a garak probe dict to a caro rule dict. Returns None to skip."""
    # Try to get severity / JEF score
    jef_score = probe.get("jef_score") or probe.get("severity_score")
    severity_str = probe.get("severity") or probe.get("risk_level", "")

    if jef_score is not None:
        risk = jef_score_to_risk(float(jef_score))
    elif severity_str:
        risk = severity_str_to_risk(severity_str)
    else:
        risk = "high"  # Default: treat unknown as high

    if risk is None:
        return None  # Skip low-severity probes

    family = probe.get("family") or probe.get("category") or "unknown"
    name = probe.get("name") or probe.get("id") or f"probe-{index:03d}"
    description = probe.get("description") or probe.get("summary") or name

    # Extract shell command pattern from payload/suffix/template
    payload_text = (
        probe.get("payload")
        or probe.get("suffix")
        or probe.get("template")
        or probe.get("prompt")
        or ""
    )
    pattern = extract_shell_pattern(payload_text)
    if not pattern:
        print(f"  SKIP {name}: could not extract shell command pattern", file=sys.stderr)
        return None

    # Generate block/allow examples from test_cases if present
    block_examples = [payload_text[:80]] if payload_text else []
    allow_examples = probe.get("benign_examples") or []

    today = date.today().isoformat()
    rule_id = f"ODIN-{today[:4]}-{index:03d}"

    return {
        "rule_id": rule_id,
        "family": family,
        "source_url": f"https://0din.ai/research/taxonomy/techniques",
        "disclosed": today,
        "risk_level": risk,
        "shell_specific": probe.get("shell_specific"),
        "pattern": pattern,
        "short_description": description[:80],
        "block_examples": block_examples[:3],
        "allow_examples": allow_examples[:3],
    }


# ─── CLI ─────────────────────────────────────────────────────────────────────

def main():
    parser = argparse.ArgumentParser(description="Convert Mozilla 0din probe data to caro YAML rules")
    parser.add_argument("--source", required=True, help="0din probe YAML directory or JSONL scan file")
    parser.add_argument("--out", default="data/cve_rules/", help="Output directory for ODIN-*.yaml files")
    parser.add_argument("--dry-run", action="store_true", help="Print to stdout, don't write files")
    parser.add_argument("--start-id", type=int, default=100, help="Starting numeric ID for ODIN rules")
    args = parser.parse_args()

    source = Path(args.source)
    out_dir = Path(args.out)

    if not args.dry_run:
        out_dir.mkdir(parents=True, exist_ok=True)

    probes = []
    if source.is_dir():
        for yaml_file in sorted(source.glob("*.yaml")):
            probes.extend(parse_garak_probe_yaml(yaml_file))
    elif source.suffix == ".jsonl":
        probes = parse_jef_jsonl(source)
    elif source.suffix in (".yaml", ".yml"):
        probes = parse_garak_probe_yaml(source)
    else:
        print(f"ERROR: Unsupported source format: {source}", file=sys.stderr)
        sys.exit(1)

    print(f"Found {len(probes)} probe(s) in {source}", file=sys.stderr)

    written = 0
    skipped = 0
    for i, probe in enumerate(probes, start=args.start_id):
        rule = convert_garak_probe(probe, i)
        if rule is None:
            skipped += 1
            continue

        yaml_content = generate_odin_yaml(**rule)
        out_path = out_dir / f"{rule['rule_id']}.yaml"

        if args.dry_run:
            print(f"\n# ─── {out_path} ───")
            print(yaml_content)
        else:
            out_path.write_text(yaml_content)
            print(f"  wrote {out_path}")
        written += 1

    print(f"\nSummary: {written} rules written, {skipped} skipped", file=sys.stderr)
    print("Next step: run `cargo build` to compile rules into the binary.", file=sys.stderr)
    print("Then run: cargo test safety -- --nocapture", file=sys.stderr)


if __name__ == "__main__":
    main()
