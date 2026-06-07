# Task #18: Screen Operation Panel Implementation Guide

## Overview
This document describes the complete implementation of the Screen Operation Panel for Robost Editor, enabling real-time screen preview, coordinate picking, and region selection for RPA automation.

## Files Created

### 1. Component Files
- **`src/renderer/components/ScreenPanel.tsx`** - Main Screen Panel component
  - Screen preview with real-time screenshot display
  - Zoom controls (25%, 50%, 100%, 200%)
  - Coordinate picker with RGB color extraction
  - Region selector (rectangle drawing)
  - Coordinate history sidebar (last 5 coordinates)
  - Copy-to-clipboard functionality

- **`src/renderer/components/ScreenPanel.css`** - Styling for ScreenPanel
  - VS Code-inspired dark and light theme support
  - Responsive grid layout with sidebar
  - Canvas overlay for region selection
  - Coordinate display tooltip
  - Collapsible sidebar sections

### 2. Hook Files
- **`src/renderer/hooks/useScreenCapture.ts`** - Screen capture logic
  - Fetches screenshots from RPA server via Electron API
  - Handles auto-refresh with configurable intervals (1-30 seconds)
  - Loading and error states
  - Base64 image encoding

### 3. Utility Files
- **`src/renderer/utils/coordinatePicker.ts`** - Geometry utilities
  - RGB color extraction at coordinates
  - Region normalization and dimension calculation
  - Canvas-to-image coordinate transformation
  - Region canvas creation
  - Coordinate formatting (CSV, JSON, hex color)

### 4. Store Updates
- **`src/renderer/store/runStore.ts`** - Extended with screen operations
  - New `PickedCoordinate` interface
  - `pickedCoordinates` state array (keeps last 50)
  - Actions: `addPickedCoordinate`, `clearPickedCoordinates`, `removePickedCoordinate`

### 5. Locale Files
Updated with new translations:
- **`src/renderer/locales/en.json`** - English
- **`src/renderer/locales/ja.json`** - Japanese
- **`src/renderer/locales/zh.json`** - Chinese (Simplified)

New keys:
```
editor:
  screen: "Screen"
  autoRefresh: "Auto-refresh"
  screenCaptureHint: "Click 'Refresh' to capture..."
  captureNow: "Capture Now"
  tools: "Tools"
  clearRegion: "Clear Region"
  clearCoordinates: "Clear History"
  region: "Region"
  size: "Size"
  coordinates: "Coordinates"
  noCoordinates: "No coordinates picked yet"
  copyAsCSV: "Copy as x,y"
  copyAsJSON: "Copy as JSON"
  selection: "Selection"

common:
  refresh: "Refresh"
```

## Files Modified

### 1. Editor Component
**`src/renderer/components/Editor.tsx`**
- Added 'screen' to `ViewMode` type
- Added Screen tab to editor tabs: `['canvas', 'list', 'code', 'screen']`
- Imported `ScreenPanel` component
- Added screen view rendering in content area

### 2. RPA Store
**`src/renderer/store/runStore.ts`**
- Added `PickedCoordinate` interface with: id, x, y, color, timestamp
- Added `pickedCoordinates` state field
- Added three new actions for coordinate management

### 3. RPA Server Hook
**`src/renderer/hooks/useRpaServer.ts`**
- Updated window.electronAPI type declaration with `rpaScreenshot` method
- Fixed stopRun calls to include required status parameter ('success', 'failed', 'stopped')

### 4. Main Process IPC
**`src/main/index.ts`**
- Added `rpa:screenshot` IPC handler
- Fetches PNG image from RPA server's `/screenshot` endpoint
- Returns base64-encoded image data
- Includes 5-second timeout and error handling

### 5. Preload Script
**`src/main/preload.ts`**
- Exposed `rpaScreenshot` method to renderer via Electron Context Bridge
- Allows secure IPC communication for screenshot fetching

## Key Features

### 1. Screen Preview
- Real-time screenshot display with canvas rendering
- Zoom levels: 25%, 50%, 100%, 200%
- Pan support: Right-click drag to pan
- Auto-fit zoom on new screenshot
- Loading indicator during capture

### 2. Coordinate Picker
- **Click to pick**: Left-click on image to select coordinates
- **RGB color display**: Shows hex color at cursor position
- **Real-time display**: Coordinates shown in top-right corner
- **History sidebar**: Last 5 picked coordinates
- **Copy options**: Copy as CSV (x,y) or JSON ({x,y})
- **Metadata**: Timestamp for each pick

### 3. Region Selector
- **Draw rectangle**: Drag on image to create region
- **Dimension display**: Shows width x height in pixels
- **Normalized coordinates**: Auto-calculates min/max corners
- **Visual feedback**: Dashed blue outline with semi-transparent fill
- **Selection info panel**: Detailed region coordinates and dimensions

### 4. Auto-Refresh
- Toggle auto-refresh with checkbox
- Configurable interval: 1-30 seconds (default: 2s)
- Captures continuously while enabled
- Maintains zoom and pan state

### 5. Sidebar Management
- Three collapsible sections: Tools, Coordinates, Selection
- Compact sidebar (240-300px width)
- Smooth section transitions
- Empty states for no data

## Architecture

### Data Flow
```
RPA Server (HTTP)
    ↓
Electron Main Process (IPC handler)
    ↓
Renderer Process (useScreenCapture hook)
    ↓
ScreenPanel Component
    ↓
Canvas API (rendering + interaction)
```

### State Management
- **useScreenCapture**: Local screenshot state
- **useRunStore**: Picked coordinates history + server port
- **ScreenPanel local state**: Zoom, pan, region, current coord, sidebar expansion

### Event Handling
- **Mouse down**: Start pan (right-click) or region selection (left-click)
- **Mouse move**: Update coordinate display, draw region
- **Mouse up**: Finalize region or add coordinate to history
- **Context menu**: Prevent default, enable custom pan

## Usage

### Basic Workflow
1. Start RPA scenario (creates HTTP server)
2. Navigate to Screen tab in Editor
3. Click "Capture Now" to get first screenshot
4. Enable "Auto-refresh" for continuous updates
5. Click on image to pick coordinates
6. View history in sidebar, copy as needed
7. Drag to select regions for reference

### Integration with Steps
Currently, ScreenPanel stores coordinates in `useRunStore` for future integration:
- `addPickedCoordinate`: Add to history
- `pickedCoordinates`: Access history in Inspector/Steps
- Future: Drag-and-drop to fill click_image coordinates

## Server Endpoint Requirements

The RPA server must provide:
```
GET http://127.0.0.1:{PORT}/screenshot
  Returns: PNG image file
  Content-Type: image/png
```

## Performance Considerations

- Canvas rendering optimized with requestAnimationFrame
- Image data cached during pan/zoom operations
- History limited to 50 coordinates to prevent memory bloat
- Auto-refresh cleanup on unmount
- Overlay canvas only redrawn when region changes

## Accessibility

- Keyboard support for tab navigation
- Semantic HTML for screen readers
- Contrast-compliant colors (VS Code palette)
- Hover tooltips on interactive elements
- Clear visual feedback for interactions

## Future Enhancements

1. **NCC Matching Visualization**
   - Display confidence scores (0-100%)
   - Highlight matched regions
   - Threshold slider (0.5-1.0)

2. **Multiple Match Display**
   - Show top-N matches
   - Cycle through matches

3. **Region Reference Storage**
   - Save selected regions as images
   - Link to step data
   - Reference in subsequent steps

4. **Click Integration**
   - Drag coordinates to Inspector
   - Auto-fill click_image step data
   - Visual feedback in canvas

5. **Advanced Drawing Tools**
   - Freehand marking
   - Color picker overlay
   - Measurement tools

## Testing Checklist

- [ ] Screenshot capture from running RPA server
- [ ] Zoom controls (all 4 levels)
- [ ] Pan with right-click drag
- [ ] Coordinate picking with left-click
- [ ] RGB color extraction
- [ ] Region selection with drag
- [ ] Region dimension calculation
- [ ] Coordinate history (max 5, then remove old)
- [ ] Copy to clipboard (CSV and JSON formats)
- [ ] Auto-refresh toggle and interval
- [ ] Sidebar collapse/expand
- [ ] Empty state messages
- [ ] Error handling (server not running)
- [ ] Light and dark theme support
- [ ] All 3 languages (en, ja, zh)

## Troubleshooting

### Screenshot not capturing
- Ensure RPA server is running (port in console logs)
- Check browser DevTools for IPC errors
- Verify `/screenshot` endpoint available on server

### Coordinates not picking
- Ensure screenshot is loaded first
- Check that canvas is in viewport
- Verify mouse events not blocked by overlay

### Region selector not working
- Ensure left-click (not right-click)
- Verify minimum region size > 5px
- Check z-index of overlay canvas

### Auto-refresh not updating
- Check interval is >= 1 second
- Ensure server connection stable
- Look for network errors in console

## Code Structure

```
ScreenPanel (main component)
├── useScreenCapture hook
│   ├── captureScreen (fetch from server)
│   ├── startAutoRefresh (setup interval)
│   └── stopAutoRefresh (cleanup)
│
├── Canvas rendering
│   ├── Image canvas (display)
│   ├── Overlay canvas (regions)
│   └── Coordinate display tooltip
│
└── Sidebar sections
    ├── Tools (clear, info)
    ├── Coordinates (history list)
    └── Selection (region details)
```

## Version Information

- Created: 2024
- React: 18.x
- Electron: Latest
- No external dependencies required

## License

Same as parent Robost project
