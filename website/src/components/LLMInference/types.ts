/**
 * MediaPipe LLM Inference Types
 * Type definitions for in-browser LLM inference with MediaPipe Tasks GenAI
 */

// Model configuration and metadata
export interface ModelInfo {
  id: string;
  name: string;
  description: string;
  size: number; // in MB
  sizeFormatted: string;
  quantization: 'int4' | 'int8' | 'fp16';
  capabilities: ('text' | 'vision' | 'audio')[];
  source: 'huggingface' | 'kaggle' | 'local';
  downloadUrl: string;
  recommended?: boolean;
}

// LLM inference configuration options
export interface LLMInferenceOptions {
  maxTokens: number;
  topK: number;
  temperature: number;
  randomSeed?: number;
}

// Default configuration
export const DEFAULT_LLM_OPTIONS: LLMInferenceOptions = {
  maxTokens: 1024,
  topK: 40,
  temperature: 0.8,
  randomSeed: 101,
};

// Available models for the terminal tutor
// Note: Some HuggingFace models require authentication. We provide both online and local options.
export const AVAILABLE_MODELS: ModelInfo[] = [
  {
    id: 'gemma3-1b',
    name: 'Gemma3 1B',
    description: 'Fast, lightweight model ideal for terminal education',
    size: 555,
    sizeFormatted: '555 MB',
    quantization: 'int4',
    capabilities: ['text'],
    source: 'huggingface',
    downloadUrl: 'https://huggingface.co/nicecui/Gemma3-1B-IT/resolve/main/gemma3-1b-it-int4.task',
    recommended: true,
  },
  {
    id: 'gemma2-2b',
    name: 'Gemma2 2B',
    description: 'Balanced performance and capability',
    size: 1350,
    sizeFormatted: '1.35 GB',
    quantization: 'int4',
    capabilities: ['text'],
    source: 'huggingface',
    downloadUrl: 'https://huggingface.co/nicecui/Gemma2-2B-IT/resolve/main/gemma2-2b-it-gpu-int4.bin',
  },
  {
    id: 'local-upload',
    name: 'Load Local Model',
    description: 'Upload a .task or .bin model file from your computer',
    size: 0,
    sizeFormatted: 'Varies',
    quantization: 'int4',
    capabilities: ['text'],
    source: 'local',
    downloadUrl: '',
  },
];

// Download links for models that require authentication
export const MODEL_DOWNLOAD_SOURCES = {
  'gemma3-1b': {
    kaggle: 'https://www.kaggle.com/models/google/gemma-3/tfLite/gemma3-1b-it-int4',
    huggingface: 'https://huggingface.co/litert-community/Gemma3-1B-IT',
  },
  'gemma2-2b': {
    kaggle: 'https://www.kaggle.com/models/google/gemma-2/tfLite/gemma2-2b-it-gpu-int4',
    huggingface: 'https://huggingface.co/litert-community/Gemma2-2B-IT',
  },
};

// Chat message structure
export interface ChatMessage {
  id: string;
  role: 'user' | 'model' | 'system';
  content: string;
  timestamp: Date;
  isStreaming?: boolean;
}

// Terminal command with educational context
export interface TerminalCommand {
  id: string;
  command: string;
  description: string;
  category: 'navigation' | 'files' | 'viewing' | 'system' | 'git' | 'advanced';
  difficulty: 'beginner' | 'intermediate' | 'advanced';
  examples: CommandExample[];
  relatedCommands?: string[];
  safetyLevel: 'safe' | 'moderate' | 'dangerous';
}

export interface CommandExample {
  input: string;
  output: string;
  explanation: string;
}

// Learning progress tracking
export interface LearningProgress {
  currentLevel: number;
  completedLessons: string[];
  commandsLearned: string[];
  totalScore: number;
  streakDays: number;
  lastSession: Date;
}

// Natural language intent classification
export type UserIntent =
  | 'navigation'
  | 'exploration'
  | 'creation'
  | 'modification'
  | 'inspection'
  | 'automation'
  | 'inquiry'
  | 'help';

// Terminal lesson structure
export interface TerminalLesson {
  id: string;
  title: string;
  description: string;
  level: number;
  objectives: string[];
  commands: string[];
  exercises: LessonExercise[];
  completionCriteria: string[];
}

export interface LessonExercise {
  id: string;
  prompt: string;
  expectedCommand: string;
  hints: string[];
  successMessage: string;
  explanation: string;
}

// Model loading state
export type ModelState =
  | { status: 'idle' }
  | { status: 'downloading'; progress: number }
  | { status: 'loading' }
  | { status: 'ready'; modelId: string }
  | { status: 'error'; message: string };

// Inference state
export type InferenceState =
  | { status: 'idle' }
  | { status: 'generating'; tokens: string }
  | { status: 'complete'; response: string }
  | { status: 'error'; message: string };

// WebGPU compatibility
export interface WebGPUSupport {
  available: boolean;
  adapter?: GPUAdapter;
  device?: GPUDevice;
  errorMessage?: string;
}

// Terminal simulation state
export interface TerminalState {
  currentDirectory: string;
  history: TerminalHistoryEntry[];
  environment: Record<string, string>;
  user: string;
  hostname: string;
}

export interface TerminalHistoryEntry {
  id: string;
  input: string;
  output: string;
  timestamp: Date;
  exitCode: number;
  isSimulated: boolean;
}

// File system simulation for learning
export interface VirtualFileSystem {
  root: VirtualDirectory;
}

export interface VirtualDirectory {
  name: string;
  type: 'directory';
  children: (VirtualDirectory | VirtualFile)[];
  permissions: string;
  owner: string;
  modified: Date;
}

export interface VirtualFile {
  name: string;
  type: 'file';
  content: string;
  size: number;
  permissions: string;
  owner: string;
  modified: Date;
}

// Event types for component communication
export interface LLMEvents {
  'model:selected': { modelId: string };
  'model:loaded': { modelId: string };
  'model:error': { error: string };
  'inference:start': { prompt: string };
  'inference:token': { token: string };
  'inference:complete': { response: string };
  'inference:error': { error: string };
  'lesson:started': { lessonId: string };
  'lesson:completed': { lessonId: string; score: number };
  'command:executed': { command: string; output: string };
}
