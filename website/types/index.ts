// Type definitions matching the Rust data structures

export type RiskLevel = 'Safe' | 'Moderate' | 'High' | 'Critical';
export type ShellType = 'Bash' | 'Zsh' | 'Fish' | 'Sh' | 'PowerShell' | 'Cmd';
export type GuideDifficulty = 'Beginner' | 'Intermediate' | 'Advanced';

export type GuardrailCategory =
  | 'FilesystemDestruction'
  | 'DiskOperations'
  | 'PrivilegeEscalation'
  | 'NetworkBackdoors'
  | 'ProcessManipulation'
  | 'SystemModification'
  | 'EnvironmentManipulation'
  | 'PackageManagement'
  | 'Containers';

export type GuideCategory =
  | 'Git'
  | 'Docker'
  | 'FileManagement'
  | 'Networking'
  | 'SystemAdministration'
  | 'Development'
  | 'Database'
  | 'Kubernetes'
  | 'Cloud'
  | 'Security'
  | 'TextProcessing'
  | 'Monitoring';

export interface CommunityNote {
  author: string;
  date: string;
  note: string;
  upvotes: number;
}

export interface GuardrailStats {
  times_triggered: number;
  times_overridden: number;
  false_positive_reports: number;
  last_triggered?: string;
}

export interface DangerPattern {
  regex: string;
  risk_level: RiskLevel;
  description: string;
  shell_specific?: ShellType;
}

export interface Guardrail {
  id: string;
  pattern: DangerPattern;
  category: GuardrailCategory;
  examples_blocked: string[];
  examples_safe: string[];
  explanation: string;
  learn_more_url?: string;
  tags: string[];
  community_notes: CommunityNote[];
  stats: GuardrailStats;
  created_at: string;
  updated_at: string;
}

export interface GuideMetrics {
  upvotes: number;
  downvotes: number;
  execution_count: number;
  success_count: number;
  failure_count: number;
  view_count: number;
  last_executed?: string;
}

export interface Guide {
  id: string;
  title: string;
  description: string;
  category: GuideCategory;
  difficulty: GuideDifficulty;
  tags: string[];
  natural_language_prompt: string;
  generated_command: string;
  shell_type: ShellType;
  risk_level: RiskLevel;
  author: string;
  created_at: string;
  updated_at: string;
  prerequisites: string[];
  expected_outcomes: string[];
  metrics: GuideMetrics;
  related_guides: string[];
  related_guardrails: string[];
  alternatives: string[];
  content: string; // Markdown content
}

// Helper types for filtering and sorting
export interface GuardrailFilter {
  category?: GuardrailCategory;
  risk_level?: RiskLevel;
  shell_type?: ShellType;
  search?: string;
}

export interface GuideFilter {
  category?: GuideCategory;
  difficulty?: GuideDifficulty;
  risk_level?: RiskLevel;
  shell_type?: ShellType;
  search?: string;
  min_quality?: number;
  popular_only?: boolean;
}

export type SortOrder = 'newest' | 'oldest' | 'best_quality' | 'most_popular' | 'alphabetical';

// Category metadata
export interface CategoryInfo {
  name: string;
  icon: string;
  description: string;
  count?: number;
}

// Search result
export interface SearchResult<T> {
  item: T;
  score: number;
  matches: string[];
}
