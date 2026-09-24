# Backend Safety Integrator Skill

**Ensure all inference backends integrate safety validation**

## What This Does

Guides integration of CommandValidator into LLM inference backends to ensure all generated commands are validated for safety.

## When to Use

- Adding new inference backend
- Updating existing backend
- Safety audit of backends

## Duration

2-4 hours per backend

## Quick Start

```bash
/backend-safety-integrator
```

## Integration Checklist

```
☐ Backend architecture understood
☐ Command generation point found
☐ Safety validator imported
☐ Validation integrated before return
☐ Dangerous commands tested (blocked)
☐ Safe commands tested (allowed)
☐ Error handling added
☐ Code documented
```

## Success Criteria

- ✅ All dangerous commands blocked
- ✅ No false positives
- ✅ Clear error messages
- ✅ Integration documented

---

*Consistent safety validation across all backends.*
