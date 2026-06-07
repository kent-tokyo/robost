import React from 'react';
import { useTranslation } from 'react-i18next';
import { useRunStore } from '../store/runStore';
import './ProgressPanel.css';

const ProgressPanel: React.FC = () => {
  const { t } = useTranslation();
  const { isRunning, isPaused, currentStepIndex, totalSteps, elapsedMs, logs, pauseRun, resumeRun } = useRunStore();

  if (!isRunning && logs.length === 0) {
    return null;
  }

  return (
    <div className="progress-panel">
      <div className="progress-header">
        <span>📋 {t('progressPanel.executionLog')}</span>
        <div style={{ display: 'flex', gap: '4px', alignItems: 'center' }}>
          <span style={{ fontSize: '11px', color: 'var(--color-text-secondary)' }}>
            {logs.length} {t('progressPanel.entries')}
          </span>
          {isRunning && (
            <>
              {!isPaused ? (
                <button
                  onClick={() => pauseRun()}
                  style={{
                    padding: '2px 6px',
                    backgroundColor: 'transparent',
                    color: 'var(--color-text-secondary)',
                    border: '1px solid var(--color-border)',
                    borderRadius: '3px',
                    cursor: 'pointer',
                    fontSize: '10px',
                    fontWeight: '500',
                  }}
                  title="Pause execution"
                >
                  ⏸
                </button>
              ) : (
                <button
                  onClick={() => resumeRun()}
                  style={{
                    padding: '2px 6px',
                    backgroundColor: 'var(--color-accent)',
                    color: '#fff',
                    border: 'none',
                    borderRadius: '3px',
                    cursor: 'pointer',
                    fontSize: '10px',
                    fontWeight: '500',
                  }}
                  title="Resume execution"
                >
                  ▶
                </button>
              )}
            </>
          )}
        </div>
      </div>

      {isRunning && (
        <div className="progress-status">
          <div className="progress-bar">
            <div
              className="progress-fill"
              style={{
                width: totalSteps > 0 ? `${(currentStepIndex / totalSteps) * 100}%` : '0%',
              }}
            />
          </div>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <span>{t('progressPanel.step')} {currentStepIndex + 1}/{totalSteps}</span>
            <span style={{ color: 'var(--color-text-secondary)' }}>
              {isPaused && <span style={{ color: '#f59e0b', fontWeight: '500' }}>PAUSED • </span>}
              {(elapsedMs / 1000).toFixed(1)}s
            </span>
          </div>
        </div>
      )}

      <div className="logs-container">
        {logs.map((log, i) => (
          <div
            key={i}
            className={`log-entry log-${log.level}`}
            title={new Date(log.timestamp).toLocaleTimeString()}
          >
            <span className="log-level">{log.level.toUpperCase()}</span>
            <span className="log-message">{log.message}</span>
          </div>
        ))}
      </div>
    </div>
  );
};

export default ProgressPanel;
