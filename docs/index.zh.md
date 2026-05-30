# robost

基于 Rust 的 RPA 自动化工具，支持图像识别、OCR 和 WASM 插件。

## 什么是 robost？

robost 无需访问 UI 框架即可自动化桌面工作流——它通过捕获屏幕并回放鼠标/键盘输入，支持 RDP/Citrix/VNC 会话。场景以纯 YAML 定义。

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

## 主要特性

- **基于图像的自动化** — 模板匹配、OCR、ML 检测
- **远程桌面支持** — 通过外部捕获方式支持 RDP/Citrix/VNC
- **WASM 插件** — 使用任意语言的沙箱插件进行扩展
- **YAML 场景** — 人类可读，对版本控制友好
- **可视化编辑器** — 拖拽式场景构建器，支持实时预览

## 快速导航

- [安装](guides/install.md)
- [快速入门](guides/quickstart.md)
- [步骤参考](steps/control_flow.md)
