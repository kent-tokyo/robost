import React from 'react';
import { useTranslation } from 'react-i18next';
import { useRunStore } from '../store/runStore';
import './ProgressPanel.css';

const ProgressPanel: React.FC = () => {
  const { t } = useTranslation();
  const { isRunning, currentStepIndex, totalSteps, elapsedMs, logs } = useRunStore();

  if (!isRunning && logs.length === 0) {
    return null;
  }

  return (
    <div className="progress-panel">
      <div className="progress-header">
        <span>📋 {t('progressPanel.executionLog')}</span>
        <span style={{ fontSize: '11px', color: 'var(--color-text-secondary)' }}>
          {logs.length} {t('progressPanel.entries')}
        </span>
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
          <span>{t('progressPanel.step')} {currentStepIndex + 1}/{totalSteps}</span>
          <span style={{ color: 'var(--color-text-secondary)' }}>
            {(elapsedMs / 1000).toFixed(1)}s
          </span>
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
