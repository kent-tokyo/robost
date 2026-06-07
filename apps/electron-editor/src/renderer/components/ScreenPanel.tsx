import React, { useState, useRef, useEffect, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { useScreenCapture } from '../hooks/useScreenCapture';
import { useRunStore } from '../store/runStore';
import {
  canvasToImageCoordinates,
  extractRGBAtCoordinate,
  formatRGBAsHex,
  formatRGBAsCSS,
  normalizeRegion,
  getRegionDimensions,
  Region,
  Coordinate,
} from '../utils/coordinatePicker';
import { RefreshIcon, AlertIcon, CameraIcon, CopyIcon, CloseIcon } from './Icons';
import './ScreenPanel.css';

const ScreenPanel: React.FC = () => {
  const { t } = useTranslation();
  const { screenshot, loading, error, autoRefreshInterval, captureScreen, startAutoRefresh, stopAutoRefresh } =
    useScreenCapture();
  const { addPickedCoordinate, pickedCoordinates, removePickedCoordinate, clearPickedCoordinates } = useRunStore();

  // State
  const [zoom, setZoom] = useState(100);
  const [isPanning, setIsPanning] = useState(false);
  const [panX, setPanX] = useState(0);
  const [panY, setPanY] = useState(0);
  const [currentCoord, setCurrentCoord] = useState<Coordinate | null>(null);
  const [currentColor, setCurrentColor] = useState<string | null>(null);
  const [isDrawing, setIsDrawing] = useState(false);
  const [selectedRegion, setSelectedRegion] = useState<Region | null>(null);
  const [regionStartCoord, setRegionStartCoord] = useState<Coordinate | null>(null);
  const [expandedSidebar, setExpandedSidebar] = useState<Set<string>>(new Set(['coordinates', 'tools']));

  // Refs
  const imageCanvasRef = useRef<HTMLCanvasElement>(null);
  const overlayCanvasRef = useRef<HTMLCanvasElement>(null);
  const previewContainerRef = useRef<HTMLDivElement>(null);
  const lastPanXRef = useRef(0);
  const lastPanYRef = useRef(0);
  const lastMouseXRef = useRef(0);
  const lastMouseYRef = useRef(0);

  // Load screenshot onto canvas when it changes
  useEffect(() => {
    if (!screenshot || !imageCanvasRef.current) return;

    const img = new Image();
    img.onload = () => {
      const canvas = imageCanvasRef.current;
      if (!canvas) return;

      canvas.width = img.width;
      canvas.height = img.height;

      const ctx = canvas.getContext('2d');
      if (ctx) {
        ctx.drawImage(img, 0, 0);
      }

      // Reset zoom and pan on new screenshot
      const zoomFitPercentage = Math.min(100, Math.max(25, (100 / (img.width + 50)) * window.innerWidth));
      setZoom(zoomFitPercentage);
      setPanX(0);
      setPanY(0);
    };

    img.src = screenshot.imageData;
  }, [screenshot]);

  // Update overlay canvas dimensions when image canvas changes
  useEffect(() => {
    if (imageCanvasRef.current && overlayCanvasRef.current) {
      overlayCanvasRef.current.width = imageCanvasRef.current.width;
      overlayCanvasRef.current.height = imageCanvasRef.current.height;
    }
  }, [screenshot]);

  // Draw region selector on overlay
  useEffect(() => {
    if (!overlayCanvasRef.current || !selectedRegion) return;

    const canvas = overlayCanvasRef.current;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    ctx.clearRect(0, 0, canvas.width, canvas.height);

    const normalized = normalizeRegion(selectedRegion);
    ctx.strokeStyle = '#007acc';
    ctx.lineWidth = 2;
    ctx.setLineDash([4, 4]);
    ctx.strokeRect(
      normalized.startX,
      normalized.startY,
      normalized.endX - normalized.startX,
      normalized.endY - normalized.startY
    );
    ctx.setLineDash([]);

    // Fill with semi-transparent color
    ctx.fillStyle = 'rgba(0, 122, 212, 0.05)';
    ctx.fillRect(
      normalized.startX,
      normalized.startY,
      normalized.endX - normalized.startX,
      normalized.endY - normalized.startY
    );
  }, [selectedRegion]);

  // Mouse down - start panning or region selection
  const handleMouseDown = (e: React.MouseEvent<HTMLCanvasElement>) => {
    if (e.button === 2) {
      // Right-click: pan
      setIsPanning(true);
      lastMouseXRef.current = e.clientX;
      lastMouseYRef.current = e.clientY;
      lastPanXRef.current = panX;
      lastPanYRef.current = panY;
    } else {
      // Left-click: region selection or coordinate pick
      const rect = imageCanvasRef.current?.getBoundingClientRect();
      if (rect) {
        const canvasX = e.clientX - rect.left;
        const canvasY = e.clientY - rect.top;
        const imageCoord = canvasToImageCoordinates(canvasX, canvasY, zoom / 100, panX, panY);

        setRegionStartCoord(imageCoord);
        setIsDrawing(true);
      }
    }
  };

  // Mouse move - update coordinates or draw region
  const handleMouseMove = (e: React.MouseEvent<HTMLCanvasElement>) => {
    if (!imageCanvasRef.current) return;

    const rect = imageCanvasRef.current.getBoundingClientRect();
    const canvasX = e.clientX - rect.left;
    const canvasY = e.clientY - rect.top;
    const imageCoord = canvasToImageCoordinates(canvasX, canvasY, zoom / 100, panX, panY);

    // Update current coordinate display
    setCurrentCoord(imageCoord);

    // Extract color at cursor
    const color = extractRGBAtCoordinate(imageCanvasRef.current, imageCoord.x, imageCoord.y);
    if (color) {
      setCurrentColor(formatRGBAsHex(color));
    }

    if (isPanning && e.buttons === 2) {
      const deltaX = e.clientX - lastMouseXRef.current;
      const deltaY = e.clientY - lastMouseYRef.current;
      setPanX(lastPanXRef.current + deltaX);
      setPanY(lastPanYRef.current + deltaY);
    }

    if (isDrawing && regionStartCoord) {
      setSelectedRegion({
        startX: regionStartCoord.x,
        startY: regionStartCoord.y,
        endX: imageCoord.x,
        endY: imageCoord.y,
      });
    }
  };

  // Mouse up - finalize region or click coordinate
  const handleMouseUp = (e: React.MouseEvent<HTMLCanvasElement>) => {
    setIsPanning(false);

    if (isDrawing) {
      setIsDrawing(false);
      return;
    }

    if (!imageCanvasRef.current || !currentCoord) return;

    // On click without dragging, pick coordinate
    if (!selectedRegion || (selectedRegion && Math.abs(selectedRegion.startX - selectedRegion.endX) < 5)) {
      const id = `coord-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
      addPickedCoordinate({
        id,
        x: currentCoord.x,
        y: currentCoord.y,
        color: currentColor || undefined,
        timestamp: Date.now(),
      });
    }
  };

  // Context menu - prevent default
  const handleContextMenu = (e: React.MouseEvent) => {
    e.preventDefault();
  };

  const zoomLevels = [25, 50, 100, 200];

  const toggleSidebar = (section: string) => {
    setExpandedSidebar((prev) => {
      const newSet = new Set(prev);
      if (newSet.has(section)) {
        newSet.delete(section);
      } else {
        newSet.add(section);
      }
      return newSet;
    });
  };

  const handleCopyCoordinate = (coord: { x: number; y: number }, format: 'csv' | 'json') => {
    const text = format === 'csv' ? `${coord.x},${coord.y}` : JSON.stringify(coord);
    navigator.clipboard.writeText(text);
  };

  const handleAutoRefreshChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = parseInt(e.target.value, 10);
    if (isNaN(value) || value <= 0) {
      stopAutoRefresh();
    } else {
      startAutoRefresh(value * 1000);
    }
  };

  const handleZoomChange = (newZoom: number) => {
    setZoom(newZoom);
  };

  return (
    <div className="screen-panel">
      {/* Header */}
      <div className="screen-panel-header">
        <div className="screen-panel-controls">
          <button onClick={captureScreen} disabled={loading} title={t('common.refresh')}>
            {loading ? <span className="loading-spinner"></span> : <RefreshIcon size={14} />} {t('common.refresh')}
          </button>

          {/* Auto-refresh toggle */}
          <div className="auto-refresh-toggle">
            <label>
              <input
                type="checkbox"
                checked={autoRefreshInterval !== null}
                onChange={(e) => {
                  if (e.target.checked) {
                    startAutoRefresh(2000);
                  } else {
                    stopAutoRefresh();
                  }
                }}
              />
              {t('editor.autoRefresh')}
            </label>
            {autoRefreshInterval !== null && (
              <input
                type="number"
                min="1"
                max="30"
                step="1"
                defaultValue={Math.round(autoRefreshInterval / 1000)}
                onChange={handleAutoRefreshChange}
                className="auto-refresh-input"
                title="Seconds"
              />
            )}
          </div>
        </div>

        {/* Zoom controls */}
        <div className="screen-panel-zoom-controls">
          <span style={{ fontSize: '12px', opacity: 0.7 }}>{zoom}%</span>
          {zoomLevels.map((z) => (
            <button key={z} onClick={() => handleZoomChange(z)} style={{ fontWeight: zoom === z ? 'bold' : 'normal' }}>
              {z}%
            </button>
          ))}
        </div>
      </div>

      {/* Error message */}
      {error && <div className="screen-error"><AlertIcon size={14} /> {error}</div>}

      {/* Content */}
      <div className="screen-panel-content">
        {/* Preview */}
        {screenshot ? (
          <div className="screen-preview-container">
            <div
              className={`screen-preview ${loading ? 'loading' : ''}`}
              ref={previewContainerRef}
              style={{
                transform: `translate(${panX}px, ${panY}px)`,
              }}
            >
              <div
                style={{
                  transform: `scale(${zoom / 100})`,
                  transformOrigin: 'top left',
                  position: 'relative',
                }}
              >
                <canvas
                  ref={imageCanvasRef}
                  onMouseDown={handleMouseDown}
                  onMouseMove={handleMouseMove}
                  onMouseUp={handleMouseUp}
                  onMouseLeave={() => setCurrentCoord(null)}
                  onContextMenu={handleContextMenu}
                />
                <div className="screen-preview-overlay" style={{ position: 'absolute' }}>
                  <canvas ref={overlayCanvasRef} />
                </div>
              </div>

              {/* Coordinate display */}
              {currentCoord && (
                <div className="coordinate-display">
                  <div className="coordinate-display-item">
                    <span className="coordinate-display-label">X:</span>
                    <span className="coordinate-display-value">{currentCoord.x}</span>
                  </div>
                  <div className="coordinate-display-item">
                    <span className="coordinate-display-label">Y:</span>
                    <span className="coordinate-display-value">{currentCoord.y}</span>
                  </div>
                  {currentColor && (
                    <div className="coordinate-display-item">
                      <span className="coordinate-display-label">RGB:</span>
                      <span className="coordinate-display-value" style={{ fontSize: '10px' }}>
                        {currentColor}
                      </span>
                    </div>
                  )}
                </div>
              )}
            </div>
          </div>
        ) : (
          <div className="screen-preview-container">
            <div className="screen-empty-state">
              <div className="screen-empty-state-icon"><CameraIcon size={32} /></div>
              <div className="screen-empty-state-text">{t('editor.screenCaptureHint')}</div>
              <button onClick={captureScreen}>{t('editor.captureNow')}</button>
            </div>
          </div>
        )}

        {/* Sidebar */}
        <div className="screen-sidebar">
          {/* Tools Section */}
          <div className="screen-sidebar-section">
            <button
              className="screen-sidebar-section-header"
              onClick={() => toggleSidebar('tools')}
            >
              <span className={`toggle-icon ${expandedSidebar.has('tools') ? 'expanded' : ''}`}>▼</span>
              <span>{t('editor.tools')}</span>
            </button>
            {expandedSidebar.has('tools') && (
              <div className="screen-sidebar-section-content">
                <button onClick={() => setSelectedRegion(null)} style={{ width: '100%' }}>
                  {t('editor.clearRegion')}
                </button>
                <button onClick={clearPickedCoordinates} style={{ width: '100%' }}>
                  {t('editor.clearCoordinates')}
                </button>
                {selectedRegion && (
                  <div className="region-selector-info">
                    <div className="region-selector-info-item">
                      <span className="region-selector-info-label">{t('editor.region')}:</span>
                      <span className="region-selector-info-value">
                        {selectedRegion.startX}, {selectedRegion.startY}
                      </span>
                    </div>
                    <div className="region-selector-info-item">
                      <span className="region-selector-info-label">{t('editor.size')}:</span>
                      <span className="region-selector-info-value">
                        {getRegionDimensions(selectedRegion).width}x{getRegionDimensions(selectedRegion).height}
                      </span>
                    </div>
                  </div>
                )}
              </div>
            )}
          </div>

          {/* Coordinates Section */}
          <div className="screen-sidebar-section">
            <button
              className="screen-sidebar-section-header"
              onClick={() => toggleSidebar('coordinates')}
            >
              <span className={`toggle-icon ${expandedSidebar.has('coordinates') ? 'expanded' : ''}`}>▼</span>
              <span>{t('editor.coordinates')} ({pickedCoordinates.length})</span>
            </button>
            {expandedSidebar.has('coordinates') && (
              <div className="screen-sidebar-section-content">
                {pickedCoordinates.length === 0 ? (
                  <div style={{ textAlign: 'center', opacity: 0.7, fontSize: '12px' }}>
                    {t('editor.noCoordinates')}
                  </div>
                ) : (
                  pickedCoordinates.slice(0, 5).map((coord) => (
                    <div key={coord.id} className="coordinate-history-item">
                      {coord.color && (
                        <div
                          className="coordinate-history-item-color"
                          style={{ backgroundColor: coord.color }}
                        />
                      )}
                      <div className="coordinate-history-item-value">
                        {coord.x}, {coord.y}
                      </div>
                      <div className="coordinate-history-item-actions">
                        <button
                          onClick={() => handleCopyCoordinate(coord, 'csv')}
                          title={t('editor.copyAsCSV')}
                        >
                          <CopyIcon size={12} />
                        </button>
                        <button
                          onClick={() => handleCopyCoordinate(coord, 'json')}
                          title={t('editor.copyAsJSON')}
                        >
                          {'{}'}
                        </button>
                        <button
                          onClick={() => removePickedCoordinate(coord.id)}
                          title={t('common.delete')}
                        >
                          <CloseIcon size={12} />
                        </button>
                      </div>
                    </div>
                  ))
                )}
              </div>
            )}
          </div>

          {/* Selection Info Section */}
          {selectedRegion && (
            <div className="screen-sidebar-section">
              <button
                className="screen-sidebar-section-header"
                onClick={() => toggleSidebar('selection')}
              >
                <span className={`toggle-icon ${expandedSidebar.has('selection') ? 'expanded' : ''}`}>▼</span>
                <span>{t('editor.selection')}</span>
              </button>
              {expandedSidebar.has('selection') && (
                <div className="screen-sidebar-section-content">
                  <div className="region-selector-info">
                    <div className="region-selector-info-item">
                      <span className="region-selector-info-label">Start:</span>
                      <span className="region-selector-info-value">
                        ({selectedRegion.startX}, {selectedRegion.startY})
                      </span>
                    </div>
                    <div className="region-selector-info-item">
                      <span className="region-selector-info-label">End:</span>
                      <span className="region-selector-info-value">
                        ({selectedRegion.endX}, {selectedRegion.endY})
                      </span>
                    </div>
                    <div className="region-selector-info-item">
                      <span className="region-selector-info-label">Width:</span>
                      <span className="region-selector-info-value">
                        {getRegionDimensions(selectedRegion).width}px
                      </span>
                    </div>
                    <div className="region-selector-info-item">
                      <span className="region-selector-info-label">Height:</span>
                      <span className="region-selector-info-value">
                        {getRegionDimensions(selectedRegion).height}px
                      </span>
                    </div>
                  </div>
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default ScreenPanel;
