# rust_rpa

A Rust-based OSS desktop automation (RPA) tool.

## Key Features

- **Image recognition automation** — multi-scale NCC template matching, OCR (Tesseract), ML detection
- **Remote desktop support** — operates RDP/Citrix/VNC sessions via external screen capture
- **Enterprise-grade template capture UX** — captures transient UI (dropdowns, tooltips) via hotkey freeze; anchor/mask/multi-scale support
- **WASM plugin extensibility** — sandboxed community plugins with explicit permission declarations
- **Rich scenario format** — YAML with variables, flow control, data sources, inline scripts, sub-scenarios

## Comparison with Other Open-Source Automation Tools

| Feature | **rust_rpa** | PyAutoGUI | SikuliX | Robot Framework |
|---|---|---|---|---|
| License | MIT / Apache-2.0 | MIT | MIT | Apache-2.0 |
| Language | Rust (YAML scenarios) | Python | Java (Jython scripts) | Python |
| Remote desktop (RDP/Citrix/VNC) | Yes — no agent needed | No | No | No |
| Image recognition | Yes — multi-scale NCC | No | Yes — pixel-exact | No (via plugins) |
| Transient UI capture (dropdowns, tooltips) | Yes — freeze + overlay | No | No | No |
| Multi-scale DPI resilience (125%/150%) | Yes — built-in | No | No | No |
| Plugin sandbox | Yes — WASM (memory-safe) | No | No | No |
| Cross-platform development | Yes — macOS/Linux/Windows | Yes | Yes | Yes |
| Scenario version control | Yes — plain YAML | Yes — Python | Partial — `.sikuli` dirs | Yes — plain text |
| Startup overhead | ~10 ms (native binary) | Python startup | JVM startup (~2 s) | Python startup |
| Inline scripting | Yes — Rhai (sandboxed) | Python itself | Jython | Python |
| OCR support | Yes (Tesseract, optional) | No | Partial | No (via plugins) |

## Why rust_rpa?

**For teams migrating from commercial RPA tools**
rust_rpa covers the same node vocabulary found in mainstream commercial RPA products (click_image, wait_image, foreach, dialog_input, …) making scenario migration straightforward. Scenarios are plain YAML — reviewable in PRs, diffable, and storable in Git without proprietary tooling.

**For RDP / remote desktop automation**
rust_rpa requires no agent installed on the remote machine. It operates by capturing the RDP window on the local machine and sending input via enigo — the same technique works for Citrix, VNC, and any windowed session. Multi-scale NCC matching handles DPI scaling (100/125/150%) that breaks pixel-perfect tools.

**For engineering teams**
- **Zero license cost** — no per-bot or per-user fees. Run as many concurrent workers as needed.
- **Git-native** — YAML scenarios are text files; `git diff` shows exactly what changed between runs.
- **Composable** — sub-scenarios, variables, inline Rhai scripts, and WASM plugins share one uniform call syntax.
- **Safe by default** — WASM plugins are sandboxed; a crashing plugin cannot bring down the runner process.
- **Fast startup** — the Rust binary starts in milliseconds; no JVM or .NET runtime warm-up.

**For open-source contributors**
The WASM plugin interface (`rpa-plugin-api`) decouples the runner from node implementations. Community plugins written in Rust, AssemblyScript, Go, or C compile to `.wasm` and integrate without forking the core. Permissions are declared in `plugin.toml` and enforced at runtime — not just documented.

## Architecture

```
crates/
├── rpa-capture/      # Screen/window capture (xcap, DPI-aware)
├── rpa-input/        # Mouse/keyboard input + window focus (enigo)
├── rpa-vision/       # Template matching (NCC), OCR, ML detection
├── rpa-backend/      # Backend trait: Local / RDP / VNC unified
├── rpa-core/         # Scenario engine: YAML parsing, step execution, retry, flow control
├── rpa-snip/         # Template capture GUI (tray app, hotkey, overlay, Japanese UI)
├── rpa-editor/       # Visual scenario editor (step list + YAML, dark theme, log panel)
├── rpa-template/     # Shared coordinate/geometry types
├── rpa-plugin-api/   # Public plugin author API (crates.io publish candidate)
├── rpa-plugin-host/  # wasmtime-based WASM plugin runner with epoch timeout
├── rpa-script/       # Rhai inline scripting (sandboxed)
├── rpa-stdlib/       # Built-in scenario node library
└── rpa-cli/          # CLI binary
```

## Quick Start

```bash
cargo build --workspace
cargo run -p rpa-cli -- run scenario.yaml
```

## Scenario Format

```yaml
name: "example"
target:
  kind: window
  title_contains: "MyApp"
variables:
  retry_count: 0
steps:
  # Image operations
  - wait_image:  { template: login_button.png, timeout_ms: 5000 }
  - click_image: { template: login_button.png, action: left, offset_x: 0, offset_y: 0 }
  - find_image:  { template: icon.png, save_as: pos }  # {found, x, y, score}
  - match_rect:
      template: badge.png
      rect: { x: 100, y: 200, width: 300, height: 100 }
      save_as: result

  # OCR (requires Tesseract + --features ocr)
  - ocr_match:
      contains: "Login"
      lang: "jpn+eng"
      timeout_ms: 5000
      save_as: ocr_result   # {found, text}

  # Input
  - type: "username"
  - type: { secret_env: PASSWORD }
  - press: Tab

  # Variables
  - set:          { name: count, value: 0 }
  - increment:    { name: count, by: 1 }
  - copy_var:     { from: src, to: dst }
  - get_datetime: { format: "%Y%m%d", save_as: today }
  - get_username: { save_as: user }
  - calc:         { expr: "count * 2", save_as: doubled }
  - to_fullwidth: { value: "abc", save_as: full }
  - to_halfwidth: { value: "ａｂｃ", save_as: half }

  # Clipboard
  - clipboard_set: { value: "{{ text }}" }
  - clipboard_get: { save_as: copied }

  # Shell
  - shell: { cmd: python3, args: [script.py], save_as: output, timeout_ms: 30000 }

  # Flow control
  - if:
      cond: "count > 10"
      then: [ { press: Escape } ]
      else: [ { wait_ms: 500 } ]
  - switch:
      on: status
      cases:
        - when: "ok"
          do: [ { click_image: { template: ok.png } } ]
      default: [ { press: Escape } ]
  - repeat:  { count: 3, do: [ { wait_ms: 1000 } ] }
  - while:   { cond: "found", do: [ { wait_image: { template: spinner.png } } ] }
  - foreach: { var: __rows__, do: [ { type: "{{ name }}" } ] }
  - try_catch:
      try:   [ { click_image: { template: btn.png } } ]
      catch: [ { set: { name: _error, value: "failed" } } ]
      finally: [ { wait_ms: 100 } ]
  - group:   { name: "login block", do: [ { type: "user" } ] }
  - break
  - continue
  - exit

  # User interaction (CLI: stdin; silent mode: uses defaults)
  - dialog_wait:   { message: "Check the screen, then press Enter.", title: "Waiting" }
  - dialog_input:  { message: "Enter filename:", default: "output.xlsx", save_as: fname }
  - dialog_select: { message: "Choose action:", options: [Save, Skip, Abort], save_as: choice }

  # Screenshot / observation
  - screenshot_save: { path: "caps/{{ today }}.png" }                    # full screen
  - screenshot_save: { path: "caps/win.png", window: "MyApp" }           # specific window
  - wait_no_image:   { template: spinner.png, timeout_ms: 30000 }        # wait until gone

  # System integration
  - url_open: { url: "https://example.com/report" }
  - notify:   { title: "Done", message: "{{ count }} rows processed" }

  # Window
  - wait_window:    { title_contains: "MyApp", state: exists, timeout_ms: 10000 }
  - window_control: { title_contains: "Notepad", action: focus }  # focus|maximize|minimize|close

  # Log
  - log_write: { file: run.log, message: "step {{ count }} done", level: info }  # info|warn|error|debug

  # File operations
  - file_exists:  { path: data.csv, save_as: exists }
  - file_copy:    { src: a.txt, dst: b.txt }
  - file_move:    { src: tmp.txt, dst: archive/tmp.txt }
  - file_delete:  { path: old.txt }
  - file_rename:  { path: a.txt, new_name: b.txt }
  - file_list:    { pattern: "logs/*.log", save_as: files }
  - file_read:    { path: notes.txt, save_as: content }
  - file_write:   { path: out.txt, content: "{{ result }}", mode: overwrite }  # overwrite|append
  - file_append:  { path: out.txt, content: "{{ line }}\n" }

  # Process operations
  - process_start:  { name: notepad.exe, wait_ms: 500 }
  - process_kill:   { name: notepad.exe }
  - process_exists: { name: notepad.exe, save_as: running }

  # Date operations
  - date_format: { value: "{{ today }}", format: "%Y/%m/%d", save_as: formatted }
  - date_add:    { value: "{{ today }}", days: 7, save_as: next_week }
  - date_diff:   { from: "{{ start }}", to: "{{ end }}", unit: days, save_as: elapsed }

  # String operations
  - string_replace:   { value: "{{ text }}", from: "old", to: "new", save_as: result }
  - string_trim:      { value: "  hello  ", save_as: trimmed }
  - string_upper:     { value: "{{ text }}", save_as: upper }
  - string_lower:     { value: "{{ text }}", save_as: lower }
  - string_substring: { value: "{{ text }}", start: 0, end: 5, save_as: sub }
  - string_length:    { value: "{{ text }}", save_as: len }
  - string_split:     { value: "a,b,c", sep: ",", save_as: parts }
  - string_join:      { values: "{{ parts }}", sep: ", ", save_as: joined }
  - string_regex:     { value: "{{ text }}", pattern: "\\d+", save_as: match }

  # JSON / Path / Env
  - json_parse:     { value: "{\"k\":1}", save_as: obj }
  - json_stringify: { value: "{{ obj }}", save_as: json_str }
  - path_join:      { parts: ["dir", "sub", "file.txt"], save_as: full_path }
  - path_basename:  { path: "/dir/file.txt", save_as: name }
  - path_dirname:   { path: "/dir/file.txt", save_as: dir }
  - env_get:        { name: HOME, save_as: home_dir }

  # Mouse coordinate operations
  - mouse_move:     { x: 500, y: 300 }
  - mouse_click_xy: { x: 500, y: 300, button: left }  # left|right|double
  - mouse_drag:     { from_x: 100, from_y: 100, to_x: 400, to_y: 400, hold_ms: 100 }
  - mouse_scroll:   { direction: down, amount: 3 }    # up|down|left|right

  # Key combination
  - key_combo: { keys: [ctrl, c] }           # Ctrl+C
  - key_combo: { keys: [ctrl, shift, tab] }  # Ctrl+Shift+Tab

  # CSV operations
  - csv_read:  { path: data.csv, has_header: true, save_as: rows }
  - csv_write: { path: out.csv, rows: "{{ rows }}", mode: overwrite }  # overwrite|append

  # HTTP (requires feature = "http")
  - http_get:  { url: "https://api.example.com/items", save_as: resp }
  - http_post: { url: "https://api.example.com/items", body: "{{ payload }}", save_as: resp }
  - http_put:  { url: "https://api.example.com/items/1", body: "{{ payload }}", save_as: resp }

  # Excel (requires feature = "excel-write")
  - excel_read_cell:  { path: data.xlsx, sheet: Sheet1, row: 2, col: 1, save_as: cell_val }
  - excel_read_range: { path: data.xlsx, sheet: Sheet1, start_row: 2, end_row: 10, save_as: range }
  - excel_write_cell: { path: data.xlsx, sheet: Sheet1, row: 2, col: 1, value: "{{ result }}" }

  # Variable persistence
  - import_vars: { path: params.xlsx, row: 2 }
  - save_vars:   { path: state.json, vars: [count, status] }
  - load_vars:   { path: state.json }

  # Sub-scenarios & scripts
  - sub_scenario:   { path: sub/login.yaml, inputs: { user: "{{ user }}" } }
  - call_scenario:  { path: "{{ path }}", save_as: result }
  - script:         { script: "let d = now(); d.format(\"%Y%m%d\")", save_as: date }
  - library:        { name: "excel-reader.read_sheet", inputs: { path: data.xlsx }, save_as: rows }
```

## Data Source

Load Excel/CSV row-by-row; column headers become variable names:

```yaml
data_source:
  file: data.xlsx
  sheet: Sheet1
steps:
  - foreach: { var: __rows__, do: [ { type: "{{ 氏名 }}" } ] }
```

Export results after run:

```bash
cargo run -p rpa-cli -- run scenario.yaml --export result.xlsx
```

## Template Capture (rpa-snip)

1. `cargo run -p rpa-snip` — starts as a tray app (no window, no focus steal)
2. Open the target UI (dropdown, dialog, tooltip, etc.)
3. Press **Ctrl+Shift+C** (or use tray menu) — freezes the screen into a fullscreen overlay
4. Drag to select the template region
5. Optionally add **anchor points** (click reference targets) and **mask regions** (exclude dynamic areas like timestamps)
6. Press **▶ Match test** to verify the match against the frozen screen
7. **Save** — PNG + metadata YAML written to `templates/`; multi-scale variants (125%, 150%) generated automatically

## Plugin System

Plugins are `.wasm` + `plugin.toml` pairs. They run in a WASM sandbox; permissions must be declared.

```bash
# Build a plugin (separate workspace)
cargo build -p my-plugin --target wasm32-wasip2

# Install with permission review
cargo run -p rpa-cli -- plugin install ./my-plugin.wasm

# Auto-approve
cargo run -p rpa-cli -- plugin install ./my-plugin.wasm -y

# Use in a scenario
# - library: { name: "my-plugin.function", inputs: { key: value }, save_as: result }
```

## CLI Reference

```
rpa run <scenario.yaml> [OPTIONS]

  --from <N>         Start at step N (0-based)
  --steps <S..E>     Run step range, e.g. "2..5"
  --data <path>      Override data_source file
  --export <path>    Export __rows__ after run (.csv or .xlsx)
  --silent           Auto-answer all dialogs with defaults
  --wait-ms <ms>     Wait N ms before starting
  --exit             Exit process when done

rpa plugin install <path.wasm> [-y]
rpa plugin list
```

## OCR Feature

OCR requires Tesseract to be installed on the host:

```bash
# macOS
brew install tesseract tesseract-lang

# Ubuntu / Debian
sudo apt install tesseract-ocr tesseract-ocr-jpn tesseract-ocr-eng

# Windows: https://github.com/UB-Mannheim/tesseract/wiki
```

Build with the `ocr` feature:

```bash
cargo build --features rpa-core/ocr
```

## Development Commands

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all

cargo run -p rpa-snip          # Template capture tool
cargo run -p rpa-editor        # Visual scenario editor
```

## Published Crates

| Crate | Version | Description |
|---|---|---|
| [rpa-vision](https://crates.io/crates/rpa-vision) | 0.1.0 | Multi-scale NCC template matching + OCR for desktop automation |

## License

MIT OR Apache-2.0
