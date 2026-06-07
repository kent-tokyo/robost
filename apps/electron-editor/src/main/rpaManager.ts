import { spawn, ChildProcess } from 'child_process';
import { BrowserWindow } from 'electron';
import path from 'path';
import fs from 'fs';

export interface ProgressEvent {
  type: 'step_start' | 'step_done' | 'log' | 'finished';
  index?: number;
  name?: string;
  elapsed_ms?: number;
  level?: string;
  message?: string;
  success?: boolean;
  error?: string;
}

export class RpaManager {
  private rpaProcess: ChildProcess | null = null;
  private eventSource: EventSource | null = null;
  private mainWindow: BrowserWindow | null = null;

  constructor(mainWindow: BrowserWindow) {
    this.mainWindow = mainWindow;
  }

  /**
   * Locate rpa binary in app resources or system PATH
   */
  private locateRpaBinary(): string {
    const appPath = process.env.APP_PATH || process.resourcesPath;
    const platform = process.platform;

    // Determine platform-specific subdirectory and binary name
    let platformDir = '';
    let binaryName = 'rpa';

    if (platform === 'win32') {
      platformDir = 'win32-x64';
      binaryName = 'rpa.exe';
    } else if (platform === 'darwin') {
      const arch = process.arch === 'arm64' ? 'arm64' : 'x64';
      platformDir = `darwin-${arch}`;
      binaryName = 'rpa';
    }

    // Try app resources first (packaged app)
    const resourcePath = path.join(appPath, 'rpa', platformDir, binaryName);
    if (fs.existsSync(resourcePath)) {
      return resourcePath;
    }

    // Try alternatives paths for dev environment
    const devPath = path.join(__dirname, '../../assets/rpa', platformDir, binaryName);
    if (fs.existsSync(devPath)) {
      return devPath;
    }

    // Fall back to PATH (system binary)
    return 'rpa';
  }

  /**
   * Run a scenario and stream progress via SSE
   */
  async runScenario(yamlPath: string): Promise<number> {
    if (this.rpaProcess) {
      throw new Error('RPA is already running');
    }

    const rpaBinary = this.locateRpaBinary();
    const args = ['run', yamlPath, '--serve', '127.0.0.1:0'];

    console.log(`[RpaManager] Spawning: ${rpaBinary} ${args.join(' ')}`);

    return new Promise((resolve, reject) => {
      this.rpaProcess = spawn(rpaBinary, args, {
        stdio: ['ignore', 'pipe', 'pipe'],
      });

      let stdoutData = '';
      let port: number | null = null;

      // Capture PORT from stdout
      this.rpaProcess!.stdout!.on('data', (data: Buffer) => {
        stdoutData += data.toString();
        const match = stdoutData.match(/PORT=(\d+)/);
        if (match && !port) {
          port = parseInt(match[1], 10);
          console.log(`[RpaManager] Server started on port ${port}`);

          // Connect to SSE
          this.connectToSSE(port!)
            .then(() => {
              this.mainWindow?.webContents.send('rpa:started', { port });
            })
            .catch((e) => {
              console.error('[RpaManager] SSE connection failed:', e);
              this.mainWindow?.webContents.send('rpa:error', { message: e.message });
            });
        }
      });

      // Handle stderr
      this.rpaProcess!.stderr!.on('data', (data: Buffer) => {
        const message = data.toString();
        console.log('[rpa stderr]', message);
        this.mainWindow?.webContents.send('rpa:log', {
          level: 'error',
          message: message.trim(),
        });
      });

      // Handle process exit
      this.rpaProcess!.on('exit', (code) => {
        console.log(`[RpaManager] Process exited with code ${code}`);
        this.rpaProcess = null;
        this.eventSource?.close();
        this.eventSource = null;
        resolve(code || 0);
      });

      this.rpaProcess!.on('error', (err) => {
        console.error('[RpaManager] Process error:', err);
        reject(err);
      });
    });
  }

  /**
   * Connect to rpa server SSE endpoint
   */
  private connectToSSE(port: number): Promise<void> {
    return new Promise((resolve, reject) => {
      const url = `http://127.0.0.1:${port}/events`;

      try {
        this.eventSource = new EventSource(url);

        this.eventSource.onopen = () => {
          console.log('[RpaManager] SSE connection opened');
          resolve();
        };

        this.eventSource.onmessage = (event: MessageEvent) => {
          try {
            const data = JSON.parse(event.data);
            console.log('[RpaManager] Event:', data.type);

            // Forward to renderer
            this.mainWindow?.webContents.send('rpa:progress', data as ProgressEvent);
          } catch (e) {
            console.error('[RpaManager] Failed to parse event:', e);
          }
        };

        this.eventSource.onerror = (err: Event) => {
          console.error('[RpaManager] SSE error:', err);
          this.eventSource?.close();
          this.eventSource = null;
          reject(new Error('SSE connection error'));
        };

        // Timeout after 5 seconds if no connection
        setTimeout(() => {
          if (this.eventSource?.readyState === EventSource.CONNECTING) {
            this.eventSource?.close();
            reject(new Error('SSE connection timeout'));
          }
        }, 5000);
      } catch (err) {
        reject(err);
      }
    });
  }

  /**
   * Stop the running rpa process
   */
  stop(): void {
    if (this.rpaProcess) {
      this.rpaProcess.kill();
      this.rpaProcess = null;
    }
    if (this.eventSource) {
      this.eventSource.close();
      this.eventSource = null;
    }
  }

  /**
   * Check if rpa is currently running
   */
  isRunning(): boolean {
    return this.rpaProcess !== null;
  }
}
