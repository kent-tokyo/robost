import { useEffect, useCallback } from 'react';

interface HotkeyHandler {
  key: string;
  ctrlKey?: boolean;
  shiftKey?: boolean;
  altKey?: boolean;
  metaKey?: boolean;
  handler: (e: KeyboardEvent) => void;
}

export const useCanvasHotkeys = (hotkeys: HotkeyHandler[]) => {
  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      hotkeys.forEach((hotkey) => {
        const matchesKey = e.key.toLowerCase() === hotkey.key.toLowerCase();
        const matchesCtrl = hotkey.ctrlKey === undefined || hotkey.ctrlKey === e.ctrlKey;
        const matchesShift = hotkey.shiftKey === undefined || hotkey.shiftKey === e.shiftKey;
        const matchesAlt = hotkey.altKey === undefined || hotkey.altKey === e.altKey;
        const matchesMeta = hotkey.metaKey === undefined || hotkey.metaKey === e.metaKey;

        if (matchesKey && matchesCtrl && matchesShift && matchesAlt && matchesMeta) {
          hotkey.handler(e);
        }
      });
    },
    [hotkeys]
  );

  useEffect(() => {
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleKeyDown]);
};

/**
 * Check if current element is editable
 */
export const isEditableElement = (): boolean => {
  const activeElement = document.activeElement;
  return (
    activeElement?.tagName === 'INPUT' ||
    activeElement?.tagName === 'TEXTAREA' ||
    (activeElement as any)?.contentEditable === 'true'
  );
};

/**
 * Create platform-aware hotkey (Cmd on Mac, Ctrl on Windows/Linux)
 */
export const createPlatformHotkey = (key: string): HotkeyHandler => {
  const isMac = /Mac|iPhone|iPad|iPod/.test(navigator.platform);
  return {
    key,
    metaKey: isMac,
    ctrlKey: !isMac,
  } as HotkeyHandler;
};
