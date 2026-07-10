# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [0.1.3] - 2026-06-27

### Added
- `doctor` and `vision-bench` CLI commands, with match-confidence tracking
- `cargo publish` workflow for automated crates.io releases
- Agent mode as the default launch behavior
- Windows UIA improvements and a new `ocrs-cjk` OCR backend
- UIA hover inspector
- Dependabot, CodeQL, and security-audit (`cargo audit` + `npm audit`) workflows
- Scenario-level step defaults, `click_image.until`, improved `click_text` reliability, `--trace`/`--screenshots` CLI flags

### Fixed
- Engine bugs: `poll_match` deadline handling, `foreach` item variable, shell exit code propagation, `resolve_coord`
- Windows UIA security fixes

## [0.1.2] - 2026-06-24

"Windows casual-user support" release.

### Added
- Embedded web editor (`embed-editor` feature): the browser-based GUI is bundled into the `rpa` binary via `rust-embed`
- Static CRT linking on Windows (no separate VC++ runtime install required)
- Inno Setup installer (Program Files install, desktop/start-menu shortcuts, PATH registration, uninstaller)
- CI: installer + portable zip automatically attached to GitHub Releases on tag push
- SmartScreen warning guidance, hidden console window, multi-size app icon
- macOS build now also includes the embedded web editor

### Fixed
- PATH entry cleanup on uninstall

## [0.1.1] - 2026-06-04

### Added
- Major rewrite of the visual editor's Canvas view across 10 iterative rounds: zoom/pan, context-menu paste, snapping, lasso selection, minimap, align/distribute, multi-drag, confirmation dialogs
- `ai_create` step: natural-language prompt → AI-generated YAML scenario step
- `windows-ocr` feature: native Windows 10/11 WinRT OCR (no Tesseract dependency required)

### Changed
- Split the editor's 7,542-line `main.rs` into 12 focused modules
- CI: `cargo fmt --check` limited to the Linux runner (CJK character-width mismatch on other platforms), Node.js 24 opt-in, `workflow_dispatch` added to the release workflow

## [0.1.0] - 2026-05-30

Initial public release of the full workspace (all 16 crates).

### Added
- All 16 workspace crates published to crates.io
- Trilingual (EN/JA/ZH) documentation site published via GitHub Pages
- GitHub Releases workflow for cross-platform binaries
- Visual scenario editor (`robost-editor`): AI chat panel (Anthropic/OpenAI), multi-step selection, drag-and-drop, full node palette, EN/JA/ZH i18n
- Broad standard-library node coverage aimed at WinActor/UiPath parity: file operations, date/string utilities, HTTP, Excel (read/write/sheet management), Windows UI Automation, browser automation (fantoccini), mail (IMAP/SMTP), list operations, scheduler, webhook notifications, OS keychain integration
- `robost-vision`: multi-scale template matching and OCR (`leptess`) integration

Note: `robost-vision` shipped earlier as a standalone pre-1.0 crate (0.1.0 on 2026-05-17, 0.1.1 on 2026-05-21) before this first workspace-wide release.
