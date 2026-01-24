# ADR Numbering Rule

**CRITICAL**: When merging multiple ADR PRs, always renumber them sequentially based on a consistent ordering principle.

## Numbering Principle

Choose ONE of these approaches and apply consistently:

### Option 1: PR Number Order (Recommended)
- Lowest PR number → Lowest ADR number
- Example: PRs #623, #624, #627, #630 → ADR-005, 006, 007, 008

### Option 2: Creation Date Order
- Earliest created → Lowest ADR number
- Check PR creation timestamp to determine order

### Option 3: Merge Order
- First merged → Lowest ADR number
- Sequential based on actual merge time

## Workflow

When merging multiple ADR PRs in a batch:

1. **List all pending ADR PRs** with their numbers and creation dates
2. **Choose ordering principle** (recommend PR number order)
3. **Renumber ADRs** sequentially before merging
4. **Update README.md** with correct sequential numbers
5. **Merge in order** (lowest number first)

## Example

**Bad** (merged with original numbers):
```
PR #630 → ADR-004 (created as 004)
PR #627 → ADR-006 (created as 006)
PR #624 → ADR-007 (created as 007)
PR #623 → ADR-008 (created as 004, renumbered to 008)
Result: ADR-005 is missing! ❌
```

**Good** (renumbered by PR order):
```
PR #623 → Renumber to ADR-005
PR #624 → Renumber to ADR-006
PR #627 → Renumber to ADR-007
PR #630 → Renumber to ADR-008
Result: Sequential ADR-005 through 008 ✓
```

## Never Skip Numbers

ADR numbers must be sequential with no gaps. If ADR-009 is missing, the next ADR should be renumbered to ADR-009, not created as ADR-010.

## Why This Matters

- **Consistency**: Clear numbering makes ADRs easy to reference
- **No gaps**: Sequential numbers prevent confusion
- **Trackability**: Order reflects project history
- **Documentation**: README index stays clean and organized

## When Renumbering

Renumbering requires updating:
- [ ] ADR filename (`ADR-XXX-title.md`)
- [ ] ADR title in the document
- [ ] `docs/adr/README.md` table entry
- [ ] Any cross-references in other ADRs
- [ ] Spec directories (`specs/XXX-feature/`)
