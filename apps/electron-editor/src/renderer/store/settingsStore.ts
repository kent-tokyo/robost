import { create } from 'zustand';
import { immer } from 'zustand/middleware/immer';
import { persist } from 'zustand/middleware';

type Theme = 'dark' | 'light';
type Locale = 'en' | 'ja' | 'zh';

export interface RecentFile {
  path: string;
  name: string;
  timestamp: number;
}

interface SettingsState {
  theme: Theme;
  locale: Locale;
  apiKeyOpenAI: string;
  apiKeyAnthropic: string;
  autoSave: boolean;
  autoSaveInterval: number;
  recentFiles: RecentFile[];

  // Actions
  setTheme: (theme: Theme) => void;
  setLocale: (locale: Locale) => void;
  setApiKeyOpenAI: (key: string) => void;
  setApiKeyAnthropic: (key: string) => void;
  setAutoSave: (enabled: boolean) => void;
  setAutoSaveInterval: (ms: number) => void;
  addRecentFile: (filePath: string) => void;
  removeRecentFile: (filePath: string) => void;
  clearRecentFiles: () => void;
  // Note: aiHistory is stored in localStorage directly by AiPanel component
}

export const useSettingsStore = create<SettingsState>()(
  persist(
    immer((set) => ({
      theme: 'dark',
      locale: 'en',
      apiKeyOpenAI: '',
      apiKeyAnthropic: '',
      autoSave: true,
      autoSaveInterval: 5000,
      recentFiles: [],

      setTheme: (theme: Theme) =>
        set((state) => {
          state.theme = theme;
        }),

      setLocale: (locale: Locale) =>
        set((state) => {
          state.locale = locale;
        }),

      setApiKeyOpenAI: (key: string) =>
        set((state) => {
          state.apiKeyOpenAI = key;
        }),

      setApiKeyAnthropic: (key: string) =>
        set((state) => {
          state.apiKeyAnthropic = key;
        }),

      setAutoSave: (enabled: boolean) =>
        set((state) => {
          state.autoSave = enabled;
        }),

      setAutoSaveInterval: (ms: number) =>
        set((state) => {
          state.autoSaveInterval = ms;
        }),

      addRecentFile: (filePath: string) =>
        set((state) => {
          // Extract filename from path
          const name = filePath.split(/[\\/]/).pop() || 'Untitled';

          // Remove duplicate if exists
          state.recentFiles = state.recentFiles.filter((f) => f.path !== filePath);

          // Add to front
          state.recentFiles.unshift({
            path: filePath,
            name,
            timestamp: Date.now(),
          });

          // Keep only last 5
          state.recentFiles = state.recentFiles.slice(0, 5);
        }),

      removeRecentFile: (filePath: string) =>
        set((state) => {
          state.recentFiles = state.recentFiles.filter((f) => f.path !== filePath);
        }),

      clearRecentFiles: () =>
        set((state) => {
          state.recentFiles = [];
        }),
    })),
    {
      name: 'robost-settings',
    }
  )
);
