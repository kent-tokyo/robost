# rust_rpa

基于 Rust 的开源桌面自动化 (RPA) 工具。

## 核心特性

- **基于图像识别的自动化** — 多尺度 NCC 模板匹配、OCR（Tesseract）、ML 检测
- **远程桌面支持** — 通过外部屏幕捕获操作 RDP/Citrix/VNC 会话
- **企业级模板采集体验** — 通过热键冻结屏幕，捕获下拉菜单、悬浮提示等瞬态 UI；支持锚点/遮罩/多尺度
- **WASM 插件扩展性** — 带权限声明的沙箱化社区插件
- **丰富的 YAML 场景格式** — 支持变量、流程控制、数据源、内联脚本、子场景

## 与其他开源自动化工具的对比

| 功能 | **rust_rpa** | PyAutoGUI | SikuliX | Robot Framework |
|---|---|---|---|---|
| 许可证 | MIT / Apache-2.0 | MIT | MIT | Apache-2.0 |
| 语言 | Rust（YAML 场景） | Python | Java（Jython 脚本） | Python |
| 远程桌面（RDP/Citrix/VNC） | 支持 — 无需代理 | 不支持 | 不支持 | 不支持 |
| 图像识别 | 支持 — 多尺度 NCC | 不支持 | 支持 — 像素精确匹配 | 不支持（需插件） |
| 瞬态 UI 捕获（下拉菜单、悬浮提示） | 支持 — 冻结 + 叠加层 | 不支持 | 不支持 | 不支持 |
| 多尺度 DPI 适应（125%/150%） | 支持 — 内置 | 不支持 | 不支持 | 不支持 |
| 插件沙箱 | 支持 — WASM（内存安全） | 不支持 | 不支持 | 不支持 |
| 跨平台开发 | 支持 — macOS/Linux/Windows | 支持 | 支持 | 支持 |
| 场景版本控制 | 支持 — 纯 YAML 文本 | 支持 — Python | 部分支持 — `.sikuli` 目录 | 支持 — 纯文本 |
| 启动开销 | 约 10 ms（原生二进制） | Python 启动开销 | JVM 启动（约 2 秒） | Python 启动开销 |
| 内联脚本 | 支持 — Rhai（沙箱化） | Python 本身 | Jython | Python |
| OCR 支持 | 支持（Tesseract，可选） | 不支持 | 部分支持 | 不支持（需插件） |

## 为什么选择 rust_rpa？

**适合从商用 RPA 工具迁移的团队**
rust_rpa 采用与主流商用 RPA 产品相同的节点词汇（click_image、wait_image、foreach、dialog_input 等），使场景迁移变得简单。场景以纯 YAML 格式保存，可在 PR 中审查，通过 `git diff` 直观查看变更，无需专有工具。

**适合 RDP / 远程桌面自动化**
无需在目标机器上安装代理。通过在本地机器上捕获 RDP 窗口并经由 enigo 发送输入，同样的方式也适用于 Citrix、VNC 及任何窗口化会话。多尺度 NCC 匹配可自动处理导致像素精确工具失效的 DPI 缩放（100/125/150%）。

**适合工程团队**
- **零许可证成本** — 不论机器人数量或用户数量，完全免费。并发工作进程不受限制。
- **Git 原生** — YAML 场景是文本文件；`git diff` 可精确展示运行间的变化。
- **可组合** — 子场景、变量、Rhai 内联脚本和 WASM 插件共享统一的调用语法。
- **默认安全** — WASM 插件在沙箱中运行；崩溃的插件不会影响运行进程。
- **快速启动** — Rust 二进制在毫秒内启动；无需 JVM 或 .NET 运行时预热。

**适合开源贡献者**
WASM 插件接口（`rpa-plugin-api`）将运行器与节点实现解耦。用 Rust、AssemblyScript、Go 或 C 编写的插件编译为 `.wasm` 后无需 fork 核心即可集成。权限在 `plugin.toml` 中声明并在运行时强制执行。

## 架构

```
crates/
├── rpa-capture/      # 屏幕/窗口捕获（xcap，DPI 感知）
├── rpa-input/        # 鼠标/键盘输入 + 窗口前置（enigo）
├── rpa-vision/       # 模板匹配（NCC）、OCR、ML 检测
├── rpa-backend/      # Backend trait：本地 / RDP / VNC 统一
├── rpa-core/         # 场景引擎：YAML 解析、步骤执行、重试、流程控制
├── rpa-snip/         # 模板采集 GUI（托盘应用、热键、叠加层、日文 UI）
├── rpa-editor/       # 可视化场景编辑器（步骤列表 + YAML、暗色主题、日志面板）
├── rpa-template/     # 共享坐标/几何类型
├── rpa-plugin-api/   # 插件作者公开 API（crates.io 发布候选）
├── rpa-plugin-host/  # 基于 wasmtime 的 WASM 插件运行器（带 epoch 超时）
├── rpa-script/       # Rhai 内联脚本（沙箱化）
├── rpa-stdlib/       # 内置场景节点库
└── rpa-cli/          # CLI 二进制
```

## 快速开始

```bash
cargo build --workspace
cargo run -p rpa-cli -- run scenario.yaml
```

## 场景格式

```yaml
name: "example"
target:
  kind: window
  title_contains: "MyApp"
variables:
  retry_count: 0
steps:
  # 图像操作
  - wait_image:  { template: login_button.png, timeout_ms: 5000 }
  - click_image: { template: login_button.png, action: left, offset_x: 0, offset_y: 0 }
  - find_image:  { template: icon.png, save_as: pos }  # {found, x, y, score}
  - match_rect:
      template: badge.png
      rect: { x: 100, y: 200, width: 300, height: 100 }
      save_as: result

  # OCR（需要 Tesseract + --features ocr）
  - ocr_match:
      contains: "Login"
      lang: "jpn+eng"
      timeout_ms: 5000
      save_as: ocr_result   # {found, text}

  # 输入操作
  - type: "username"
  - type: { secret_env: PASSWORD }
  - press: Tab

  # 变量操作
  - set:          { name: count, value: 0 }
  - increment:    { name: count, by: 1 }
  - copy_var:     { from: src, to: dst }
  - get_datetime: { format: "%Y%m%d", save_as: today }
  - get_username: { save_as: user }
  - calc:         { expr: "count * 2", save_as: doubled }
  - to_fullwidth: { value: "abc", save_as: full }
  - to_halfwidth: { value: "ａｂｃ", save_as: half }

  # 剪贴板
  - clipboard_set: { value: "{{ text }}" }
  - clipboard_get: { save_as: copied }

  # Shell 执行
  - shell: { cmd: python3, args: [script.py], save_as: output, timeout_ms: 30000 }

  # 流程控制
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

  # 用户交互（CLI: stdin；静默模式: 使用默认值）
  - dialog_wait:   { message: "Check the screen, then press Enter.", title: "Waiting" }
  - dialog_input:  { message: "Enter filename:", default: "output.xlsx", save_as: fname }
  - dialog_select: { message: "Choose action:", options: [Save, Skip, Abort], save_as: choice }

  # 截图 / 观测
  - screenshot_save: { path: "caps/{{ today }}.png" }                    # 全屏
  - screenshot_save: { path: "caps/win.png", window: "MyApp" }           # 指定窗口
  - wait_no_image:   { template: spinner.png, timeout_ms: 30000 }        # 等待图像消失

  # 系统集成
  - url_open: { url: "https://example.com/report" }
  - notify:   { title: "Done", message: "{{ count }} rows processed" }

  # 窗口操作
  - wait_window:    { title_contains: "MyApp", state: exists, timeout_ms: 10000 }
  - window_control: { title_contains: "Notepad", action: focus }  # focus|maximize|minimize|close

  # 日志输出
  - log_write: { file: run.log, message: "step {{ count }} done", level: info }  # info|warn|error|debug

  # 文件操作
  - file_exists:  { path: data.csv, save_as: exists }
  - file_copy:    { src: a.txt, dst: b.txt }
  - file_move:    { src: tmp.txt, dst: archive/tmp.txt }
  - file_delete:  { path: old.txt }
  - file_rename:  { path: a.txt, new_name: b.txt }
  - file_list:    { pattern: "logs/*.log", save_as: files }
  - file_read:    { path: notes.txt, save_as: content }
  - file_write:   { path: out.txt, content: "{{ result }}", mode: overwrite }  # overwrite|append
  - file_append:  { path: out.txt, content: "{{ line }}\n" }

  # 进程操作
  - process_start:  { name: notepad.exe, wait_ms: 500 }
  - process_kill:   { name: notepad.exe }
  - process_exists: { name: notepad.exe, save_as: running }

  # 日期操作
  - date_format: { value: "{{ today }}", format: "%Y/%m/%d", save_as: formatted }
  - date_add:    { value: "{{ today }}", days: 7, save_as: next_week }
  - date_diff:   { from: "{{ start }}", to: "{{ end }}", unit: days, save_as: elapsed }

  # 字符串操作
  - string_replace:   { value: "{{ text }}", from: "old", to: "new", save_as: result }
  - string_trim:      { value: "  hello  ", save_as: trimmed }
  - string_upper:     { value: "{{ text }}", save_as: upper }
  - string_lower:     { value: "{{ text }}", save_as: lower }
  - string_substring: { value: "{{ text }}", start: 0, end: 5, save_as: sub }
  - string_length:    { value: "{{ text }}", save_as: len }
  - string_split:     { value: "a,b,c", sep: ",", save_as: parts }
  - string_join:      { values: "{{ parts }}", sep: ", ", save_as: joined }
  - string_regex:     { value: "{{ text }}", pattern: "\\d+", save_as: match }

  # JSON / 路径 / 环境变量
  - json_parse:     { value: "{\"k\":1}", save_as: obj }
  - json_stringify: { value: "{{ obj }}", save_as: json_str }
  - path_join:      { parts: ["dir", "sub", "file.txt"], save_as: full_path }
  - path_basename:  { path: "/dir/file.txt", save_as: name }
  - path_dirname:   { path: "/dir/file.txt", save_as: dir }
  - env_get:        { name: HOME, save_as: home_dir }

  # 鼠标坐标操作
  - mouse_move:     { x: 500, y: 300 }
  - mouse_click_xy: { x: 500, y: 300, button: left }  # left|right|double
  - mouse_drag:     { from_x: 100, from_y: 100, to_x: 400, to_y: 400, hold_ms: 100 }
  - mouse_scroll:   { direction: down, amount: 3 }    # up|down|left|right

  # 组合键
  - key_combo: { keys: [ctrl, c] }           # Ctrl+C
  - key_combo: { keys: [ctrl, shift, tab] }  # Ctrl+Shift+Tab

  # CSV 操作
  - csv_read:  { path: data.csv, has_header: true, save_as: rows }
  - csv_write: { path: out.csv, rows: "{{ rows }}", mode: overwrite }  # overwrite|append

  # HTTP（需要 feature = "http"）
  - http_get:  { url: "https://api.example.com/items", save_as: resp }
  - http_post: { url: "https://api.example.com/items", body: "{{ payload }}", save_as: resp }
  - http_put:  { url: "https://api.example.com/items/1", body: "{{ payload }}", save_as: resp }

  # Excel（需要 feature = "excel-write"）
  - excel_read_cell:   { path: data.xlsx, sheet: Sheet1, row: 2, col: 1, save_as: cell_val }
  - excel_read_range:  { path: data.xlsx, sheet: Sheet1, start_row: 2, end_row: 10, save_as: range }
  - excel_write_cell:  { path: data.xlsx, sheet: Sheet1, row: 2, col: 1, value: "{{ result }}" }
  - excel_write_range: { path: data.xlsx, sheet: Sheet1, start_cell: A2, data: "{{ rows }}" }

  # Webhook 通知
  - notify_slack: { url: "{{ SLACK_WEBHOOK }}", message: "{{ count }} rows processed" }
  - notify_teams: { url: "{{ TEAMS_WEBHOOK }}", title: "Done", message: "{{ count }} rows processed" }

  # 操作系统密钥链（macOS Keychain / Windows 凭据管理器 / Linux Secret Service）
  - keychain_set:    { service: myapp, account: api_key, value: "{{ secret }}" }
  - keychain_get:    { service: myapp, account: api_key, save_as: secret }
  - keychain_delete: { service: myapp, account: api_key }

  # 调度器（参见 `rpa schedule` CLI）
  # 场景通过 cron 触发 — 无需内联步骤

  # 变量持久化
  - import_vars: { path: params.xlsx, row: 2 }
  - save_vars:   { path: state.json, vars: [count, status] }
  - load_vars:   { path: state.json }

  # 子场景与脚本
  - sub_scenario:   { path: sub/login.yaml, inputs: { user: "{{ user }}" } }
  - call_scenario:  { path: "{{ path }}", save_as: result }
  - script:         { script: "let d = now(); d.format(\"%Y%m%d\")", save_as: date }
  - library:        { name: "excel-reader.read_sheet", inputs: { path: data.xlsx }, save_as: rows }
```

## 数据源

逐行加载 Excel/CSV，列标题自动映射为变量名：

```yaml
data_source:
  file: data.xlsx
  sheet: Sheet1
steps:
  - foreach: { var: __rows__, do: [ { type: "{{ 氏名 }}" } ] }
```

运行后导出结果：

```bash
cargo run -p rpa-cli -- run scenario.yaml --export result.xlsx
```

## 模板采集（rpa-snip）

1. `cargo run -p rpa-snip` — 以托盘应用方式启动（无窗口，不抢占焦点）
2. 打开目标 UI（下拉菜单、对话框、悬浮提示等）
3. 按 **Ctrl+Shift+C**（或使用托盘菜单）— 将屏幕冻结为全屏叠加层
4. 拖动选择模板区域
5. 可选：添加**锚点**（单击参考目标）和**遮罩区域**（排除时间戳等动态内容）
6. 点击 **▶ 匹配测试** 验证对冻结屏幕的匹配效果
7. **保存** — PNG + 元数据 YAML 写入 `templates/`；自动生成多尺度变体（125%、150%）

## 插件系统

插件以 `.wasm` + `plugin.toml` 的形式配对分发。在 WASM 沙箱中运行，必须声明权限。

```bash
# 构建插件（独立 workspace）
cargo build -p my-plugin --target wasm32-wasip2

# 带权限审查安装
cargo run -p rpa-cli -- plugin install ./my-plugin.wasm

# 跳过确认自动安装
cargo run -p rpa-cli -- plugin install ./my-plugin.wasm -y

# 在场景中使用
# - library: { name: "my-plugin.function", inputs: { key: value }, save_as: result }
```

## CLI 参考

```
rpa run <scenario.yaml> [选项]

  --from <N>         从第 N 步开始执行（0-based）
  --steps <S..E>     执行步骤范围，例如 "2..5"
  --data <path>      覆盖 data_source 文件
  --export <path>    运行后导出 __rows__（.csv 或 .xlsx）
  --silent           使用默认值自动回答所有对话框
  --wait-ms <ms>     启动后等待 N 毫秒再执行
  --exit             完成后退出进程

rpa plugin install <path.wasm> [-y]
rpa plugin list

rpa schedule add --cron "<expr>" --scenario <path.yaml> [--name <name>]
rpa schedule list
rpa schedule remove <id|name>
rpa schedule run           # 启动调度器守护进程
```

## OCR 功能

OCR 需要在宿主机上安装 Tesseract：

```bash
# macOS
brew install tesseract tesseract-lang

# Ubuntu / Debian
sudo apt install tesseract-ocr tesseract-ocr-jpn tesseract-ocr-eng

# Windows: https://github.com/UB-Mannheim/tesseract/wiki
```

启用 `ocr` feature 进行构建：

```bash
cargo build --features rpa-core/ocr
```

## 开发命令

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all

cargo run -p rpa-snip          # 模板采集工具
cargo run -p rpa-editor        # 可视化场景编辑器
```

## 已发布 Crate

| Crate | 版本 | 说明 |
|---|---|---|
| [rpa-vision](https://crates.io/crates/rpa-vision) | 0.1.0 | 面向桌面自动化的多尺度 NCC 模板匹配 + OCR |

## 许可证

MIT OR Apache-2.0
