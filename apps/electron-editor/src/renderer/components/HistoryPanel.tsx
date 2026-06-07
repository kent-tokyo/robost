import React, { useState, useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import { useHistoryStore } from '../store/historyStore';
import { useRunStore, ExecutionRecord } from '../store/runStore';
import ExecutionReplay from './ExecutionReplay';
import './HistoryPanel.css';

type FilterStatus = 'all' | 'success' | 'failed' | 'stopped';
type ViewMode = 'list' | 'details' | 'replay';

interface HistoryPanelProps {
  onSelectExecution?: (record: ExecutionRecord) => void;
}

const HistoryPanel: React.FC<HistoryPanelProps> = ({ onSelectExecution }) => {
  const { t } = useTranslation();
  const { records, removeRecord, clearAll, exportAsJSON, exportAsCSV, importFromJSON } = useHistoryStore();
  const { deleteHistoryRecord } = useRunStore();

  const [filterStatus, setFilterStatus] = useState<FilterStatus>('all');
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [viewMode, setViewMode] = useState<ViewMode>('list');

  const filteredRecords = useMemo(() => {
    let filtered = records;

    if (filterStatus !== 'all') {
      filtered = filtered.filter((r) => r.status === filterStatus);
    }

    if (searchQuery) {
      const lowerQuery = searchQuery.toLowerCase();
      filtered = filtered.filter(
        (r) =>
          r.scenarioName.toLowerCase().includes(lowerQuery) ||
          r.id.toLowerCase().includes(lowerQuery)
      );
    }

    return filtered;
  }, [records, filterStatus, searchQuery]);

  const selectedRecord = selectedId ? records.find((r) => r.id === selectedId) : null;

  const handleDelete = (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    removeRecord(id);
    deleteHistoryRecord(id);
    if (selectedId === id) {
      setSelectedId(null);
      setViewMode('list');
    }
  };

  const handleClearAll = () => {
    if (window.confirm(t('history.confirmClearAll') || 'Clear all execution history?')) {
      clearAll();
      setSelectedId(null);
      setViewMode('list');
    }
  };

  const handleExportJSON = () => {
    const json = exportAsJSON();
    const blob = new Blob([json], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `execution-history-${Date.now()}.json`;
    a.click();
    URL.revokeObjectURL(url);
  };

  const handleExportCSV = () => {
    const csv = exportAsCSV();
    const blob = new Blob([csv], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `execution-history-${Date.now()}.csv`;
    a.click();
    URL.revokeObjectURL(url);
  };

  const handleImportJSON = async () => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.json';
    input.onchange = (e: any) => {
      const file = e.target.files[0];
      if (file) {
        const reader = new FileReader();
        reader.onload = (event: any) => {
          const success = importFromJSON(event.target.result);
          if (success) {
            alert(t('history.importSuccess') || 'History imported successfully');
          } else {
            alert(t('history.importFailed') || 'Failed to import history');
          }
        };
        reader.readAsText(file);
      }
    };
    input.click();
  };

  const formatTime = (timestamp: number) => {
    return new Date(timestamp).toLocaleString();
  };

  const formatDuration = (ms: number) => {
    const seconds = Math.floor(ms / 1000);
    const minutes = Math.floor(seconds / 60);
    if (minutes > 0) {
      return `${minutes}m ${seconds % 60}s`;
    }
    return `${seconds}s`;
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'success':
        return '#4ade80';
      case 'failed':
        return '#ef4444';
      case 'stopped':
        return '#f97316';
      default:
        return '#6b7280';
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'success':
        return '✓';
      case 'failed':
        return '✕';
      case 'stopped':
        return '⏹';
      default:
        return '◯';
    }
  };

  return (
    <div className="history-panel">
      {viewMode === 'replay' && selectedRecord ? (
        <ExecutionReplay
          record={selectedRecord}
          onClose={() => {
            setViewMode('details');
          }}
        />
      ) : viewMode === 'list' ? (
        <>
          {/* Toolbar */}
          <div className="history-toolbar">
            <div className="history-search">
              <input
                type="text"
                placeholder={t('history.searchPlaceholder') || 'Search executions...'}
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                style={{
                  width: '100%',
                  padding: '6px 8px',
                  border: '1px solid var(--color-border)',
                  borderRadius: '4px',
                  backgroundColor: 'var(--color-bg-secondary)',
                  color: 'var(--color-text)',
                  fontSize: '11px',
                }}
              />
            </div>

            {/* Filter buttons */}
            <div className="history-filters">
              {(['all', 'success', 'failed', 'stopped'] as FilterStatus[]).map((status) => (
                <button
                  key={status}
                  onClick={() => setFilterStatus(status)}
                  style={{
                    padding: '4px 8px',
                    border: filterStatus === status ? '1px solid var(--color-accent)' : '1px solid var(--color-border)',
                    backgroundColor: filterStatus === status ? 'var(--color-accent)' : 'transparent',
                    color: filterStatus === status ? '#fff' : 'var(--color-text-secondary)',
                    borderRadius: '3px',
                    cursor: 'pointer',
                    fontSize: '10px',
                    fontWeight: '500',
                    textTransform: 'capitalize',
                  }}
                >
                  {status}
                </button>
              ))}
            </div>

            {/* Action buttons */}
            <div className="history-actions">
              <button
                onClick={handleExportJSON}
                title={t('history.exportJSON')}
                style={{
                  padding: '4px 8px',
                  backgroundColor: 'var(--color-bg-tertiary)',
                  color: 'var(--color-text)',
                  border: '1px solid var(--color-border)',
                  borderRadius: '3px',
                  cursor: 'pointer',
                  fontSize: '10px',
                }}
              >
                {t('history.exportJSON') || 'Export JSON'}
              </button>
              <button
                onClick={handleExportCSV}
                title={t('history.exportCSV')}
                style={{
                  padding: '4px 8px',
                  backgroundColor: 'var(--color-bg-tertiary)',
                  color: 'var(--color-text)',
                  border: '1px solid var(--color-border)',
                  borderRadius: '3px',
                  cursor: 'pointer',
                  fontSize: '10px',
                }}
              >
                {t('history.exportCSV') || 'Export CSV'}
              </button>
              <button
                onClick={handleImportJSON}
                title={t('history.import')}
                style={{
                  padding: '4px 8px',
                  backgroundColor: 'var(--color-bg-tertiary)',
                  color: 'var(--color-text)',
                  border: '1px solid var(--color-border)',
                  borderRadius: '3px',
                  cursor: 'pointer',
                  fontSize: '10px',
                }}
              >
                {t('history.import') || 'Import'}
              </button>
              <button
                onClick={handleClearAll}
                title={t('history.clearAll')}
                style={{
                  padding: '4px 8px',
                  backgroundColor: 'var(--color-bg-tertiary)',
                  color: 'var(--color-text)',
                  border: '1px solid var(--color-border)',
                  borderRadius: '3px',
                  cursor: 'pointer',
                  fontSize: '10px',
                }}
              >
                {t('history.clearAll') || 'Clear All'}
              </button>
            </div>
          </div>

          {/* Results info */}
          <div className="history-info">
            {filteredRecords.length === 0 ? (
              <p style={{ color: 'var(--color-text-secondary)', fontSize: '12px' }}>
                {records.length === 0
                  ? t('history.noExecutions') || 'No execution history yet'
                  : t('history.noResults') || 'No matching executions'}
              </p>
            ) : (
              <p style={{ color: 'var(--color-text-secondary)', fontSize: '11px' }}>
                {filteredRecords.length} of {records.length} execution(s)
              </p>
            )}
          </div>

          {/* Execution list */}
          <div className="history-list">
            {filteredRecords.map((record) => (
              <div
                key={record.id}
                className="history-item"
                onClick={() => {
                  setSelectedId(record.id);
                  setViewMode('details');
                  onSelectExecution?.(record);
                }}
                style={{
                  padding: '8px',
                  marginBottom: '4px',
                  backgroundColor: selectedId === record.id ? 'var(--color-bg-tertiary)' : 'transparent',
                  border: selectedId === record.id ? '1px solid var(--color-accent)' : '1px solid var(--color-border)',
                  borderRadius: '4px',
                  cursor: 'pointer',
                  transition: 'all 0.2s',
                }}
                onMouseOver={(e) => {
                  if (selectedId !== record.id) {
                    (e.currentTarget as HTMLDivElement).style.backgroundColor = 'var(--color-bg-secondary)';
                  }
                }}
                onMouseOut={(e) => {
                  if (selectedId !== record.id) {
                    (e.currentTarget as HTMLDivElement).style.backgroundColor = 'transparent';
                  }
                }}
              >
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '8px', flex: 1 }}>
                    <span
                      style={{
                        color: getStatusColor(record.status),
                        fontWeight: 'bold',
                        fontSize: '12px',
                      }}
                    >
                      {getStatusIcon(record.status)}
                    </span>
                    <div style={{ flex: 1 }}>
                      <div style={{ fontWeight: '500', fontSize: '11px', color: 'var(--color-text)' }}>
                        {record.scenarioName}
                      </div>
                      <div
                        style={{
                          fontSize: '10px',
                          color: 'var(--color-text-secondary)',
                          marginTop: '2px',
                        }}
                      >
                        {formatTime(record.timestamp)}
                      </div>
                    </div>
                  </div>
                  <button
                    onClick={(e) => handleDelete(record.id, e)}
                    style={{
                      background: 'none',
                      border: 'none',
                      color: 'var(--color-text-secondary)',
                      cursor: 'pointer',
                      fontSize: '14px',
                      padding: '0 4px',
                    }}
                    title={t('history.delete')}
                  >
                    ✕
                  </button>
                </div>

                {/* Execution stats */}
                <div style={{ marginTop: '6px', display: 'flex', gap: '12px', fontSize: '10px', color: 'var(--color-text-secondary)' }}>
                  <span>{record.completedSteps}/{record.totalSteps} steps</span>
                  <span>{formatDuration(record.duration)}</span>
                  <span>{record.logs.length} logs</span>
                </div>
              </div>
            ))}
          </div>
        </>
      ) : selectedRecord ? (
        <ExecutionDetailsView
          record={selectedRecord}
          onBack={() => {
            setViewMode('list');
            setSelectedId(null);
          }}
          onReplay={() => {
            setViewMode('replay');
          }}
        />
      ) : null}
    </div>
  );
};

interface ExecutionDetailsViewProps {
  record: ExecutionRecord;
  onBack: () => void;
  onReplay?: () => void;
}

const ExecutionDetailsView: React.FC<ExecutionDetailsViewProps> = ({ record, onBack, onReplay }) => {
  const { t } = useTranslation();
  const [logFilter, setLogFilter] = useState<'all' | 'info' | 'warn' | 'error'>('all');

  const filteredLogs = useMemo(() => {
    if (logFilter === 'all') return record.logs;
    return record.logs.filter((l) => l.level === logFilter);
  }, [record.logs, logFilter]);

  const formatDuration = (ms: number) => {
    const seconds = Math.floor(ms / 1000);
    const minutes = Math.floor(seconds / 60);
    if (minutes > 0) {
      return `${minutes}m ${seconds % 60}s`;
    }
    return `${seconds}s`;
  };

  const getLogLevelColor = (level: string) => {
    switch (level) {
      case 'info':
        return '#3b82f6';
      case 'warn':
        return '#f59e0b';
      case 'error':
        return '#ef4444';
      case 'debug':
        return '#8b5cf6';
      default:
        return '#6b7280';
    }
  };

  return (
    <div className="execution-details">
      {/* Header */}
      <div style={{ padding: '8px', borderBottom: '1px solid var(--color-border)' }}>
        <div style={{ display: 'flex', gap: '8px', marginBottom: '8px' }}>
          <button
            onClick={onBack}
            style={{
              background: 'none',
              border: 'none',
              color: 'var(--color-accent)',
              cursor: 'pointer',
              fontSize: '12px',
              padding: '0',
            }}
          >
            ← Back
          </button>
          {record.stepExecutions.length > 0 && onReplay && (
            <button
              onClick={onReplay}
              style={{
                background: 'none',
                border: 'none',
                color: 'var(--color-accent)',
                cursor: 'pointer',
                fontSize: '12px',
                marginLeft: 'auto',
                padding: '0 4px',
              }}
              title="Replay execution"
            >
              ▶ Replay
            </button>
          )}
        </div>
        <div style={{ fontWeight: '600', fontSize: '12px', color: 'var(--color-text)' }}>
          {record.scenarioName}
        </div>
        <div style={{ fontSize: '10px', color: 'var(--color-text-secondary)', marginTop: '2px' }}>
          {new Date(record.timestamp).toLocaleString()}
        </div>
      </div>

      {/* Stats */}
      <div style={{ padding: '8px', borderBottom: '1px solid var(--color-border)', fontSize: '11px' }}>
        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '8px' }}>
          <div>
            <span style={{ color: 'var(--color-text-secondary)' }}>Status: </span>
            <span style={{ fontWeight: '500', color: record.status === 'success' ? '#4ade80' : '#ef4444' }}>
              {record.status}
            </span>
          </div>
          <div>
            <span style={{ color: 'var(--color-text-secondary)' }}>Duration: </span>
            <span style={{ fontWeight: '500' }}>{formatDuration(record.duration)}</span>
          </div>
          <div>
            <span style={{ color: 'var(--color-text-secondary)' }}>Steps: </span>
            <span style={{ fontWeight: '500' }}>
              {record.completedSteps}/{record.totalSteps}
            </span>
          </div>
          <div>
            <span style={{ color: 'var(--color-text-secondary)' }}>Logs: </span>
            <span style={{ fontWeight: '500' }}>{record.logs.length}</span>
          </div>
        </div>
      </div>

      {/* Progress bar */}
      <div style={{ padding: '8px', borderBottom: '1px solid var(--color-border)' }}>
        <div style={{ fontSize: '10px', color: 'var(--color-text-secondary)', marginBottom: '4px' }}>
          Progress
        </div>
        <div
          style={{
            width: '100%',
            height: '4px',
            backgroundColor: 'var(--color-bg-tertiary)',
            borderRadius: '2px',
            overflow: 'hidden',
          }}
        >
          <div
            style={{
              width: `${(record.completedSteps / record.totalSteps) * 100}%`,
              height: '100%',
              backgroundColor: record.status === 'success' ? '#4ade80' : '#ef4444',
            }}
          />
        </div>
      </div>

      {/* Log filters */}
      <div style={{ padding: '8px', borderBottom: '1px solid var(--color-border)', display: 'flex', gap: '4px' }}>
        {(['all', 'info', 'warn', 'error'] as const).map((level) => (
          <button
            key={level}
            onClick={() => setLogFilter(level)}
            style={{
              padding: '4px 8px',
              border: logFilter === level ? '1px solid var(--color-accent)' : '1px solid var(--color-border)',
              backgroundColor: logFilter === level ? 'var(--color-bg-tertiary)' : 'transparent',
              color: logFilter === level ? 'var(--color-accent)' : 'var(--color-text-secondary)',
              borderRadius: '3px',
              cursor: 'pointer',
              fontSize: '10px',
              fontWeight: '500',
              textTransform: 'capitalize',
            }}
          >
            {level}
          </button>
        ))}
      </div>

      {/* Logs */}
      <div style={{ flex: 1, overflow: 'auto', padding: '8px' }}>
        {filteredLogs.length === 0 ? (
          <p style={{ color: 'var(--color-text-secondary)', fontSize: '11px' }}>No logs</p>
        ) : (
          filteredLogs.map((log, i) => (
            <div
              key={i}
              style={{
                padding: '4px 0',
                borderBottom: '1px solid var(--color-border)',
                fontSize: '10px',
                fontFamily: 'monospace',
              }}
            >
              <div style={{ display: 'flex', gap: '8px' }}>
                <span
                  style={{
                    color: getLogLevelColor(log.level),
                    fontWeight: '600',
                    minWidth: '40px',
                  }}
                >
                  {log.level.toUpperCase()}
                </span>
                <span style={{ color: 'var(--color-text-secondary)', minWidth: '100px' }}>
                  {new Date(log.timestamp).toLocaleTimeString()}
                </span>
                <span style={{ color: 'var(--color-text)', flex: 1 }}>{log.message}</span>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
};

export default HistoryPanel;
