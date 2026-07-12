import { create } from 'zustand'
import * as yaml from 'js-yaml'
import { api, type FolderEntry } from '../api/client'
import { addStepToYaml, deleteStepFromYaml, setDataSourceInYaml } from '../utils/yamlFlow'

function uniqueScenarioName(base: string, ext: string, existing: string[], sep = '_'): string {
  let name = `${base}${ext}`
  let n = 2
  while (existing.includes(name)) {
    name = `${base}${sep}${n}${ext}`
    n++
  }
  return name
}

function allScenarioNames(scenarios: string[], folders: FolderEntry[]): string[] {
  return [...scenarios, ...folders.flatMap((f) => f.scenarios.map((s) => `${f.name}/${s}`))]
}

interface ScenarioStore {
  scenarios: string[]
  folders: FolderEntry[]
  activeScenario: string | null
  yaml: string
  dirty: boolean
  loading: boolean

  loadList: () => Promise<void>
  openScenario: (name: string) => Promise<void>
  setYaml: (yaml: string) => void
  save: () => Promise<void>
  newScenario: (name: string, folder?: string) => Promise<void>
  createScenarioFromYaml: (yamlContent: string) => Promise<void>
  duplicateScenario: (name: string) => Promise<void>
  deleteScenario: (name: string) => Promise<void>
  addStep: (stepType: string, defaults?: Record<string, unknown>, atIndex?: number) => void
  deleteStep: (index: number) => void
  setDataSource: (file: string, sheet?: string) => void
  moveScenario: (from: string, to: string) => Promise<void>
  createFolder: (name: string) => Promise<void>
  deleteFolder: (name: string) => Promise<void>
}

export const useScenarioStore = create<ScenarioStore>((set, get) => ({
  scenarios: [],
  folders: [],
  activeScenario: null,
  yaml: '',
  dirty: false,
  loading: false,

  loadList: async () => {
    const result = await api.listScenarios()
    set({ scenarios: result.scenarios, folders: result.folders })
  },

  openScenario: async (name) => {
    set({ loading: true })
    try {
      const yaml = await api.getScenario(name)
      set({ activeScenario: name, yaml, dirty: false })
    } finally {
      set({ loading: false })
    }
  },

  setYaml: (yaml) => set({ yaml, dirty: true }),

  save: async () => {
    const { activeScenario, yaml } = get()
    if (!activeScenario) return
    await api.saveScenario(activeScenario, yaml)
    set({ dirty: false })
  },

  newScenario: async (name, folder) => {
    const fullName = folder ? `${folder}/${name}` : name
    const displayName = name.replace(/\.ya?ml$/i, '')
    const initialYaml = `name: ${displayName}\nsteps: []\n`
    await api.saveScenario(fullName, initialYaml)
    await get().loadList()
    set({ activeScenario: fullName, yaml: initialYaml, dirty: false })
  },

  createScenarioFromYaml: async (yamlContent) => {
    let parsedName: string | undefined
    try {
      const parsed = yaml.load(yamlContent) as { name?: unknown } | undefined
      if (typeof parsed?.name === 'string') parsedName = parsed.name
    } catch {
      // malformed YAML from the AI — fall back to a generic name
    }
    const base = (parsedName || 'ai-scenario').replace(/[/\\]/g, '_')
    const { scenarios, folders } = get()
    const name = uniqueScenarioName(base, '.yaml', allScenarioNames(scenarios, folders))

    await api.saveScenario(name, yamlContent)
    await get().loadList()
    set({ activeScenario: name, yaml: yamlContent, dirty: false })
  },

  duplicateScenario: async (name) => {
    const content = await api.getScenario(name)
    const slash = name.lastIndexOf('/')
    const folder = slash >= 0 ? name.slice(0, slash + 1) : ''
    const base = name.slice(slash + 1).replace(/\.ya?ml$/i, '')
    const ext = name.match(/\.ya?ml$/i)?.[0] ?? '.yaml'
    const { scenarios, folders } = get()

    // sep='' preserves the original on-disk convention: foo_copy2.yaml, not foo_copy_2.yaml
    const copyName = uniqueScenarioName(`${folder}${base}_copy`, ext, allScenarioNames(scenarios, folders), '')

    await api.saveScenario(copyName, content)
    await get().loadList()
  },

  deleteScenario: async (name) => {
    await api.deleteScenario(name)
    const { activeScenario } = get()
    await get().loadList()
    if (activeScenario === name) {
      set({ activeScenario: null, yaml: '', dirty: false })
    }
  },

  addStep: (stepType, defaults = {}, atIndex) => {
    const { yaml } = get()
    const newYaml = addStepToYaml(yaml, stepType, defaults, atIndex)
    set({ yaml: newYaml, dirty: true })
  },

  deleteStep: (index) => {
    const { yaml } = get()
    const newYaml = deleteStepFromYaml(yaml, index)
    set({ yaml: newYaml, dirty: true })
  },

  moveScenario: async (from, to) => {
    await api.moveScenario(from, to)
    const { activeScenario } = get()
    await get().loadList()
    if (activeScenario === from) {
      set({ activeScenario: to })
    }
  },

  setDataSource: (file, sheet) => {
    const { yaml } = get()
    const newYaml = setDataSourceInYaml(yaml, file, sheet)
    set({ yaml: newYaml, dirty: true })
  },

  createFolder: async (name) => {
    await api.createFolder(name)
    await get().loadList()
  },

  deleteFolder: async (name) => {
    await api.deleteFolder(name)
    await get().loadList()
  },
}))
