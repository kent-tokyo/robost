import { useCallback } from 'react';
import * as YAML from 'js-yaml';
import { useScenarioStore } from '../store/scenarioStore';
import { useEditorStore } from '../store/editorStore';
import { useSettingsStore } from '../store/settingsStore';
import { Scenario } from '../types';

// electronAPI is declared globally in useRpaServer.ts, extend it here for file operations

interface FileManagerResult {
  success: boolean;
  filePath?: string;
  error?: string;
}

export const useFileManager = () => {
  const { scenario, setScenario } = useScenarioStore();
  const { setScenarioPath } = useEditorStore();
  const { addRecentFile, recentFiles } = useSettingsStore();

  /**
   * Convert scenario object to YAML string
   */
  const scenarioToYaml = useCallback((scen: Scenario): string => {
    return YAML.dump(scen, { lineWidth: 0 });
  }, []);

  /**
   * Convert YAML string to scenario object
   */
  const yamlToScenario = useCallback((yamlContent: string): Scenario => {
    try {
      const parsed = YAML.load(yamlContent);
      if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) {
        throw new Error('YAML must contain an object');
      }
      if (!('steps' in parsed)) {
        (parsed as any).steps = [];
      }
      return parsed as Scenario;
    } catch (err: any) {
      console.error('Failed to parse YAML:', err);
      throw new Error(`Invalid YAML format: ${err.message}`);
    }
  }, []);

  /**
   * Create a new blank scenario
   */
  const newScenario = useCallback((): void => {
    const blankScenario: Scenario = {
      name: 'Untitled Scenario',
      steps: [],
      variables: {},
    };
    setScenario(blankScenario);
    setScenarioPath('');
  }, [setScenario, setScenarioPath]);

  /**
   * Open a scenario file by path
   */
  const openScenarioByPath = useCallback(async (filePath: string): Promise<FileManagerResult> => {
    try {
      if (!window.electronAPI || !window.electronAPI.readFile) {
        throw new Error('File API not available');
      }

      const result = await window.electronAPI.readFile(filePath);
      if (!result.success || !result.content) {
        return { success: false };
      }

      const scen = yamlToScenario(result.content);
      setScenario(scen);
      setScenarioPath(filePath);
      addRecentFile(filePath);

      return { success: true, filePath };
    } catch (err: any) {
      console.error('Error opening scenario by path:', err);
      return { success: false, error: err.message };
    }
  }, [yamlToScenario, setScenario, setScenarioPath, addRecentFile]);

  /**
   * Open a scenario file from disk
   */
  const openScenario = useCallback(async (): Promise<FileManagerResult> => {
    try {
      if (!window.electronAPI || !window.electronAPI.openScenario) {
        throw new Error('File API not available');
      }

      const result = await window.electronAPI.openScenario();
      if (!result.success || !result.filePath || !result.content) {
        return { success: false };
      }

      const scen = yamlToScenario(result.content);
      setScenario(scen);
      setScenarioPath(result.filePath);
      addRecentFile(result.filePath);

      return { success: true, filePath: result.filePath };
    } catch (err: any) {
      console.error('Error opening scenario:', err);
      return { success: false, error: err.message };
    }
  }, [yamlToScenario, setScenario, setScenarioPath, addRecentFile]);

  /**
   * Save scenario to its current path
   */
  const saveScenario = useCallback(async (): Promise<FileManagerResult> => {
    const currentPath = useEditorStore.getState().scenarioPath;
    if (!currentPath) {
      return await saveAsScenario();
    }

    try {
      if (!window.electronAPI || !window.electronAPI.saveScenario) {
        throw new Error('File API not available');
      }

      const yamlContent = scenarioToYaml(scenario);
      const result = await window.electronAPI.saveScenario(currentPath, yamlContent);

      if (!result.success) {
        return { success: false };
      }

      addRecentFile(currentPath);
      return { success: true, filePath: result.filePath };
    } catch (err: any) {
      console.error('Error saving scenario:', err);
      return { success: false, error: err.message };
    }
  }, [scenario, scenarioToYaml, addRecentFile]);

  /**
   * Save scenario with a new filename
   */
  const saveAsScenario = useCallback(async (): Promise<FileManagerResult> => {
    try {
      if (!window.electronAPI || !window.electronAPI.saveAsScenario) {
        throw new Error('File API not available');
      }

      const yamlContent = scenarioToYaml(scenario);
      const result = await window.electronAPI.saveAsScenario(yamlContent);

      if (!result.success || !result.filePath) {
        return { success: false };
      }

      setScenarioPath(result.filePath);
      addRecentFile(result.filePath);

      return { success: true, filePath: result.filePath };
    } catch (err: any) {
      console.error('Error saving scenario as:', err);
      return { success: false, error: err.message };
    }
  }, [scenario, scenarioToYaml, setScenarioPath, addRecentFile]);

  return {
    newScenario,
    openScenario,
    openScenarioByPath,
    saveScenario,
    saveAsScenario,
    scenarioToYaml,
    yamlToScenario,
    recentFiles,
  };
};
