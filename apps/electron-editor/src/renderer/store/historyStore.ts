import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { ExecutionRecord } from './runStore';

const STORAGE_KEY = 'robost-execution-history';
const MAX_STORED_RECORDS = 50;

interface HistoryState {
  records: ExecutionRecord[];

  // Actions
  addRecord: (record: ExecutionRecord) => void;
  removeRecord: (id: string) => void;
  clearAll: () => void;
  getRecord: (id: string) => ExecutionRecord | null;
  getAllRecords: () => ExecutionRecord[];
  searchRecords: (query: string) => ExecutionRecord[];
  exportAsJSON: () => string;
  exportAsCSV: () => string;
  importFromJSON: (jsonString: string) => boolean;
}

export const useHistoryStore = create<HistoryState>()(
  persist(
    (set, get) => ({
      records: [],

      addRecord: (record: ExecutionRecord) =>
        set((state) => {
          const newRecords = [record, ...state.records];
          // Keep only the last MAX_STORED_RECORDS
          if (newRecords.length > MAX_STORED_RECORDS) {
            newRecords.splice(MAX_STORED_RECORDS);
          }
          return { records: newRecords };
        }),

      removeRecord: (id: string) =>
        set((state) => ({
          records: state.records.filter((r) => r.id !== id),
        })),

      clearAll: () =>
        set(() => ({
          records: [],
        })),

      getRecord: (id: string) => {
        return get().records.find((r) => r.id === id) || null;
      },

      getAllRecords: () => {
        return get().records;
      },

      searchRecords: (query: string) => {
        const lowerQuery = query.toLowerCase();
        return get().records.filter(
          (r) =>
            r.scenarioName.toLowerCase().includes(lowerQuery) ||
            r.status.toLowerCase().includes(lowerQuery) ||
            r.id.toLowerCase().includes(lowerQuery)
        );
      },

      exportAsJSON: () => {
        const records = get().records;
        return JSON.stringify(records, null, 2);
      },

      exportAsCSV: () => {
        const records = get().records;
        if (records.length === 0) {
          return 'timestamp,scenarioName,status,totalSteps,completedSteps,duration,logCount,stepCount\n';
        }

        const headers = ['timestamp', 'scenarioName', 'status', 'totalSteps', 'completedSteps', 'duration', 'logCount', 'stepCount'];
        const rows = records.map((r) => [
          new Date(r.timestamp).toISOString(),
          `"${r.scenarioName}"`,
          r.status,
          r.totalSteps,
          r.completedSteps,
          r.duration,
          r.logs.length,
          r.stepExecutions.length,
        ]);

        const csv = [
          headers.join(','),
          ...rows.map((r) => r.join(',')),
        ].join('\n');

        return csv;
      },

      importFromJSON: (jsonString: string) => {
        try {
          const imported = JSON.parse(jsonString);
          if (!Array.isArray(imported)) {
            throw new Error('Invalid format: expected array');
          }

          // Validate structure
          for (const record of imported) {
            if (!record.id || !record.scenarioName || !record.timestamp) {
              throw new Error('Invalid record structure');
            }
          }

          set((state) => {
            const newRecords = [...imported, ...state.records];
            if (newRecords.length > MAX_STORED_RECORDS) {
              newRecords.splice(MAX_STORED_RECORDS);
            }
            return { records: newRecords };
          });

          return true;
        } catch (error) {
          console.error('Failed to import history:', error);
          return false;
        }
      },
    }),
    {
      name: STORAGE_KEY,
      version: 1,
    }
  )
);
