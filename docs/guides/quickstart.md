# Quick Start

## Windows — No build required

1. Download [rpa-x86_64-windows.zip](https://github.com/kent-tokyo/robost/releases/latest/download/rpa-x86_64-windows.zip)
2. Extract anywhere, e.g. `C:\Tools\robost`
3. Double-click **`rpa.exe`** — browser opens to the visual editor automatically

No Rust, Cargo, or Visual Studio required.

Run a scenario from the command line:

```
rpa run examples\windows\calculator.yaml
rpa run examples\windows\calculator.yaml --dry-run
```

> **Developers (build from source):** `cargo build --workspace` — see the [Development](#development) section below.

---

## 1. Launch the editor

**Installed / Portable ZIP:**

```
rpa.exe
```

**From source:**

```bash
cargo run -p robost-editor
```

## 2. Build your first scenario

1. Enter a **Scenario name** in the toolbar
2. Drag a node from the **Nodes** panel (left) into **Steps**, or double-click to append
3. Select a step to edit its properties in the center panel
4. Press **Save** (Cmd+S) to write the YAML file
5. Press **Run** (F5) to execute

## 3. Write YAML directly

robost scenarios are plain YAML. Create `hello.yaml`:

```yaml
name: hello
steps:
  - wait_ms: 500
  - type: "Hello from robost!"
  - press: Enter
```

Run it:

```
rpa run hello.yaml
```

## 4. Use variables

```yaml
name: with_variables
variables:
  target_app: "Notepad"
steps:
  - wait_window:
      title_contains: "{{ target_app }}"
      timeout_ms: 10000
  - type: "Automated by robost"
```

## 5. Capture a template image

1. Open the target application
2. Launch `robost-snip.exe` (installed) or **from source:** `cargo run -p robost-snip`
3. Press **Ctrl+Shift+C** to freeze the screen
4. Draw a rectangle around the UI element
5. The template PNG is saved to your project folder
