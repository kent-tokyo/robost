# robost

[🇯🇵 日本語](https://kent-tokyo.github.io/robost/ja/) | [🇨🇳 中文](https://kent-tokyo.github.io/robost/zh/)

Rust-based RPA automation tool with image recognition, OCR, and WASM plugins.

## What is robost?

robost automates desktop workflows without needing UI framework access — it works with RDP/Citrix/VNC sessions by capturing the screen and replaying mouse/keyboard input. Scenarios are defined in plain YAML.

```yaml
name: login_example
steps:
  - wait_image:
      template: login_button.png
      timeout_ms: 5000
  - click_image:
      template: login_button.png
  - type: "myusername"
  - press: Tab
  - type:
      secret_env: PASSWORD
  - press: Enter
```

## Key features

- **Image-based automation** — template matching, OCR, ML detection
- **Remote desktop support** — works with RDP/Citrix/VNC via external capture
- **WASM plugins** — extend with sandboxed plugins in any language
- **YAML scenarios** — human-readable, version-control friendly
- **Visual editor** — drag-and-drop scenario builder with live preview

## Quick navigation

- [Installation](guides/install.md)
- [Quick Start](guides/quickstart.md)
- [Step Reference](steps/control_flow.md)
