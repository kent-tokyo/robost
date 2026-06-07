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
  deleteStepWithCascade: (id: string) => void;
  reorderSteps: (stepIds: string[]) => void;

  // Canvas layout actions
  setCanvasLayout: (layout: CanvasLayout) => void;
  updateCanvasNodes: (nodes: Node[]) => void;
  updateCanvasEdges: (edges: Edge[]) => void;
  updateCanvasZoom: (zoom: number) => void;
  updateCanvasPan: (panX: number, panY: number) => void;

  // Grouping actions
  groupSteps: (stepIds: string[], groupName: string) => string;
  ungroupSteps: (groupId: string) => void;
  duplicateStep: (stepId: string) => string;
  pasteStep: (stepData: ScenarioStep, afterId?: string) => string;
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

    deleteStepWithCascade: (id: string) =>
      set((state) => {
        const deleteRecursive = (steps: ScenarioStep[], targetId: string) => {
          return steps.filter((s) => {
            if (s.id === targetId) {
              return false;
            }
            if (s.childSteps) {
              s.childSteps = deleteRecursive(s.childSteps, targetId);
            }
            return true;
          });
        };
        state.scenario.steps = deleteRecursive(state.scenario.steps, id);
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

    groupSteps: (stepIds: string[], groupName: string) => {
      let groupId = '';
      set((state) => {
        const stepsToGroup: ScenarioStep[] = [];
        const remainingSteps: ScenarioStep[] = [];

        state.scenario.steps.forEach((step) => {
          if (stepIds.includes(step.id)) {
            stepsToGroup.push(step);
          } else {
            remainingSteps.push(step);
          }
        });

        if (stepsToGroup.length === 0) return;

        groupId = `group-${Date.now()}`;
        const groupStep: ScenarioStep = {
          id: groupId,
          name: groupName,
          type: 'group',
          data: { group_name: groupName },
          childSteps: stepsToGroup,
        };

        // Insert group at position of first grouped step
        const firstIndex = state.scenario.steps.findIndex((s) => s.id === stepIds[0]);
        state.scenario.steps = [
          ...remainingSteps.slice(0, firstIndex),
          groupStep,
          ...remainingSteps.slice(firstIndex),
        ];
      });
      return groupId;
    },

    ungroupSteps: (groupId: string) =>
      set((state) => {
        const ungroupRecursive = (steps: ScenarioStep[]): ScenarioStep[] => {
          const result: ScenarioStep[] = [];
          steps.forEach((step) => {
            if (step.id === groupId && step.childSteps) {
              result.push(...step.childSteps);
            } else {
              if (step.childSteps) {
                step.childSteps = ungroupRecursive(step.childSteps);
              }
              result.push(step);
            }
          });
          return result;
        };
        state.scenario.steps = ungroupRecursive(state.scenario.steps);
      }),

    duplicateStep: (stepId: string) => {
      let newStepId = '';
      set((state) => {
        const findAndDuplicate = (steps: ScenarioStep[]): ScenarioStep[] => {
          return steps.flatMap((step) => {
            if (step.id === stepId) {
              const duplicated = JSON.parse(JSON.stringify(step));
              newStepId = `${stepId}-${Date.now()}`;
              duplicated.id = newStepId;
              if (duplicated.childSteps) {
                duplicated.childSteps = duplicated.childSteps.map((child: any) => ({
                  ...child,
                  id: `${child.id}-${Date.now()}`,
                }));
              }
              return [step, duplicated];
            }
            if (step.childSteps) {
              step.childSteps = findAndDuplicate(step.childSteps);
            }
            return [step];
          });
        };
        state.scenario.steps = findAndDuplicate(state.scenario.steps);
      });
      return newStepId;
    },

    pasteStep: (stepData: ScenarioStep, afterId?: string) => {
      let newStepId = '';
      set((state) => {
        const pasted = JSON.parse(JSON.stringify(stepData));
        newStepId = `${pasted.id}-${Date.now()}`;
        pasted.id = newStepId;

        if (pasted.childSteps) {
          pasted.childSteps = pasted.childSteps.map((child: any) => ({
            ...child,
            id: `${child.id}-${Date.now()}`,
          }));
        }

        if (afterId) {
          const index = state.scenario.steps.findIndex((s) => s.id === afterId);
          if (index >= 0) {
            state.scenario.steps.splice(index + 1, 0, pasted);
          } else {
            state.scenario.steps.push(pasted);
          }
        } else {
          state.scenario.steps.push(pasted);
        }
      });
      return newStepId;
    },
  }))
);
