/**
 * MediaPipe LLM Inference Engine
 * Core engine for in-browser LLM inference using MediaPipe Tasks GenAI
 */

import type {
  LLMInferenceOptions,
  ModelInfo,
  ModelState,
  InferenceState,
  WebGPUSupport,
  ChatMessage,
  UserIntent,
  DEFAULT_LLM_OPTIONS,
} from './types';

// MediaPipe types (loaded dynamically)
declare global {
  interface Window {
    FilesetResolver: any;
    LlmInference: any;
  }
}

/**
 * Terminal Tutor LLM Engine
 * Handles model loading, inference, and natural language to command translation
 */
export class TerminalTutorEngine {
  private llmInference: any = null;
  private modelState: ModelState = { status: 'idle' };
  private inferenceState: InferenceState = { status: 'idle' };
  private conversationHistory: ChatMessage[] = [];
  private options: LLMInferenceOptions;
  private listeners: Map<string, Set<Function>> = new Map();

  constructor(options: Partial<LLMInferenceOptions> = {}) {
    this.options = {
      maxTokens: options.maxTokens ?? 1024,
      topK: options.topK ?? 40,
      temperature: options.temperature ?? 0.8,
      randomSeed: options.randomSeed ?? 101,
    };
  }

  /**
   * Check WebGPU availability
   */
  async checkWebGPUSupport(): Promise<WebGPUSupport> {
    if (!navigator.gpu) {
      return {
        available: false,
        errorMessage: 'WebGPU is not supported in this browser. Please use Chrome 113+ or Edge 113+.',
      };
    }

    try {
      const adapter = await navigator.gpu.requestAdapter();
      if (!adapter) {
        return {
          available: false,
          errorMessage: 'No suitable GPU adapter found.',
        };
      }

      const device = await adapter.requestDevice();
      return {
        available: true,
        adapter,
        device,
      };
    } catch (error) {
      return {
        available: false,
        errorMessage: `WebGPU initialization failed: ${error}`,
      };
    }
  }

  /**
   * Load MediaPipe scripts dynamically
   */
  private async loadMediaPipeScripts(): Promise<void> {
    if (window.FilesetResolver && window.LlmInference) {
      return; // Already loaded
    }

    return new Promise((resolve, reject) => {
      const script = document.createElement('script');
      script.type = 'module';
      script.innerHTML = `
        import { FilesetResolver, LlmInference } from 'https://cdn.jsdelivr.net/npm/@mediapipe/tasks-genai@latest';
        window.FilesetResolver = FilesetResolver;
        window.LlmInference = LlmInference;
        window.dispatchEvent(new Event('mediapipe-loaded'));
      `;

      window.addEventListener('mediapipe-loaded', () => resolve(), { once: true });
      script.onerror = () => reject(new Error('Failed to load MediaPipe scripts'));
      document.head.appendChild(script);
    });
  }

  /**
   * Load a model for inference
   */
  async loadModel(model: ModelInfo, onProgress?: (progress: number) => void): Promise<void> {
    try {
      this.updateModelState({ status: 'downloading', progress: 0 });
      this.emit('model:downloading', { modelId: model.id, progress: 0 });

      // Load MediaPipe scripts
      await this.loadMediaPipeScripts();

      // Initialize FilesetResolver
      const genai = await window.FilesetResolver.forGenAiTasks(
        'https://cdn.jsdelivr.net/npm/@mediapipe/tasks-genai@latest/wasm'
      );

      this.updateModelState({ status: 'loading' });
      this.emit('model:loading', { modelId: model.id });

      // Fetch model with progress tracking
      const modelBlob = await this.fetchModelWithProgress(model.downloadUrl, (progress) => {
        this.updateModelState({ status: 'downloading', progress });
        this.emit('model:downloading', { modelId: model.id, progress });
        onProgress?.(progress);
      });

      // Create LLM Inference instance
      this.llmInference = await window.LlmInference.createFromOptions(genai, {
        baseOptions: {
          modelAssetBuffer: await modelBlob.arrayBuffer(),
        },
        maxTokens: this.options.maxTokens,
        topK: this.options.topK,
        temperature: this.options.temperature,
        randomSeed: this.options.randomSeed,
      });

      this.updateModelState({ status: 'ready', modelId: model.id });
      this.emit('model:loaded', { modelId: model.id });

      // Cache model info
      localStorage.setItem('lastLoadedModel', model.id);
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Unknown error loading model';
      this.updateModelState({ status: 'error', message });
      this.emit('model:error', { error: message });
      throw error;
    }
  }

  /**
   * Load a model from a local file
   */
  async loadModelFromFile(
    file: File,
    modelInfo: ModelInfo,
    onProgress?: (progress: number) => void
  ): Promise<void> {
    try {
      this.updateModelState({ status: 'loading' });
      this.emit('model:loading', { modelId: modelInfo.id });
      onProgress?.(10);

      // Load MediaPipe scripts
      await this.loadMediaPipeScripts();
      onProgress?.(30);

      // Initialize FilesetResolver
      const genai = await window.FilesetResolver.forGenAiTasks(
        'https://cdn.jsdelivr.net/npm/@mediapipe/tasks-genai@latest/wasm'
      );
      onProgress?.(50);

      // Read file as ArrayBuffer
      const arrayBuffer = await file.arrayBuffer();
      onProgress?.(70);

      // Create LLM Inference instance
      this.llmInference = await window.LlmInference.createFromOptions(genai, {
        baseOptions: {
          modelAssetBuffer: arrayBuffer,
        },
        maxTokens: this.options.maxTokens,
        topK: this.options.topK,
        temperature: this.options.temperature,
        randomSeed: this.options.randomSeed,
      });

      onProgress?.(100);
      this.updateModelState({ status: 'ready', modelId: modelInfo.id });
      this.emit('model:loaded', { modelId: modelInfo.id });

    } catch (error) {
      const message = error instanceof Error ? error.message : 'Unknown error loading model';
      this.updateModelState({ status: 'error', message });
      this.emit('model:error', { error: message });
      throw error;
    }
  }

  /**
   * Fetch model with download progress tracking
   */
  private async fetchModelWithProgress(
    url: string,
    onProgress: (progress: number) => void
  ): Promise<Blob> {
    const response = await fetch(url);

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}. Access to model may be restricted.`);
    }

    const contentLength = response.headers.get('content-length');
    const total = contentLength ? parseInt(contentLength, 10) : 0;

    if (!response.body) {
      // Fallback if streaming not supported
      const blob = await response.blob();
      onProgress(100);
      return blob;
    }

    const reader = response.body.getReader();
    const chunks: Uint8Array[] = [];
    let loaded = 0;

    while (true) {
      const { done, value } = await reader.read();

      if (done) break;

      chunks.push(value);
      loaded += value.length;

      if (total > 0) {
        onProgress(Math.round((loaded / total) * 100));
      }
    }

    const blob = new Blob(chunks);
    onProgress(100);
    return blob;
  }

  /**
   * Generate response using the loaded model
   */
  async generateResponse(
    prompt: string,
    onToken?: (token: string) => void
  ): Promise<string> {
    if (!this.llmInference) {
      throw new Error('Model not loaded. Please load a model first.');
    }

    try {
      this.updateInferenceState({ status: 'generating', tokens: '' });
      this.emit('inference:start', { prompt });

      // Format prompt with Gemma template
      const formattedPrompt = this.formatPrompt(prompt);

      let fullResponse = '';

      // Use streaming if callback provided
      if (onToken) {
        await this.llmInference.generateResponse(formattedPrompt, (partialResult: string, done: boolean) => {
          if (partialResult) {
            fullResponse = partialResult;
            this.updateInferenceState({ status: 'generating', tokens: partialResult });
            this.emit('inference:token', { token: partialResult });
            onToken(partialResult);
          }

          if (done) {
            this.updateInferenceState({ status: 'complete', response: fullResponse });
            this.emit('inference:complete', { response: fullResponse });
          }
        });
      } else {
        // Non-streaming response
        fullResponse = await this.llmInference.generateResponse(formattedPrompt);
        this.updateInferenceState({ status: 'complete', response: fullResponse });
        this.emit('inference:complete', { response: fullResponse });
      }

      // Add to conversation history
      this.addToHistory('user', prompt);
      this.addToHistory('model', fullResponse);

      return fullResponse;
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Inference failed';
      this.updateInferenceState({ status: 'error', message });
      this.emit('inference:error', { error: message });
      throw error;
    }
  }

  /**
   * Format prompt using Gemma template
   */
  private formatPrompt(userMessage: string): string {
    const systemPrompt = this.getTerminalTutorSystemPrompt();

    // Build conversation context
    let prompt = `<start_of_turn>user\n${systemPrompt}\n\n`;

    // Add conversation history (last 5 exchanges)
    const recentHistory = this.conversationHistory.slice(-10);
    for (const msg of recentHistory) {
      if (msg.role === 'user') {
        prompt += `User: ${msg.content}\n`;
      } else if (msg.role === 'model') {
        prompt += `<end_of_turn>\n<start_of_turn>model\n${msg.content}\n`;
      }
    }

    // Add current message
    prompt += `User: ${userMessage}\n<end_of_turn>\n<start_of_turn>model\n`;

    return prompt;
  }

  /**
   * Get the system prompt for terminal tutoring
   */
  private getTerminalTutorSystemPrompt(): string {
    return `You are Caro, a friendly and knowledgeable terminal tutor. Your role is to teach users how to use Unix/Linux terminal commands through natural language understanding.

CORE PRINCIPLES:
1. SEMANTIC FIRST: Understand the user's goal, then provide the terminal solution
2. TEACH CONCEPTS: Explain the WHY behind commands, not just syntax
3. PROGRESSIVE COMPLEXITY: Start simple, gradually introduce advanced features
4. SAFETY FIRST: Warn about destructive operations
5. ENCOURAGEMENT: Every interaction is a learning opportunity

When users provide natural language input, help them by:
1. [TRANSLATION]: "You want to [user's goal]. Let's do that with:"
2. [COMMAND]: Show the actual terminal command in a code block
3. [EXPLANATION]: Explain each part of the command briefly
4. [NEXT STEP]: Suggest what they might want to learn next

For navigation commands (pwd, ls, cd), explain like exploring a folder structure.
For file operations (touch, mkdir, cp, mv, rm), emphasize careful usage.
For viewing commands (cat, head, tail), explain when to use each.

Keep responses concise but educational. Use friendly language and celebrate learning milestones.

If the user asks something dangerous (like rm -rf /), explain why it's dangerous and suggest safer alternatives.

Always format commands in code blocks for clarity.`;
  }

  /**
   * Classify user intent from natural language
   */
  classifyIntent(input: string): UserIntent {
    const normalizedInput = input.toLowerCase().trim();

    // Navigation patterns
    if (/\b(go to|navigate|move to|change.*(directory|folder)|cd|where am i|current.*(location|directory|folder))\b/.test(normalizedInput)) {
      return 'navigation';
    }

    // Exploration patterns
    if (/\b(show|list|see|what.*(here|inside|folder)|ls|find|search|look)\b/.test(normalizedInput)) {
      return 'exploration';
    }

    // Creation patterns
    if (/\b(create|make|new|touch|mkdir|add)\b/.test(normalizedInput)) {
      return 'creation';
    }

    // Modification patterns
    if (/\b(copy|move|rename|edit|change|update|cp|mv)\b/.test(normalizedInput)) {
      return 'modification';
    }

    // Inspection patterns
    if (/\b(read|view|open|cat|head|tail|less|more|content)\b/.test(normalizedInput)) {
      return 'inspection';
    }

    // Deletion patterns (dangerous)
    if (/\b(delete|remove|rm|rmdir)\b/.test(normalizedInput)) {
      return 'modification';
    }

    // Help patterns
    if (/\b(help|how|what does|explain|teach|learn)\b/.test(normalizedInput)) {
      return 'help';
    }

    // Automation patterns
    if (/\b(script|automate|pipe|chain|schedule|cron)\b/.test(normalizedInput)) {
      return 'automation';
    }

    return 'inquiry';
  }

  /**
   * Translate natural language to terminal command
   */
  async translateToCommand(naturalLanguage: string): Promise<{
    command: string;
    explanation: string;
    safetyLevel: 'safe' | 'moderate' | 'dangerous';
  }> {
    const prompt = `Translate this natural language request to a Unix terminal command:
"${naturalLanguage}"

Respond in this exact JSON format (no markdown, just JSON):
{
  "command": "the exact command",
  "explanation": "brief explanation of what it does",
  "safetyLevel": "safe|moderate|dangerous"
}`;

    const response = await this.generateResponse(prompt);

    try {
      // Extract JSON from response
      const jsonMatch = response.match(/\{[\s\S]*\}/);
      if (jsonMatch) {
        return JSON.parse(jsonMatch[0]);
      }
    } catch {
      // Fallback parsing
    }

    return {
      command: 'echo "Could not translate command"',
      explanation: 'Unable to translate the request',
      safetyLevel: 'safe',
    };
  }

  /**
   * Add message to conversation history
   */
  private addToHistory(role: 'user' | 'model' | 'system', content: string): void {
    this.conversationHistory.push({
      id: crypto.randomUUID(),
      role,
      content,
      timestamp: new Date(),
    });

    // Keep only last 20 messages
    if (this.conversationHistory.length > 20) {
      this.conversationHistory = this.conversationHistory.slice(-20);
    }
  }

  /**
   * Clear conversation history
   */
  clearHistory(): void {
    this.conversationHistory = [];
    this.emit('history:cleared', {});
  }

  /**
   * Get current conversation history
   */
  getHistory(): ChatMessage[] {
    return [...this.conversationHistory];
  }

  /**
   * Update model state and notify listeners
   */
  private updateModelState(state: ModelState): void {
    this.modelState = state;
  }

  /**
   * Update inference state
   */
  private updateInferenceState(state: InferenceState): void {
    this.inferenceState = state;
  }

  /**
   * Get current model state
   */
  getModelState(): ModelState {
    return this.modelState;
  }

  /**
   * Get current inference state
   */
  getInferenceState(): InferenceState {
    return this.inferenceState;
  }

  /**
   * Update inference options
   */
  updateOptions(options: Partial<LLMInferenceOptions>): void {
    this.options = { ...this.options, ...options };
  }

  /**
   * Get current options
   */
  getOptions(): LLMInferenceOptions {
    return { ...this.options };
  }

  /**
   * Event emitter methods
   */
  on(event: string, callback: Function): void {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, new Set());
    }
    this.listeners.get(event)!.add(callback);
  }

  off(event: string, callback: Function): void {
    this.listeners.get(event)?.delete(callback);
  }

  private emit(event: string, data: any): void {
    this.listeners.get(event)?.forEach((callback) => callback(data));
  }

  /**
   * Cleanup resources
   */
  dispose(): void {
    this.llmInference = null;
    this.conversationHistory = [];
    this.listeners.clear();
    this.updateModelState({ status: 'idle' });
    this.updateInferenceState({ status: 'idle' });
  }
}

// Singleton instance for global access
let engineInstance: TerminalTutorEngine | null = null;

export function getTerminalTutorEngine(options?: Partial<LLMInferenceOptions>): TerminalTutorEngine {
  if (!engineInstance) {
    engineInstance = new TerminalTutorEngine(options);
  }
  return engineInstance;
}

export function resetTerminalTutorEngine(): void {
  engineInstance?.dispose();
  engineInstance = null;
}
