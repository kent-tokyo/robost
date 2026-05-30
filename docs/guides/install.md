# Installation

## Prerequisites

- Rust stable (≥ 1.75)
- On Windows: Visual Studio Build Tools
- On macOS: Xcode command-line tools
- For OCR: Tesseract (`brew install tesseract` / `apt install tesseract-ocr`)

## Build from source

```bash
git clone https://github.com/kent-tokyo/robost
cd robost
cargo build --release
```

Binaries are placed in `target/release/`:

| Binary | Description |
|--------|-------------|
| `robost-editor` | Visual scenario editor |
| `robost-cli` | Command-line runner |
| `robost-snip` | Template capture tool (tray app) |

## Run the editor

```bash
cargo run -p robost-editor
```

## Run a scenario from CLI

```bash
cargo run -p robost-cli -- run scenario.yaml
```
