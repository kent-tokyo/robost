import yaml from 'js-yaml'
import type { Node, Edge } from '@xyflow/react'

export interface ScenarioStep {
  name?: string
  type?: string
  [key: string]: unknown
}

interface Scenario {
  name?: string
  steps?: unknown[]
  [key: string]: unknown
}

// Rust YAML uses tagged-union keys: { window_control: { ... } }
// This list maps those keys to their display type string.
const KNOWN_STEP_TYPES = [
  'window_control', 'click_text', 'move_to_text', 'wait_ms', 'type', 'press',
  'click_image', 'wait_image', 'find_image', 'script', 'shell',
  'ocr_match', 'ml_detect', 'wait_window', 'group', 'if',
  'foreach', 'while', 'repeat', 'set', 'calc', 'log',
  'call_scenario', 'try_catch', 'key_combo', 'mouse_move',
  'mouse_click_xy', 'mouse_scroll', 'mouse_drag', 'sub_scenario',
  'excel_read_sheet', 'excel_read_range', 'import_vars', 'csv_read',
] as const

type KnownStepType = typeof KNOWN_STEP_TYPES[number]

/** Convert a raw Rust-format step object into a flat ScenarioStep. */
export function normalizeStep(raw: Record<string, unknown>): ScenarioStep {
  for (const key of KNOWN_STEP_TYPES) {
    if (key in raw) {
      const val = raw[key]
      const data =
        typeof val === 'object' && val !== null && !Array.isArray(val)
          ? (val as Record<string, unknown>)
          : val !== null && val !== undefined
          ? { value: val }
          : {}
      return {
        type: key,
        name: raw.name as string | undefined,
        enabled: (raw.enabled as boolean) ?? true,
        ...data,
      }
    }
  }
  // Unknown step type — pass through as-is
  return raw as ScenarioStep
}

/** Convert a normalized ScenarioStep back to Rust-format YAML object. */
export function denormalizeStep(step: ScenarioStep): Record<string, unknown> {
  const { type, name, enabled, ...data } = step as {
    type?: string
    name?: string
    enabled?: boolean
    [key: string]: unknown
  }
  if (type && (KNOWN_STEP_TYPES as readonly string[]).includes(type)) {
    const stepData = Object.keys(data).length > 0 ? data : null
    const result: Record<string, unknown> = { [type]: stepData }
    if (name) result.name = name
    if (enabled === false) result.enabled = false
    return result
  }
  return step as Record<string, unknown>
}

export function yamlToNodes(yamlText: string): { nodes: Node[]; edges: Edge[] } {
  let scenario: Scenario
  try {
    scenario = (yaml.load(yamlText) as Scenario) ?? {}
  } catch {
    return { nodes: [], edges: [] }
  }

  const rawSteps = scenario.steps ?? []
  const steps: ScenarioStep[] = rawSteps.map((s) =>
    normalizeStep(s as Record<string, unknown>),
  )

  const nodes: Node[] = steps.map((step, i) => ({
    id: `step-${i}`,
    position: { x: 240, y: i * 110 },
    data: {
      label: step.name ?? step.type ?? `Step ${i + 1}`,
      type: step.type,
      step,
      stepIndex: i,
    },
    type: 'default',
  }))

  const edges: Edge[] = steps.slice(0, -1).map((_, i) => ({
    id: `e${i}-${i + 1}`,
    source: `step-${i}`,
    target: `step-${i + 1}`,
    type: 'smoothstep',
  }))

  return { nodes, edges }
}

export function nodesToYaml(nodes: Node[], _edges: Edge[], originalYaml: string): string {
  let scenario: Scenario
  try {
    scenario = (yaml.load(originalYaml) as Scenario) ?? {}
  } catch {
    return originalYaml
  }

  const sorted = [...nodes].sort((a, b) => a.position.y - b.position.y)
  const rawSteps = (scenario.steps ?? []) as Record<string, unknown>[]
  const stepMap = new Map(rawSteps.map((s, i) => [`step-${i}`, s]))

  const reorderedSteps = sorted
    .map((n) => stepMap.get(n.id))
    .filter((s): s is Record<string, unknown> => s !== undefined)

  if (reorderedSteps.length > 0) {
    scenario.steps = reorderedSteps
  }

  return yaml.dump(scenario, { lineWidth: -1, noRefs: true })
}

/** Update one step in the YAML by index. Returns the new YAML string. */
export function updateStepInYaml(
  yamlText: string,
  stepIndex: number,
  patch: Partial<ScenarioStep>,
): string {
  let scenario: Scenario
  try {
    scenario = (yaml.load(yamlText) as Scenario) ?? {}
  } catch {
    return yamlText
  }

  const rawSteps = (scenario.steps ?? []) as Record<string, unknown>[]
  if (stepIndex < 0 || stepIndex >= rawSteps.length) return yamlText

  // Normalize → patch → denormalize
  const normalized = normalizeStep(rawSteps[stepIndex])
  const patched: ScenarioStep = { ...normalized, ...patch }
  rawSteps[stepIndex] = denormalizeStep(patched)
  scenario.steps = rawSteps

  return yaml.dump(scenario, { lineWidth: -1, noRefs: true })
}

/** Remove the step at `stepIndex` from the scenario YAML. */
export function deleteStepFromYaml(yamlText: string, stepIndex: number): string {
  let scenario: Scenario
  try {
    scenario = (yaml.load(yamlText) as Scenario) ?? {}
  } catch {
    return yamlText
  }

  const rawSteps = (scenario.steps ?? []) as Record<string, unknown>[]
  if (stepIndex < 0 || stepIndex >= rawSteps.length) return yamlText

  rawSteps.splice(stepIndex, 1)
  scenario.steps = rawSteps

  return yaml.dump(scenario, { lineWidth: -1, noRefs: true })
}

/** Append a new step of the given type to the scenario YAML. */
export function addStepToYaml(
  yamlText: string,
  stepType: KnownStepType | string,
  defaults: Record<string, unknown> = {},
): string {
  let scenario: Scenario
  try {
    scenario = (yaml.load(yamlText) as Scenario) ?? {}
  } catch {
    scenario = {}
  }

  if (!scenario.steps) scenario.steps = []
  const stepData = Object.keys(defaults).length > 0 ? defaults : null
  ;(scenario.steps as unknown[]).push({ [stepType]: stepData })

  return yaml.dump(scenario, { lineWidth: -1, noRefs: true })
}

/** Set (or clear) the scenario-level data_source for Excel/CSV row iteration. */
export function setDataSourceInYaml(
  yamlText: string,
  file: string,
  sheet?: string,
): string {
  let scenario: Scenario
  try {
    scenario = (yaml.load(yamlText) as Scenario) ?? {}
  } catch {
    scenario = {}
  }

  if (file.trim()) {
    scenario.data_source = sheet?.trim()
      ? { file: file.trim(), sheet: sheet.trim() }
      : { file: file.trim() }
  } else {
    delete scenario.data_source
  }

  return yaml.dump(scenario, { lineWidth: -1, noRefs: true })
}

/** Read the data_source from the YAML, returning file/sheet if present. */
export function getDataSourceFromYaml(
  yamlText: string,
): { file: string; sheet: string } {
  try {
    const scenario = (yaml.load(yamlText) as Scenario) ?? {}
    const ds = scenario.data_source as { file?: string; sheet?: string } | undefined
    return { file: ds?.file ?? '', sheet: ds?.sheet ?? '' }
  } catch {
    return { file: '', sheet: '' }
  }
}
