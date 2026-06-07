import React, { useState } from 'react';
import { useRunStore } from '../store/runStore';
import { useScenarioStore } from '../store/scenarioStore';
import './BreakpointManager.css';

const BreakpointManager: React.FC = () => {
  const { breakpoints, addBreakpoint, removeBreakpoint } = useRunStore();
  const { scenario } = useScenarioStore();
  const [expandedSteps, setExpandedSteps] = useState<Set<string>>(new Set());

  const handleToggleBreakpoint = (stepId: string) => {
    if (breakpoints.has(stepId)) {
      removeBreakpoint(stepId);
    } else {
      addBreakpoint(stepId);
    }
  };

  const toggleExpanded = (stepId: string) => {
    const newExpanded = new Set(expandedSteps);
    if (newExpanded.has(stepId)) {
      newExpanded.delete(stepId);
    } else {
      newExpanded.add(stepId);
    }
    setExpandedSteps(newExpanded);
  };

  const renderStep = (step: any, depth: number = 0) => {
    const hasChildren = step.childSteps && step.childSteps.length > 0;
    const isExpanded = expandedSteps.has(step.id);
    const hasBreakpoint = breakpoints.has(step.id);

    return (
      <div key={step.id} className="breakpoint-step" style={{ marginLeft: `${depth * 12}px` }}>
        <div className="breakpoint-step-header">
          {hasChildren && (
            <button
              onClick={() => toggleExpanded(step.id)}
              style={{
                background: 'none',
                border: 'none',
                cursor: 'pointer',
                padding: '0 2px',
                color: 'var(--color-text-secondary)',
                fontSize: '12px',
              }}
            >
              {isExpanded ? '▼' : '▶'}
            </button>
          )}
          {!hasChildren && <span style={{ width: '16px' }} />}

          <button
            onClick={() => handleToggleBreakpoint(step.id)}
            className={`breakpoint-dot ${hasBreakpoint ? 'active' : ''}`}
            title={hasBreakpoint ? 'Remove breakpoint' : 'Add breakpoint'}
          >
            ●
          </button>

          <span className="breakpoint-step-name">{step.name}</span>
          <span className="breakpoint-step-type">{step.type}</span>
        </div>

        {hasChildren && isExpanded && (
          <div className="breakpoint-children">
            {step.childSteps.map((child: any) => renderStep(child, depth + 1))}
          </div>
        )}
      </div>
    );
  };

  return (
    <div className="breakpoint-manager">
      <div className="breakpoint-header">
        <span>Breakpoints: {breakpoints.size}</span>
      </div>

      <div className="breakpoint-list">
        {scenario.steps.length === 0 ? (
          <p style={{ color: 'var(--color-text-secondary)', fontSize: '11px', padding: '8px' }}>
            No steps to set breakpoints on
          </p>
        ) : (
          scenario.steps.map((step) => renderStep(step))
        )}
      </div>
    </div>
  );
};

export default BreakpointManager;
