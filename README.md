# robost

**robost** = **robo**t + **ro**bu**st** + **Rust**

A Rust-based OSS desktop automation (RPA) tool.

[![CI](https://github.com/kent-tokyo/robost/actions/workflows/ci.yml/badge.svg)](https://github.com/kent-tokyo/robost/actions/workflows/ci.yml)
[![Security audit](https://github.com/kent-tokyo/robost/actions/workflows/security-audit.yml/badge.svg)](https://github.com/kent-tokyo/robost/actions/workflows/security-audit.yml)
[![Release](https://github.com/kent-tokyo/robost/actions/workflows/release.yml/badge.svg)](https://github.com/kent-tokyo/robost/actions/workflows/release.yml)
[![Docs](https://github.com/kent-tokyo/robost/actions/workflows/docs.yml/badge.svg)](https://github.com/kent-tokyo/robost/actions/workflows/docs.yml)

[日本語](README_ja.md) | [中文](README_zh.md) | [Documentation](https://kent-tokyo.github.io/robost/)

## Visual Scenario Editor

| Canvas View — step flowchart | YAML Editor — direct edit with live canvas |
|:---:|:---:|
| ![Canvas View](assets/screenshots/editor_canvas_new.png) | ![YAML Editor](assets/screenshots/editor_yaml_new.png) |

| AI Assistant — describe automation in natural language | CLI Help |
|:---:|:---:|
| ![AI Assistant](assets/screenshots/editor_ai_new.png) | ![CLI Help](assets/screenshots/cli_help.png) |

## Download

> **Latest release**: [GitHub Releases](https://github.com/kent-tokyo/robost/releases/latest)

### Windows — Installer (recommended)

**[⬇ robost-setup.exe](https://github.com/kent-tokyo/robost/releases/latest/download/robost-setup.exe)** — double-click to install. No extra dependencies required.

- Installs to `Program Files\robost`, creates Start Menu and Desktop shortcuts
- Launch from the shortcut → browser opens automatically to the visual editor
- Uninstall cleanly via Windows Settings → Apps

> **SmartScreen warning**: Windows may show "Windows protected your PC" because the installer is not code-signed.
> Click **More info → Run anyway** to proceed. This is standard for open-source software without a paid signing certificate.

### macOS

| Platform | Download |
|---|---|
| macOS (Apple Silicon) | [rpa-aarch64-apple-darwin.tar.gz](https://github.com/kent-tokyo/robost/releases/latest/download/rpa-aarch64-apple-darwin.tar.gz) |

### Windows — Portable ZIP

| Platform | Download |
|---|---|
| Windows | [rpa-x86_64-windows.zip](https://github.com/kent-tokyo/robost/releases/latest/download/rpa-x86_64-windows.zip) |

- Extract and **double-click `rpa.exe`** — browser opens automatically to the visual editor
- No installation required; runs from any folder

## Key Features

- **Image recognition** — NCC template matching (multi-scale), OCR via Tesseract or Windows built-in WinRT (no install needed)
- **Remote desktop** — captures the RDP/Citrix/VNC window locally; no agent needed on the target machine
- **Transient UI capture** — hotkey freezes the screen so you can select dropdowns and tooltips that would otherwise disappear
- **WASM plugins** — sandboxed extensions; a crashing plugin can't bring down the runner
- **Plain YAML scenarios** — variables, loops, branches, inline Rhai scripts, sub-scenarios, data sources
- **Visual editor** — list and Canvas view (free-placement, zoom/pan, minimap, snap, align/distribute), AI step creation from natural language, AI scenario assistant (Anthropic/OpenAI), full i18n (EN/JA/ZH)

## Comparison with Automation Tools

| Feature | **robost** | WinActor | UiPath | PyAutoGUI | SikuliX | Robot Framework |
|---|---|---|---|---|---|---|
| License | MIT / Apache-2.0 | Commercial | Commercial | MIT | MIT | Apache-2.0 |
| Language | Rust (YAML scenarios) | Proprietary GUI | Proprietary GUI | Python | Java (Jython) | Python |
| Open source | Yes | No | No | Yes | Yes | Yes |
| Remote desktop (RDP/Citrix/VNC) | Yes — no agent needed | Yes | Yes (agent required) | No | No | No |
| Image recognition | Yes — multi-scale NCC | Yes | Yes — AI-assisted | No | Yes — pixel-exact | No (via plugins) |
| Web browser automation | Yes — WebDriver | Yes | Yes | No | No | Yes (via SeleniumLibrary) |
| Excel automation | Yes — cell/sheet/formula | Yes | Yes | No | No | No (via plugins) |
| Word / PowerPoint | — Phase 2 | Yes | Yes | No | No | No |
| Scenario recorder | — Phase 2 | Yes | Yes | No | No | No |
| Transient UI capture (dropdowns, tooltips) | Yes — freeze + overlay | Yes | Partial | No | No | No |
| Multi-scale DPI resilience (125%/150%) | Yes — built-in | Partial | Partial | No | No | No |
| WASM plugin sandbox | Yes — memory-safe | No | No | No | No | No |
| Inline scripting | Yes — Rhai (sandboxed) | Partial | VB.NET / C# | Python itself | Jython | Python |
| Scenario version control | Yes — plain YAML | No | Partial | Yes — Python | Partial | Yes — plain text |
| Startup overhead | ~10 ms (native binary) | Several seconds | Several seconds | Python startup | JVM startup (~2 s) | Python startup |
| OCR support | Yes (Tesseract or Windows WinRT, optional) | Yes | Yes | No | Partial | No (via plugins) |

## Why robost?

The main reason to reach for robost over PyAutoGUI or SikuliX is **RDP/Citrix support without an agent**. It captures the remote desktop window on the local machine and sends input through enigo, so it works regardless of what's running on the other end. Multi-scale NCC matching also handles DPI scaling (100/125/150%) that breaks pixel-perfect tools — see [docs/benchmarks.md](docs/benchmarks.md) for the actual cost of that resilience.

The scenario format is close to WinActor's node vocabulary (`click_image`, `wait_image`, `foreach`, `dialog_input`, …), so migrating existing automations is fairly direct. Scenarios are plain YAML — readable in any text editor and diffable in git with no proprietary tooling involved.

The `robost` crate on crates.io is a documentation-only entry point — the actual functionality lives in `robost-core`, `robost-cli`, and the other workspace crates under `crates/`.

Plugins run in a WASM sandbox: permissions are declared in `plugin.toml` and enforced at runtime. A plugin can only access what it declared, and if it panics, the runner keeps going. Plugins can be written in Rust, AssemblyScript, Go, or C — anything that compiles to `.wasm`.

## Quick Start

### Windows — No build required

1. Download **[robost-setup.exe](https://github.com/kent-tokyo/robost/releases/latest/download/robost-setup.exe)** and install (or use the [portable ZIP](https://github.com/kent-tokyo/robost/releases/latest/download/rpa-x86_64-windows.zip))
2. Launch from the Desktop shortcut — browser opens automatically to the visual editor
3. Load an example scenario and run it:

```
rpa run examples\windows\notepad.yaml
rpa run examples\windows\calculator.yaml
```

### Inspect UI elements (Windows)

```
rpa inspect --window "メモ帳"       # list all UIA elements in a window
rpa inspect --hover 5               # hover cursor over element; captures after 5 seconds
rpa inspect --point 800 400         # element at specific screen coordinates
```

### Write a scenario

```yaml
# notepad_demo.yaml
steps:
  - shell: { command: "notepad.exe" }
  - wait: { ms: 1000 }
  - uia_click: { window: "メモ帳", by: { name: "ファイル" } }
  - uia_click: { window: "メモ帳", by: { name: "新規" } }
  - type_text: { text: "robost で自動入力しました" }
```

```
rpa run notepad_demo.yaml
rpa run notepad_demo.yaml --dry-run   # preview without executing
```

Full step reference: [Documentation → Step Reference](https://kent-tokyo.github.io/robost/)

## Template Capture (robost-snip)

1. `robost-snip.exe` (installed) or **from source:** `cargo run -p robost-snip` — starts as a tray app
2. Open the target UI (dropdown, tooltip, etc.)
3. **Ctrl+Shift+C** — freezes the screen into a fullscreen overlay
4. Drag to select the template region; press **Match test** to verify
5. **Save** — PNG + metadata YAML written to `templates/`; multi-scale variants generated automatically

## Plugin System

Plugins are `.wasm` + `plugin.toml` pairs running in a WASM sandbox.

```bash
cargo build -p my-plugin --target wasm32-wasip2
rpa plugin install ./my-plugin.wasm   # review permissions
# Use: - library: { name: "my-plugin.function", inputs: { key: value }, save_as: result }
```

## Development

```bash
cargo build --workspace
cargo test --workspace
cargo run -p robost-snip                                    # template capture tool
cargo run -p robost-cli --features embed-editor -- agent   # visual scenario editor (opens in browser)
```

All crates are published on [crates.io](https://crates.io/search?q=robost) (v0.1.2).

## Roadmap

| Phase | Status | Highlights |
|---|---|---|
| **Phase 1** | ✅ Complete | 200+ scenario nodes · CLI · visual editor (AI chat, DnD, i18n) · snip tool · Web/UIA/Excel/Mail/OCR/Scheduler · all crates on crates.io |
| **Phase 2** | 🔜 Planned | Scenario recorder · Word/SFTP/ML detection/Parallel execution/Registry/M365 |

**Phase 2 priorities** (internal planning, subject to change):
- 🔴 High: Word document automation, SFTP, more DB drivers (MySQL/PostgreSQL), stronger ML-based detection, OCR preprocessing, a `parallel` execution node, Windows registry/eventlog steps
- 🟡 Medium: Microsoft 365 / Google Workspace integration, PDF field extraction, screen recording, SAP GUI, rotation-invariant template matching
- 🟢 Low (Phase 3 candidate): Orchestrator, Process Mining, AI-assisted scenario generation, Test Suite

## Contributing

Issues and PRs welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) if it exists, otherwise open an issue first for large changes.

## Security

To report a vulnerability, open a GitHub issue or contact the maintainer directly.

## License

MIT OR Apache-2.0
