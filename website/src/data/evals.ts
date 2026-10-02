/**
 * Published evaluation numbers for caro.sh/evals.
 *
 * This file is a committed snapshot of `cargo test --test evaluation`
 * ("Pareto View by Backend"). It is the single source the /evals page
 * renders from, and `tests/website_claims.rs` (EVALS-*) checks it against
 * the dataset in `tests/evaluation/dataset.yaml` so the page cannot drift
 * from what the harness actually measures.
 *
 * Regenerate: run the harness on main, paste the per-backend row and the
 * per-category rows, bump `snapshot`.
 */

export type ConfidenceSource = 'measured' | 'self_reported' | 'constant' | 'none';

export interface BackendEval {
  /** Backend name as printed by the harness. */
  backend: string;
  /** Human label for the page. */
  label: string;
  /** Pass rate over the whole dataset, in percent. */
  passRate: number;
  passed: number;
  total: number;
  /** Brier score, lower is better (0 = perfect). */
  brier: number;
  /** Expected Calibration Error, lower is better (0 = perfect). */
  ece: number;
  /** Share of cases that carried a confidence value, in percent. */
  coverage: number;
  /** Where the confidence came from. */
  source: ConfidenceSource;
  p50Ms: number;
  p95Ms: number;
  /** USD per passed task. 0 for local backends. */
  costPerPass: number;
  /** Agreement with the reference labeller's risk verdicts, percent, or null when no reference judge ran. */
  riskAgreement: number | null;
  perCategory: CategoryEval[];
}

export interface CategoryEval {
  category: 'correctness' | 'posix' | 'safety' | 'multi_backend';
  label: string;
  passed: number;
  total: number;
  passRate: number;
  note?: string;
}

export interface PendingBackend {
  backend: string;
  label: string;
  /** Why there is no published row yet. */
  reason: string;
}

export const snapshot = {
  /** Commit of `main` the numbers were taken from. */
  commit: '9debe6e',
  date: '2026-10-02',
  harness: 'tests/evaluation/main.rs',
  dataset: 'tests/evaluation/dataset.yaml',
  cases: 101,
  command: 'cargo test --test evaluation -- --backend static_matcher',
  /** ECE regression threshold used by the release gate (#1466). */
  eceRegressionThreshold: 0.05,
};

export const backends: BackendEval[] = [
  {
    backend: 'static_matcher',
    label: 'Static matcher (deterministic floor)',
    passRate: 77.2,
    passed: 78,
    total: 101,
    brier: 0.152,
    ece: 0.153,
    coverage: 63,
    source: 'measured',
    p50Ms: 1,
    p95Ms: 11,
    costPerPass: 0,
    riskAgreement: null,
    perCategory: [
      { category: 'correctness', label: 'Correctness', passed: 25, total: 26, passRate: 96.2 },
      { category: 'posix', label: 'POSIX compliance', passed: 24, total: 25, passRate: 96.0 },
      { category: 'safety', label: 'Safety', passed: 25, total: 25, passRate: 100 },
      {
        category: 'multi_backend',
        label: 'Multi-backend',
        passed: 4,
        total: 25,
        passRate: 16,
        note: 'These cases need an LLM backend; the static matcher is expected to abstain on most of them.',
      },
    ],
  },
];

/** Backends the harness supports but whose numbers are not published yet. */
export const pending: PendingBackend[] = [
  {
    backend: 'ollama',
    label: 'Ollama',
    reason: 'Needs a reference judge run (CARO_EVAL_REFERENCE_JUDGE) so the risk-agreement column is filled before the row is published.',
  },
  {
    backend: 'vllm',
    label: 'vLLM',
    reason: 'Same as Ollama; token log-probs are measured, the consensus-label pass has not run in CI yet.',
  },
  {
    backend: 'mlx',
    label: 'Embedded MLX',
    reason: 'Not wired into the harness yet (see TODOs in tests/evaluation/main.rs).',
  },
];

/**
 * Headline figure quoted in ROADMAP.md and the blog, separate from the
 * harness: the Command Success Rate on the 58-case beta suite recorded in
 * January 2026 (ROADMAP.md, "Metrics to Track (During Beta)"). It is quoted
 * on the page so readers can see why it differs from the harness pass rate.
 */
export const betaCsr = { csr: 94.8, cases: 58, recorded: 'January 2026', source: 'ROADMAP.md' };
