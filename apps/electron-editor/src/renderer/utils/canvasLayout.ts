import { Node } from 'reactflow';
import { ScenarioStep } from '../types';

interface LayoutNode {
  id: string;
  level: number;
  index: number;
  childCount: number;
}

/**
 * Auto-layout algorithm using hierarchical tree layout
 * Positions nodes vertically by depth, horizontally by sibling position
 */
export const autoLayoutNodes = (steps: ScenarioStep[]): Record<string, { x: number; y: number }> => {
  const positions: Record<string, { x: number; y: number }> = {};
  const LEVEL_HEIGHT = 150;
  const NODE_WIDTH = 200;
  const H_SPACING = 250;

  let levelNodeCounts: Record<number, number> = {};

  const traverse = (
    currentSteps: ScenarioStep[],
    level: number = 0,
    parentX: number = 0,
    startIndex: number = 0
  ): { maxX: number; nextIndex: number } => {
    let maxX = parentX;
    let nodeIndex = startIndex;

    const levelCount = levelNodeCounts[level] || 0;
    let horizontalOffset = parentX - (currentSteps.length * H_SPACING) / 2;

    currentSteps.forEach((step, i) => {
      const x = horizontalOffset + i * H_SPACING;
      const y = level * LEVEL_HEIGHT;

      positions[step.id] = { x, y };
      maxX = Math.max(maxX, x + NODE_WIDTH);

      levelNodeCounts[level] = (levelNodeCounts[level] || 0) + 1;
      nodeIndex++;

      // Recursively layout child steps if it's a group
      if (step.childSteps && step.childSteps.length > 0) {
        const childResult = traverse(step.childSteps, level + 1, x + NODE_WIDTH / 2, nodeIndex);
        maxX = Math.max(maxX, childResult.maxX);
        nodeIndex = childResult.nextIndex;
      }
    });

    return { maxX, nextIndex: nodeIndex };
  };

  traverse(steps);
  return positions;
};

/**
 * Get breadcrumb trail for nested groups
 */
export const getBreadcrumbTrail = (
  steps: ScenarioStep[],
  targetId: string,
  path: ScenarioStep[] = []
): ScenarioStep[] | null => {
  for (const step of steps) {
    if (step.id === targetId) {
      return [...path, step];
    }

    if (step.childSteps) {
      const result = getBreadcrumbTrail(step.childSteps, targetId, [...path, step]);
      if (result) {
        return result;
      }
    }
  }

  return null;
};

/**
 * Find step in hierarchy by ID
 */
export const findStep = (steps: ScenarioStep[], id: string): ScenarioStep | null => {
  for (const step of steps) {
    if (step.id === id) {
      return step;
    }

    if (step.childSteps) {
      const found = findStep(step.childSteps, id);
      if (found) {
        return found;
      }
    }
  }

  return null;
};

/**
 * Zoom to fit selected nodes
 */
export const getZoomBounds = (nodeIds: string[], positions: Record<string, { x: number; y: number }>) => {
  if (nodeIds.length === 0) {
    return { x: 0, y: 0, zoom: 1 };
  }

  let minX = Infinity;
  let maxX = -Infinity;
  let minY = Infinity;
  let maxY = -Infinity;

  nodeIds.forEach((id) => {
    const pos = positions[id];
    if (pos) {
      minX = Math.min(minX, pos.x);
      maxX = Math.max(maxX, pos.x + 200);
      minY = Math.min(minY, pos.y);
      maxY = Math.max(maxY, pos.y + 100);
    }
  });

  const width = maxX - minX;
  const height = maxY - minY;
  const padding = 50;

  return {
    x: minX - padding,
    y: minY - padding,
    zoom: Math.min(1, Math.min(800 / (width + padding * 2), 600 / (height + padding * 2))),
  };
};

/**
 * Get condition expression preview
 */
export const getConditionPreview = (step: ScenarioStep): string | null => {
  if (step.type === 'if' || step.type === 'while') {
    return step.data.cond ? `${step.data.cond.substring(0, 30)}${step.data.cond.length > 30 ? '...' : ''}` : null;
  }
  if (step.type === 'foreach') {
    return `${step.data.var} → ${step.data.item_name || 'item'}`;
  }
  return null;
};
