export interface ExecutionRecord {
  id?: number;
  timestamp: string;
  prompt: string;
  generated_command: string;
  explanation: string | null;
  shell_type: string;
  risk_level: string;
  executed: boolean;
  blocked_reason: string | null;
  generation_time_ms: number;
  execution_time_ms: number;
  warnings: string[];
  alternatives: string[];
  backend_used: string;
}

export interface ExecutionRating {
  id?: number;
  execution_id: number;
  rating: number;
  feedback: string | null;
  created_at: string;
}

export interface ExecutionVote {
  id?: number;
  execution_id: number;
  vote_type: "up" | "down";
  created_at: string;
}

export interface Analytics {
  total_executions: number;
  successful_executions: number;
  blocked_executions: number;
  average_generation_time_ms: number;
  most_used_shell: string;
  risk_level_distribution: Record<string, number>;
  backend_usage: Record<string, number>;
}

export interface HistoryFilter {
  shell_type?: string | null;
  risk_level?: string | null;
  executed?: boolean | null;
  blocked?: boolean | null;
  search_query?: string | null;
  limit?: number | null;
  offset?: number | null;
}

export interface CommandGenerationRequest {
  prompt: string;
  shell?: string | null;
  safety_level?: string | null;
  dry_run: boolean;
}

export interface CommandGenerationResponse {
  generated_command: string;
  explanation: string;
  risk_level: string;
  warnings: string[];
  alternatives: string[];
  requires_confirmation: boolean;
  blocked_reason: string | null;
  generation_time_ms: number;
  backend_used: string;
}

export interface UserConfiguration {
  safety_level: string;
  default_shell: string | null;
  log_level: string;
  default_model: string | null;
  cache_max_size_gb: number;
  log_rotation_days: number;
}
