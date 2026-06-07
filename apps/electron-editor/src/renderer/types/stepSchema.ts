export type FieldType = 'text' | 'number' | 'boolean' | 'select' | 'array';

export interface Field {
  name: string;
  type: FieldType;
  label: string;
  required?: boolean;
  defaultValue?: any;
  options?: { label: string; value: string }[];
  placeholder?: string;
  min?: number;
  max?: number;
}

export interface StepSchema {
  name: string;
  fields: Field[];
}

export const STEP_SCHEMAS: Record<string, StepSchema> = {
  click_image: {
    name: 'Click Image',
    fields: [
      { name: 'template', type: 'text', label: 'Template Path', required: true },
      { name: 'timeout_ms', type: 'number', label: 'Timeout (ms)', defaultValue: 10000 },
      { name: 'similarity_threshold', type: 'number', label: 'Similarity %', min: 0, max: 100, defaultValue: 87 },
      { name: 'click_button', type: 'select', label: 'Button', options: [
        { label: 'Left', value: 'left' },
        { label: 'Right', value: 'right' },
        { label: 'Middle', value: 'middle' },
      ]},
      { name: 'double_click', type: 'boolean', label: 'Double Click' },
    ],
  },

  wait_image: {
    name: 'Wait Image',
    fields: [
      { name: 'template', type: 'text', label: 'Template Path', required: true },
      { name: 'timeout_ms', type: 'number', label: 'Timeout (ms)', defaultValue: 10000 },
      { name: 'similarity_threshold', type: 'number', label: 'Similarity %', min: 0, max: 100, defaultValue: 87 },
    ],
  },

  type: {
    name: 'Type Text',
    fields: [
      { name: 'text', type: 'text', label: 'Text to Type', required: true },
      { name: 'interval_ms', type: 'number', label: 'Key Interval (ms)', defaultValue: 50 },
    ],
  },

  press: {
    name: 'Press Key',
    fields: [
      { name: 'key', type: 'text', label: 'Key Name', required: true, placeholder: 'e.g. Return, Escape, Tab' },
      { name: 'repeat', type: 'number', label: 'Repeat Count', defaultValue: 1 },
    ],
  },

  script: {
    name: 'Run Script',
    fields: [
      { name: 'script', type: 'text', label: 'Script (Rhai)', required: true },
      { name: 'save_as', type: 'text', label: 'Save As' },
    ],
  },

  if: {
    name: 'If Statement',
    fields: [
      { name: 'cond', type: 'text', label: 'Condition', required: true },
    ],
  },

  foreach: {
    name: 'For Each Loop',
    fields: [
      { name: 'var', type: 'text', label: 'List Variable', required: true },
      { name: 'item_name', type: 'text', label: 'Item Variable Name', defaultValue: 'item' },
    ],
  },

  while: {
    name: 'While Loop',
    fields: [
      { name: 'cond', type: 'text', label: 'Condition', required: true },
    ],
  },

  repeat: {
    name: 'Repeat N Times',
    fields: [
      { name: 'count', type: 'number', label: 'Count', required: true, min: 1 },
    ],
  },

  set: {
    name: 'Set Variable',
    fields: [
      { name: 'name', type: 'text', label: 'Variable Name', required: true },
      { name: 'value', type: 'text', label: 'Value', required: true },
    ],
  },

  calc: {
    name: 'Calculate',
    fields: [
      { name: 'expr', type: 'text', label: 'Expression', required: true },
      { name: 'save_as', type: 'text', label: 'Save As', required: true },
    ],
  },

  log: {
    name: 'Log Message',
    fields: [
      { name: 'message', type: 'text', label: 'Message', required: true },
      { name: 'level', type: 'select', label: 'Level', options: [
        { label: 'Info', value: 'info' },
        { label: 'Warn', value: 'warn' },
        { label: 'Error', value: 'error' },
      ]},
    ],
  },

  shell: {
    name: 'Run Shell',
    fields: [
      { name: 'cmd', type: 'text', label: 'Command', required: true },
      { name: 'args', type: 'array', label: 'Arguments' },
      { name: 'save_as', type: 'text', label: 'Save Output As' },
    ],
  },

  call_scenario: {
    name: 'Call Scenario',
    fields: [
      { name: 'scenario', type: 'text', label: 'Scenario File', required: true },
    ],
  },

  try_catch: {
    name: 'Try-Catch',
    fields: [
      { name: 'catch_message', type: 'text', label: 'Catch Variable' },
    ],
  },

  group: {
    name: 'Group',
    fields: [
      { name: 'group_name', type: 'text', label: 'Group Name' },
    ],
  },
};

export function getStepSchema(stepType: string): StepSchema | undefined {
  return STEP_SCHEMAS[stepType];
}
