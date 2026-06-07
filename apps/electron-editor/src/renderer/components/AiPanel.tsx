import React, { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { useAiAssistant, AiSuggestion } from '../hooks/useAiAssistant';
import { useScenarioStore } from '../store/scenarioStore';
import * as YAML from 'js-yaml';
import './AiPanel.css';

interface AiHistory {
  query: string;
  suggestions: AiSuggestion[];
  timestamp: number;
}

const AiPanel: React.FC = () => {
  const { t } = useTranslation();
  const { generateSteps, suggestionToStep, loading, error, suggestions, clear } =
    useAiAssistant();
  const { addStep } = useScenarioStore();

  const [query, setQuery] = useState('');
  const [history, setHistory] = useState<AiHistory[]>([]);
  const [selectedSuggestions, setSelectedSuggestions] = useState<Set<number>>(
    new Set()
  );
  const [copiedIndex, setCopiedIndex] = useState<number | null>(null);

  // Load history from localStorage
  useEffect(() => {
    const savedHistory = localStorage.getItem('ai-suggestions-history');
    if (savedHistory) {
      try {
        const parsed = JSON.parse(savedHistory);
        setHistory(Array.isArray(parsed) ? parsed.slice(0, 10) : []);
      } catch (err) {
        console.error('Failed to load AI history:', err);
      }
    }
  }, []);

  // Save history to localStorage whenever it changes
  useEffect(() => {
    localStorage.setItem('ai-suggestions-history', JSON.stringify(history.slice(0, 10)));
  }, [history]);

  const handleGenerateSteps = async () => {
    try {
      const generated = await generateSteps(query);

      // Add to history
      setHistory((prev) => [
        { query, suggestions: generated, timestamp: Date.now() },
        ...prev,
      ]);

      setSelectedSuggestions(new Set());
    } catch (err) {
      console.error('Error generating steps:', err);
    }
  };

  const handleAddSelected = () => {
    const toAdd = Array.from(selectedSuggestions)
      .sort((a, b) => a - b)
      .map((idx) => suggestions[idx]);

    toAdd.forEach((suggestion) => {
      const step = suggestionToStep(suggestion);
      addStep(step);
    });

    setSelectedSuggestions(new Set());
  };

  const handleAddAll = () => {
    suggestions.forEach((suggestion) => {
      const step = suggestionToStep(suggestion);
      addStep(step);
    });

    setSelectedSuggestions(new Set());
  };

  const handleCopyYaml = (suggestion: AiSuggestion) => {
    const yaml = YAML.dump(
      {
        name: suggestion.name,
        type: suggestion.type,
        data: suggestion.data,
      },
      { lineWidth: 0 }
    );

    navigator.clipboard.writeText(yaml).then(() => {
      setCopiedIndex(suggestions.indexOf(suggestion));
      setTimeout(() => setCopiedIndex(null), 2000);
    });
  };

  const toggleSelection = (index: number) => {
    const newSelection = new Set(selectedSuggestions);
    if (newSelection.has(index)) {
      newSelection.delete(index);
    } else {
      newSelection.add(index);
    }
    setSelectedSuggestions(newSelection);
  };

  return (
    <div className="ai-panel">
      {/* Input Section */}
      <div className="ai-panel-section">
        <div className="ai-input-group">
          <textarea
            className="ai-input"
            placeholder={t('ai.descriptionPlaceholder') || 'Describe what you want to do...'}
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            disabled={loading}
            onKeyDown={(e) => {
              if (e.key === 'Enter' && (e.ctrlKey || e.metaKey) && query.trim()) {
                handleGenerateSteps();
              }
            }}
          />
          <button
            className="ai-generate-button"
            onClick={handleGenerateSteps}
            disabled={loading || !query.trim()}
          >
            {loading ? (
              <>
                <span className="spinner"></span> {t('common.loading') || 'Generating...'}
              </>
            ) : (
              <>✨ {t('ai.generateSteps') || 'Generate Steps'}</>
            )}
          </button>
        </div>
      </div>

      {/* Error Message */}
      {error && (
        <div className="ai-error-message">
          <span>{error}</span>
          <button className="close-error" onClick={() => clear()}>
            ×
          </button>
        </div>
      )}

      {/* Suggestions Section */}
      {suggestions.length > 0 && (
        <div className="ai-panel-section">
          <div className="ai-suggestions-header">
            <h3>
              {t('ai.suggestions') || 'Suggestions'} ({suggestions.length})
            </h3>
            <div className="ai-actions">
              <button
                className="ai-button secondary"
                onClick={handleAddSelected}
                disabled={selectedSuggestions.size === 0}
                title={
                  selectedSuggestions.size === 0
                    ? 'Select suggestions to add'
                    : 'Add selected suggestions'
                }
              >
                + {t('ai.addSelected') || 'Add Selected'} ({selectedSuggestions.size})
              </button>
              <button
                className="ai-button primary"
                onClick={handleAddAll}
              >
                + {t('ai.addAll') || 'Add All'}
              </button>
            </div>
          </div>

          <div className="ai-suggestions-list">
            {suggestions.map((suggestion, index) => (
              <div key={index} className="ai-suggestion-item">
                <input
                  type="checkbox"
                  checked={selectedSuggestions.has(index)}
                  onChange={() => toggleSelection(index)}
                  className="ai-checkbox"
                />
                <div className="ai-suggestion-content">
                  <div className="ai-suggestion-header">
                    <strong>{suggestion.name}</strong>
                    <span className="ai-suggestion-type">{suggestion.type}</span>
                  </div>
                  {suggestion.description && (
                    <div className="ai-suggestion-description">
                      {suggestion.description}
                    </div>
                  )}
                  {Object.keys(suggestion.data).length > 0 && (
                    <div className="ai-suggestion-data">
                      <small className="ai-data-label">{t('ai.data') || 'Data'}:</small>
                      <pre className="ai-data-preview">
                        {JSON.stringify(suggestion.data, null, 2).slice(0, 150)}
                        {JSON.stringify(suggestion.data, null, 2).length > 150 ? '...' : ''}
                      </pre>
                    </div>
                  )}
                </div>
                <button
                  className="ai-copy-button"
                  onClick={() => handleCopyYaml(suggestion)}
                  title={t('ai.copyYaml') || 'Copy YAML'}
                >
                  {copiedIndex === index ? '✓' : '📋'}
                </button>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* History Section */}
      {history.length > 0 && suggestions.length === 0 && (
        <div className="ai-panel-section">
          <h3>{t('ai.history') || 'Recent Suggestions'}</h3>
          <div className="ai-history-list">
            {history.slice(0, 5).map((item, idx) => (
              <div
                key={idx}
                className="ai-history-item"
                onClick={() => {
                  setQuery(item.query);
                }}
                title={item.query}
              >
                <div className="ai-history-query">{item.query.slice(0, 40)}...</div>
                <div className="ai-history-count">
                  {item.suggestions.length} {t('ai.steps') || 'steps'}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Empty State */}
      {!loading && suggestions.length === 0 && history.length === 0 && (
        <div className="ai-empty-state">
          <div className="ai-empty-icon">✨</div>
          <p>{t('ai.emptyState') || 'Describe what you want to automate'}</p>
          <small>{t('ai.emptyStateHint') || 'AI will suggest steps to add'}</small>
        </div>
      )}
    </div>
  );
};

export default AiPanel;
