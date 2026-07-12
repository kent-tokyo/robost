import { create } from 'zustand'
import { persist } from 'zustand/middleware'

export type Page = 'scenario' | 'settings'
export type Theme = 'purple' | 'blue' | 'green'

interface AppStore {
  page: Page
  setPage: (p: Page) => void
  appTitle: string
  setAppTitle: (title: string) => void
  theme: Theme
  setTheme: (theme: Theme) => void
}

export const useAppStore = create<AppStore>()(
  persist(
    (set) => ({
      page: 'scenario',
      setPage: (page) => set({ page }),
      appTitle: 'robost',
      setAppTitle: (appTitle) => set({ appTitle: appTitle.trim() || 'robost' }),
      theme: 'purple',
      setTheme: (theme) => set({ theme }),
    }),
    {
      name: 'robost-app-settings',
      partialize: (s) => ({ appTitle: s.appTitle, theme: s.theme }),
    },
  ),
)
