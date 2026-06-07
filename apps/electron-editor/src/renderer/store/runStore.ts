import { create } from 'zustand';
import { immer } from 'zustand/middleware/immer';

export interface LogEntry {
  timestamp: number;
  level: 'info' | 'warn' | 'error' | 'debug';
  message: string;
}

export interface StepExecution {
  stepId: string;
  stepName: string;
  stepType: string;
  status: 'started' | 'completed' | 'failed' | 'skipped';
  startTime: number;
  endTime?: number;
  duration?: number;
  errorMessage?: string;
  variables?: Record<string, any>;
}

export interface ExecutionRecord {
  id: string;
  scenarioName: string;
  timestamp: number;
  status: 'success' | 'failed' | 'stopped';
  totalSteps: number;
  completedSteps: number;
  duration: number;
  logs: LogEntry[];
  stepExecutions: StepExecution[];
  variables?: Record<string, any>;
}

export interface WatchVariable {
  name: string;
  value: any;
  history: Array<{ timestamp: number; value: any }>;
}

export interface PickedCoordinate {
  id: string;
  x: number;
  y: number;
  color?: string;
  timestamp: number;
}

interface RunState {
  isRunning: boolean;
  isPaused: boolean;
  currentStepIndex: number;
  totalSteps: number;
  elapsedMs: number;
  logs: LogEntry[];
  serverPort: number | null;

  // History & execution tracking
  executionHistory: ExecutionRecord[];
  currentExecution: {
    id: string;
    scenarioName: string;
    startTime: number;
    stepExecutions: StepExecution[];
    variables: Record<string, any>;
  } | null;

  // Debugging features
  breakpoints: Set<string>;
  watchVariables: Map<string, WatchVariable>;

  // Screen operations
  pickedCoordinates: PickedCoordinate[];

  // Actions
  startRun: (totalSteps: number, scenarioName: string) => void;
  stopRun: (status: 'success' | 'failed' | 'stopped') => void;
  pauseRun: () => void;
  resumeRun: () => void;
  setCurrentStep: (index: number, elapsedMs: number) => void;
  addLog: (level: LogEntry['level'], message: string) => void;
  clearLogs: () => void;
  setServerPort: (port: number) => void;

  // Execution tracking
  recordStepExecution: (execution: StepExecution) => void;
  setExecutionVariables: (variables: Record<string, any>) => void;
  saveExecutionRecord: () => void;

  // Breakpoints
  addBreakpoint: (stepId: string) => void;
  removeBreakpoint: (stepId: string) => void;
  toggleBreakpoint: (stepId: string) => void;
  isBreakpoint: (stepId: string) => boolean;

  // Watch variables
  addWatchVariable: (name: string) => void;
  removeWatchVariable: (name: string) => void;
  updateWatchVariable: (name: string, value: any) => void;

  // History management
  clearExecutionHistory: () => void;
  deleteHistoryRecord: (id: string) => void;
  getHistoryRecord: (id: string) => ExecutionRecord | null;

  // Screen operations
  addPickedCoordinate: (coord: PickedCoordinate) => void;
  clearPickedCoordinates: () => void;
  removePickedCoordinate: (id: string) => void;
}

export const useRunStore = create<RunState>()(
  immer((set, get) => ({
    isRunning: false,
    isPaused: false,
    currentStepIndex: 0,
    totalSteps: 0,
    elapsedMs: 0,
    logs: [],
    serverPort: null,
    executionHistory: [],
    currentExecution: null,
    breakpoints: new Set<string>(),
    watchVariables: new Map<string, WatchVariable>(),
    pickedCoordinates: [],

    startRun: (totalSteps: number, scenarioName: string) =>
      set((state) => {
        const executionId = `exec-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
        state.isRunning = true;
        state.isPaused = false;
        state.currentStepIndex = 0;
        state.totalSteps = totalSteps;
        state.elapsedMs = 0;
        state.logs = [];
        state.currentExecution = {
          id: executionId,
          scenarioName,
          startTime: Date.now(),
          stepExecutions: [],
          variables: {},
        };
      }),

    stopRun: (status: 'success' | 'failed' | 'stopped') =>
      set((state) => {
        state.isRunning = false;
        state.isPaused = false;
        if (state.currentExecution) {
          const duration = Date.now() - state.currentExecution.startTime;
          const record: ExecutionRecord = {
            id: state.currentExecution.id,
            scenarioName: state.currentExecution.scenarioName,
            timestamp: state.currentExecution.startTime,
            status,
            totalSteps: state.totalSteps,
            completedSteps: state.currentStepIndex,
            duration,
            logs: [...state.logs],
            stepExecutions: [...state.currentExecution.stepExecutions],
            variables: state.currentExecution.variables,
          };
          state.executionHistory.unshift(record);
          // Keep only last 100 records
          if (state.executionHistory.length > 100) {
            state.executionHistory = state.executionHistory.slice(0, 100);
          }
        }
        state.currentExecution = null;
      }),

    pauseRun: () =>
      set((state) => {
        state.isPaused = true;
      }),

    resumeRun: () =>
      set((state) => {
        state.isPaused = false;
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

    recordStepExecution: (execution: StepExecution) =>
      set((state) => {
        if (state.currentExecution) {
          state.currentExecution.stepExecutions.push(execution);
        }
      }),

    setExecutionVariables: (variables: Record<string, any>) =>
      set((state) => {
        if (state.currentExecution) {
          state.currentExecution.variables = { ...state.currentExecution.variables, ...variables };
        }
      }),

    saveExecutionRecord: () => {
      // Called manually if needed
      const state = get();
      if (state.currentExecution) {
        const duration = Date.now() - state.currentExecution.startTime;
        const record: ExecutionRecord = {
          id: state.currentExecution.id,
          scenarioName: state.currentExecution.scenarioName,
          timestamp: state.currentExecution.startTime,
          status: 'success',
          totalSteps: state.totalSteps,
          completedSteps: state.currentStepIndex,
          duration,
          logs: [...state.logs],
          stepExecutions: [...state.currentExecution.stepExecutions],
          variables: state.currentExecution.variables,
        };
        set((s) => {
          s.executionHistory.unshift(record);
          if (s.executionHistory.length > 100) {
            s.executionHistory = s.executionHistory.slice(0, 100);
          }
        });
      }
    },

    addBreakpoint: (stepId: string) =>
      set((state) => {
        state.breakpoints.add(stepId);
      }),

    removeBreakpoint: (stepId: string) =>
      set((state) => {
        state.breakpoints.delete(stepId);
      }),

    toggleBreakpoint: (stepId: string) =>
      set((state) => {
        if (state.breakpoints.has(stepId)) {
          state.breakpoints.delete(stepId);
        } else {
          state.breakpoints.add(stepId);
        }
      }),

    isBreakpoint: (stepId: string) => {
      return get().breakpoints.has(stepId);
    },

    addWatchVariable: (name: string) =>
      set((state) => {
        if (!state.watchVariables.has(name)) {
          state.watchVariables.set(name, {
            name,
            value: undefined,
            history: [],
          });
        }
      }),

    removeWatchVariable: (name: string) =>
      set((state) => {
        state.watchVariables.delete(name);
      }),

    updateWatchVariable: (name: string, value: any) =>
      set((state) => {
        const watch = state.watchVariables.get(name);
        if (watch) {
          watch.value = value;
          watch.history.push({
            timestamp: Date.now(),
            value,
          });
          // Keep last 50 history entries
          if (watch.history.length > 50) {
            watch.history = watch.history.slice(-50);
          }
        }
      }),

    clearExecutionHistory: () =>
      set((state) => {
        state.executionHistory = [];
      }),

    deleteHistoryRecord: (id: string) =>
      set((state) => {
        state.executionHistory = state.executionHistory.filter((r) => r.id !== id);
      }),

    getHistoryRecord: (id: string) => {
      return get().executionHistory.find((r) => r.id === id) || null;
    },

    addPickedCoordinate: (coord: PickedCoordinate) =>
      set((state) => {
        state.pickedCoordinates.unshift(coord);
        // Keep only last 50 coordinates
        if (state.pickedCoordinates.length > 50) {
          state.pickedCoordinates = state.pickedCoordinates.slice(0, 50);
        }
      }),

    clearPickedCoordinates: () =>
      set((state) => {
        state.pickedCoordinates = [];
      }),

    removePickedCoordinate: (id: string) =>
      set((state) => {
        state.pickedCoordinates = state.pickedCoordinates.filter((c) => c.id !== id);
      }),
  }))
);
