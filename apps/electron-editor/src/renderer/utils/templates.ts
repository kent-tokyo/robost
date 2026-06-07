import { ScenarioStep, StepType } from '../types';

export interface TemplateDefinition {
  id: string;
  name: string;
  description: string;
  iconKey: string;
  category: 'action' | 'control' | 'utility';
  generateSteps: () => ScenarioStep[];
}

const generateId = () => `step-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;

export const templates: TemplateDefinition[] = [
  // Actions
  {
    id: 'template-click-image',
    name: 'Click Image',
    description: 'Click on an image target on screen',
    iconKey: 'click_image',
    category: 'action',
    generateSteps: () => [
      {
        id: generateId(),
        name: 'Click Image',
        type: 'click_image',
        data: {
          imagePath: '',
          threshold: 0.8,
          offset: { x: 0, y: 0 },
        },
        enabled: true,
      },
    ],
  },
  {
    id: 'template-wait-image',
    name: 'Wait Image',
    description: 'Wait for an image to appear on screen',
    iconKey: 'wait_image',
    category: 'action',
    generateSteps: () => [
      {
        id: generateId(),
        name: 'Wait Image',
        type: 'wait_image',
        data: {
          imagePath: '',
          timeout: 10000,
          threshold: 0.8,
        },
        enabled: true,
      },
    ],
  },
  {
    id: 'template-type-text',
    name: 'Type Text',
    description: 'Type text into the active input field',
    iconKey: 'type',
    category: 'action',
    generateSteps: () => [
      {
        id: generateId(),
        name: 'Type Text',
        type: 'type',
        data: {
          text: '',
          delay: 50,
        },
        enabled: true,
      },
    ],
  },

  // Control Flow
  {
    id: 'template-if',
    name: 'If Statement',
    description: 'Conditional branch - execute steps if condition is true',
    iconKey: 'if',
    category: 'control',
    generateSteps: () => [
      {
        id: generateId(),
        name: 'If Statement',
        type: 'if',
        data: {
          condition: '',
          steps: [],
        },
        enabled: true,
      },
    ],
  },
  {
    id: 'template-foreach',
    name: 'For Each Loop',
    description: 'Loop through items in a collection',
    iconKey: 'foreach',
    category: 'control',
    generateSteps: () => [
      {
        id: generateId(),
        name: 'For Each Loop',
        type: 'foreach',
        data: {
          items: '',
          itemVar: 'item',
          steps: [],
        },
        enabled: true,
      },
    ],
  },
  {
    id: 'template-while',
    name: 'While Loop',
    description: 'Loop while condition is true',
    iconKey: 'while',
    category: 'control',
    generateSteps: () => [
      {
        id: generateId(),
        name: 'While Loop',
        type: 'while',
        data: {
          condition: '',
          maxIterations: 100,
          steps: [],
        },
        enabled: true,
      },
    ],
  },

  // Error Handling
  {
    id: 'template-try-catch',
    name: 'Try-Catch',
    description: 'Handle errors gracefully with try-catch block',
    iconKey: 'try_catch',
    category: 'utility',
    generateSteps: () => [
      {
        id: generateId(),
        name: 'Try Block',
        type: 'try_catch',
        data: {
          trySteps: [],
          catchSteps: [],
          errorVariable: 'error',
        },
        enabled: true,
      },
    ],
  },

  // Organization
  {
    id: 'template-group',
    name: 'Group',
    description: 'Group related steps together for organization',
    iconKey: 'group',
    category: 'utility',
    generateSteps: () => [
      {
        id: generateId(),
        name: 'Group',
        type: 'group',
        data: {
          description: 'Grouped steps',
          steps: [],
          collapsed: false,
        },
        enabled: true,
      },
    ],
  },
];

export const getTemplateById = (id: string): TemplateDefinition | undefined => {
  return templates.find((t) => t.id === id);
};

export const getTemplatesByCategory = (category: string): TemplateDefinition[] => {
  return templates.filter((t) => t.category === category);
};
