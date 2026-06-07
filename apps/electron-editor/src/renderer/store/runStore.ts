import { create } from 'zustand';
import { immer } from 'zustand/middleware/immer';

export interface LogEntry {
  timestamp: number;
  level: 'info' | 'warn' | 'error' | 'debug';
  message: string;
}

interface RunState {
  isRunning: boolean;
  currentStepIndex: number;
  totalSteps: number;
  elapsedMs: number;
  logs: LogEntry[];
  serverPort: number | null;

  // Actions
  startRun: (totalSteps: number) => void;
  stopRun: () => void;
  setCurrentStep: (index: number, elapsedMs: number) => void;
  addLog: (level: LogEntry['level'], message: string) => void;
  clearLogs: () => void;
  setServerPort: (port: number) => void;
}

export const useRunStore = create<RunState>()(
  immer((set) => ({
    isRunning: false,
    currentStepIndex: 0,
    totalSteps: 0,
    elapsedMs: 0,
    logs: [],
    serverPort: null,

    startRun: (totalSteps: number) =>
      set((state) => {
        state.isRunning = true;
        state.currentStepIndex = 0;
        state.totalSteps = totalSteps;
        state.elapsedMs = 0;
        state.logs = [];
      }),

    stopRun: () =>
      set((state) => {
        state.isRunning = false;
      }),

    setCurrentStep: (index: number, elapsedMs: number) =>
      set((state) => {
        state.currentStepIndex = index;
        state.elapsedMs = elapsedMs;
      }),

    addLog: (level: LogEntry['level'], message: string) =>
      set((state) => {
        state.logs.push({
          timestamp: Date.now(),
          level,
          message,
        });
      }),

    clearLogs: () =>
      set((state) => {
        state.logs = [];
      }),

    setServerPort: (port: number) =>
      set((state) => {
        state.serverPort = port;
      }),
  }))
);
