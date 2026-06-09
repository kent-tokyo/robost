# SPEC.md — robost Interaction Specification

Detailed interaction and layout specs for robost-editor and robost-snip.
Visual tokens (colors, spacing, typography) are in DESIGN.md.

---

## 0. Font Constraints

egui bundles Ubuntu-Light as the base Proportional font. robost overrides the priority order:
`[cjk, phosphor, Ubuntu-Light]`

Mixed Japanese/Latin UI labels must use the same CJK metrics box to avoid uneven text height.

**Unicode characters that render correctly (in Ubuntu-Light):**
- Basic Latin, Latin-1 Supplement (U+0000–U+00FF)
- Common arrows: ↑ ↓ ← → (U+2190–U+21FF Arrows block)
- Box drawing, block elements (U+2500–U+259F)

**Characters that appear as □ (NOT in Ubuntu-Light or Phosphor):**

| Do NOT use | Codepoint | Use instead |
|------------|-----------|-------------|
| `✕` | U+2715 | `×` (U+00D7) |
| `✗` | U+2717 | `×` (U+00D7) |
| `▸` | U+25B8 | omit, or use CollapsingHeader's built-in indicator |
| `✓` | U+2713 | `egui_phosphor::regular::CHECK` (Phosphor) |

**Rule:** For any non-ASCII character in UI labels or buttons, verify it renders before committing. If unsure, prefer Latin-1 (U+0000–U+00FF) or Phosphor icons (`ph::*`).

---

## 1. Editor Layout

```
┌─────────────────────────────────────────────────────────┐
│ Menu bar (File / Edit / View / Run / Help)               │  24 px
├─────────────────────────────────────────────────────────┤
│ Toolbar (undo/redo · name · run/stop)                    │  34 px
├────┬──────────┬─────────────────────┬───────────────────┤
│Act │ Unified  │ Canvas editor surface                   │ Inspector
│48px│ Sidebar  │ Freeform node canvas                    │ Canvas overlay
│    │ 220 px   │                                        │ 340 px default
│    │min 180   │                                        │
├──────────┴──────────────────────────┴───────────────────┤
│ Bottom panel: Variables · Log · Problems        160 px   │  resizable
├─────────────────────────────────────────────────────────┤
│ Status bar                                               │  22 px fixed
└─────────────────────────────────────────────────────────┘
```

### Workbench panels

The Activity Bar is always the leftmost 48 px strip. Its icon clicks select one unified sidebar content surface:

| Activity | Sidebar content | Extra behavior |
|----------|-----------------|----------------|
| Steps | Scenario step list | Selects canvas nodes |
| Nodes | Searchable step palette | Drag/drop or double-click to add steps |
| Templates | PNG template gallery | Sets template paths on selected image steps |

The unified sidebar is resizable, defaults to 220 px, and has an 11 px uppercase title (`SCENARIO`, `STEP PALETTE`, `TEMPLATES`). There is no separate List or Flow editor mode; Canvas is the only central editing surface.

### Inspector

Selecting a step opens the canvas inspector overlay. The inspector contains:

1. Compact selected-step header: category stripe, 1-based index, display name.
2. One-line description of what the selected action does.
3. Tabs: `フォーム` first, `YAML` second.
4. The property form and YAML editor for the selected step.

The inspector is hidden when nothing is selected. When visible, it is an immediate right-edge overlay inside the canvas area and must not resize or push the canvas left.
Floating utility windows, including Manual, take precedence over the canvas inspector; while Manual is open, the inspector overlay is hidden so it cannot cover the guide.

### Central editor surface

The central editor surface is always the freeform Canvas with start/end terminals, grid, minimap, comments, lasso, context menus, and drag/drop insertion.

---

## 2. Status Bar Zones

```
Canvas | ステップ 3: click_image  保存しました …        ● 実行中 ステップ 5 | 100%
```

| Zone | Content | Clears |
|------|---------|--------|
| Far left | Fixed editor mode text: `Canvas` | Static |
| Left-center | Selected step summary, or total step count | Every frame |
| Center | Temporary message (save result, undo description, error) | After 3 s |
| Right-center | `● 実行中 ステップ N` / `■ 停止中` | Every frame |
| Far right | Zoom percentage | Every frame |

The status bar uses StatusBarBg with white text. Temporary messages currently use white text; logs and toasts carry success/error color.

---

## 3. Bottom Panel

The bottom panel is resizable, defaults to 160 px, and has a 60 px minimum height.

| Tab | Content |
|-----|---------|
| Variables | Add/delete scenario variables, view initial values, click a variable name to highlight referencing steps |
| Log | Runtime/editor log, newest output sticks to bottom, Clear button visible while selected |
| Problems | Scenario validation issues with jump-to-step buttons |

Problems auto-opens when the validation issue count increases. The right side of the tab row shows the current scenario path when a file is open.

---

## 4. Toolbar Buttons

| Button | Tooltip format | Notes |
|--------|---------------|-------|
| Undo | `"アンドゥ: {action} (Cmd+Z)"` when action is known, else `"アンドゥ (Cmd+Z)"` | Action name injected from `last_undo_name` |
| Redo | Same pattern with `"リドゥ"` | Uses top of redo stack's action name |
| Run | `"シナリオを実行 (F5)"` | |
| Stop | `"実行を停止 (F5)"` | |
| Theme / Language | Located under View menu | Display settings are not toolbar controls |

Tooltip delay: 300 ms. All interactive controls must have a tooltip.

Toolbar layout, left to right:

1. Undo and redo icon buttons.
2. Scenario name text field, fixed 200 px.
3. Run or Stop button with Phosphor icon and label.
4. Right-aligned hint that theme and language live under View.

---

## 5. Keyboard Shortcuts

### Global (tray app always running)

| Key | Action |
|-----|--------|
| `Ctrl+Shift+C` | Start template capture (snip overlay) |
| `Ctrl+Shift+S` | Open / focus scenario editor |

### Editor — Always active

| Key | Action |
|-----|--------|
| `Cmd+N` | New scenario |
| `Cmd+O` | Open file |
| `Cmd+S` | Save |
| `Cmd+Shift+S` | Save As |
| `Cmd+Z` | Undo |
| `Cmd+Shift+Z` | Redo |
| `Cmd+C / X / V / D` | Copy / Cut / Paste / Duplicate selected steps |
| `F5` | Run / Stop toggle |
| `Cmd+R` | Run (when stopped only) |
| `Cmd+Shift+F5` | Run selection |
| `Cmd+Shift+A` | Open Add Step popup |
| `Delete` | Delete selected step(s) — shows confirm dialog |
| `Backspace` | Delete selected step(s) in Canvas mode — shows confirm dialog |
| `Esc` | Deselect / close overlay |
| `Cmd+,` | Settings |

### Editor — Canvas mode only

| Key | Action |
|-----|--------|
| `Cmd+A` | Select all |
| `Cmd+F` / `Cmd+G` | Toggle node search bar |
| `?` | Toggle keyboard shortcut overlay |
| `Cmd+0` | Fit all nodes in view |
| `Cmd+1` | Reset zoom to 100% and center nodes |
| `Cmd+↑ / ↓` | Move selected step(s) up / down |
| `↑ / ↓` | Select previous / next node and center it |
| `Ctrl+Scroll` | Zoom (range: 25 %–200 %) |
| Pinch | Zoom (range: 25 %–200 %) |
| `Middle drag` or background drag | Pan |
| `Shift+drag` on background | Lasso selection |
| `Cmd+click` on node | Toggle selection |
| `Shift+click` on node | Range-select from anchor |

### Snip overlay

| Key | Action |
|-----|--------|
| `Esc` | Cancel capture, close overlay |
| `V` | Switch to View mode |
| `A` | Switch to Add Anchor mode |
| `M` | Switch to Add Mask mode |

---

## 6. Canvas Interaction Model

### Node drag

1. Pointer enters node rect → `Grab` cursor.
2. Pointer down + move > `DragThreshold` (4 px) → drag starts; cursor becomes `Grabbing`.
3. During drag: if snap is on, position snaps to nearest 40-unit grid point; node border flashes SnapFlash color for 300 ms on each snap.
4. Pointer up → drag ends; final position is snap-resolved.

### Edge connection

1. Pointer enters the output port area (bottom-center of node, within 10 px at current zoom) → cursor becomes `Crosshair`.
2. Drag from port → ghost bezier curve follows cursor.
3. Release over another node → step is reordered to that position.
4. Release on empty canvas → drag cancelled.

### Multi-select

- `Shift+drag` on background → lasso; all nodes touched by the rectangle are added to selection.
- `Cmd+click` on node → toggle node in/out of selection.
- `Shift+click` on node → range-select from the current anchor to the clicked node.
- `Cmd+A` → select all.
- `Esc` → clear selection.

### Double-click

- On node → select that step and focus the canvas inspector.

### Snap grid

- Grid size: 40 logical units (scales with zoom; hidden when rendered spacing < 8 px).
- Snap is toggled per-user in settings; default off.
- Visual feedback: SnapFlash color border on node for 300 ms on each snap event. No sound or haptic.

### Search

- `Cmd+F` or `Cmd+G` opens a 320 px floating search bar at the top-center of the canvas.
- Match rule: `step_matches(step, query)` over step key and serialized step content.
- `Enter` jumps to the next match; `Shift+Enter` jumps to the previous match.
- Jumping selects the matched node and pans it to the center of the viewport.
- `Esc` closes search and clears the query.

### Minimap

- Size: 160 x 100 px, bottom-right, 8 px margin.
- Shown automatically when the scenario has 15 or more top-level steps, or when enabled from the canvas context menu/settings.
- Dragging inside the minimap recenters the canvas viewport. The minimap keeps the drag latch even if the pointer briefly exits the minimap rect.

### Comments

- Canvas comments are sticky-note annotations stored only in the sidecar layout file, not in scenario YAML.
- Background context menu → `コメントを追加` creates a comment at the clicked canvas position and enters edit mode.
- Double-click a comment to edit text.
- Drag a comment to move it.
- Comment context menu supports delete and color changes.

### Layout persistence

For `scenario.yaml`, canvas positions and comments are saved to sibling file `scenario.yaml.layout.json`:

```json
{
  "positions": { "0": [40.0, 40.0] },
  "comments": []
}
```

---

## 7. Context Menus

**Rule: never show a grayed-out item. If an action does not apply, omit it entirely.**

### Node right-click

| Item | Condition to show |
|------|------------------|
| ● 有効化 / ○ 無効化 | Always |
| コピー | Selection is non-empty |
| カット | Selection is non-empty |
| 複製 | Selection is non-empty |
| 貼り付け | Clipboard is non-empty |
| 削除 | Selection is non-empty |
| ▶ ここから実行 | Scenario is not currently running |
| 整列 (← ↑) | `multi_selected.len() >= 2` |
| 等間隔 (↔ ↕) | `multi_selected.len() >= 3` |

### Canvas background right-click

| Item | Condition to show |
|------|------------------|
| ここにステップを追加 | Always |
| コメントを追加 | Always |
| 全選択 | Always |
| 貼り付け | Clipboard is non-empty |
| 整列 / 等間隔 | `multi_selected.len() >= 2` / `>= 3` |
| キャンバスリセット / ビューを合わせる | Always |
| スナップ / ミニマップ | Always (toggle label) |

---

## 8. Cursor Specification

| Situation | Cursor |
|-----------|--------|
| Canvas: hovering a draggable node (not during edge drag) | `Grab` |
| Canvas: dragging a node | `Grabbing` |
| Canvas: hovering output port (edge-drag ready) | `Crosshair` |
| Canvas: `Shift` held over background (lasso ready) | `Crosshair` |
| Canvas: hovering edge `+` insertion button | `PointingHand` |
| Canvas: hovering canvas comment | `Grab` |
| Snip: AddAnchor mode, pointer over template | `Crosshair` |
| Snip: AddMask mode, pointer over template | `Crosshair` |
| Snip: View mode | default arrow |
| Snip: selection drag (overlay) | `Crosshair` |

---

## 9. Undo System

The undo stack stores complete state snapshots. Limit: 50 entries; oldest discarded when exceeded.

Each snapshot carries an `action_name` field set by the named push function. The plain push (used for drag operations and property edits) inherits the name from the previous push.

| Operation | Action name |
|-----------|-------------|
| Step deleted | `"ステップ削除"` |
| Step added from palette | `"ステップ追加"` |
| Paste | `"貼り付け"` |
| Move up / down (keyboard) | `"移動"` |
| Node drag (end) | `"移動"` |
| Property edit | `"編集"` |

After Undo: status bar center shows `"取り消し: {action_name}"` for 3 s.
After Redo: status bar center shows `"やり直し: {action_name}"` for 3 s.
Undo button tooltip: `"アンドゥ: {action_name} (Cmd+Z)"` when name is non-empty.

---

## 10. Add Step Popup

Triggered by `Cmd+Shift+A`, Edit → Add Step, the sidebar Add Step button, edge midpoint `+`, or canvas context menu `ここにステップを追加`.

- Window title: `"ステップを追加"`.
- Default size: 300 x 500 px, resizable, non-collapsible.
- Top-right `X` icon closes the popup without inserting.
- Text input at top, placeholder `"検索…"`, focused on open.
- Empty query shows a category tree, default-open, with category colors.
- Non-empty query shows a flat filtered list.
- Match rule: step name, display name, or category contains lowercased query.
- `↑ / ↓` navigate visible results.
- `Enter` inserts the highlighted result.
- `Esc` closes.

Insertion target:

| Opened from | Insert position |
|-------------|-----------------|
| Normal add | After selected step, or append when nothing is selected |
| Edge midpoint `+` | After the source edge step |
| Canvas background `ここにステップを追加` | At clicked canvas position |
| Branch `+` in a compound step | Append to that branch sub-list |

The Nodes sidebar uses the same step templates. Double-clicking a palette row inserts after the selected step; drag/drop to the canvas inserts at the drop position. During a compatible drag, Canvas shows a drop-acceptance cue over the working area; releasing anywhere inside the canvas inserts the new node at the release position, including when the canvas is empty.

---

## 11. AI Assistant And Settings

### Floating AI assistant

- Floating action button: bottom-right, 48 x 48 px circular button.
- Normal fill: Accent. Unread response fill: Warning.
- Click toggles a fixed 360 x 320 px, non-resizable assistant window.
- Window contains header (`AI アシスタント` / localized), Clear button, Close button, scrollable messages, and a two-row input.
- `Ctrl+Enter` sends while the input is focused.
- AI responses render Markdown. YAML code blocks are extracted and shown with insert buttons.
- Insert button preview shows parsed step count and first step keys when YAML is valid.

### AI Create step

`ai_create` is a special step type. Its property form contains a prompt field and an `AI で生成` action. Generation runs in the background and replaces the `ai_create` step with the returned YAML steps only if the original step still exists at the same index.

### Settings

Settings are opened from File → Settings or `Cmd+,`.

| Setting | UI |
|---------|----|
| AI provider | Combo box: Anthropic (Claude) / OpenAI |
| API key | Password text field; stored in OS keychain |
| Model | Free-form text field plus provider-specific preset quick-fill buttons |
| Test connection | Background test with spinner and success/error result |

Non-secret settings are saved to `~/.config/robost/settings.toml` with `0600` permissions on Unix. API keys are never written to TOML.

Theme and language are changed from the View menu and persisted with the same settings file.

### Manual and About

- Help → Manual opens a resizable 720 x 640 px operation guide with searchable/category-filtered step docs and insertable snippets.
- Help → About opens a compact non-resizable about dialog.

---

## 12. Snip Overlay

### Capture flow

1. Global hotkey received → full-screen capture → overlay window shown. Total: ≤ 50 ms.
2. Overlay: frozen screenshot at 1:1 pixels, semi-transparent dark layer on top.
3. Hint text at top: `"ドラッグで範囲を選択（Escape でキャンセル）"`.
4. User drags to define selection. Live w×h label renders near the cursor (not in a panel).
5. Release with selection ≥ 4×4 px → transition to Edit mode.
6. Release with selection < 4×4 px → show `"選択範囲が小さすぎます"` warning for 1.5 s, remain in Selecting state.

### Edit mode panels

Editing panel anchored to bottom-center of screen.

| Section | Contents |
|---------|---------|
| Template preview | Scaled image (max 420×280 px, 0.5×–8×); anchors and masks overlaid |
| Mode selector | View / + アンカー / + マスク — radio buttons with tooltips |
| Anchor label | Text field (AddAnchor mode only) |
| Anchors list | Collapsible; each entry shows index, coordinates, label; delete button |
| Masks list | Collapsible; each entry shows index, bounding rect; delete button |
| Multi-scale | Checkbox: generate 125 % / 150 % DPI variants |
| Live test | `"▶ マッチング確認"` button; shows match score and position on success |
| Actions | `"💾 保存"` and `"✕ キャンセル"` buttons |

### Cursor in edit mode

- AddAnchor mode + pointer over template → `Crosshair`.
- AddMask mode + pointer over template → `Crosshair`.
- View mode → default arrow.

---

## 13. Error Messages

Button labels start with a verb. Error messages always include a suggested next action.

| Situation | Message |
|-----------|---------|
| Template not found | `"テンプレート画像 '{name}' が見つかりません — 📸 Snip で再採取してください"` |
| Match failed | `"テンプレートが見つかりませんでした — 閾値を下げるか再採取してください"` |
| Save failed | `"保存に失敗しました: {reason} — 保存先フォルダの書き込み権限を確認してください"` |
| Save OK | `"保存しました: {path}"` |
| Delete confirm | `"'{label}' を削除しますか？Cmd+Z で元に戻せます。"` |
| Undo limit | `"アンドゥ履歴の上限 (50件) に達しました"` |
| Plugin permission | `"'{plugin}' にファイル読み取りを許可しますか？"` |

---

## 14. Snip Tray Menu

Maximum one level of nesting.

```
robost Snip
├── テンプレート採取 (Ctrl+Shift+C)
├── エディターを開く (Ctrl+Shift+S)
├── 使い方
└── 終了
```

---

## 15. Activity Bar (VS Code style)

Derived from VS Code `activitybarPart.ts`. Fixed 48 px wide, leftmost panel, non-resizable.

### Icons (top to bottom)

| Icon | Phosphor | Sidebar content when active |
|------|----------|-----------------------------|
| Steps | `LIST_BULLETS` | Current scenario step list |
| Nodes | `TREE_STRUCTURE` | Step template palette (add new steps) |
| Templates | `IMAGE` | PNG template gallery |

### Visual rules

- Background: `ActivityBarBg` (#333333 in both themes) — always darker than sidebar.
- Active icon: white (`#FFFFFF`), left 2 px `Accent` border.
- Inactive icon: white at **40% opacity** (matches VS Code's `activityBar.inactiveForeground`).
- No text labels on icons — icon only.
- Hover: subtle white fill at low alpha and brighter icon.
- Icons are 22 px, centered in the 48 px cell.

### Interaction

- Click any icon → switches sidebar tab.
- Click Steps → shows the scenario step list in the unified sidebar while keeping Canvas active.
- Click Nodes or Templates → keeps Canvas active.
- Sidebar state persists per session.

---

## 16. Sidebar (unified)

Single conceptual sidebar controlled by Activity Bar. In implementation this is the `step_palette` side panel and its content is selected by `sidebar_tab`.

### Tabs

| `SidebarTab` | Source | Notes |
|--------------|--------|-------|
| `Steps` | Scenario list | Select step; double-click opens List edit; Add Step button |
| `Nodes` | Step palette | Expand/collapse all, search, double-click insert, drag to canvas |
| `Templates` | Template gallery | PNG thumbnails from scenario directory or `~/Documents/robost_templates`; double-click sets selected image step template |

### Visual rules

- Background: `SidebarBg` (#252526 dark / #F3F3F3 light).
- Section header: 1 px bottom border `#3C3C3C`, text `FgDim`.
- Row hover: `ListHover` background fill (no border).
- Active row: `ListSelection` background fill + `#FFFFFF` text.
- No tab bar inside sidebar — tab switching is done via Activity Bar.

---

## 17. Interaction Design Principles (from VS Code)

**Rule: hover = background fill only. Never add a border on hover.**

VS Code source `list.hoverBackground` = `#2A2D2E` — a subtle fill.
`list.activeSelectionBackground` = `#04395E` — high contrast for selection.
`list.inactiveSelectionBackground` = `#37373D` — reduced contrast when panel is unfocused.

**Focus**: keyboard focus uses 1 px `FocusBorder` (#007FD4) outline — not a fill.

**Opacity for inactive states**: inactive icons, disabled text → opacity 40% of the active color.
Never use a separate dim color — always derive from the active color via alpha.

**Color hierarchy rule**: `EditorBg` (darkest) < `SidebarBg` < `ActivityBarBg` (lightest in dark mode).
Separation between regions is achieved by background value contrast, not borders.
