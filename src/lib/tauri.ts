import { invoke } from "@tauri-apps/api/core";
import type {
  ExecutionRecord,
  ExecutionRating,
  ExecutionVote,
  Analytics,
  HistoryFilter,
  CommandGenerationRequest,
  CommandGenerationResponse,
  UserConfiguration,
} from "../types";

export async function getConfig(): Promise<UserConfiguration> {
  return await invoke("get_config");
}

export async function updateConfig(config: UserConfiguration): Promise<void> {
  return await invoke("update_config", { configJson: config });
}

export async function getExecutionHistory(
  filter: HistoryFilter
): Promise<ExecutionRecord[]> {
  return await invoke("get_execution_history", { filter });
}

export async function addExecution(
  record: ExecutionRecord
): Promise<number> {
  return await invoke("add_execution", { record });
}

export async function getExecutionById(
  id: number
): Promise<ExecutionRecord | null> {
  return await invoke("get_execution_by_id", { id });
}

export async function deleteExecution(id: number): Promise<void> {
  return await invoke("delete_execution", { id });
}

export async function rateExecution(
  executionId: number,
  rating: number,
  feedback?: string
): Promise<number> {
  return await invoke("rate_execution", {
    executionId,
    rating,
    feedback: feedback || null,
  });
}

export async function getExecutionRatings(
  executionId: number
): Promise<ExecutionRating[]> {
  return await invoke("get_execution_ratings", { executionId });
}

export async function voteExecution(
  executionId: number,
  voteType: "up" | "down"
): Promise<number> {
  return await invoke("vote_execution", { executionId, voteType });
}

export async function getExecutionVotes(
  executionId: number
): Promise<ExecutionVote[]> {
  return await invoke("get_execution_votes", { executionId });
}

export async function generateCommand(
  request: CommandGenerationRequest
): Promise<CommandGenerationResponse> {
  return await invoke("generate_command", { request });
}

export async function getAnalytics(): Promise<Analytics> {
  return await invoke("get_analytics");
}

export async function exportHistory(format: string): Promise<string> {
  return await invoke("export_history", { format });
}
