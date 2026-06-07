import React, { useState, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { ReactFlowProvider } from 'reactflow';
import Canvas from './Canvas';
import YAMLEditor from './YAMLEditor';
import ScreenPanel from './ScreenPanel';
import { useEditorStore } from '../store/editorStore';

interface EditorProps {
  scenarioPath: string;
  onNodeSelect?: (nodeId: string | null) => void;
}

type ViewMode = 'canvas' | 'list' | 'code' | 'screen';

const Editor: React.FC<EditorProps> = ({ scenarioPath, onNodeSelect }) => {
  const { t } = useTranslation();
  const [viewMode, setViewMode] = useState<ViewMode>('canvas');
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null);
  const { canUndo, canRedo, undo, redo } = useEditorStore();

  const handleNodeSelect = useCallback((nodeId: string | null) => {
    setSelectedNodeId(nodeId);
    onNodeSelect?.(nodeId);
  }, [onNodeSelect]);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', width: '100%', height: '100%' }}>
      {/* View mode tabs with undo/redo */}
      <div style={{
        display: 'flex',
        gap: '0',
        justifyContent: 'space-between',
        alignItems: 'center',
        borderBottom: '1px solid var(--color-border)',
        backgroundColor: 'var(--color-bg-secondary)',
      }}>
        <div style={{ display: 'flex', gap: '0' }}>
          {(['canvas', 'list', 'code', 'screen'] as const).map((mode) => (
            <button
              key={mode}
              onClick={() => setViewMode(mode)}
              style={{
                padding: '8px 16px',
                border: 'none',
                backgroundColor: viewMode === mode ? 'var(--color-bg-tertiary)' : 'transparent',
                color: viewMode === mode ? 'var(--color-accent)' : 'var(--color-text-secondary)',
                cursor: 'pointer',
                borderBottom: viewMode === mode ? '2px solid var(--color-accent)' : 'none',
                fontSize: '12px',
                textTransform: 'capitalize',
              }}
            >
              {mode === 'canvas' && `🎨 ${t('editor.canvas')}`}
              {mode === 'list' && `📋 ${t('editor.list')}`}
              {mode === 'code' && `📝 ${t('editor.code')}`}
              {mode === 'screen' && `🖥️ ${t('editor.screen')}`}
            </button>
          ))}
        </div>

        {/* Undo/Redo buttons */}
        <div style={{ display: 'flex', gap: 'var(--spacing-sm)', padding: '0 var(--spacing-md)' }}>
          <button
            onClick={undo}
            disabled={!canUndo()}
            style={{
              padding: '4px 8px',
              fontSize: '12px',
              opacity: canUndo() ? 1 : 0.5,
              cursor: canUndo() ? 'pointer' : 'not-allowed',
            }}
          >
            ↶ {t('editor.undo')}
          </button>
          <button
            onClick={redo}
            disabled={!canRedo()}
            style={{
              padding: '4px 8px',
              fontSize: '12px',
              opacity: canRedo() ? 1 : 0.5,
              cursor: canRedo() ? 'pointer' : 'not-allowed',
            }}
          >
            ↷ {t('editor.redo')}
          </button>
        </div>
      </div>

      {/* View content */}
      <div style={{ flex: 1, overflow: 'hidden' }}>
        {viewMode === 'canvas' && (
          <ReactFlowProvider>
            <Canvas onNodeSelect={handleNodeSelect} />
          </ReactFlowProvider>
        )}
        {viewMode === 'list' && (
          <div style={{
            padding: 'var(--spacing-lg)',
            color: 'var(--color-text-secondary)',
          }}>
            {t('editor.listViewComingSoon')}
          </div>
        )}
        {viewMode === 'code' && <YAMLEditor />}
        {viewMode === 'screen' && <ScreenPanel />}
      </div>
    </div>
  );
};

export default Editor;
