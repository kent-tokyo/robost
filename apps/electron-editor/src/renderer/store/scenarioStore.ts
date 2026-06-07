import { create } from 'zustand';
import { immer } from 'zustand/middleware/immer';
import { ScenarioStep, Scenario } from '../types';
import { Node, Edge } from 'reactflow';

interface CanvasLayout {
  nodes: Node[];
  edges: Edge[];
  zoom: number;
  panX: number;
  panY: number;
}

interface ScenarioState {
  scenario: Scenario;
  canvasLayout: CanvasLayout;

  // Scenario actions
  setScenario: (scenario: Scenario) => void;
  addStep: (step: ScenarioStep, afterId?: string) => void;
  updateStep: (id: string, patch: Partial<ScenarioStep>) => void;
  deleteStep: (id: string) => void;
  reorderSteps: (stepIds: string[]) => void;

  // Canvas layout actions
  setCanvasLayout: (layout: CanvasLayout) => void;
  updateCanvasNodes: (nodes: Node[]) => void;
  updateCanvasEdges: (edges: Edge[]) => void;
  updateCanvasZoom: (zoom: number) => void;
  updateCanvasPan: (panX: number, panY: number) => void;
}

export const useScenarioStore = create<ScenarioState>()(
  immer((set) => ({
    scenario: {
      name: 'Untitled Scenario',
      steps: [],
      variables: {},
    },
    canvasLayout: {
      nodes: [],
      edges: [],
      zoom: 1,
      panX: 0,
      panY: 0,
    },

    setScenario: (scenario: Scenario) =>
      set((state) => {
        state.scenario = scenario;
      }),

    addStep: (step: ScenarioStep, afterId?: string) =>
      set((state) => {
        if (afterId) {
          const index = state.scenario.steps.findIndex((s) => s.id === afterId);
          if (index >= 0) {
            state.scenario.steps.splice(index + 1, 0, step);
          } else {
            state.scenario.steps.push(step);
          }
        } else {
          state.scenario.steps.push(step);
        }
      }),

    updateStep: (id: string, patch: Partial<ScenarioStep>) =>
      set((state) => {
        const step = state.scenario.steps.find((s) => s.id === id);
        if (step) {
          Object.assign(step, patch);
        }
      }),

    deleteStep: (id: string) =>
      set((state) => {
        state.scenario.steps = state.scenario.steps.filter((s) => s.id !== id);
      }),

    reorderSteps: (stepIds: string[]) =>
      set((state) => {
        const stepMap = new Map(state.scenario.steps.map((s) => [s.id, s]));
        state.scenario.steps = stepIds
          .map((id) => stepMap.get(id))
          .filter((s) => s !== undefined) as ScenarioStep[];
      }),

    setCanvasLayout: (layout: CanvasLayout) =>
      set((state) => {
        state.canvasLayout = layout;
      }),

    updateCanvasNodes: (nodes: Node[]) =>
      set((state) => {
        state.canvasLayout.nodes = nodes;
      }),

    updateCanvasEdges: (edges: Edge[]) =>
      set((state) => {
        state.canvasLayout.edges = edges;
      }),

    updateCanvasZoom: (zoom: number) =>
      set((state) => {
        state.canvasLayout.zoom = zoom;
      }),

    updateCanvasPan: (panX: number, panY: number) =>
      set((state) => {
        state.canvasLayout.panX = panX;
        state.canvasLayout.panY = panY;
      }),
  }))
);
