import { useEffect, useCallback } from 'react';
import { useRunStore } from '../store/runStore';
import { useScenarioStore } from '../store/scenarioStore';

declare global {
  interface Window {
    electronAPI?: {
      // File operations
      openScenario: () => Promise<{ success: boolean; filePath?: string; content?: string }>;
      saveScenario: (filePath: string, content: string) => Promise<{ success: boolean; filePath?: string }>;
      saveAsScenario: (content: string) => Promise<{ success: boolean; filePath?: string }>;
      readFile: (filePath: string) => Promise<{ success: boolean; content?: string }>;

      // RPA execution
      rpaRun: (yamlPath: string) => Promise<{ success: boolean }>;
      rpaStop: () => Promise<{ success: boolean }>;
      rpaIsRunning: () => Promise<boolean>;

      // IPC messaging
      ipcOn: (
        channel: string,
        callback: (data: any) => void
      ) => () => void;
      ipcSend: (channel: string, data: any) => void;
    };
  }
}

export const useRpaServer = () => {
  const {
    startRun,
    stopRun,
    setCurrentStep,
    addLog,
    setServerPort,
  } = useRunStore();
  const { scenario } = useScenarioStore();

  // Listen for RPA events from main process
  useEffect(() => {
    if (!window.electronAPI) return;

    const unsubProgress = window.electronAPI.ipcOn('rpa:progress', (event) => {
      console.log('[Renderer] Progress event:', event);

      switch (event.type) {
        case 'scenario_start':
          startRun(event.total || 0);
          break;

        case 'step_start':
          // Update UI to show current step
          if (event.index !== undefined) {
            setCurrentStep(event.index, 0);
            addLog('info', `Starting: ${event.name}`);
          }
          break;

        case 'step_done':
          if (event.index !== undefined && event.elapsed_ms !== undefined) {
            setCurrentStep(event.index, event.elapsed_ms);
            addLog('info', `Completed in ${event.elapsed_ms}ms`);
          }
          break;

        case 'log':
          if (event.message) {
            addLog(event.level || 'info', event.message);
          }
          break;

        case 'finished':
          stopRun();
          if (event.success) {
            addLog('info', 'Scenario completed successfully');
          } else if (event.error) {
            addLog('error', `Scenario failed: ${event.error}`);
          }
          break;
      }
    });

    const unsubStarted = window.electronAPI.ipcOn('rpa:started', (data) => {
      console.log('[Renderer] RPA server started on port', data.port);
      setServerPort(data.port);
      addLog('info', `Server started on port ${data.port}`);
    });

    const unsubError = window.electronAPI.ipcOn('rpa:error', (data) => {
      console.error('[Renderer] RPA error:', data);
      addLog('error', data.message || 'Unknown error');
      stopRun();
    });

    const unsubLog = window.electronAPI.ipcOn('rpa:log', (data) => {
      addLog(data.level || 'info', data.message);
    });

    return () => {
      unsubProgress();
      unsubStarted();
      unsubError();
      unsubLog();
    };
  }, [startRun, stopRun, setCurrentStep, addLog, setServerPort]);

  // Run scenario
  const runScenario = useCallback(
    async (yamlPath?: string) => {
      if (!window.electronAPI) {
        throw new Error('Electron API not available');
      }

      const path = yamlPath || '/tmp/scenario.yaml';

      try {
        addLog('info', `Running scenario: ${path}`);
        const result = await window.electronAPI.rpaRun(path);
        console.log('[Renderer] runScenario result:', result);
      } catch (err: any) {
        addLog('error', err.message || 'Failed to run scenario');
        stopRun();
      }
    },
    [addLog, stopRun]
  );

  // Stop scenario
  const stopScenario = useCallback(async () => {
    if (!window.electronAPI) return;

    try {
      await window.electronAPI.rpaStop();
      addLog('info', 'Stopped by user');
      stopRun();
    } catch (err: any) {
      addLog('error', err.message || 'Failed to stop scenario');
    }
  }, [addLog, stopRun]);

  // Check if running
  const checkIfRunning = useCallback(async () => {
    if (!window.electronAPI) return false;
    return window.electronAPI.rpaIsRunning();
  }, []);

  return {
    runScenario,
    stopScenario,
    checkIfRunning,
  };
};
