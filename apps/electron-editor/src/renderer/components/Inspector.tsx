import React, { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useScenarioStore } from '../store/scenarioStore';
import { useEditorStore } from '../store/editorStore';
import { getStepSchema } from '../types/stepSchema';
import './Inspector.css';

interface InspectorProps {
  selectedNodeId: string | null;
}

const Inspector: React.FC<InspectorProps> = ({ selectedNodeId }) => {
  const { t } = useTranslation();
  const { scenario, updateStep } = useScenarioStore();
  const { saveSnapshot } = useEditorStore();
  const [expandedSections, setExpandedSections] = useState<Set<string>>(
    new Set(['general', 'properties'])
  );

  if (!selectedNodeId) {
    return (
      <div className="inspector empty">
        <div style={{ padding: 'var(--spacing-lg)', textAlign: 'center', color: 'var(--color-text-secondary)' }}>
          <p>{t('inspector.selectStepToEdit')}</p>
        </div>
      </div>
    );
  }

  const step = scenario.steps.find((s) => s.id === selectedNodeId);
  if (!step) {
    return (
      <div className="inspector empty">
        <p>{t('inspector.stepNotFound')}</p>
      </div>
    );
  }

  const schema = getStepSchema(step.type);

  const toggleSection = (section: string) => {
    setExpandedSections((prev) => {
      const newSet = new Set(prev);
      if (newSet.has(section)) {
        newSet.delete(section);
      } else {
        newSet.add(section);
      }
      return newSet;
    });
  };

  const handleFieldChange = (fieldName: string, value: any) => {
    updateStep(step.id, {
      ...step,
      data: {
        ...step.data,
        [fieldName]: value,
      },
    });
    saveSnapshot(`Update ${fieldName}`);
  };

  const handleNameChange = (value: string) => {
    updateStep(step.id, { name: value });
    saveSnapshot('Rename step');
  };

  const handleEnabledChange = (value: boolean) => {
    updateStep(step.id, { enabled: value });
    saveSnapshot(value ? 'Enable step' : 'Disable step');
  };

  return (
    <div className="inspector">
      {/* General Section */}
      <div className="inspector-section">
        <button
          className="inspector-section-header"
          onClick={() => toggleSection('general')}
        >
          <span className="toggle-icon">{expandedSections.has('general') ? '▼' : '▶'}</span>
          <span>{t('inspector.general')}</span>
        </button>

        {expandedSections.has('general') && (
          <div className="inspector-section-content">
            <div className="form-group">
              <label>{t('inspector.name')}</label>
              <input
                type="text"
                value={step.name}
                onChange={(e) => handleNameChange(e.target.value)}
                placeholder={t('inspector.stepName')}
              />
            </div>

            <div className="form-group">
              <label>{t('inspector.type')}</label>
              <div className="readonly-field">{step.type}</div>
            </div>

            <div className="form-group">
              <label>
                <input
                  type="checkbox"
                  checked={step.enabled !== false}
                  onChange={(e) => handleEnabledChange(e.target.checked)}
                />
                {t('inspector.enabled')}
              </label>
            </div>
          </div>
        )}
      </div>

      {/* Properties Section */}
      {schema && (
        <div className="inspector-section">
          <button
            className="inspector-section-header"
            onClick={() => toggleSection('properties')}
          >
            <span className="toggle-icon">{expandedSections.has('properties') ? '▼' : '▶'}</span>
            <span>{t('inspector.properties')}</span>
          </button>

          {expandedSections.has('properties') && (
            <div className="inspector-section-content">
              {schema.fields.map((field) => (
                <div key={field.name} className="form-group">
                  <label>
                    {field.label}
                    {field.required && <span className="required">*</span>}
                  </label>

                  {field.type === 'text' && (
                    <input
                      type="text"
                      value={step.data[field.name] || ''}
                      onChange={(e) => handleFieldChange(field.name, e.target.value)}
                      placeholder={field.placeholder}
                    />
                  )}

                  {field.type === 'number' && (
                    <input
                      type="number"
                      value={step.data[field.name] ?? ''}
                      onChange={(e) =>
                        handleFieldChange(field.name, e.target.value ? Number(e.target.value) : '')
                      }
                      min={field.min}
                      max={field.max}
                    />
                  )}

                  {field.type === 'boolean' && (
                    <label className="checkbox-label">
                      <input
                        type="checkbox"
                        checked={step.data[field.name] || false}
                        onChange={(e) => handleFieldChange(field.name, e.target.checked)}
                      />
                      {field.label}
                    </label>
                  )}

                  {field.type === 'select' && field.options && (
                    <select
                      value={step.data[field.name] || ''}
                      onChange={(e) => handleFieldChange(field.name, e.target.value)}
                    >
                      <option value="">Select {field.label}</option>
                      {field.options.map((opt) => (
                        <option key={opt.value} value={opt.value}>
                          {opt.label}
                        </option>
                      ))}
                    </select>
                  )}

                  {field.type === 'array' && (
                    <textarea
                      value={
                        Array.isArray(step.data[field.name])
                          ? step.data[field.name].join('\n')
                          : ''
                      }
                      onChange={(e) =>
                        handleFieldChange(field.name, e.target.value.split('\n').filter(Boolean))
                      }
                      placeholder={`Enter items (one per line)`}
                      rows={4}
                    />
                  )}
                </div>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
};

export default Inspector;
