import { contextBridge, ipcRenderer } from 'electron';

contextBridge.exposeInMainWorld('electronAPI', {
  // File/App
  getAppPath: () => ipcRenderer.invoke('get-app-path'),

  // File operations
  openScenario: () => ipcRenderer.invoke('file:open-scenario'),
  saveScenario: (filePath: string, content: string) => ipcRenderer.invoke('file:save-scenario', filePath, content),
  saveAsScenario: (content: string) => ipcRenderer.invoke('file:save-as-scenario', content),
  readFile: (filePath: string) => ipcRenderer.invoke('file:read', filePath),

  // IPC messaging
  ipcSend: (channel: string, data: any) => ipcRenderer.send(channel, data),
  ipcOn: (channel: string, callback: (data: any) => void) => {
    const listener = (event: any, data: any) => callback(data);
    ipcRenderer.on(channel, listener);
    return () => ipcRenderer.off(channel, listener);
  },

  // RPA execution
  rpaRun: (yamlPath: string) => ipcRenderer.invoke('rpa:run', yamlPath),
  rpaStop: () => ipcRenderer.invoke('rpa:stop'),
  rpaIsRunning: () => ipcRenderer.invoke('rpa:is-running'),
});

export {};
