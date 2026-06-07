import { useCallback, useMemo } from 'react';
import { useScenarioStore } from '../store/scenarioStore';
import { useCanvasStore } from '../store/canvasStore';
import { ScenarioStep } from '../types';

export const useCanvasSearch = () => {
  const { scenario } = useScenarioStore();
  const { setSearchHighlights, setSearchQuery, searchQuery } = useCanvasStore();

  const flattenSteps = useCallback((steps: ScenarioStep[], parentPath: string[] = []): Array<{ step: ScenarioStep; path: string[] }> => {
    const result: Array<{ step: ScenarioStep; path: string[] }> = [];
    steps.forEach((step) => {
      const currentPath = [...parentPath, step.id];
      result.push({ step, path: currentPath });
      if (step.childSteps) {
        result.push(...flattenSteps(step.childSteps, currentPath));
      }
    });
    return result;
  }, []);

  const search = useCallback(
    (query: string) => {
      if (!query.trim()) {
        setSearchHighlights(new Set());
        setSearchQuery('');
        return [];
      }

      const lowerQuery = query.toLowerCase();
      const flatSteps = flattenSteps(scenario.steps);

      const matches = flatSteps.filter(({ step }) => {
        const matchesName = step.name.toLowerCase().includes(lowerQuery);
        const matchesType = step.type.toLowerCase().includes(lowerQuery);
        const matchesData = Object.values(step.data).some((value) => {
          if (typeof value === 'string') return value.toLowerCase().includes(lowerQuery);
          if (typeof value === 'object') return JSON.stringify(value).toLowerCase().includes(lowerQuery);
          return false;
        });
        const matchesComment = step.comment?.toLowerCase().includes(lowerQuery);

        return matchesName || matchesType || matchesData || matchesComment;
      });

      setSearchQuery(query);
      setSearchHighlights(new Set(matches.map(({ step }) => step.id)));
      return matches.map(({ step }) => step.id);
    },
    [scenario.steps, flattenSteps, setSearchHighlights, setSearchQuery]
  );

  const filterByType = useCallback(
    (stepType: string | null) => {
      if (!stepType) return scenario.steps;

      const filter = (steps: ScenarioStep[]): ScenarioStep[] => {
        return steps
          .filter((step) => step.type === stepType)
          .map((step) => ({
            ...step,
            childSteps: step.childSteps ? filter(step.childSteps) : undefined,
          }));
      };

      return filter(scenario.steps);
    },
    [scenario.steps]
  );

  return {
    search,
    filterByType,
    flattenSteps,
  };
};
