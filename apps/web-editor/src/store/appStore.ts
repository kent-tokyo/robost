import { create } from 'zustand'

export type Page = 'scenario' | 'settings'

interface AppStore {
  page: Page
  setPage: (p: Page) => void
}

export const useAppStore = create<AppStore>((set) => ({
  page: 'scenario',
  setPage: (page) => set({ page }),
}))
