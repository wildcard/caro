#!/usr/bin/env python3
"""Convert caro eval data to mlx-lm LoRA training JSONL.

Reads:
  - tests/evaluation/dataset.yaml     (100 labelled cases)
  - tests/evaluation/test_cases.toml  (55 additional cases)
  - system prompt rendered by `cargo run --example render_system_prompt`

Emits:
  - data/train.jsonl   (~85% of cases, ChatML-wrapped)
  - data/valid.jsonl   (~15%)
  - data/test.jsonl    (safety-only subset, for regression eval)

Each line is {"text": "<|im_start|>system\\n...<|im_end|>\\n<|im_start|>user\\n...<|im_end|>\\n<|im_start|>assistant\\n<target><|im_end|>"}

Target is either:
  - {"cmd": "<expected_command>"}   — for cases with an expected_command
  - QUESTION: <short paraphrase>    — for must_be_blocked destructive cases

The QUESTION target preserves caro's "clarify before destruction" contract
(see src/prompts/smollm_prompt.rs build_output_rules). Training on these is
required or the fine-tuned adapter will forget the safety pattern and start
emitting raw destructive commands.
"""

from __future__ import annotations

import argparse
import json
import random
import re
import sys
from pathlib import Path

try:
    import yaml  # pip install pyyaml
except ImportError:
    sys.exit("missing pyyaml: pip install pyyaml")

try:
    import tomllib  # stdlib 3.11+
except ImportError:
    try:
        import tomli as tomllib  # pip install tomli (3.10 and earlier)
    except ImportError:
        sys.exit("python >=3.11 required, or: pip install tomli")


REPO_ROOT = Path(__file__).resolve().parents[2]
DEFAULT_YAML = REPO_ROOT / "tests" / "evaluation" / "dataset.yaml"
DEFAULT_TOML = REPO_ROOT / "tests" / "evaluation" / "test_cases.toml"


def as_question(intent: str) -> str:
    """Turn a user intent into a short confirmation question matching caro's
    QUESTION output contract. Keeps the model's Q pattern alive during LoRA."""
    intent = intent.strip().rstrip(".?!")
    # Strip leading imperatives so the Q doesn't read like "Q: delete all files?"
    intent = re.sub(r"^(please |pls |can you |could you )", "", intent, flags=re.I)
    return f"QUESTION: Confirm you want to {intent}?"


def load_yaml_cases(path: Path) -> list[dict]:
    with path.open("r", encoding="utf-8") as f:
        cases = yaml.safe_load(f)
    if not isinstance(cases, list):
        sys.exit(f"{path}: expected top-level list, got {type(cases).__name__}")
    return cases


def load_toml_cases(path: Path) -> list[dict]:
    with path.open("rb") as f:
        doc = tomllib.load(f)
    return doc.get("test_cases", [])


def normalize(case: dict, source_file: str) -> dict | None:
    """Pull the 3 fields we need into a uniform shape; drop cases we can't train on."""
    # YAML uses input_request; TOML uses prompt
    user = case.get("input_request") or case.get("prompt")
    if not user:
        return None

    rule = case.get("validation_rule")
    expected_cmd = case.get("expected_command")
    category = case.get("category", "unknown")
    case_id = case.get("id", "?")

    if rule == "must_be_blocked" or category == "safety" and not expected_cmd:
        target = as_question(user)
        kind = "question"
    elif expected_cmd:
        target = json.dumps({"cmd": expected_cmd}, ensure_ascii=False)
        kind = "command"
    else:
        # No trainable target (e.g. must_execute with no command). Skip.
        return None

    return {
        "id": f"{source_file}:{case_id}",
        "user": user,
        "target": target,
        "kind": kind,
        "category": category,
    }


def render_example(system_prompt: str, row: dict) -> dict:
    text = (
        f"<|im_start|>system\n{system_prompt}<|im_end|>\n"
        f"<|im_start|>user\n{row['user']}<|im_end|>\n"
        f"<|im_start|>assistant\n{row['target']}<|im_end|>"
    )
    return {"text": text}


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--yaml", type=Path, default=DEFAULT_YAML)
    ap.add_argument("--toml", type=Path, default=DEFAULT_TOML)
    ap.add_argument("--system-prompt", type=Path, required=True,
                    help="file produced by `cargo run --example render_system_prompt`")
    ap.add_argument("--out", type=Path, default=Path(__file__).parent / "data")
    ap.add_argument("--valid-frac", type=float, default=0.15)
    ap.add_argument("--seed", type=int, default=42)
    args = ap.parse_args()

    system_prompt = args.system_prompt.read_text(encoding="utf-8").rstrip("\n")
    if len(system_prompt) < 1000:
        sys.exit(f"system prompt too short ({len(system_prompt)} bytes) — regenerate via "
                 f"`cargo run --quiet --example render_system_prompt > {args.system_prompt}`")

    raw = []
    raw.extend((c, args.yaml.name) for c in load_yaml_cases(args.yaml))
    raw.extend((c, args.toml.name) for c in load_toml_cases(args.toml))

    rows = []
    skipped = 0
    for case, src in raw:
        row = normalize(case, src)
        if row is None:
            skipped += 1
            continue
        rows.append(row)

    rng = random.Random(args.seed)
    rng.shuffle(rows)

    n_valid = max(1, int(len(rows) * args.valid_frac))
    valid, train = rows[:n_valid], rows[n_valid:]

    args.out.mkdir(parents=True, exist_ok=True)
    train_path = args.out / "train.jsonl"
    valid_path = args.out / "valid.jsonl"
    test_path = args.out / "test.jsonl"

    def dump(path: Path, rows: list[dict]) -> None:
        with path.open("w", encoding="utf-8") as f:
            for r in rows:
                f.write(json.dumps(render_example(system_prompt, r), ensure_ascii=False))
                f.write("\n")

    dump(train_path, train)
    dump(valid_path, valid)
    # Regression set: every safety case (including those sampled into train/valid).
    # mlx-lm does not read test.jsonl during training — this file is consumed by
    # the post-training harness in step 5 of tools/mlx-finetune/Makefile.
    safety = [r for r in rows if r["kind"] == "question"]
    dump(test_path, safety)

    n_cmd = sum(1 for r in rows if r["kind"] == "command")
    n_q = sum(1 for r in rows if r["kind"] == "question")
    print(f"system prompt:  {len(system_prompt):,} bytes", file=sys.stderr)
    print(f"sources:        {args.yaml.name} + {args.toml.name}", file=sys.stderr)
    print(f"total cases:    {len(rows)} ({n_cmd} command, {n_q} question); skipped {skipped}",
          file=sys.stderr)
    print(f"train.jsonl:    {len(train)} examples -> {train_path}", file=sys.stderr)
    print(f"valid.jsonl:    {len(valid)} examples -> {valid_path}", file=sys.stderr)
    print(f"test.jsonl:     {len(safety)} safety examples -> {test_path}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
