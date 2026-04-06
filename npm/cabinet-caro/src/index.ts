/**
 * @caro/cabinet — TypeScript client for caro-server
 *
 * Provides typed access to caro's command generation, execution,
 * and knowledge APIs over HTTP and WebSocket.
 */

// ─── Types ───

export type ApiStatus = "ok" | "blocked" | "error";
export type RiskLevel = "safe" | "moderate" | "high" | "critical";
export type ShellType = "bash" | "zsh" | "sh" | "fish";
export type SafetyLevel = "permissive" | "moderate" | "strict";

export interface CommandRequest {
  input: string;
  shell?: ShellType;
  safety_level?: SafetyLevel;
  context?: string;
  request_id?: string;
  agent_id?: string;
}

export interface CommandResponse {
  request_id: string;
  status: ApiStatus;
  command?: string;
  explanation?: string;
  risk_level?: RiskLevel;
  estimated_impact?: string;
  alternatives?: string[];
  backend_used?: string;
  generation_time_ms?: number;
  confidence_score?: number;
  warnings: string[];
  error?: string;
  reason?: string;
}

export interface ExecuteRequest {
  command: string;
  confirmed: boolean;
  request_id?: string;
  timeout_ms?: number;
}

export interface ExecuteResponse {
  request_id: string;
  status: ApiStatus;
  exit_code?: number;
  stdout?: string;
  stderr?: string;
  execution_time_ms?: number;
  error?: string;
  risk_level?: RiskLevel;
  warnings: string[];
}

export interface HealthResponse {
  status: string;
  version: string;
  backends: {
    static_matcher: boolean;
    embedded: boolean;
    ollama: boolean;
    claude: boolean;
  };
  safety_patterns: number;
  uptime_seconds: number;
}

export interface KnowledgeResult {
  input: string;
  command: string;
  context?: string;
  similarity: number;
  timestamp: string;
  entry_type?: string;
}

export interface KnowledgeSearchResponse {
  results: KnowledgeResult[];
  total: number;
}

export interface KnowledgeRecordRequest {
  input: string;
  command: string;
  context?: string;
  success?: boolean;
  agent_id?: string;
}

export interface KnowledgeRecordResponse {
  status: ApiStatus;
  message: string;
}

export interface KnowledgeExportResponse {
  status: ApiStatus;
  entries: KnowledgeResult[];
  total: number;
}

export interface KnowledgeImportEntry {
  input: string;
  command: string;
  context?: string;
}

export interface KnowledgeImportResponse {
  status: ApiStatus;
  imported: number;
  skipped: number;
}

// ─── WebSocket types ───

export type WsMessage =
  | { type: "command_request"; id: string; input: string; agent_id?: string }
  | { type: "command_result"; id: string; status: ApiStatus; command?: string; explanation?: string; risk_level?: RiskLevel; warnings?: string[]; error?: string }
  | { type: "execution_request"; id: string; command: string; confirmed: boolean }
  | { type: "execution_result"; id: string; exit_code: number; stdout: string; stderr: string; execution_time_ms: number }
  | { type: "knowledge_update"; entries: KnowledgeResult[] }
  | { type: "heartbeat" }
  | { type: "error"; message: string };

// ─── Client options ───

export interface CaroClientOptions {
  /** Base URL of the caro-server (e.g. "http://localhost:3847") */
  url: string;
  /** Bearer token for authentication */
  token?: string;
  /** Request timeout in milliseconds (default: 30000) */
  timeout?: number;
}

// ─── Errors ───

export class CaroApiError extends Error {
  constructor(
    public readonly status: number,
    public readonly body: unknown,
  ) {
    super(`caro-server responded with ${status}`);
    this.name = "CaroApiError";
  }
}

// ─── HTTP Client ───

export class CaroClient {
  private readonly baseUrl: string;
  private readonly headers: Record<string, string>;
  private readonly timeout: number;

  constructor(options: CaroClientOptions) {
    this.baseUrl = options.url.replace(/\/+$/, "");
    this.timeout = options.timeout ?? 30_000;
    this.headers = { "Content-Type": "application/json" };
    if (options.token) {
      this.headers["Authorization"] = `Bearer ${options.token}`;
    }
  }

  // ─── Core API ───

  /** Check server health. */
  async health(): Promise<HealthResponse> {
    return this.get<HealthResponse>("/api/v1/health");
  }

  /** Generate a shell command from natural language. */
  async generate(input: string | CommandRequest): Promise<CommandResponse> {
    const body: CommandRequest =
      typeof input === "string" ? { input } : input;
    return this.post<CommandResponse>("/api/v1/generate", body);
  }

  /** Execute a previously generated command. */
  async execute(
    command: string,
    options?: { confirmed?: boolean; timeout_ms?: number; request_id?: string },
  ): Promise<ExecuteResponse> {
    const body: ExecuteRequest = {
      command,
      confirmed: options?.confirmed ?? true,
      timeout_ms: options?.timeout_ms,
      request_id: options?.request_id,
    };
    return this.post<ExecuteResponse>("/api/v1/execute", body);
  }

  // ─── Knowledge API ───

  /** Search the knowledge index. */
  async searchKnowledge(
    query: string,
    limit?: number,
  ): Promise<KnowledgeSearchResponse> {
    const params = new URLSearchParams({ q: query });
    if (limit !== undefined) params.set("limit", String(limit));
    return this.get<KnowledgeSearchResponse>(
      `/api/v1/knowledge/search?${params}`,
    );
  }

  /** Record a command result in the knowledge base. */
  async recordKnowledge(
    entry: KnowledgeRecordRequest,
  ): Promise<KnowledgeRecordResponse> {
    return this.post<KnowledgeRecordResponse>("/api/v1/knowledge/record", entry);
  }

  /** Export all knowledge entries. */
  async exportKnowledge(): Promise<KnowledgeExportResponse> {
    return this.get<KnowledgeExportResponse>("/api/v1/knowledge/export");
  }

  /** Import knowledge entries. */
  async importKnowledge(
    entries: KnowledgeImportEntry[],
  ): Promise<KnowledgeImportResponse> {
    return this.post<KnowledgeImportResponse>("/api/v1/knowledge/import", {
      entries,
    });
  }

  // ─── WebSocket ───

  /**
   * Open a WebSocket connection to caro-server.
   *
   * Returns a `CaroWebSocket` wrapper with typed send/receive helpers.
   * Requires a WebSocket implementation (browser-native or `ws` package in Node).
   */
  connectWebSocket(): CaroWebSocket {
    const wsUrl = this.baseUrl
      .replace(/^http/, "ws")
      .concat("/api/v1/ws");
    return new CaroWebSocket(wsUrl, this.headers["Authorization"]);
  }

  // ─── Internal fetch helpers ───

  private async get<T>(path: string): Promise<T> {
    const resp = await fetch(`${this.baseUrl}${path}`, {
      method: "GET",
      headers: this.headers,
      signal: AbortSignal.timeout(this.timeout),
    });
    if (!resp.ok) throw new CaroApiError(resp.status, await resp.json().catch(() => null));
    return resp.json() as Promise<T>;
  }

  private async post<T>(path: string, body: unknown): Promise<T> {
    const resp = await fetch(`${this.baseUrl}${path}`, {
      method: "POST",
      headers: this.headers,
      body: JSON.stringify(body),
      signal: AbortSignal.timeout(this.timeout),
    });
    if (!resp.ok) throw new CaroApiError(resp.status, await resp.json().catch(() => null));
    return resp.json() as Promise<T>;
  }
}

// ─── WebSocket wrapper ───

export type WsEventHandler = (message: WsMessage) => void;

export class CaroWebSocket {
  private ws: WebSocket | null = null;
  private handlers: WsEventHandler[] = [];
  private heartbeatInterval: ReturnType<typeof setInterval> | null = null;

  constructor(
    private readonly url: string,
    private readonly auth?: string,
  ) {}

  /** Open the WebSocket connection. Resolves when connected. */
  connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      const protocols = this.auth ? [this.auth] : undefined;
      this.ws = new WebSocket(this.url, protocols);

      this.ws.onopen = () => {
        this.startHeartbeat();
        resolve();
      };
      this.ws.onerror = (ev) => reject(ev);
      this.ws.onmessage = (ev) => {
        try {
          const msg: WsMessage = JSON.parse(
            typeof ev.data === "string" ? ev.data : "",
          );
          for (const handler of this.handlers) handler(msg);
        } catch {
          // ignore malformed messages
        }
      };
      this.ws.onclose = () => this.stopHeartbeat();
    });
  }

  /** Register a handler for incoming messages. */
  onMessage(handler: WsEventHandler): void {
    this.handlers.push(handler);
  }

  /** Send a typed message. */
  send(message: WsMessage): void {
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
      throw new Error("WebSocket is not connected");
    }
    this.ws.send(JSON.stringify(message));
  }

  /** Request command generation over WebSocket. */
  requestCommand(id: string, input: string, agentId?: string): void {
    this.send({
      type: "command_request",
      id,
      input,
      agent_id: agentId,
    });
  }

  /** Request command execution over WebSocket. */
  requestExecution(
    id: string,
    command: string,
    confirmed: boolean = true,
  ): void {
    this.send({
      type: "execution_request",
      id,
      command,
      confirmed,
    });
  }

  /** Close the WebSocket connection. */
  close(): void {
    this.stopHeartbeat();
    this.ws?.close();
    this.ws = null;
  }

  private startHeartbeat(): void {
    this.heartbeatInterval = setInterval(() => {
      if (this.ws?.readyState === WebSocket.OPEN) {
        this.send({ type: "heartbeat" });
      }
    }, 25_000);
  }

  private stopHeartbeat(): void {
    if (this.heartbeatInterval) {
      clearInterval(this.heartbeatInterval);
      this.heartbeatInterval = null;
    }
  }
}
