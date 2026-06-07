import React, { useState } from 'react';
import { useRunStore, WatchVariable } from '../store/runStore';
import './VariableInspector.css';

const VariableInspector: React.FC = () => {
  const { currentExecution, watchVariables, addWatchVariable, removeWatchVariable } = useRunStore();
  const [newVarName, setNewVarName] = useState('');
  const [expandedVars, setExpandedVars] = useState<Set<string>>(new Set());

  const handleAddWatch = () => {
    if (newVarName.trim()) {
      addWatchVariable(newVarName.trim());
      setNewVarName('');
    }
  };

  const toggleExpanded = (varName: string) => {
    const newExpanded = new Set(expandedVars);
    if (newExpanded.has(varName)) {
      newExpanded.delete(varName);
    } else {
      newExpanded.add(varName);
    }
    setExpandedVars(newExpanded);
  };

  const getTypeInfo = (value: any): { type: string; icon: string; color: string } => {
    if (value === null) {
      return { type: 'null', icon: '◯', color: '#6b7280' };
    }
    if (Array.isArray(value)) {
      return { type: 'array', icon: '[ ]', color: '#3b82f6' };
    }
    const type = typeof value;
    switch (type) {
      case 'string':
        return { type: 'string', icon: '"', color: '#10b981' };
      case 'number':
        return { type: 'number', icon: '#', color: '#f59e0b' };
      case 'boolean':
        return { type: 'boolean', icon: '?', color: '#8b5cf6' };
      case 'object':
        return { type: 'object', icon: '{}', color: '#06b6d4' };
      default:
        return { type, icon: '•', color: '#6b7280' };
    }
  };

  const formatValue = (value: any): string => {
    if (value === null) return 'null';
    if (value === undefined) return 'undefined';
    if (typeof value === 'string') return `"${value}"`;
    if (typeof value === 'boolean') return value ? 'true' : 'false';
    if (typeof value === 'number') return value.toString();
    if (Array.isArray(value)) return `[${value.length}]`;
    if (typeof value === 'object') return '{...}';
    return String(value);
  };

  const renderValue = (value: any, depth: number = 0): React.ReactNode => {
    if (value === null || value === undefined) {
      return <span style={{ color: '#6b7280' }}>{String(value)}</span>;
    }

    const type = typeof value;

    if (type === 'string') {
      return <span style={{ color: '#10b981' }}>"{value}"</span>;
    }

    if (type === 'number') {
      return <span style={{ color: '#f59e0b' }}>{value}</span>;
    }

    if (type === 'boolean') {
      return <span style={{ color: '#8b5cf6' }}>{value ? 'true' : 'false'}</span>;
    }

    if (Array.isArray(value)) {
      return (
        <div style={{ marginLeft: '12px' }}>
          <div style={{ color: '#3b82f6' }}>
            {'['}
            {value.length > 0 && (
              <>
                <br />
                {value.map((item, i) => (
                  <div key={i} style={{ paddingLeft: '12px' }}>
                    <span style={{ color: '#6b7280' }}>{i}: </span>
                    {renderValue(item, depth + 1)}
                    {i < value.length - 1 && <span style={{ color: '#6b7280' }}>{','}</span>}
                    <br />
                  </div>
                ))}
              </>
            )}
            {']'}
          </div>
        </div>
      );
    }

    if (type === 'object') {
      return (
        <div style={{ marginLeft: '12px' }}>
          <div style={{ color: '#06b6d4' }}>
            {'{'}
            {Object.keys(value).length > 0 && (
              <>
                <br />
                {Object.entries(value).map(([key, val], i, arr) => (
                  <div key={key} style={{ paddingLeft: '12px' }}>
                    <span style={{ color: '#8b5cf6' }}>{key}</span>
                    <span style={{ color: '#6b7280' }}>: </span>
                    {renderValue(val, depth + 1)}
                    {i < arr.length - 1 && <span style={{ color: '#6b7280' }}>{','}</span>}
                    <br />
                  </div>
                ))}
              </>
            )}
            {'}'}
          </div>
        </div>
      );
    }

    return <span style={{ color: '#6b7280' }}>{String(value)}</span>;
  };

  return (
    <div className="variable-inspector">
      <div className="variable-header">
        <span>Variables</span>
        <span style={{ fontSize: '10px', color: 'var(--color-text-secondary)' }}>
          {watchVariables.size}
        </span>
      </div>

      {/* Add watch variable form */}
      <div className="variable-add-watch">
        <input
          type="text"
          placeholder="Variable name..."
          value={newVarName}
          onChange={(e) => setNewVarName(e.target.value)}
          onKeyPress={(e) => {
            if (e.key === 'Enter') handleAddWatch();
          }}
          style={{
            flex: 1,
            padding: '4px 6px',
            border: '1px solid var(--color-border)',
            borderRadius: '3px',
            backgroundColor: 'var(--color-bg-secondary)',
            color: 'var(--color-text)',
            fontSize: '10px',
          }}
        />
        <button
          onClick={handleAddWatch}
          style={{
            padding: '4px 8px',
            backgroundColor: 'var(--color-accent)',
            color: '#fff',
            border: 'none',
            borderRadius: '3px',
            cursor: 'pointer',
            fontSize: '10px',
            fontWeight: '500',
          }}
        >
          +
        </button>
      </div>

      {/* Current execution variables */}
      <div className="variable-section">
        <div className="variable-section-title">Current Execution</div>
        <div className="variable-list">
          {currentExecution && Object.keys(currentExecution.variables).length > 0 ? (
            Object.entries(currentExecution.variables).map(([name, value]) => {
              const typeInfo = getTypeInfo(value);
              return (
                <div key={name} className="variable-item">
                  <div
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      gap: '4px',
                      fontSize: '10px',
                    }}
                  >
                    <span style={{ color: typeInfo.color, fontWeight: '600' }}>
                      {typeInfo.icon}
                    </span>
                    <span style={{ fontWeight: '500', color: 'var(--color-text)' }}>
                      {name}
                    </span>
                    <span style={{ color: 'var(--color-text-secondary)' }}>
                      {typeInfo.type}
                    </span>
                  </div>
                  <div
                    style={{
                      marginTop: '2px',
                      fontSize: '9px',
                      color: 'var(--color-text-secondary)',
                      fontFamily: 'monospace',
                    }}
                  >
                    {formatValue(value)}
                  </div>
                </div>
              );
            })
          ) : (
            <p style={{ color: 'var(--color-text-secondary)', fontSize: '10px' }}>
              No variables
            </p>
          )}
        </div>
      </div>

      {/* Watch variables */}
      <div className="variable-section">
        <div className="variable-section-title">Watch List</div>
        <div className="variable-list">
          {watchVariables.size === 0 ? (
            <p style={{ color: 'var(--color-text-secondary)', fontSize: '10px' }}>
              No watched variables
            </p>
          ) : (
            Array.from(watchVariables.values()).map((watch: WatchVariable) => {
              const typeInfo = getTypeInfo(watch.value);
              const isExpanded = expandedVars.has(watch.name);

              return (
                <div key={watch.name} className="variable-item">
                  <div style={{ display: 'flex', gap: '4px', alignItems: 'flex-start' }}>
                    {watch.history.length > 0 && (
                      <button
                        onClick={() => toggleExpanded(watch.name)}
                        style={{
                          background: 'none',
                          border: 'none',
                          cursor: 'pointer',
                          padding: '0 2px',
                          color: 'var(--color-text-secondary)',
                          fontSize: '10px',
                        }}
                      >
                        {isExpanded ? '▼' : '▶'}
                      </button>
                    )}
                    {watch.history.length === 0 && <span style={{ width: '12px' }} />}

                    <div style={{ flex: 1 }}>
                      <div
                        style={{
                          display: 'flex',
                          alignItems: 'center',
                          gap: '4px',
                          fontSize: '10px',
                        }}
                      >
                        <span style={{ color: typeInfo.color, fontWeight: '600' }}>
                          {typeInfo.icon}
                        </span>
                        <span style={{ fontWeight: '500', color: 'var(--color-text)' }}>
                          {watch.name}
                        </span>
                      </div>
                      <div
                        style={{
                          marginTop: '2px',
                          fontSize: '9px',
                          color: 'var(--color-text-secondary)',
                          fontFamily: 'monospace',
                        }}
                      >
                        {formatValue(watch.value)}
                      </div>

                      {/* History */}
                      {isExpanded && watch.history.length > 0 && (
                        <div style={{ marginTop: '4px', paddingLeft: '8px', borderLeft: '1px solid var(--color-border)' }}>
                          <div style={{ fontSize: '9px', color: 'var(--color-text-secondary)', marginBottom: '2px' }}>
                            History ({watch.history.length})
                          </div>
                          {watch.history.slice(-10).reverse().map((entry, i) => (
                            <div key={i} style={{ fontSize: '9px', padding: '2px 0', color: 'var(--color-text-secondary)' }}>
                              <span>{new Date(entry.timestamp).toLocaleTimeString()}: </span>
                              <span style={{ fontFamily: 'monospace', color: 'var(--color-text)' }}>
                                {formatValue(entry.value)}
                              </span>
                            </div>
                          ))}
                        </div>
                      )}
                    </div>

                    <button
                      onClick={() => removeWatchVariable(watch.name)}
                      style={{
                        background: 'none',
                        border: 'none',
                        color: 'var(--color-text-secondary)',
                        cursor: 'pointer',
                        fontSize: '12px',
                        padding: '0 2px',
                      }}
                      title="Remove from watch list"
                    >
                      ✕
                    </button>
                  </div>
                </div>
              );
            })
          )}
        </div>
      </div>
    </div>
  );
};

export default VariableInspector;
