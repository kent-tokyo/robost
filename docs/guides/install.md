# Installation

> **Latest release**: [GitHub Releases](https://github.com/kent-tokyo/robost/releases/latest)

## Windows — installer (recommended)

**[⬇ robost-setup.exe](https://github.com/kent-tokyo/robost/releases/latest/download/robost-setup.exe)** — double-click to install. No extra dependencies required.

- Installs to `Program Files\robost`, creates Start Menu and Desktop shortcuts
- Launch from the shortcut → browser opens automatically to the visual editor
- Uninstall cleanly via Windows Settings → Apps

!!! warning "SmartScreen warning"
    Windows may show "Windows protected your PC" because the installer is not code-signed.
    Click **More info → Run anyway** to proceed. This is standard for open-source software without a paid signing certificate.

## Windows — portable ZIP

| Platform | Download |
|---|---|
| Windows | [rpa-x86_64-windows.zip](https://github.com/kent-tokyo/robost/releases/latest/download/rpa-x86_64-windows.zip) |

- Extract and **double-click `rpa.exe`** — browser opens automatically to the visual editor
- No installation required; runs from any folder

## macOS

| Platform | Download |
|---|---|
| macOS (Apple Silicon) | [rpa-aarch64-apple-darwin.tar.gz](https://github.com/kent-tokyo/robost/releases/latest/download/rpa-aarch64-apple-darwin.tar.gz) |

## Build from source

Prerequisites:
- Rust stable (≥ 1.75)
- On Windows: Visual Studio Build Tools
- On macOS: Xcode command-line tools
- For OCR: Tesseract (`brew install tesseract` / `apt install tesseract-ocr`)

```bash
git clone https://github.com/kent-tokyo/robost
cd robost
cargo build --release --features embed-editor
```

Binaries are placed in `target/release/`:

| Binary | Description |
|--------|-------------|
| `rpa` | CLI + agent — `agent` opens the visual editor in your browser |
| `robost-snip` | Template capture tool (tray app) |

## Run the editor

```bash
./target/release/rpa agent
```
Opens `http://localhost:9921` in your browser automatically (use `--no-browser` to skip that, or `--port` to change it).

## Run a scenario from CLI

```bash
./target/release/rpa run scenario.yaml
```
