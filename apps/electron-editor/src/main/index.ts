import { app, BrowserWindow, Menu, ipcMain, dialog } from 'electron';
import path from 'path';
import isDev from 'electron-is-dev';
import { promises as fs } from 'fs';
import { RpaManager } from './rpaManager';

let mainWindow: BrowserWindow | null = null;
let rpaManager: RpaManager | null = null;

const createWindow = () => {
  mainWindow = new BrowserWindow({
    width: 1400,
    height: 900,
    minWidth: 800,
    minHeight: 600,
    webPreferences: {
      nodeIntegration: false,
      contextIsolation: true,
      preload: path.join(__dirname, '..', '..', '.webpack', 'main', 'preload.js'),
    },
  });

  const startURL = isDev
    ? 'http://localhost:3000'
    : `file://${path.join(__dirname, '..', '..', '.webpack', 'renderer', 'main_window', 'index.html')}`;

  mainWindow.loadURL(startURL);

  // Show window when ready
  mainWindow.once('ready-to-show', () => {
    mainWindow?.show();
  });

  if (isDev) {
    mainWindow.webContents.openDevTools();
  }

  mainWindow.on('closed', () => {
    mainWindow = null;
    if (rpaManager) {
      rpaManager.stop();
    }
  });

  createMenu();

  // Initialize RpaManager
  rpaManager = new RpaManager(mainWindow);
};

const createMenu = () => {
  const template: Electron.MenuItemConstructorOptions[] = [
    {
      label: 'File',
      submenu: [
        {
          label: 'Exit',
          accelerator: 'CmdOrCtrl+Q',
          click: () => {
            app.quit();
          },
        },
      ],
    },
    {
      label: 'Edit',
      submenu: [
        { role: 'undo' },
        { role: 'redo' },
        { type: 'separator' },
        { role: 'cut' },
        { role: 'copy' },
        { role: 'paste' },
      ],
    },
    {
      label: 'View',
      submenu: [
        { role: 'reload' },
        { role: 'forceReload' },
        { role: 'toggleDevTools' },
      ],
    },
  ];

  const menu = Menu.buildFromTemplate(template);
  Menu.setApplicationMenu(menu);
};

app.on('ready', createWindow);

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

app.on('activate', () => {
  if (mainWindow === null) {
    createWindow();
  }
});

// IPC handlers
ipcMain.on('get-app-path', (event) => {
  event.reply('app-path', app.getAppPath());
});

// File operation handlers
ipcMain.handle('file:read', async (event, filePath: string) => {
  try {
    const content = await fs.readFile(filePath, 'utf-8');
    return { success: true, content };
  } catch (err: any) {
    throw new Error(`Failed to read file: ${err.message}`);
  }
});

ipcMain.handle('file:open-scenario', async (event) => {
  if (!mainWindow) throw new Error('Main window not available');

  const result = await dialog.showOpenDialog(mainWindow, {
    properties: ['openFile'],
    filters: [
      { name: 'YAML Files', extensions: ['yaml', 'yml'] },
      { name: 'All Files', extensions: ['*'] },
    ],
  });

  if (result.canceled || result.filePaths.length === 0) {
    return { success: false };
  }

  const filePath = result.filePaths[0];
  try {
    const content = await fs.readFile(filePath, 'utf-8');
    return { success: true, filePath, content };
  } catch (err: any) {
    throw new Error(`Failed to read file: ${err.message}`);
  }
});

ipcMain.handle('file:save-scenario', async (event, filePath: string, content: string) => {
  try {
    await fs.writeFile(filePath, content, 'utf-8');
    return { success: true, filePath };
  } catch (err: any) {
    throw new Error(`Failed to save file: ${err.message}`);
  }
});

ipcMain.handle('file:save-as-scenario', async (event, content: string) => {
  if (!mainWindow) throw new Error('Main window not available');

  const result = await dialog.showSaveDialog(mainWindow, {
    defaultPath: 'scenario.yaml',
    filters: [
      { name: 'YAML Files', extensions: ['yaml', 'yml'] },
      { name: 'All Files', extensions: ['*'] },
    ],
  });

  if (result.canceled || !result.filePath) {
    return { success: false };
  }

  try {
    await fs.writeFile(result.filePath, content, 'utf-8');
    return { success: true, filePath: result.filePath };
  } catch (err: any) {
    throw new Error(`Failed to save file: ${err.message}`);
  }
});

// RPA execution handlers
ipcMain.handle('rpa:run', async (event, yamlPath: string) => {
  if (!rpaManager) {
    throw new Error('RpaManager not initialized');
  }

  try {
    // Run in background
    rpaManager.runScenario(yamlPath).catch((err) => {
      console.error('[RpaManager] Runtime error:', err);
      mainWindow?.webContents.send('rpa:error', { message: err.message });
    });

    return { success: true };
  } catch (err: any) {
    throw new Error(err.message || 'Failed to start RPA');
  }
});

ipcMain.handle('rpa:stop', () => {
  if (rpaManager) {
    rpaManager.stop();
  }
  return { success: true };
});

ipcMain.handle('rpa:is-running', () => {
  return rpaManager?.isRunning() || false;
});

// Screenshot handler
ipcMain.handle('rpa:screenshot', async (event, serverPort: number) => {
  if (!serverPort) {
    throw new Error('Server port not available');
  }

  return new Promise<string>((resolve, reject) => {
    const http = require('http');

    const request = http.get(`http://127.0.0.1:${serverPort}/screenshot`, (response: any) => {
      if (response.statusCode !== 200) {
        reject(new Error(`HTTP ${response.statusCode}: ${response.statusMessage}`));
        return;
      }

      const chunks: Buffer[] = [];

      response.on('data', (chunk: Buffer) => {
        chunks.push(chunk);
      });

      response.on('end', () => {
        const buffer = Buffer.concat(chunks);
        const base64 = buffer.toString('base64');
        resolve(base64);
      });
    });

    request.on('error', (err: any) => {
      reject(new Error(`Failed to fetch screenshot: ${err.message}`));
    });

    // Set timeout to avoid hanging
    request.setTimeout(5000, () => {
      request.destroy();
      reject(new Error('Screenshot request timeout'));
    });
  });
});
