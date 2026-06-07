# DESIGN.md — robost Design Tokens

Design intent and visual language for robost-editor and robost-snip.
Derived from VS Code's design system (colorRegistry.ts, workbench theme).
Implementation details and interaction specs live in SPEC.md.

---

## Philosophy

| Principle | Meaning |
|-----------|---------|
| Minimum color | ~20 colors total. Grayscale + one accent. Never add a new color without removing one. |
| Background not border | Hover and selection are communicated by background fill change, never border addition. |
| Transparency for depth | Inactive states use opacity reduction, not a different color. |
| Flat | No shadows. No gradients. 1 px separators that match the background hue. |
| Discoverable | Every action reachable by keyboard. Tooltips teach shortcuts. |
| Workbench first | The editor is an operational tool, not a landing page. Prioritize dense scanning, stable controls, and persistent context over decorative panels. |

---

## Workbench Composition

robost-editor uses a VS Code-style workbench with persistent chrome:

| Zone | Role | Visual rule |
|------|------|-------------|
| Menu bar | File / Edit / View / Run / Help commands | Same background as toolbar, compact text buttons |
| Toolbar | View selector, undo/redo, scenario name, run/stop | Single fixed-height row; icon buttons for undo/redo/run actions |
| Activity bar | Top-level navigation between Steps / Nodes / Templates | Fixed 48 px, icon-only, dark in both themes |
| Unified sidebar | Scenario list, step palette, or template gallery | Compact rows and grids, no card framing |
| Legacy step list | List/Flow left working column | Visible in List and Flow modes only |
| Editor surface | Canvas / Flow / List work area | Full-bleed background with no nested cards |
| Inspector | Selected step details | Right-side panel in Canvas and Flow, property-first tabs |
| Bottom panel | Variables / Log / Problems | Resizable, tabbed, operational diagnostics |
| Status bar | Mode, selection, run state, zoom | Fixed 22 px accent bar at the bottom |

The main editor surface must remain visually dominant. Sidebars, inspectors, and diagnostics use subdued backgrounds and small headings so the user can scan the scenario without fighting chrome. Floating utility windows are limited to AI assistant, Settings, Manual, About, and step insertion.

---

## Color — Dark Mode

Derived from VS Code Dark+ (`colorRegistry.ts`, `workbench/common/theme.ts`).

### Layout regions

| Token | Hex | VS Code source | Usage |
|-------|-----|----------------|-------|
| ActivityBarBg | `#333333` | activityBar.background | Leftmost 48 px icon strip |
| SidebarBg | `#252526` | sideBar.background | Sidebar panel |
| EditorBg | `#1E1E1E` | editor.background | Canvas / List / Flow area |
| PanelBg | `#1E1E1E` | panel.background | Bottom Variables/Log/Problems |
| ToolbarBg | `#2B2B2B` | workbench chrome approximation | Menu bar and toolbar |
| StatusBarBg | `#007ACC` | statusBar.background | Bottom status bar |

Text: FgDefault `#CCCCCC` · FgDim `#858585` · activity icons `#FFF` @ 40%/100% opacity.

### Interaction (background-only — never borders)

| Token | Hex | VS Code source | Usage |
|-------|-----|----------------|-------|
| ListHover | `#2A2D2E` | list.hoverBackground | Row hover |
| ListSelection | `#04395E` | list.activeSelectionBackground | Active selection |
| ListInactive | `#37373D` | list.inactiveSelectionBackground | Unfocused selection |
| FocusBorder | `#007FD4` | focusBorder | Keyboard focus outline only |

### Semantic

Accent `#0078D4` · Success `#6CCB5F` · Warning `#FCE100` · Error `#FF99A4` · SnapFlash `#20C0A0` · BadgeBg `#4D4D4D`

Canvas structural colors: NodeBg `#2D2D2D`, NodeBgSelected = ListSelection, NodeBgRunning `#302800`, EdgeColor `#5F5F5F`.

Step category color is shown as a narrow stripe on nodes and step rows, not as a filled card/background. Current categories include AI, image, control flow, input, dialogs, variables, wait, scripts, clipboard, library, data, files, Excel, string, date, JSON, path, mouse, process, HTTP, mail, web, UIA, CSV, list, and utility. Category colors must be centralized with the other UI tokens before adding new categories.

Light mode uses `egui::Visuals::light()` as base. Override: ActivityBarBg `#333333` (stays dark), SidebarBg `#F3F3F3`, EditorBg `#FFFFFF`, PanelBg `#FAFAFA`, ToolbarBg `#F7F7F7`, Border `#D6D6D6`, ListHover `#E8E8E8`, FgDefault `#1A1A1A`.

---

## Typography

Font priority order: `[CJK(W4), Phosphor, Ubuntu-Light]`
CJK is first so Latin and Japanese share one metrics box in mixed labels.
Phosphor remains before the bundled fallback so icon glyphs resolve.

| Role | Size | Usage |
|------|------|-------|
| Body | 13 px | Labels, buttons (matches VS Code default) |
| Small | 11 px | Status bar, hints |
| Monospace | 13 px | YAML, log, variable values |
| Heading | 15 px | Section titles |

Sentence case for UI labels. `"Save file"` ✓ `"Save File"` ✗

---

## Spacing

Base unit: **4 px** for layout blocks. Text-bearing controls use explicit comfort tokens:
item spacing 10 x 6 px, button padding 10 x 5 px, window margin 12 px, menu margin 8 px.

---

## Shape

| Token | Value | Usage |
|-------|-------|-------|
| RoundingUi | 2 px | Buttons, list items, small widgets |
| RoundingCard | 4 px | Canvas nodes, panels, cards |

VS Code uses sharp corners (0 px) for most surfaces.
robost uses 2 px minimum to soften egui's defaults without losing the VS Code feel.

---

## Motion

Off by default. Snap flash: 300 ms teal border. All others: instant.

---

## Layout Constants

ActivityBarWidth 48 px · StatusBarHeight 22 px · ToolbarHeight 34 px · SidebarWidthDefault 240 px · SidebarWidthMin 180 px · InspectorWidthDefault 340 px · InspectorWidthMin 280 px · InspectorWidthCollapsed 48 px · InspectorWidthMax 560 px · StepRowHeight 28 px · DragThreshold 4 px

Canvas: NodeW 180 px · NodeH 72 px · Grid 40 logical units · Zoom 25%–200% · Minimap 160 x 100 px, bottom-right, shown automatically for 15+ nodes or when enabled in settings.

Timing: tooltip delay 300 ms · status message 3 s · toast 4 s (see SPEC.md §2–4)
