# Robost Editor - Phase 3 Features Test Plan

## Test Checklist

### Task #16: AI Assistant Integration

#### Setup
- [ ] Navigate to Sidebar > AI tab
- [ ] Go to Settings > API Keys section
- [ ] Paste Anthropic API key
- [ ] Click "Save"

#### Basic AI Feature Test
- [ ] Click "AI" tab in Sidebar
- [ ] Type description: "Click the OK button"
- [ ] Click "Generate Steps" button
- [ ] Verify suggested steps appear in list
- [ ] Verify step has: name, type, data fields
- [ ] Click suggested step → preview appears
- [ ] Click "Add to Canvas" → step added to canvas
- [ ] Verify step appears in Canvas

#### Advanced AI Features
- [ ] Click "Copy YAML" button → YAML copied to clipboard
- [ ] Click "Add All" button → all suggestions added
- [ ] Check History sidebar → last 5 suggestions shown
- [ ] Click history item → re-populate suggestions
- [ ] Test error handling: Remove API key, try Generate → error message appears
- [ ] Test multiple suggestions: Generate several times → different results

---

### Task #17: Advanced Canvas Features

#### Node Grouping
- [ ] Select 2-3 steps on canvas
- [ ] Hold Cmd (macOS) or Ctrl (Windows) + click additional nodes
- [ ] Right-click selection → "Group Selected Steps" appears
- [ ] Click "Group" → nodes grouped visually (purple box)
- [ ] Click group → expand/collapse
- [ ] Right-click group → "Ungroup Steps" option
- [ ] Click Ungroup → group dissolved

#### Conditional Visualization
- [ ] Add "if" step to canvas
- [ ] Verify node is blue diamond shape
- [ ] Add "while" step → green dashed border
- [ ] Add "foreach" step → loops icon
- [ ] Add "try_catch" step → red striped border
- [ ] Verify hover shows condition preview (if/while)

#### Search & Filter
- [ ] Press Cmd+F (macOS) or Ctrl+F (Windows)
- [ ] Type "click" → search modal opens
- [ ] Verify matching steps highlighted on canvas
- [ ] Verify match count shows
- [ ] Type step type in filter → results filtered
- [ ] Press Escape → search modal closes

#### Advanced Node Operations
- [ ] Select node on canvas
- [ ] Press Cmd+C → copy to clipboard
- [ ] Press Cmd+V → pasted node appears on canvas
- [ ] Select node → Press Cmd+D → duplicate appears
- [ ] Right-click node → "Add Comment" option
- [ ] Type comment → Inspector shows comment field
- [ ] Select multiple nodes → Delete → nodes removed
- [ ] Press Ctrl+Z → undo removes deletion

#### Layout & Navigation
- [ ] Click "Auto Layout" button → nodes rearrange hierarchically
- [ ] Nodes with children show in hierarchy
- [ ] Right-click node → "Zoom to Selection" zooms to node
- [ ] Breadcrumb shows group nesting (if groups exist)

---

### Task #18: Screen Operation Panel

#### Screen Capture
- [ ] Click "Screen" tab in Editor area (next to Canvas, Code, List)
- [ ] Screen preview displays (if RPA server running)
- [ ] Click refresh button → new screenshot captured
- [ ] Zoom controls: 25%, 50%, 100%, 200% work
- [ ] Pan screenshot: right-click drag moves view
- [ ] Auto-refresh toggle enabled → refreshes periodically

#### Coordinate Picker
- [ ] Click on screenshot image → crosshair cursor appears
- [ ] Move mouse → coordinates display in top-right (x, y)
- [ ] Left-click location → coordinates + RGB color shown
- [ ] "Last 5 Coordinates" sidebar shows history
- [ ] Click history item → coordinates populated
- [ ] Click "Copy CSV" → "x,y" format copied
- [ ] Click "Copy JSON" → JSON format copied

#### Region Selector
- [ ] Draw rectangle on screenshot (click + drag)
- [ ] Rectangle outline appears with dimensions
- [ ] Release mouse → region captured
- [ ] "Region Size" shows: width × height
- [ ] "Save Region" button → region saved to reference
- [ ] Region preview appears in Inspector (if edit click_image)

---

### Task #19: Execution History & Debugging

#### Execution History Panel
- [ ] Click "History" tab in Sidebar (or ActivityBar)
- [ ] History panel shows list of past executions
- [ ] Each execution shows: timestamp, status, duration, step count
- [ ] Filter by status: success/failed/stopped
- [ ] Search by scenario name
- [ ] Click execution → details view opens
- [ ] Details show: full logs, step timeline, error messages

#### Breakpoint Manager
- [ ] Right-click step on canvas → "Set Breakpoint" option
- [ ] Red dot appears on step node
- [ ] Right-click breakpoint → "Remove Breakpoint"
- [ ] Breakpoint tab shows all breakpoints hierarchically
- [ ] Expand/collapse group to see nested breakpoints

#### Variable Inspector
- [ ] Run scenario (or mock execution)
- [ ] Variables panel shows scenario variables
- [ ] Shows: variable name, type, current value
- [ ] Click variable → watch it
- [ ] Watched variables tracked across steps
- [ ] Variable history shows previous values
- [ ] Type color coding: string (blue), number (green), boolean (orange), array (purple)

#### Execution Replay
- [ ] After execution completes, click "Replay" button
- [ ] Replay timeline appears
- [ ] Play/pause controls work
- [ ] Next/previous step buttons advance execution
- [ ] Speed control: 0.5x, 1x, 2x
- [ ] Click timeline to seek to step
- [ ] Step details show at each point
- [ ] Variables update as replay progresses

#### Pause/Resume
- [ ] Run scenario
- [ ] Click "Pause" button in ProgressPanel
- [ ] Execution pauses at current step
- [ ] UI shows "PAUSED" indicator
- [ ] Click "Resume" → continues execution
- [ ] Step stays highlighted during pause

---

## Integration Tests

### Cross-Feature Tests

#### AI + Canvas
- [ ] Generate AI suggestions
- [ ] Add multiple suggestions to canvas
- [ ] Switch to Canvas tab → verify steps present
- [ ] Edit AI-generated steps in Inspector

#### Canvas + History
- [ ] Run scenario with grouped nodes
- [ ] Check History panel → execution logged
- [ ] Click execution → replay shows grouped nodes

#### Screen + click_image
- [ ] Use Screen picker to get coordinates
- [ ] Drag coordinates to Inspector click_image field
- [ ] Values auto-fill in step data

#### Breakpoints + Replay
- [ ] Set breakpoints on several steps
- [ ] Run scenario
- [ ] Execution pauses at first breakpoint
- [ ] Resume → pauses at next breakpoint
- [ ] Replay shows which steps had breakpoints

---

## Performance Tests

- [ ] Canvas with 50+ steps: smooth scrolling & zoom
- [ ] Search with 100+ steps: instant results
- [ ] History with 50 executions: instant load
- [ ] Variable inspector with 100+ variables: no lag
- [ ] Replay at 2x speed: smooth playback

---

## Localization Tests (i18n)

- [ ] Change language in Settings: EN/JA/ZH
- [ ] AI tab label translates
- [ ] Screen panel labels translate
- [ ] History panel labels translate
- [ ] Breakpoint/Variable labels translate
- [ ] All UI text reflects chosen language

---

## Theme Tests

- [ ] Switch to Light mode in Settings
- [ ] All new components use light palette
- [ ] Text readable on light background
- [ ] Switch back to Dark → components match dark theme
- [ ] CSS variables applied correctly

---

## Edge Cases & Error Handling

- [ ] AI API key missing → error message
- [ ] AI API quota exceeded → error message
- [ ] Screen panel: RPA server offline → "Connection failed" message
- [ ] Breakpoint on non-existent step → graceful handling
- [ ] History export: empty history → "No data" message
- [ ] Replay: no recorded execution → "No replay data"
- [ ] Screen picker: click outside image → ignored
- [ ] Group empty nodes → no visible change

---

## Success Criteria

✅ All checkboxes passed = **Features ready for production**

---

## Notes

- Ensure TypeScript builds without errors
- Check browser console (F12) for JS errors
- Monitor performance in DevTools
- Test on macOS and Windows if possible
- Verify responsive design on different screen sizes
