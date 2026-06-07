import React, { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import ActivityBar from './components/ActivityBar';
import Sidebar from './components/Sidebar';
import Editor from './components/Editor';
import StatusBar from './components/StatusBar';
import ProgressPanel from './components/ProgressPanel';
import Inspector from './components/Inspector';
import { useRpaServer } from './hooks/useRpaServer';
import { useSettingsStore } from './store/settingsStore';
import { useEditorStore } from './store/editorStore';
import './App.css';

type Panel = 'explorer' | 'search' | 'run' | 'extensions' | 'settings' | 'history' | null;

const App: React.FC = () => {
  const { i18n, t } = useTranslation();
  const { theme } = useSettingsStore();
  const { scenarioPath } = useEditorStore();
  const [activePanel, setActivePanel] = useState<Panel>('explorer');
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null);
  const [showInspector, setShowInspector] = useState(true);

  // Initialize RPA server hooks
  useRpaServer();

  // Apply theme to root element
  useEffect(() => {
    const root = document.documentElement;
    if (theme === 'light') {
      root.setAttribute('data-theme', 'light');
    } else {
      root.removeAttribute('data-theme');
    }
  }, [theme]);

  return (
    <div className="workbench">
      {/* Activity Bar (left) */}
      <ActivityBar activePanel={activePanel} onPanelChange={setActivePanel} />

      {/* Main content area */}
      <div style={{ display: 'flex', flexDirection: 'column', flex: 1, overflow: 'hidden' }}>
        {/* Sidebar + Editor + Inspector */}
        <div style={{ display: 'flex', flex: 1, overflow: 'hidden' }}>
          {/* Sidebar */}
          {activePanel && <Sidebar panel={activePanel} scenarioPath={scenarioPath || ''} />}

          {/* Main Editor Area */}
          <div className="editor-area">
            <Editor scenarioPath={scenarioPath || ''} onNodeSelect={setSelectedNodeId} />
          </div>

          {/* Inspector Panel */}
          {showInspector && (
            <div style={{
              width: '340px',
              borderLeft: '1px solid var(--color-border)',
              overflow: 'hidden',
              display: 'flex',
              flexDirection: 'column',
            }}>
              <div style={{
                padding: '8px 16px',
                borderBottom: '1px solid var(--color-border)',
                fontSize: '12px',
                fontWeight: '600',
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'center',
              }}>
                <span>{t('inspector.inspector')}</span>
                <button
                  onClick={() => setShowInspector(false)}
                  style={{
                    background: 'none',
                    border: 'none',
                    color: 'var(--color-text-secondary)',
                    cursor: 'pointer',
                    fontSize: '14px',
                  }}
                >
                  ×
                </button>
              </div>
              <Inspector selectedNodeId={selectedNodeId} />
            </div>
          )}
        </div>

        {/* Progress/Log Panel */}
        <ProgressPanel />
      </div>

      {/* Status Bar (bottom) */}
      <StatusBar />
    </div>
  );
};

export default App;
