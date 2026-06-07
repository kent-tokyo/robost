import React, { useState, useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import { ExplorerIcon, SearchIcon, RunIcon, ExtensionsIcon, SettingsIcon, NewFileIcon, OpenFolderIcon, SaveIcon, SaveAsIcon, SparklesIcon, getStepTypeIcon } from './Icons';
import { useFileManager } from '../hooks/useFileManager';
import { templates, TemplateDefinition } from '../utils/templates';
import SettingsPanel from './SettingsPanel';
import AiPanel from './AiPanel';
import HistoryPanel from './HistoryPanel';
import BreakpointManager from './BreakpointManager';
import VariableInspector from './VariableInspector';

type Panel = 'explorer' | 'search' | 'run' | 'extensions' | 'settings' | 'history' | null;
type TabType = 'explorer' | 'recent' | 'templates' | 'ai' | 'breakpoints' | 'variables';

interface SidebarProps {
  panel: Panel;
  scenarioPath: string;
}

const Sidebar: React.FC<SidebarProps> = ({ panel, scenarioPath }) => {
  const { t } = useTranslation();
  const [activeTab, setActiveTab] = useState<TabType>('explorer');
  const { newScenario, openScenario, openScenarioByPath, saveScenario, saveAsScenario, recentFiles } = useFileManager();

  const handleNewScenario = async () => {
    newScenario();
  };

  const handleOpenScenario = async () => {
    const result = await openScenario();
    if (!result.success) {
      console.error('Failed to open scenario:', result.error);
    }
  };

  const handleSaveScenario = async () => {
    const result = await saveScenario();
    if (!result.success) {
      console.error('Failed to save scenario:', result.error);
    }
  };

  const handleSaveAsScenario = async () => {
    const result = await saveAsScenario();
    if (!result.success) {
      console.error('Failed to save scenario as:', result.error);
    }
  };

  const handleOpenRecentFile = async (filePath: string) => {
    const result = await openScenarioByPath(filePath);
    if (!result.success) {
      console.error('Failed to open recent file:', result.error);
    }
  };

  const handleDragStart = (e: React.DragEvent<HTMLDivElement>, template: TemplateDefinition) => {
    e.dataTransfer.effectAllowed = 'move';
    e.dataTransfer.setData('stepType', template.id);
    e.dataTransfer.setData('templateData', JSON.stringify(template.generateSteps()));
  };

  const groupedTemplates = useMemo(() => {
    return {
      action: templates.filter((t) => t.category === 'action'),
      control: templates.filter((t) => t.category === 'control'),
      utility: templates.filter((t) => t.category === 'utility'),
    };
  }, []);

  return (
    <div className="sidebar">
      {/* Sidebar Header with Title */}
      <div
        className="sidebar-header"
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          padding: '8px 16px',
          borderBottom: '1px solid var(--color-border)',
        }}
      >
        <span style={{ fontSize: '13px', fontWeight: '600' }}>
          {panel === 'explorer' && `📁 ${t('sidebar.explorer')}`}
          {panel === 'search' && `🔍 ${t('sidebar.search')}`}
          {panel === 'run' && `▶️ ${t('sidebar.run')}`}
          {panel === 'extensions' && `🧩 ${t('sidebar.extensions')}`}
          {panel === 'settings' && `⚙️ ${t('sidebar.settings')}`}
        </span>
      </div>

      {/* Explorer Panel */}
      {panel === 'explorer' && (
        <div className="sidebar-content" style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
          {/* File Action Buttons */}
          <div
            style={{
              display: 'grid',
              gridTemplateColumns: '1fr 1fr',
              gap: '4px',
              padding: '8px',
              borderBottom: '1px solid var(--color-border)',
            }}
          >
            <button
              onClick={handleNewScenario}
              style={{
                padding: '6px 8px',
                backgroundColor: 'var(--color-bg-tertiary)',
                color: 'var(--color-text)',
                border: '1px solid var(--color-border)',
                borderRadius: '4px',
                cursor: 'pointer',
                fontSize: '11px',
                fontWeight: '500',
              }}
              title="Create a new scenario"
            >
              📄 New
            </button>
            <button
              onClick={handleOpenScenario}
              style={{
                padding: '6px 8px',
                backgroundColor: 'var(--color-bg-tertiary)',
                color: 'var(--color-text)',
                border: '1px solid var(--color-border)',
                borderRadius: '4px',
                cursor: 'pointer',
                fontSize: '11px',
                fontWeight: '500',
              }}
              title="Open a scenario file"
            >
              📂 Open
            </button>
            <button
              onClick={handleSaveScenario}
              disabled={!scenarioPath}
              style={{
                padding: '6px 8px',
                backgroundColor: 'var(--color-bg-tertiary)',
                color: scenarioPath ? 'var(--color-text)' : 'var(--color-text-secondary)',
                border: '1px solid var(--color-border)',
                borderRadius: '4px',
                cursor: scenarioPath ? 'pointer' : 'not-allowed',
                fontSize: '11px',
                fontWeight: '500',
                opacity: scenarioPath ? 1 : 0.5,
              }}
              title="Save the current scenario"
            >
              💾 Save
            </button>
            <button
              onClick={handleSaveAsScenario}
              style={{
                padding: '6px 8px',
                backgroundColor: 'var(--color-bg-tertiary)',
                color: 'var(--color-text)',
                border: '1px solid var(--color-border)',
                borderRadius: '4px',
                cursor: 'pointer',
                fontSize: '11px',
                fontWeight: '500',
              }}
              title="Save with a new filename"
            >
              ⤳ Save As
            </button>
          </div>

          {/* Tabs */}
          <div
            style={{
              display: 'flex',
              gap: '0',
              borderBottom: '1px solid var(--color-border)',
              padding: '4px',
              overflowX: 'auto',
            }}
          >
            {(['explorer', 'recent', 'templates', 'ai'] as TabType[]).map((tab) => (
              <button
                key={tab}
                onClick={() => setActiveTab(tab)}
                style={{
                  padding: '6px 12px',
                  border: 'none',
                  backgroundColor: activeTab === tab ? 'var(--color-bg-tertiary)' : 'transparent',
                  color: activeTab === tab ? 'var(--color-accent)' : 'var(--color-text-secondary)',
                  cursor: 'pointer',
                  fontSize: '11px',
                  fontWeight: '500',
                  borderBottom: activeTab === tab ? '2px solid var(--color-accent)' : 'none',
                  textTransform: 'capitalize',
                  whiteSpace: 'nowrap',
                }}
              >
                {tab === 'explorer' && 'Explorer'}
                {tab === 'recent' && 'Recent'}
                {tab === 'templates' && 'Templates'}
                {tab === 'ai' && '✨ AI'}
              </button>
            ))}
          </div>

          {/* Tab Content */}
          <div style={{ flex: 1, overflow: 'auto', padding: '8px' }}>
            {/* Explorer Tab */}
            {activeTab === 'explorer' && (
              <div>
                <p style={{ color: 'var(--color-text-secondary)', fontSize: '12px', margin: '8px 0' }}>
                  {scenarioPath ? (
                    <>
                      <strong>Current:</strong>
                      <br />
                      {scenarioPath}
                    </>
                  ) : (
                    'Open or create a scenario file to get started'
                  )}
                </p>
              </div>
            )}

            {/* Recent Tab */}
            {activeTab === 'recent' && (
              <div>
                {recentFiles.length > 0 ? (
                  <div>
                    {recentFiles.map((file) => (
                      <div
                        key={file.path}
                        onClick={() => handleOpenRecentFile(file.path)}
                        style={{
                          padding: '6px 8px',
                          marginBottom: '4px',
                          backgroundColor: 'var(--color-bg-tertiary)',
                          borderRadius: '4px',
                          cursor: 'pointer',
                          fontSize: '11px',
                          borderLeft: '2px solid var(--color-accent)',
                          transition: 'all 0.2s',
                        }}
                        onMouseOver={(e) => {
                          (e.currentTarget as HTMLDivElement).style.backgroundColor = 'var(--color-bg-quaternary)';
                        }}
                        onMouseOut={(e) => {
                          (e.currentTarget as HTMLDivElement).style.backgroundColor = 'var(--color-bg-tertiary)';
                        }}
                        title={file.path}
                      >
                        <strong>{file.name}</strong>
                        <div style={{ fontSize: '10px', color: 'var(--color-text-secondary)', marginTop: '2px' }}>
                          {new Date(file.timestamp).toLocaleDateString()}
                        </div>
                      </div>
                    ))}
                  </div>
                ) : (
                  <p style={{ color: 'var(--color-text-secondary)', fontSize: '12px' }}>No recent files</p>
                )}
              </div>
            )}

            {/* Templates Tab */}
            {activeTab === 'templates' && (
              <div>
                {/* Action Templates */}
                {groupedTemplates.action.length > 0 && (
                  <div style={{ marginBottom: '16px' }}>
                    <div style={{ fontSize: '11px', fontWeight: '600', color: 'var(--color-accent)', marginBottom: '6px' }}>
                      Actions
                    </div>
                    {groupedTemplates.action.map((template) => (
                      <div
                        key={template.id}
                        draggable
                        onDragStart={(e) => handleDragStart(e, template)}
                        style={{
                          padding: '8px',
                          marginBottom: '4px',
                          backgroundColor: 'var(--color-bg-tertiary)',
                          borderRadius: '4px',
                          cursor: 'grab',
                          fontSize: '11px',
                          border: '1px solid var(--color-border)',
                          transition: 'all 0.2s',
                        }}
                        onMouseOver={(e) => {
                          (e.currentTarget as HTMLDivElement).style.backgroundColor = 'var(--color-bg-quaternary)';
                          (e.currentTarget as HTMLDivElement).style.cursor = 'grabbing';
                        }}
                        onMouseOut={(e) => {
                          (e.currentTarget as HTMLDivElement).style.backgroundColor = 'var(--color-bg-tertiary)';
                          (e.currentTarget as HTMLDivElement).style.cursor = 'grab';
                        }}
                      >
                        <div style={{ fontWeight: '500' }}>
                          {template.icon} {template.name}
                        </div>
                        <div style={{ fontSize: '10px', color: 'var(--color-text-secondary)', marginTop: '2px' }}>
                          {template.description}
                        </div>
                      </div>
                    ))}
                  </div>
                )}

                {/* Control Flow Templates */}
                {groupedTemplates.control.length > 0 && (
                  <div style={{ marginBottom: '16px' }}>
                    <div style={{ fontSize: '11px', fontWeight: '600', color: 'var(--color-accent)', marginBottom: '6px' }}>
                      Control Flow
                    </div>
                    {groupedTemplates.control.map((template) => (
                      <div
                        key={template.id}
                        draggable
                        onDragStart={(e) => handleDragStart(e, template)}
                        style={{
                          padding: '8px',
                          marginBottom: '4px',
                          backgroundColor: 'var(--color-bg-tertiary)',
                          borderRadius: '4px',
                          cursor: 'grab',
                          fontSize: '11px',
                          border: '1px solid var(--color-border)',
                          transition: 'all 0.2s',
                        }}
                        onMouseOver={(e) => {
                          (e.currentTarget as HTMLDivElement).style.backgroundColor = 'var(--color-bg-quaternary)';
                          (e.currentTarget as HTMLDivElement).style.cursor = 'grabbing';
                        }}
                        onMouseOut={(e) => {
                          (e.currentTarget as HTMLDivElement).style.backgroundColor = 'var(--color-bg-tertiary)';
                          (e.currentTarget as HTMLDivElement).style.cursor = 'grab';
                        }}
                      >
                        <div style={{ fontWeight: '500' }}>
                          {template.icon} {template.name}
                        </div>
                        <div style={{ fontSize: '10px', color: 'var(--color-text-secondary)', marginTop: '2px' }}>
                          {template.description}
                        </div>
                      </div>
                    ))}
                  </div>
                )}

                {/* Utility Templates */}
                {groupedTemplates.utility.length > 0 && (
                  <div style={{ marginBottom: '16px' }}>
                    <div style={{ fontSize: '11px', fontWeight: '600', color: 'var(--color-accent)', marginBottom: '6px' }}>
                      Utilities
                    </div>
                    {groupedTemplates.utility.map((template) => (
                      <div
                        key={template.id}
                        draggable
                        onDragStart={(e) => handleDragStart(e, template)}
                        style={{
                          padding: '8px',
                          marginBottom: '4px',
                          backgroundColor: 'var(--color-bg-tertiary)',
                          borderRadius: '4px',
                          cursor: 'grab',
                          fontSize: '11px',
                          border: '1px solid var(--color-border)',
                          transition: 'all 0.2s',
                        }}
                        onMouseOver={(e) => {
                          (e.currentTarget as HTMLDivElement).style.backgroundColor = 'var(--color-bg-quaternary)';
                          (e.currentTarget as HTMLDivElement).style.cursor = 'grabbing';
                        }}
                        onMouseOut={(e) => {
                          (e.currentTarget as HTMLDivElement).style.backgroundColor = 'var(--color-bg-tertiary)';
                          (e.currentTarget as HTMLDivElement).style.cursor = 'grab';
                        }}
                      >
                        <div style={{ fontWeight: '500' }}>
                          {template.icon} {template.name}
                        </div>
                        <div style={{ fontSize: '10px', color: 'var(--color-text-secondary)', marginTop: '2px' }}>
                          {template.description}
                        </div>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            )}

            {/* AI Tab */}
            {activeTab === 'ai' && <AiPanel />}

            {/* Breakpoints Tab */}
            {activeTab === 'breakpoints' && <BreakpointManager />}

            {/* Variables Tab */}
            {activeTab === 'variables' && <VariableInspector />}
          </div>
        </div>
      )}

      {/* History Panel */}
      {panel === 'history' && (
        <div className="sidebar-content" style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
          <HistoryPanel />
        </div>
      )}

      {/* Settings Panel */}
      {panel === 'settings' && (
        <div className="sidebar-content" style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
          <SettingsPanel />
        </div>
      )}

      {/* Other Panels */}
      {panel !== 'explorer' && panel !== 'settings' && (
        <div className="sidebar-content">
          <p style={{ color: 'var(--color-text-secondary)', fontSize: '12px' }}>
            {panel === 'search' && t('sidebar.searchDesc')}
            {panel === 'run' && t('sidebar.runDesc')}
            {panel === 'extensions' && t('sidebar.extensionsDesc')}
          </p>
        </div>
      )}
    </div>
  );
};

export default Sidebar;
