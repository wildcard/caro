/**
 * Flox Use-Case Integration — Regression Guard
 *
 * Locks down the four surfaces added when Flox became a first-class use
 * case + distribution channel (see .claude/plans, the LinkedIn exchange
 * with Ben Futoriansky / Flox, 2026-05). Each surface has a wiring seam
 * that a future edit could silently break:
 *
 *   1. Dev env       — .flox/env/manifest.toml builds caro (MSRV-aligned)
 *   2. Use-case page — website/src/pages/use-cases/flox.astro renders
 *   3. Index entry   — the flox persona is registered in the hub
 *   4. Dev rule      — coder-agent-isolation.md is registered in the
 *                      constitution's Tier 3
 *   5. Packaging     — packages.yml has a flox job in the summary needs[]
 *
 * These are read-as-text assertions on the actual source files, so they
 * fail loudly if a refactor drops the flox page from the personas array,
 * un-registers the rule, or removes the CI job. That is the whole point:
 * keep the feature working and green as other agents keep building.
 */

import { describe, it, expect } from 'vitest';
import { readFileSync, existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

// This test file lives at website/src/__tests__/, so the repo root is
// four levels up. Resolve everything from the file location, not from
// process.cwd(), so the test is runnable from any working directory.
const HERE = dirname(fileURLToPath(import.meta.url));
const WEBSITE_ROOT = resolve(HERE, '..', '..'); // website/
const REPO_ROOT = resolve(WEBSITE_ROOT, '..'); // repo root

function readRepo(relPath: string): string {
  const abs = resolve(REPO_ROOT, relPath);
  expect(existsSync(abs), `expected file to exist: ${relPath}`).toBe(true);
  return readFileSync(abs, 'utf-8');
}

// ============================================================================
// Surface 1 — Flox dev environment manifest
// ============================================================================

describe('Surface 1: Flox dev env manifest', () => {
  it('exists and declares manifest version 1', () => {
    const manifest = readRepo('.flox/env/manifest.toml');
    expect(manifest).toMatch(/^version\s*=\s*1/m);
  });

  it('installs the Rust build toolchain', () => {
    const manifest = readRepo('.flox/env/manifest.toml');
    expect(manifest).toContain('rustup.pkg-path');
    expect(manifest).toContain('openssl.pkg-path');
    expect(manifest).toContain('pkg-config.pkg-path');
  });

  it('pins the MSRV so the env matches Cargo.toml', () => {
    const manifest = readRepo('.flox/env/manifest.toml');
    const cargo = readRepo('Cargo.toml');
    const msrvMatch = cargo.match(/rust-version\s*=\s*"([\d.]+)"/);
    expect(msrvMatch, 'Cargo.toml must declare rust-version').not.toBeNull();
    const msrv = msrvMatch![1];
    // The manifest's on-activate hook defaults rustup to the MSRV. If the
    // MSRV bumps, this test forces the Flox env to bump with it.
    expect(manifest).toContain(`rustup default ${msrv}`);
  });

  it('targets all four Flox-supported systems', () => {
    const manifest = readRepo('.flox/env/manifest.toml');
    for (const sys of [
      'aarch64-darwin',
      'x86_64-darwin',
      'aarch64-linux',
      'x86_64-linux',
    ]) {
      expect(manifest, `manifest must target ${sys}`).toContain(sys);
    }
  });

  it('ignores generated Flox state from git', () => {
    const gitignore = readRepo('.flox/.gitignore');
    expect(gitignore).toContain('cache/');
    expect(gitignore).toContain('run/');
  });
});

// ============================================================================
// Surface 2 — Use-case landing page
// ============================================================================

describe('Surface 2: Flox use-case page', () => {
  const PAGE = 'website/src/pages/use-cases/flox.astro';

  it('exists', () => {
    expect(existsSync(resolve(REPO_ROOT, PAGE))).toBe(true);
  });

  it('renders the three-layer model (env / boundary / command-safety)', () => {
    const page = readRepo(PAGE);
    expect(page).toMatch(/Environment/);
    expect(page).toMatch(/Boundary/);
    expect(page).toMatch(/Command safety/);
    // The boundary progression Ben described, lightest to strongest.
    expect(page).toContain('sandflox');
    expect(page).toMatch(/Container/);
    expect(page).toMatch(/VM/);
  });

  it('documents the two Flox packages (binary + skill)', () => {
    const page = readRepo(PAGE);
    expect(page).toContain('caro-skill');
    expect(page).toContain('.claude/skills/caro-shell');
  });

  it('does not contain a bare "{" that would trip esbuild JSX parsing', () => {
    // Guards .claude/rules/astro-esbuild-shell-syntax.md: shell snippets
    // with an unescaped "{" in .astro template text crash the build.
    // We check the template body (after the frontmatter fence).
    const page = readRepo(PAGE);
    const parts = page.split(/^---\s*$/m);
    // parts[0] = '', parts[1] = frontmatter, parts[2..] = template + style
    const template = parts.slice(2).join('---');
    // A raw "{" immediately followed by a non-identifier, non-whitespace,
    // non-JSX-map character inside a <code>/<pre> shell snippet is the
    // failure mode. We assert no ":(){"-style fork-bomb text is present
    // unescaped (the canonical crash case from the rule).
    expect(template).not.toMatch(/<code>[^<]*:\(\)\{/);
  });
});

// ============================================================================
// Surface 3 — Registration in the use-case hub
// ============================================================================

describe('Surface 3: Flox persona registered in hub', () => {
  const INDEX = 'website/src/pages/use-cases/index.astro';

  it('lists the flox persona in the personas array', () => {
    const index = readRepo(INDEX);
    expect(index).toMatch(/slug:\s*['"]flox['"]/);
  });

  it('gives the flox persona its jobs-to-be-done', () => {
    const index = readRepo(INDEX);
    // The slug and its three jobs must travel together — a persona card
    // with no jobs renders empty.
    const floxBlock = index.slice(index.indexOf("slug: 'flox'"));
    expect(floxBlock).toContain('Reproducible coder-agent envs');
    expect(floxBlock).toContain('Boundary-flexible isolation');
    expect(floxBlock).toContain('Inspect-before-build supply chain');
  });
});

// ============================================================================
// Surface 4 — Dev-process rule + constitution registration
// ============================================================================

describe('Surface 4: coder-agent-isolation rule', () => {
  it('exists as a rule file', () => {
    expect(
      existsSync(resolve(REPO_ROOT, '.claude/rules/coder-agent-isolation.md')),
    ).toBe(true);
  });

  it('is registered in the constitution Tier 3', () => {
    const constitution = readRepo('.claude/rules/constitution.md');
    expect(constitution).toContain('coder-agent-isolation.md');
    // Must sit in Tier 3 (workflow hygiene), below Tier 2 engineering rules.
    const tier3Idx = constitution.indexOf('## Tier 3');
    const ruleIdx = constitution.indexOf('coder-agent-isolation.md');
    expect(tier3Idx).toBeGreaterThan(-1);
    expect(ruleIdx).toBeGreaterThan(tier3Idx);
  });

  it('articulates the boundary-vs-environment principle', () => {
    const rule = readRepo('.claude/rules/coder-agent-isolation.md');
    expect(rule).toMatch(/independent choices/i);
    expect(rule).toContain('sandflox');
    expect(rule).toContain('Flox');
  });
});

// ============================================================================
// Surface 5 — CI packaging job
// ============================================================================

describe('Surface 5: Flox packaging in CI', () => {
  const WORKFLOW = '.github/workflows/packages.yml';

  it('defines a flox job', () => {
    const wf = readRepo(WORKFLOW);
    expect(wf).toMatch(/^\s{2}flox:/m);
  });

  it('wires flox into the summary job needs[] so it is not orphaned', () => {
    const wf = readRepo(WORKFLOW);
    expect(wf).toMatch(/needs:\s*\[prepare,\s*docker,\s*npm,\s*nuget,\s*flox\]/);
  });

  it('passes version/tag via env, never inline in run: (injection guard)', () => {
    // Mirrors the security_reminder_hook guidance: no ${{ }} interpolation
    // inside the flox job's run: blocks — values flow through env:.
    const wf = readRepo(WORKFLOW);
    const floxStart = wf.indexOf('\n  flox:');
    const summaryStart = wf.indexOf('\n  summary:', floxStart);
    const floxJob = wf.slice(floxStart, summaryStart);
    expect(floxJob).toContain('VERSION: ${{ needs.prepare.outputs.version }}');
    // The run: bodies should reference $VERSION / $TAG / $REPO shell vars.
    expect(floxJob).toMatch(/\$\{?VERSION\}?/);
  });

  it('ships a flox/README.md documenting the distribution channel', () => {
    const readme = readRepo('flox/README.md');
    expect(readme).toContain('caro-skill');
    expect(readme).toMatch(/flox\.flake|caro\.flake/);
  });
});
