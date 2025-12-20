import { create } from "zustand";
import type {
  ExecutionRecord,
  Analytics,
  UserConfiguration,
  HistoryFilter,
} from "./types";

interface AppStore {
  // Config
  config: UserConfiguration | null;
  setConfig: (config: UserConfiguration) => void;

  // History
  history: ExecutionRecord[];
  setHistory: (history: ExecutionRecord[]) => void;
  addToHistory: (record: ExecutionRecord) => void;

  // Analytics
  analytics: Analytics | null;
  setAnalytics: (analytics: Analytics) => void;

  // UI State
  currentTab: "command" | "history" | "config" | "analytics";
  setCurrentTab: (tab: "command" | "history" | "config" | "analytics") => void;

  historyFilter: HistoryFilter;
  setHistoryFilter: (filter: HistoryFilter) => void;

  selectedExecution: ExecutionRecord | null;
  setSelectedExecution: (execution: ExecutionRecord | null) => void;
}

export const useAppStore = create<AppStore>((set) => ({
  // Config
  config: null,
  setConfig: (config) => set({ config }),

  // History
  history: [],
  setHistory: (history) => set({ history }),
  addToHistory: (record) =>
    set((state) => ({ history: [record, ...state.history] })),

  // Analytics
  analytics: null,
  setAnalytics: (analytics) => set({ analytics }),

  // UI State
  currentTab: "command",
  setCurrentTab: (tab) => set({ currentTab: tab }),

  historyFilter: {
    limit: 50,
    offset: 0,
  },
  setHistoryFilter: (filter) => set({ historyFilter: filter }),

  selectedExecution: null,
  setSelectedExecution: (execution) => set({ selectedExecution: execution }),
}));
