# robost

**robost** = **robo**t（机器人）+ **ro**bu**st**（健壮）+ **Rust**（编程语言）

基于 Rust 的开源桌面自动化 (RPA) 工具。

[English](README.md) | [日本語](README_ja.md) | [详细文档](https://kent-tokyo.github.io/robost/)

## 可视化场景编辑器

| Canvas 视图 — 步骤流程图 | YAML 编辑器 — 直接编辑与实时画布 |
|:---:|:---:|
| ![Canvas View](assets/screenshots/editor_canvas_new.png) | ![YAML Editor](assets/screenshots/editor_yaml_new.png) |

| AI 助手 — 用自然语言描述自动化 | CLI 帮助 |
|:---:|:---:|
| ![AI Assistant](assets/screenshots/editor_ai_new.png) | ![CLI Help](assets/screenshots/cli_help.png) |

## 下载

> **最新版本**: [GitHub Releases](https://github.com/kent-tokyo/robost/releases/latest)

### Windows — 安装程序（推荐）

**[⬇ robost-setup.exe](https://github.com/kent-tokyo/robost/releases/latest/download/robost-setup.exe)** — 双击安装，无需额外依赖。

- 安装到 `Program Files\robost`，自动创建开始菜单和桌面快捷方式
- 点击快捷方式后浏览器自动打开可视化编辑器
- 可通过 Windows「设置 → 应用」完整卸载

> **SmartScreen 警告**：由于安装程序未进行代码签名，Windows 可能显示「Windows 已保护你的电脑」。
> 请点击**「更多信息」→「仍要运行」**继续安装。这对于没有付费签名证书的开源软件来说是正常现象。

### macOS

| 平台 | 下载 |
|---|---|
| macOS (Apple Silicon) | [rpa-aarch64-apple-darwin.tar.gz](https://github.com/kent-tokyo/robost/releases/latest/download/rpa-aarch64-apple-darwin.tar.gz) |

### Windows — 便携 ZIP

| 平台 | 下载 |
|---|---|
| Windows | [rpa-x86_64-windows.zip](https://github.com/kent-tokyo/robost/releases/latest/download/rpa-x86_64-windows.zip) |

- 解压后**双击 `rpa.exe`** — 浏览器自动打开可视化编辑器
- 无需安装，可在任意目录运行

## 特性

- **图像识别** — 多尺度 NCC 模板匹配、Tesseract OCR 或 Windows 内置 WinRT OCR（无需安装）
- **远程桌面支持** — 在本地捕获 RDP/Citrix/VNC 窗口，目标机器无需安装代理
- **瞬态 UI 采集** — 热键冻结屏幕，可以选取平时会消失的下拉菜单和悬浮提示
- **WASM 插件** — 沙箱内运行，插件崩溃不影响主进程
- **纯 YAML 场景** — 支持变量、循环、分支、Rhai 内联脚本、子场景、数据源
- **可视化编辑器** — 列表和 Canvas 视图、从自然语言 AI 生成步骤、AI 场景助手（Anthropic/OpenAI）、完整多语言支持（EN/JA/ZH）

## 自动化工具对比

| 功能 | **robost** | WinActor | UiPath | PyAutoGUI | SikuliX | Robot Framework |
|---|---|---|---|---|---|---|
| 许可证 | MIT / Apache-2.0 | 商业授权 | 商业授权 | MIT | MIT | Apache-2.0 |
| 语言 | Rust（YAML 场景） | 专有 GUI | 专有 GUI | Python | Java（Jython） | Python |
| 开源 | 是 | 否 | 否 | 是 | 是 | 是 |
| 远程桌面（RDP/Citrix/VNC） | 是 — 无需代理 | 是 | 是（需要代理） | 否 | 否 | 否 |
| 图像识别 | 是 — 多尺度 NCC | 是 | 是 — AI 辅助 | 否 | 是 — 像素精确 | 否（通过插件） |
| Web 浏览器自动化 | 是 — WebDriver | 是 | 是 | 否 | 否 | 是（SeleniumLibrary） |
| Excel 自动化 | 是 — 单元格/工作表/公式 | 是 | 是 | 否 | 否 | 否（通过插件） |
| Word / PowerPoint | — Phase 2 | 是 | 是 | 否 | 否 | 否 |
| 场景录制 | — Phase 2 | 是 | 是 | 否 | 否 | 否 |
| 瞬态 UI 捕获（下拉菜单等） | 是 — 冻结+覆盖层 | 是 | 部分 | 否 | 否 | 否 |
| 多尺度 DPI 适配（125%/150%） | 是 — 内置 | 部分 | 部分 | 否 | 否 | 否 |
| WASM 插件沙箱 | 是 — 内存安全 | 否 | 否 | 否 | 否 | 否 |
| 内联脚本 | 是 — Rhai（沙箱） | 部分 | VB.NET / C# | Python 本身 | Jython | Python |
| 场景版本控制 | 是 — 纯 YAML | 否 | 部分 | 是 — Python | 部分 | 是 — 纯文本 |
| 启动开销 | 约 10 ms（原生二进制） | 数秒 | 数秒 | Python 启动 | JVM 启动（约 2 秒） | Python 启动 |
| OCR 支持 | 是（Tesseract 或 Windows 内置 WinRT，可选） | 是 | 是 | 否 | 部分 | 否（通过插件） |

## 为什么选 robost？

相比 PyAutoGUI 和 SikuliX，最主要的区别是**无需在目标机器安装代理就能操作 RDP/Citrix**。它在本地捕获远程桌面窗口并通过 enigo 发送输入，不依赖对端运行的环境。多尺度 NCC 匹配也能自动处理会让像素精确工具失效的 DPI 缩放（100/125/150%）。

场景格式的节点词汇贴近 WinActor（`click_image`、`wait_image`、`foreach`、`dialog_input` 等），从现有自动化流程迁移比较直接。场景是纯 YAML，能在文本编辑器里看，用 git 管理变更，不需要专有工具。

插件在 WASM 沙箱里运行：权限在 `plugin.toml` 里声明并在运行时强制检查。插件只能访问它声明过的资源，崩溃了主进程也继续运行。用 Rust、AssemblyScript、Go 或 C 编译成 `.wasm` 就能集成，不需要 fork 核心。

## 快速开始

```yaml
# scenario.yaml
name: "login"
target:
  kind: window
  title_contains: "MyApp"
steps:
  - wait_image:  { template: login_button.png, timeout_ms: 5000 }
  - click_image: { template: login_button.png }
  - type: "username"
  - type: { secret_env: PASSWORD }
  - press: Tab
  - if:
      cond: "logged_in"
      then: [ { wait_image: { template: dashboard.png } } ]
      else: [ { press: Escape } ]
  - foreach:
      var: __rows__
      do: [ { type: "{{ 氏名 }}" }, { press: Tab } ]
```

```bash
cargo build -p robost-cli
./target/debug/rpa run scenario.yaml

# 带数据源（逐行读取 Excel）
./target/debug/rpa run scenario.yaml --data data.xlsx
```

完整步骤参考：[详细文档 → 步骤参考](https://kent-tokyo.github.io/robost/)

## 模板截取（robost-snip）

1. `cargo run -p robost-snip` — 以托盘应用启动
2. 打开目标 UI（下拉菜单、悬浮提示等）
3. **Ctrl+Shift+C** — 将屏幕冻结为全屏覆盖层
4. 拖动选择模板区域；点击**匹配测试**进行验证
5. **保存** — PNG + 元数据 YAML 写入 `templates/`；自动生成多尺度变体

## 插件系统

插件是 `.wasm` + `plugin.toml` 的组合，在 WASM 沙箱中运行。

```bash
cargo build -p my-plugin --target wasm32-wasip2
rpa plugin install ./my-plugin.wasm   # 确认权限后安装
# 使用：- library: { name: "my-plugin.function", inputs: { key: value }, save_as: result }
```

## 开发

```bash
cargo build --workspace
cargo test --workspace
cargo run -p robost-snip     # 模板截取工具
cargo run -p robost-editor   # 可视化场景编辑器
```

所有 crate 已发布至 [crates.io](https://crates.io/search?q=robost)（v0.1.2）。

## 路线图

| 阶段 | 状态 | 主要内容 |
|---|---|---|
| **Phase 1** | ✅ 已完成 | 200+ 场景节点 · CLI · 可视化编辑器（AI 聊天·拖放·多语言）· snip 工具 · Web/UIA/Excel/Mail/OCR/调度器 · 所有 crate 已发布至 crates.io |
| **Phase 2** | 🔜 计划中 | 场景录制 · Word/SFTP/ML 检测/并行执行/注册表/M365 |

## 贡献

欢迎提交 Issue 和 PR。较大的改动请先通过 Issue 讨论。

## 许可证

MIT OR Apache-2.0
