import { create } from 'zustand';
import { immer } from 'zustand/middleware/immer';

export interface EditorSnapshot {
  scenarioPath: string;
  isDirty: boolean;
  timestamp: number;
  actionName: string;
}

interface EditorState {
  scenarioPath: string;
  isDirty: boolean;
  selectedNodeId: string | null;
  history: EditorSnapshot[];
  historyIndex: number;

  // Actions
  setScenarioPath: (path: string) => void;
  setDirty: (dirty: boolean) => void;
  setSelectedNodeId: (id: string | null) => void;

  // History (undo/redo)
  saveSnapshot: (actionName: string) => void;
  undo: () => void;
  redo: () => void;
  canUndo: () => boolean;
  canRedo: () => boolean;
}

export const useEditorStore = create<EditorState>()(
  immer((set, get) => ({
    scenarioPath: '',
    isDirty: false,
    selectedNodeId: null,
    history: [],
    historyIndex: -1,

    setScenarioPath: (path: string) =>
      set((state) => {
        state.scenarioPath = path;
      }),

    setDirty: (dirty: boolean) =>
      set((state) => {
        state.isDirty = dirty;
      }),

    setSelectedNodeId: (id: string | null) =>
      set((state) => {
        state.selectedNodeId = id;
      }),

    saveSnapshot: (actionName: string) =>
      set((state) => {
        // Remove any redo history
        state.history = state.history.slice(0, state.historyIndex + 1);

        // Add new snapshot (max 50)
        state.history.push({
          scenarioPath: state.scenarioPath,
          isDirty: true,
          timestamp: Date.now(),
          actionName,
        });

        if (state.history.length > 50) {
          state.history.shift();
        } else {
          state.historyIndex++;
        }
      }),

    undo: () =>
      set((state) => {
        if (state.historyIndex > 0) {
          state.historyIndex--;
          const snapshot = state.history[state.historyIndex];
          state.scenarioPath = snapshot.scenarioPath;
          state.isDirty = true;
        }
      }),

    redo: () =>
      set((state) => {
        if (state.historyIndex < state.history.length - 1) {
          state.historyIndex++;
          const snapshot = state.history[state.historyIndex];
          state.scenarioPath = snapshot.scenarioPath;
          state.isDirty = true;
        }
      }),

    canUndo: () => {
      const state = get();
      return state.historyIndex > 0;
    },

    canRedo: () => {
      const state = get();
      return state.historyIndex < state.history.length - 1;
    },
  }))
);
