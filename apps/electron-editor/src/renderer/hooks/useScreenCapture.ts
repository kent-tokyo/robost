import { useCallback, useState, useEffect, useRef } from 'react';
import { useRunStore } from '../store/runStore';

export interface ScreenCapture {
  imageData: string; // base64 encoded image
  timestamp: number;
}

export const useScreenCapture = () => {
  const [screenshot, setScreenshot] = useState<ScreenCapture | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [autoRefreshInterval, setAutoRefreshInterval] = useState<number | null>(null);
  const serverPort = useRunStore((state) => state.serverPort);
  const autoRefreshRef = useRef<NodeJS.Timeout | null>(null);

  /**
   * Fetch screenshot from RPA server
   */
  const captureScreen = useCallback(async (): Promise<boolean> => {
    if (!serverPort) {
      setError('RPA server not running');
      return false;
    }

    setLoading(true);
    setError(null);

    try {
      if (!window.electronAPI?.rpaScreenshot) {
        throw new Error('Electron API not available');
      }

      const base64Data = await window.electronAPI.rpaScreenshot(serverPort);
      const imageData = `data:image/png;base64,${base64Data}`;
      setScreenshot({
        imageData,
        timestamp: Date.now(),
      });
      return true;
    } catch (err: any) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      setError(`Failed to capture screen: ${errorMsg}`);
      console.error('Screenshot error:', err);
      return false;
    } finally {
      setLoading(false);
    }
  }, [serverPort]);

  /**
   * Start auto-refresh
   */
  const startAutoRefresh = useCallback(
    (intervalMs: number) => {
      // Clear existing interval
      if (autoRefreshRef.current) {
        clearInterval(autoRefreshRef.current);
      }

      setAutoRefreshInterval(intervalMs);

      // Capture immediately
      captureScreen();

      // Set up interval
      autoRefreshRef.current = setInterval(() => {
        captureScreen();
      }, intervalMs);
    },
    [captureScreen]
  );

  /**
   * Stop auto-refresh
   */
  const stopAutoRefresh = useCallback(() => {
    if (autoRefreshRef.current) {
      clearInterval(autoRefreshRef.current);
      autoRefreshRef.current = null;
    }
    setAutoRefreshInterval(null);
  }, []);

  /**
   * Cleanup on unmount
   */
  useEffect(() => {
    return () => {
      if (autoRefreshRef.current) {
        clearInterval(autoRefreshRef.current);
      }
    };
  }, []);

  return {
    screenshot,
    loading,
    error,
    autoRefreshInterval,
    captureScreen,
    startAutoRefresh,
    stopAutoRefresh,
  };
};
