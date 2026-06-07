import React, { useState, useEffect } from 'react';
import { ExecutionRecord, StepExecution } from '../store/runStore';
import './ExecutionReplay.css';

interface ExecutionReplayProps {
  record: ExecutionRecord;
  onClose: () => void;
}

type PlaybackSpeed = 0.5 | 1 | 2;

const ExecutionReplay: React.FC<ExecutionReplayProps> = ({ record, onClose }) => {
  const [isPlaying, setIsPlaying] = useState(false);
  const [currentStepIndex, setCurrentStepIndex] = useState(0);
  const [playbackSpeed, setPlaybackSpeed] = useState<PlaybackSpeed>(1);

  useEffect(() => {
    if (!isPlaying) return;

    const currentStep = record.stepExecutions[currentStepIndex];
    if (!currentStep) {
      setIsPlaying(false);
      return;
    }

    const duration = (currentStep.duration || 500) / playbackSpeed;
    const timer = setTimeout(() => {
      if (currentStepIndex < record.stepExecutions.length - 1) {
        setCurrentStepIndex(currentStepIndex + 1);
      } else {
        setIsPlaying(false);
      }
    }, duration);

    return () => clearTimeout(timer);
  }, [isPlaying, currentStepIndex, record.stepExecutions, playbackSpeed]);

  const currentStep = record.stepExecutions[currentStepIndex];

  const handlePlayPause = () => {
    setIsPlaying(!isPlaying);
  };

  const handleNext = () => {
    if (currentStepIndex < record.stepExecutions.length - 1) {
      setCurrentStepIndex(currentStepIndex + 1);
      setIsPlaying(false);
    }
  };

  const handlePrevious = () => {
    if (currentStepIndex > 0) {
      setCurrentStepIndex(currentStepIndex - 1);
      setIsPlaying(false);
    }
  };

  const handleSeek = (index: number) => {
    setCurrentStepIndex(Math.max(0, Math.min(index, record.stepExecutions.length - 1)));
    setIsPlaying(false);
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'completed':
        return '#4ade80';
      case 'failed':
        return '#ef4444';
      case 'skipped':
        return '#6b7280';
      default:
        return '#3b82f6';
    }
  };

  return (
    <div className="execution-replay">
      {/* Header */}
      <div className="replay-header">
        <h3 style={{ margin: '0 0 8px 0', fontSize: '12px' }}>Execution Replay</h3>
        <button
          onClick={onClose}
          style={{
            position: 'absolute',
            top: '8px',
            right: '8px',
            background: 'none',
            border: 'none',
            color: 'var(--color-text-secondary)',
            cursor: 'pointer',
            fontSize: '14px',
          }}
        >
          ✕
        </button>
      </div>

      {/* Current Step */}
      <div className="replay-current-step">
        {currentStep && (
          <>
            <div className="step-info">
              <span
                style={{
                  color: getStatusColor(currentStep.status),
                  fontWeight: '600',
                  fontSize: '12px',
                  marginRight: '8px',
                }}
              >
                {currentStep.status.toUpperCase()}
              </span>
              <span style={{ fontWeight: '500', fontSize: '11px', flex: 1 }}>
                {currentStep.stepName}
              </span>
              <span style={{ fontSize: '10px', color: 'var(--color-text-secondary)' }}>
                {currentStep.duration ? `${currentStep.duration}ms` : 'pending'}
              </span>
            </div>
            {currentStep.errorMessage && (
              <div style={{ fontSize: '10px', color: '#ef4444', marginTop: '4px' }}>
                Error: {currentStep.errorMessage}
              </div>
            )}
          </>
        )}
      </div>

      {/* Playback Controls */}
      <div className="replay-controls">
        <button
          onClick={handlePrevious}
          disabled={currentStepIndex === 0}
          style={{
            padding: '4px 8px',
            backgroundColor: 'var(--color-bg-tertiary)',
            color: currentStepIndex === 0 ? 'var(--color-text-secondary)' : 'var(--color-text)',
            border: '1px solid var(--color-border)',
            borderRadius: '3px',
            cursor: currentStepIndex === 0 ? 'not-allowed' : 'pointer',
            fontSize: '10px',
            opacity: currentStepIndex === 0 ? 0.5 : 1,
          }}
          title="Previous step"
        >
          ◀
        </button>

        <button
          onClick={handlePlayPause}
          style={{
            padding: '4px 12px',
            backgroundColor: isPlaying ? 'var(--color-accent)' : 'var(--color-bg-tertiary)',
            color: isPlaying ? '#fff' : 'var(--color-text)',
            border: 'none',
            borderRadius: '3px',
            cursor: 'pointer',
            fontSize: '10px',
            fontWeight: '500',
          }}
        >
          {isPlaying ? '⏸ Pause' : '▶ Play'}
        </button>

        <button
          onClick={handleNext}
          disabled={currentStepIndex === record.stepExecutions.length - 1}
          style={{
            padding: '4px 8px',
            backgroundColor: 'var(--color-bg-tertiary)',
            color: currentStepIndex === record.stepExecutions.length - 1 ? 'var(--color-text-secondary)' : 'var(--color-text)',
            border: '1px solid var(--color-border)',
            borderRadius: '3px',
            cursor: currentStepIndex === record.stepExecutions.length - 1 ? 'not-allowed' : 'pointer',
            fontSize: '10px',
            opacity: currentStepIndex === record.stepExecutions.length - 1 ? 0.5 : 1,
          }}
          title="Next step"
        >
          ▶
        </button>

        <div style={{ marginLeft: 'auto', display: 'flex', gap: '4px', alignItems: 'center' }}>
          <span style={{ fontSize: '10px', color: 'var(--color-text-secondary)' }}>Speed:</span>
          {([0.5, 1, 2] as PlaybackSpeed[]).map((speed) => (
            <button
              key={speed}
              onClick={() => setPlaybackSpeed(speed)}
              style={{
                padding: '2px 6px',
                backgroundColor: playbackSpeed === speed ? 'var(--color-accent)' : 'transparent',
                color: playbackSpeed === speed ? '#fff' : 'var(--color-text-secondary)',
                border: playbackSpeed === speed ? 'none' : '1px solid var(--color-border)',
                borderRadius: '3px',
                cursor: 'pointer',
                fontSize: '9px',
                fontWeight: '500',
              }}
            >
              {speed}x
            </button>
          ))}
        </div>
      </div>

      {/* Timeline */}
      <div className="replay-timeline">
        <div className="timeline-label">Timeline</div>
        <div
          style={{
            display: 'flex',
            gap: '2px',
            padding: '0 8px',
            overflow: 'auto',
          }}
        >
          {record.stepExecutions.map((step, index) => (
            <div
              key={index}
              onClick={() => handleSeek(index)}
              style={{
                flex: '0 0 24px',
                height: '24px',
                backgroundColor: getStatusColor(step.status),
                border: currentStepIndex === index ? '2px solid var(--color-accent)' : 'none',
                borderRadius: '3px',
                cursor: 'pointer',
                opacity: step.status === 'skipped' ? 0.5 : 1,
              }}
              title={`${index + 1}. ${step.stepName}`}
            />
          ))}
        </div>
      </div>

      {/* Step List */}
      <div className="replay-step-list">
        <div style={{ fontSize: '10px', fontWeight: '600', padding: '4px 8px', color: 'var(--color-text-secondary)' }}>
          Steps ({record.stepExecutions.length})
        </div>
        {record.stepExecutions.map((step, index) => (
          <div
            key={index}
            onClick={() => handleSeek(index)}
            style={{
              padding: '6px 8px',
              marginBottom: '2px',
              backgroundColor: currentStepIndex === index ? 'var(--color-bg-tertiary)' : 'transparent',
              border: currentStepIndex === index ? '1px solid var(--color-accent)' : '1px solid transparent',
              borderRadius: '3px',
              cursor: 'pointer',
              fontSize: '10px',
              transition: 'all 0.2s',
            }}
            onMouseOver={(e) => {
              if (currentStepIndex !== index) {
                (e.currentTarget as HTMLDivElement).style.backgroundColor = 'var(--color-bg-secondary)';
              }
            }}
            onMouseOut={(e) => {
              if (currentStepIndex !== index) {
                (e.currentTarget as HTMLDivElement).style.backgroundColor = 'transparent';
              }
            }}
          >
            <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
              <span
                style={{
                  color: getStatusColor(step.status),
                  fontWeight: '600',
                  minWidth: '60px',
                }}
              >
                {index + 1}. {step.status}
              </span>
              <span style={{ color: 'var(--color-text)', flex: 1 }}>{step.stepName}</span>
              <span style={{ color: 'var(--color-text-secondary)' }}>
                {step.duration}ms
              </span>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};

export default ExecutionReplay;
