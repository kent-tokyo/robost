import { create } from 'zustand';
import { immer } from 'zustand/middleware/immer';
import { ScenarioStep } from '../types';

interface CanvasState {
  selectedNodeIds: Set<string>;
  clipboard: ScenarioStep | null;
  searchQuery: string;
  searchHighlightIds: Set<string>;
  filterType: string | null;
  expandedGroupIds: Set<string>;

  // Selection actions
  selectNode: (nodeId: string) => void;
  deselectNode: (nodeId: string) => void;
  toggleNodeSelection: (nodeId: string) => void;
  clearSelection: () => void;
  isNodeSelected: (nodeId: string) => boolean;
  getSelectedNodeIds: () => string[];

  // Clipboard actions
  copyToClipboard: (step: ScenarioStep) => void;
  getFromClipboard: () => ScenarioStep | null;
  clearClipboard: () => void;

  // Search actions
  setSearchQuery: (query: string) => void;
  setSearchHighlights: (ids: Set<string>) => void;
  getSearchQuery: () => string;

  // Filter actions
  setFilterType: (type: string | null) => void;
  getFilterType: () => string | null;

  // Group expansion actions
  toggleGroupExpanded: (groupId: string) => void;
  isGroupExpanded: (groupId: string) => boolean;
}

export const useCanvasStore = create<CanvasState>()(
  immer((set, get) => ({
    selectedNodeIds: new Set<string>(),
    clipboard: null,
    searchQuery: '',
    searchHighlightIds: new Set<string>(),
    filterType: null,
    expandedGroupIds: new Set<string>(),

    selectNode: (nodeId: string) =>
      set((state) => {
        state.selectedNodeIds.add(nodeId);
      }),

    deselectNode: (nodeId: string) =>
      set((state) => {
        state.selectedNodeIds.delete(nodeId);
      }),

    toggleNodeSelection: (nodeId: string) =>
      set((state) => {
        if (state.selectedNodeIds.has(nodeId)) {
          state.selectedNodeIds.delete(nodeId);
        } else {
          state.selectedNodeIds.add(nodeId);
        }
      }),

    clearSelection: () =>
      set((state) => {
        state.selectedNodeIds.clear();
      }),

    isNodeSelected: (nodeId: string) => {
      return get().selectedNodeIds.has(nodeId);
    },

    getSelectedNodeIds: () => {
      return Array.from(get().selectedNodeIds);
    },

    copyToClipboard: (step: ScenarioStep) =>
      set((state) => {
        state.clipboard = JSON.parse(JSON.stringify(step));
      }),

    getFromClipboard: () => {
      return get().clipboard;
    },

    clearClipboard: () =>
      set((state) => {
        state.clipboard = null;
      }),

    setSearchQuery: (query: string) =>
      set((state) => {
        state.searchQuery = query;
      }),

    setSearchHighlights: (ids: Set<string>) =>
      set((state) => {
        state.searchHighlightIds = ids;
      }),

    getSearchQuery: () => {
      return get().searchQuery;
    },

    setFilterType: (type: string | null) =>
      set((state) => {
        state.filterType = type;
      }),

    getFilterType: () => {
      return get().filterType;
    },

    toggleGroupExpanded: (groupId: string) =>
      set((state) => {
        if (state.expandedGroupIds.has(groupId)) {
          state.expandedGroupIds.delete(groupId);
        } else {
          state.expandedGroupIds.add(groupId);
        }
      }),

    isGroupExpanded: (groupId: string) => {
      return get().expandedGroupIds.has(groupId);
    },
  }))
);
