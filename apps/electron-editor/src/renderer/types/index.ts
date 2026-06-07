export type StepType =
  | 'click_image'
  | 'wait_image'
  | 'type'
  | 'press'
  | 'script'
  | 'if'
  | 'foreach'
  | 'while'
  | 'repeat'
  | 'set'
  | 'calc'
  | 'log'
  | 'shell'
  | 'library'
  | 'call_scenario'
  | 'try_catch'
  | 'group';

export interface ScenarioStep {
  id: string;
  name: string;
  type: StepType;
  data: Record<string, any>;
  enabled?: boolean;
  comment?: string;
  childSteps?: ScenarioStep[];
  parentGroupId?: string;
}

export interface Scenario {
  name: string;
  steps: ScenarioStep[];
  variables?: Record<string, any>;
}

export type ViewMode = 'canvas' | 'list' | 'flow';
