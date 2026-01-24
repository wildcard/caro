# 008: Self-Update Mechanism

**Status**: Specified
**Priority**: Medium
**Target Version**: v1.1.0 (Phase 1), v1.2.0 (Phase 2)

## Overview

Add self-update functionality to Caro, allowing users to update their installation with a single command (`caro update`).

## Documents

| Document | Description |
|----------|-------------|
| [spec.md](./spec.md) | Product Requirements Document |
| [plan.md](./plan.md) | Implementation Architecture |
| [tasks.md](./tasks.md) | Actionable Implementation Tasks |

## Related

- [ADR-004: Self-Update Mechanism](../../docs/adr/ADR-004-self-update-mechanism.md) - Architecture decision record
- [Release Process](../../docs/RELEASE_PROCESS.md) - Current release workflow

## Quick Summary

**Problem**: Users must manually check and download updates.

**Solution**: `caro update` command using `self_update` crate to check GitHub Releases and replace binary in-place.

**Scope**:
- `caro update` - Install latest version
- `caro update --check` - Check without installing
- `caro update --force` - Force reinstall
- Build type awareness (binary vs source vs dev)

## Key Decisions

1. **Crate**: `self_update` v0.42+ (6.5M+ downloads, GitHub native)
2. **Backend**: GitHub Releases (existing release channel)
3. **Feature**: Optional via `--features self-update`
4. **Security**: HTTPS only, user confirmation, future signature verification

## Phases

| Phase | Version | Scope |
|-------|---------|-------|
| 1 | v1.1.0 | `--check` only, build type detection |
| 2 | v1.2.0 | Full self-update with progress |
| 3 | v1.3.0 | Quiet mode, optional startup check |
| 4 | v2.0.0 | Ed25519 signature verification |
