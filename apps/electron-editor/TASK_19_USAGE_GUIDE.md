# Task #19: Execution History & Debugging Features - Usage Guide

## Quick Start

### Accessing the Features

1. **Execution History Panel**
   - Click the 📊 icon in the Activity Bar (left side)
   - Browse all past executions with filters
   - Click any execution to view details

2. **Breakpoints**
   - Open Explorer panel (📁)
   - Click "Breakpoints" tab
   - Click the red dot (●) to toggle breakpoints on steps
   - Steps with breakpoints will pause execution (when implemented)

3. **Variable Inspector**
   - Open Explorer panel (📁)
   - Click "Variables" tab
   - View current execution variables
   - Add variables to watch list during execution

## Execution History Panel

### List View
The main history view shows:
- **Execution Status**: ✓ (success), ✕ (failed), ⏹ (stopped)
- **Scenario Name**: Name of the executed scenario
- **Timestamp**: When the execution started
- **Statistics**: Steps completed, duration, log count

### Filtering & Search
- **Status Filter**: All, Success, Failed, Stopped
- **Search Box**: Find executions by name
- **Results**: Shows filtered count vs. total

### Actions
- **View Details**: Click any execution
- **Delete**: Click ✕ on execution card
- **Clear All**: Clear entire history with confirmation

### Export & Import
- **Export as JSON**: Full execution records with all data
- **Export as CSV**: Spreadsheet-compatible format
- **Import**: Load previously exported history

### Detailed View
Clicking an execution shows:
- **Header**: Execution info with back button and replay option
- **Statistics**: Status, duration, steps, logs
- **Progress Bar**: Visual completion percentage
- **Log Filters**: View by level (info, warn, error, all)
- **Logs**: Full timestamped execution log

## Breakpoint Manager

### Visual Hierarchy
- **Expandable Sections**: Groups and nested steps show expand/collapse arrows
- **Step List**: All steps in execution order
- **Breakpoint Indicator**: Red dot (●) shows active breakpoints
- **Step Type**: Small label showing step category

### Setting Breakpoints
1. Locate the step in the breakpoints list
2. Click the red dot button next to the step
3. Dot will light up with a glow effect when active
4. Breakpoint counter at top shows total active breakpoints

### Step Hierarchy
Supports nested structures:
- Groups (containers of steps)
- Control flow (if/foreach/while)
- Try/catch blocks

Click expand arrow (▶/▼) to show/hide child steps.

## Variable Inspector

### Current Execution Section
Shows all variables from the current execution with:
- **Type Icon**: Visual indicator of value type
- **Variable Name**: The variable identifier
- **Type Label**: String, Number, Boolean, Array, Object, etc.
- **Value**: Current value or visual representation

### Watch List
Add specific variables to monitor across execution:

1. **Add Variable**:
   - Type variable name in input field
   - Press Enter or click + button
   - Variable added to watch list

2. **Watch List Display**:
   - Current value with type indicator
   - Change history (last 10 changes shown)
   - Expand history with ▶ arrow

3. **Remove from Watch**:
   - Click ✕ next to variable name
   - Removed from watch list

### Variable Types & Colors
- **String** (green): `"text value"`
- **Number** (amber): `42` or `3.14`
- **Boolean** (purple): `true` or `false`
- **Array** (blue): `[array with 5 items]`
- **Object** (cyan): `{nested properties...}`
- **Null** (gray): `null`

### Nested Structure Viewing
Objects and arrays show:
```
{
  key: "value",
  nested: {
    prop: 123
  }
}
```

Click items to expand/collapse nested structures.

## Execution Replay

### Starting Replay
1. Go to History panel
2. Click on any past execution to view details
3. Click "▶ Replay" button in header
4. Replay interface opens

### Playback Controls
- **Play/Pause**: Toggle automatic step advancement
- **Previous (◀)**: Go to previous step
- **Next (▶)**: Go to next step
- **Speed Control**: 0.5x, 1x, or 2x playback speed

### Navigation
- **Timeline**: Visual bar showing all steps
  - Green: completed successfully
  - Red: failed
  - Gray: skipped
  - Click any step to jump to it

- **Step List**: Detailed list below timeline
  - Shows step name, status, duration
  - Click to seek to that step
  - Highlighted step is current

### Current Step Display
Shows:
- **Status**: STARTED, COMPLETED, or FAILED
- **Step Name**: The step identifier
- **Duration**: Time taken (ms)
- **Error Message**: If step failed

### Auto-Advance
When playing, steps auto-advance based on their recorded duration:
- 0.5x: Each step takes 2x recorded time
- 1x: Each step takes recorded time
- 2x: Each step takes 1/2 recorded time

## Progress Panel Updates

### During Execution
New controls appear when execution is running:
- **Status Display**: Shows progress and elapsed time
- **Pause Button** (⏸): Pauses the execution
- **Resume Button** (▶): Resumes paused execution
- **PAUSED Status**: Yellow indicator when paused

## localStorage Persistence

### Auto-Save
Execution history automatically saves to browser localStorage:
- **Storage Key**: `robost-execution-history`
- **Max Stored**: 50 execution records
- **Persistence**: Survives app restart

### Manual Backup
1. Export as JSON for full backup
2. Store in version control or cloud storage
3. Import when needed

## Performance Notes

### Memory Management
- **In-Session**: Up to 100 executions in memory
- **Persistent**: Up to 50 executions in localStorage
- **Watch Variables**: 50 history entries per variable
- **Step Executions**: Full details stored with each execution

### Optimization Tips
- Regularly export and clear old executions
- Use filters to narrow down search
- Archive important executions separately

## Tips & Tricks

1. **Export Before Clear**: Always export history before clearing all
2. **Watch Key Variables**: Add variables you're debugging to watch list
3. **Use Filters**: Status filter helps find failed executions quickly
4. **Replay Speed**: Slow down (0.5x) to see each step clearly
5. **Compare Executions**: View multiple executions to compare behavior

## Known Limitations & Future Features

### Current Implementation
- Replay is visual only (doesn't re-execute)
- Breakpoints don't pause execution yet
- No conditional breakpoints
- No variable modification during debugging

### Planned Enhancements
- Breakpoint execution pause
- Conditional breakpoints ("break if X > 5")
- Screenshot capture at each step
- Side-by-side execution comparison
- Variable state modification
- Performance analytics and graphs

## Troubleshooting

### History Not Persisting
- Check browser localStorage is enabled
- Verify no private/incognito mode
- Clear cache and reload

### Replay Not Working
- Ensure execution has step details recorded
- Check execution has completed
- Verify step executions list is populated

### Variables Not Showing
- Make sure execution is active or has completed
- Check variable names are correct
- Variables require RPA server to capture them

### Breakpoints Not Pausing
- Feature planned for future release
- Currently for visual marking only
- Use pause button in ProgressPanel instead

## API Integration

For developers integrating execution capture:

### Send Execution Variables
```javascript
window.electronAPI.ipcSend('rpa:progress', {
  type: 'variables',
  variables: { varName: varValue }
});
```

### Record Step Details
```javascript
// At step start
{
  type: 'step_start',
  index: 0,
  name: 'Step Name',
  id: 'step-1',
  type: 'click_image'
}

// At step completion
{
  type: 'step_done',
  index: 0,
  name: 'Step Name',
  elapsed_ms: 1234
}

// At step failure
{
  type: 'step_failed',
  index: 0,
  error: 'Element not found',
  elapsed_ms: 500
}
```

### Execution Completion
```javascript
{
  type: 'finished',
  success: true/false,
  error: 'error message if failed'
}
```

## Keyboard Shortcuts (Future)
- `Ctrl+H`: Open history panel
- `Ctrl+B`: Toggle breakpoints
- `Ctrl+V`: Open variables
- `Space`: Play/pause replay
- `Left/Right Arrows`: Previous/next step in replay
