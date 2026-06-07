import React, { useState, useEffect, useRef, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { useCanvasStore } from '../store/canvasStore';
import { useScenarioStore } from '../store/scenarioStore';
import { useCanvasSearch } from '../hooks/useCanvasSearch';
import './SearchCanvas.css';

interface SearchCanvasProps {
  onClose: () => void;
  isOpen: boolean;
}

const SearchCanvas: React.FC<SearchCanvasProps> = ({ onClose, isOpen }) => {
  const { t } = useTranslation();
  const [query, setQuery] = useState('');
  const [filterType, setFilterType] = useState<string | null>(null);
  const [results, setResults] = useState<string[]>([]);
  const inputRef = useRef<HTMLInputElement>(null);
  const { search, filterByType } = useCanvasSearch();
  const { scenario } = useScenarioStore();
  const { setFilterType: setStoreFilterType, setSearchHighlights } = useCanvasStore();

  // Get unique step types
  const stepTypes = Array.from(
    new Set(
      scenario.steps.reduce((types: string[], step) => {
        types.push(step.type);
        if (step.childSteps) {
          step.childSteps.forEach((child) => types.push(child.type));
        }
        return types;
      }, [])
    )
  ).sort();

  useEffect(() => {
    if (isOpen && inputRef.current) {
      inputRef.current.focus();
    }
  }, [isOpen]);

  const handleSearch = useCallback(
    (searchQuery: string) => {
      setQuery(searchQuery);
      if (searchQuery.trim()) {
        const matches = search(searchQuery);
        setResults(matches);
      } else {
        setResults([]);
        setSearchHighlights(new Set());
      }
    },
    [search, setSearchHighlights]
  );

  const handleFilterChange = useCallback(
    (type: string | null) => {
      setFilterType(type);
      setStoreFilterType(type);
    },
    [setStoreFilterType]
  );

  const handleClearAll = useCallback(() => {
    setQuery('');
    setFilterType(null);
    setResults([]);
    setSearchHighlights(new Set());
    setStoreFilterType(null);
  }, [setSearchHighlights, setStoreFilterType]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLInputElement>) => {
      if (e.key === 'Escape') {
        onClose();
      }
    },
    [onClose]
  );

  if (!isOpen) return null;

  return (
    <div className="search-canvas-overlay" onClick={onClose}>
      <div className="search-canvas-modal" onClick={(e) => e.stopPropagation()}>
        <div className="search-canvas-header">
          <h2>Search Canvas</h2>
          <button className="search-canvas-close" onClick={onClose}>×</button>
        </div>

        <div className="search-canvas-content">
          <div className="search-canvas-search-box">
            <input
              ref={inputRef}
              type="text"
              placeholder="Search by name, type, or data..."
              value={query}
              onChange={(e) => handleSearch(e.target.value)}
              onKeyDown={handleKeyDown}
              className="search-canvas-input"
            />
            <span className="search-canvas-count">
              {results.length > 0 ? `${results.length} match${results.length !== 1 ? 'es' : ''}` : ''}
            </span>
          </div>

          <div className="search-canvas-filters">
            <label className="search-canvas-filter-label">Filter by type:</label>
            <div className="search-canvas-filter-buttons">
              <button
                className={`search-canvas-filter-btn ${filterType === null ? 'active' : ''}`}
                onClick={() => handleFilterChange(null)}
              >
                All
              </button>
              {stepTypes.map((type) => (
                <button
                  key={type}
                  className={`search-canvas-filter-btn ${filterType === type ? 'active' : ''}`}
                  onClick={() => handleFilterChange(type)}
                  title={type}
                >
                  {type}
                </button>
              ))}
            </div>
          </div>

          <div className="search-canvas-actions">
            {(query || filterType) && (
              <button className="search-canvas-clear-btn" onClick={handleClearAll}>
                Clear All
              </button>
            )}
            <button className="search-canvas-close-btn" onClick={onClose}>
              Close
            </button>
          </div>

          {results.length === 0 && query && (
            <div className="search-canvas-empty">
              <p>No matches found for "{query}"</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default SearchCanvas;
