import { create } from 'zustand'
import { api } from '../api/client'

export type RunStatus = 'idle' | 'running' | 'success' | 'failed'

interface LogEntry {
  level: 'info' | 'error' | 'warn'
  message: string
  time: number
}

interface RunStore {
  status: RunStatus
  currentStep: number
  logs: LogEntry[]
  disconnect: (() => void) | null

  startRun: (scenarioName: string) => Promise<void>
  startRunStep: (scenarioName: string, stepIndex: number) => Promise<void>
  stopRun: () => void
  addLog: (level: LogEntry['level'], message: string) => void
  connectEvents: () => void
}

export const useRunStore = create<RunStore>((set, get) => ({
  status: 'idle',
  currentStep: -1,
  logs: [],
  disconnect: null,

  // Fix: await api.run() so 409 / errors surface instead of being silently dropped.
  startRun: async (scenarioName) => {
    get().connectEvents()
    try {
      await api.run(scenarioName)
      set({ status: 'running', currentStep: 0, logs: [] })
    } catch (err) {
      get().disconnect?.()
      set({ disconnect: null })
      get().addLog('error', `Failed to start run: ${err}`)
    }
  },

  startRunStep: async (scenarioName, stepIndex) => {
    get().connectEvents()
    try {
      await api.runStep(scenarioName, stepIndex)
      set({ status: 'running', currentStep: stepIndex, logs: [] })
    } catch (err) {
      get().disconnect?.()
      set({ disconnect: null })
      get().addLog('error', `ステップ実行に失敗: ${err}`)
    }
  },

  stopRun: () => {
    api.stop()
    get().disconnect?.()
    set({ status: 'idle', disconnect: null })
  },

  addLog: (level, message) => {
    set((s) => ({
      logs: [...s.logs.slice(-200), { level, message, time: Date.now() }],
    }))
  },

  connectEvents: () => {
    get().disconnect?.()
    const off = api.connectEvents((raw: unknown) => {
      const e = raw as Record<string, unknown>
      const { addLog } = get()
      switch (e.type) {
        case 'scenario_start':
          set({ status: 'running', currentStep: 0 })
          break
        case 'step_start':
          set({ currentStep: (e.index as number) ?? 0 })
          addLog('info', `▶ ${e.name}`)
          break
        case 'step_done':
          addLog('info', `✓ ${e.name} (${e.elapsed_ms}ms)`)
          break
        case 'log':
          addLog((e.level as LogEntry['level']) || 'info', e.message as string)
          break
        case 'finished':
          set({ status: e.success ? 'success' : 'failed' })
          if (!e.success && e.error) addLog('error', e.error as string)
          break
      }
    })
    set({ disconnect: off })
  },
}))
