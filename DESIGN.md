# DESIGN.md — robost Design Guidelines

> This document references the official Windows application design guidelines ([Microsoft Design Guidelines](https://learn.microsoft.com/en-us/windows/apps/design/guidelines-overview))
> and reinterprets them to fit robost's technology stack and target users.

---

## 0. Design Philosophy

robost's users are **RPA practitioners and engineers** who prioritize **speed and accuracy** over visual aesthetics.

| Principle | Meaning |
|------|------|
| **Transparency** | Users always know what the tool is doing |
| **Non-intrusive** | The tray icon and overlay never block the target UI the user is operating |
| **No hidden errors** | Failures are surfaced immediately with a screenshot |
| **Consistency** | Built-in / plugin / script all share the same invocation syntax and the same visual feedback |

---

## 1. Color

Microsoft guideline: *Use color to establish hierarchy, meaning, and visual identity*

### 1.1 Semantic Colors

| Purpose | Recommended (Light) | Recommended (Dark) | Meaning |
|------|--------------|--------------|------|
| **Accent** | `#0078D4` (Windows Blue) | `#60CDFF` | Selected state, in progress |
| **Success** | `#107C10` | `#6CCB5F` | Match succeeded, step completed |
| **Warning** | `#C19C00` | `#FCE100` | Approaching timeout, near threshold lower bound |
| **Error** | `#C42B1C` | `#FF99A4` | Match failed, step error |
| **Overlay BG** | `rgba(0,0,0,0.55)` | — | Frozen overlay background for the snip tool |
| **Selection** | `rgba(0,120,212,0.35)` | `rgba(96,205,255,0.35)` | Snip rectangle selection |

### 1.2 Color Usage Rules

- Do not overuse color beyond step icons and status badges (maximum 3 accent colors)
- Text on background must meet **WCAG AA or higher** (contrast ratio ≥ 4.5:1)
- Do not convey error state through accent color alone — always combine with an icon and text

### 1.3 Theme System — Dark / Light Mode

robost supports two visual themes. **Light mode is the default.**

| Property | Light | Dark |
|----------|-------|------|
| Base visuals | `egui::Visuals::light()` | `egui::Visuals::dark()` |
| Default | ✅ yes | — |
| Semantic tokens | Adapted (see §1.1 Light column) | Adapted (see §1.1 Dark column) |
| Canvas background | `#F5F5F5` | `#1A1B1E` |
| Node background | `#FFFFFF` (with shadow) | `#282829` |

#### Toggle

A **☀ / 🌙 icon button** is placed in the toolbar area (right side). Clicking it toggles between Light and Dark. The selected theme is persisted in `settings.toml` under the `theme` key.

#### Implementation

```rust
// settings.rs
#[derive(Default, Serialize, Deserialize, Clone, PartialEq)]
pub enum Theme { #[default] Light, Dark }

// main.rs — apply_style()
let base = match settings.theme {
    Theme::Light => egui::Visuals::light(),
    Theme::Dark  => egui::Visuals::dark(),
};
style.visuals = base;
// Then overlay semantic tokens (ACCENT, SUCCESS, etc.) on top.
```

#### Rules

- Custom semantic colors (Accent, Success, Warning, Error) are applied **on top of** the base visuals, not instead of them. This ensures all egui built-in widgets (scrollbars, separators, etc.) also adapt correctly.
- Snip tool overlay is always dark regardless of theme (the frozen screenshot must be clearly distinguishable from the overlay).
- Do not hardcode `Color32::WHITE` / `Color32::BLACK` in widget code — reference tokens or `ui.visuals()` instead.

---

## 2. Typography

Microsoft guideline: *Set tone and improve readability through consistent typefaces and hierarchy*

**Phase 1 (current):** CJK font support via OS-path probing (`setup_fonts()` searches standard system font directories on macOS, Windows, and Linux). This avoids adding a large binary asset (~4 MB) to the repository in the early phase.

**Phase 2:** Embed **Noto Sans / Noto Sans JP** using `include_bytes!()` so the binary is fully self-contained. The font file should be stored in `crates/robost-editor/assets/NotoSansJP-Regular.ttf` and committed to the repo.

### 2.1 Type Scale

| Role | Size | Weight | Usage |
|--------|--------|---------|------|
| `Display` | 28 px | SemiBold | Window title |
| `Title` | 20 px | SemiBold | Panel heading |
| `Body` | 14 px | Regular | General text, labels |
| `Caption` | 12 px | Regular | Metadata, hints |
| `Monospace` | 13 px | Regular | Scenario YAML, log output |

### 2.2 Rules

- UI labels use sentence case (capitalize only the first letter). `"Save File"` ✗ → `"Save file"` ✓
- When wrapping long text, use `line-height: 1.5` as the baseline
- Scenario YAML is always displayed in monospace. Never use a proportional font.

---

## 3. Geometry & Layout

Microsoft guideline: *Create a balanced and predictable layout through shape, size, and spatial relationships*

### 3.1 Grid and Spacing

- Base unit: **4 px**
- Element spacing: one of 4 / 8 / 12 / 16 / 24 / 32 px
- Corner radius: controls 4 px, panels 8 px, modals 12 px
- Panel minimum widths (scenario editor):
  - Node palette panel: min 200 px, default 220 px
  - Step list panel: min 240 px (DESIGN.md §3.1 general guideline)
  - Inspector / central area: remaining width

### 3.2 Component Sizes

| Component | Recommended Height | Notes |
|--------------|---------|------|
| Button (primary) | 32 px | min-width: 80 px |
| Text field | 32 px | |
| Step row (scenario editor) | 32 px | Compact density; drag handle overlay on hover |
| Tray menu item | 32 px | |
| Snip selection handle | 10×10 px | For visibility |

### 3.3 Snip Overlay Specifics

- The overlay covers **all monitors**
- The frozen capture image is displayed at 1:1 pixels (no scaling)
- Selection rectangle dimensions (w × h px) are shown in real time near the cursor
- The toolbar is positioned at the **bottom center of the desktop** to avoid obstructing the target UI

---

## 4. Elevation

Microsoft guideline: *Use depth and layers to guide focus and reinforce structure*

Since robost uses egui, Mica/Acrylic materials are unavailable. Depth is expressed instead via **shadows and border colors**.

| Layer | Expression | Used For |
|---------|------|---------|
| L0: Background | `bg_color` | Application background |
| L1: Panel | `bg_color + 10` (brightness offset) | Side panels, toolbar |
| L2: Card | `bg_color + 20` + 1px shadow | Step cards, settings panel |
| L3: Popover | `bg_color + 30` + 4px shadow | Dropdowns, tooltips |
| L4: Modal | dim overlay + 8px shadow | Confirmation dialogs |

---

## 5. Motion

Microsoft guideline: *Use motion for feedback and directing attention to create a smooth interaction feel*

**Core policy: optional animations are OFF by default.** Avoid adding visual noise during RPA execution.

### 5.1 Duration Guidelines

| Type | Duration | Easing |
|------|------|---------|
| Slide in/out | 150 ms | `ease-out` |
| Fade in/out | 100 ms | linear |
| Progress bar update | — | linear |
| Scroll | Immediate | — |

### 5.2 Snip Tool Timing Constraint

> Capture → overlay display: **within 50 ms**

- From hotkey received to capture complete: ≤ 30 ms
- egui window creation and rendering: ≤ 20 ms
- Do not add any animation (e.g., fade-in) to the snip overlay that would break this constraint

---

## 6. Navigation

Microsoft guideline: *Guide users through a predictable structure*

In the current phase, GUI components follow a **single-window, flat structure** as the baseline.

### 6.1 Scenario Editor

```
┌─────────────────────────────────────────────┐
│ Toolbar (run / stop / save / snip)          │
├────────────┬────────────────────────────────┤
│ Step List  │  Step Inspector (selected step) │
│ (left)     │  (right)                        │
└────────────┴────────────────────────────────┘
```

- The selected step is reflected immediately in the Inspector — no separate window opens
- `Esc` deselects, `Delete` removes the step, `Ctrl+Z` undoes

### 6.2 Tray Menu Structure

```
robost
├── Open Scenario Editor
├── New Template (snip)
├── Recent Scenarios >
├── ─────────────────
├── Settings
└── Quit
```

- Maximum 2 levels of nesting. Anything deeper moves into the editor.

---

## 7. Commanding

Microsoft guideline: *Present what users can do through clear, consistent patterns*

### 7.1 Global Shortcuts (always active while tray app is running)

| Key | Action |
|------|------|
| `Ctrl+Shift+C` | Launch snip tool (template capture) |
| `Ctrl+Shift+S` | Launch / bring scenario editor to front |

### 7.2 Inside Snip Overlay

| Key | Action |
|------|------|
| `Escape` | Cancel snip, close overlay |
| `Enter` | Confirm selection, proceed to save dialog |
| `Space` | Toggle mask region mode |

### 7.3 Inside Scenario Editor

| Key | Action |
|------|------|
| `Ctrl+R` | Run scenario |
| `Ctrl+S` | Save |
| `Ctrl+Z` / `Ctrl+Shift+Z` | Undo / Redo |
| `Del` | Delete selected step |
| `↑` / `↓` | Move step selection |
| `Ctrl+↑` / `Ctrl+↓` | Reorder steps |

---

## 8. Iconography

Microsoft guideline: *Convey actions and concepts quickly with familiar, purposeful icons*

- Use **Fluent System Icons** ([microsoft/fluentui-system-icons](https://github.com/microsoft/fluentui-system-icons))
- Icon sizes: 16 px (toolbar with label), 20 px (toolbar icon only), 24 px (large buttons)
- Custom icons, when needed, are created as SVG (single color, consistent stroke) and stored in `assets/icons/`

### 8.1 Step Type Icon Mapping

| Step Type | Icon Name | Color |
|------------|----------|-----|
| `click_image` | `cursor_click` | Accent |
| `wait_image` | `timer` | Warning |
| `type` | `keyboard` | neutral |
| `press` | `keyboard` | neutral |
| `script` | `code` | neutral |
| `library` (plugin) | `puzzle` | `#7B68EE` (Plugin Purple) |
| `foreach` | `arrow_repeat` | neutral |

---

## 9. Usability

Microsoft guideline: *Ensure intuitive operation, clear affordances, and accessibility*

### 9.1 Accessibility

- All features must be operable by keyboard alone (set correct Tab order)
- Enable egui's `AccessKit` integration
- Always show focus rings. Use distinct styles for `:hover` and `:focus`
- Communicate errors and warnings with icon + text (do not rely on color alone)

### 9.2 Interactive States

Every control must define styles for the following 4 states:

| State | Visual Change |
|------|---------|
| Default | Base style |
| Hover | Background brightness +10%, cursor `pointer` |
| Pressed | Background brightness -10%, 2px inset |
| Disabled | opacity 0.38, cursor `not-allowed` |

### 9.3 Errors and Feedback

- On match failure: immediately display the **failure screenshot + match score** in the error panel
- Always show progress feedback during processing (indeterminate progress bar or spinner)
- Destructive operations (template overwrite, step deletion) require a confirmation dialog
- Scenario completion / failure is reported via tray balloon notification

---

## 10. Materials

Microsoft guideline: *Add depth and warmth with Mica, Acrylic, and similar materials*

OS-native materials are not available in egui, but the following substitutes are used:

- **Header background**: Accent color at 10% opacity for a subtle gradient
- **Panel dividers**: 1 px separator (`stroke_color`)
- **Scrollbar**: Narrow (4 px), visible only on hover

If the editor is migrated to a WebView2 / WinUI base in the future, Mica Base will be adopted.

---

## 11. Widgets

Microsoft guideline: *Provide interactive surfaces that convey key information at a glance*

Windows Widgets are not implemented in the current phase, but the following serve as design principles:

- Each step card has self-contained text that **communicates intent in a single line**
- Template images are always shown as thumbnails (48×48 px) in the preview
- Match score is displayed simultaneously as a number and a progress bar

---

## 12. Writing

Microsoft guideline: *Reduce cognitive load and aid understanding through clear, concise, and friendly language*

### 12.1 Core Policy

- UI text is designed with **both Japanese and English support** in mind (UI strings are isolated in `i18n/`)
- Button labels start with a verb: `"Save"` / `"Cancel"` / `"Run scenario"`
- Do not write error messages that blame the user

### 12.2 Good vs. Bad Examples

| Situation | Bad | Good |
|------|----|----|
| Match failure | "Error: template not found" | "Template not found — try lowering the threshold or recapturing" |
| Save complete | "Done." | "Saved to `templates/login_button.png`" |
| Delete confirmation | "Are you sure?" | "Delete 'login_button' template? This can't be undone." |
| Plugin permission | "Permission required" | "'excel-reader' needs filesystem read access. Allow?" |

### 12.3 Tone

- Polite but concise. Prefer imperative form over verbose "please do X" phrasing.
- Always include a **suggested next action** with error messages
- Use technical terms as-is (template, threshold, OCR). The audience is RPA practitioners — no need to simplify.

---

## Appendix A: egui Implementation Notes

- Define tokens centrally in `egui::Style` — do not hardcode values
- Use `egui::Context::set_pixels_per_point()` to correctly reflect DPI
- **Font embedding** (Phase 2): Embed font files in the binary via `include_bytes!()`. In Phase 1, OS-path probing is used.
- Use `egui_extras`'s `TableBuilder` for virtualized scrolling in the step list (to support large scenarios)
- **AccessKit**: eframe 0.34 integrates AccessKit automatically via the winit backend. No additional configuration is required in `NativeOptions`. Keyboard navigation and screen reader support are available by default.

## Appendix B: Design Tokens (Provisional)

```rust
// crates/robost-ui/src/tokens.rs (tentative)
pub const ACCENT:       egui::Color32 = egui::Color32::from_rgb(0x00, 0x78, 0xD4);
pub const SUCCESS:      egui::Color32 = egui::Color32::from_rgb(0x10, 0x7C, 0x10);
pub const WARNING:      egui::Color32 = egui::Color32::from_rgb(0xC1, 0x9C, 0x00);
pub const ERROR:        egui::Color32 = egui::Color32::from_rgb(0xC4, 0x2B, 0x1C);
pub const PLUGIN_PURPLE: egui::Color32 = egui::Color32::from_rgb(0x7B, 0x68, 0xEE);

pub const SPACING_XS: f32 = 4.0;
pub const SPACING_SM: f32 = 8.0;
pub const SPACING_MD: f32 = 16.0;
pub const SPACING_LG: f32 = 24.0;

pub const ROUNDING_SM: egui::Rounding = egui::Rounding::same(4.0);
pub const ROUNDING_MD: egui::Rounding = egui::Rounding::same(8.0);

pub const STEP_ROW_HEIGHT: f32 = 48.0;
pub const TOOLBAR_HEIGHT:  f32 = 40.0;
```
