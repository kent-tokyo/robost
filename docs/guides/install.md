# Installation

## Windows: installer or portable ZIP (no build required)

See the [Download section in the README](https://github.com/kent-tokyo/robost#download) — `robost-setup.exe` or a portable ZIP, no Rust toolchain needed.

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
