# 安装

> **最新版本**: [GitHub Releases](https://github.com/kent-tokyo/robost/releases/latest)

## Windows — 安装程序（推荐）

**[⬇ robost-setup.exe](https://github.com/kent-tokyo/robost/releases/latest/download/robost-setup.exe)** — 双击安装，无需额外依赖。

- 安装到 `Program Files\robost`，自动创建开始菜单和桌面快捷方式
- 点击快捷方式后浏览器自动打开可视化编辑器
- 可通过 Windows「设置 → 应用」完整卸载

!!! warning "SmartScreen 警告"
    由于安装程序未进行代码签名，Windows 可能显示「Windows 已保护你的电脑」。
    请点击**「更多信息」→「仍要运行」**继续安装。这对于没有付费签名证书的开源软件来说是正常现象。

## Windows — 便携 ZIP

| 平台 | 下载 |
|---|---|
| Windows | [rpa-x86_64-windows.zip](https://github.com/kent-tokyo/robost/releases/latest/download/rpa-x86_64-windows.zip) |

- 解压后**双击 `rpa.exe`** — 浏览器自动打开可视化编辑器
- 无需安装，可在任意目录运行

## macOS

| 平台 | 下载 |
|---|---|
| macOS (Apple Silicon) | [rpa-aarch64-apple-darwin.tar.gz](https://github.com/kent-tokyo/robost/releases/latest/download/rpa-aarch64-apple-darwin.tar.gz) |

## 从源码构建

前提条件：
- Rust stable（≥ 1.75）
- Windows：Visual Studio Build Tools
- macOS：Xcode 命令行工具
- OCR 功能：Tesseract（`brew install tesseract` / `apt install tesseract-ocr`）

```bash
git clone https://github.com/kent-tokyo/robost
cd robost
cargo build --release --features embed-editor
```

二进制文件位于 `target/release/`：

| 二进制文件 | 说明 |
|-----------|------|
| `rpa` | CLI + 代理 — `agent` 会在浏览器中打开可视化编辑器 |
| `robost-snip` | 模板采集工具（托盘应用） |

## 启动编辑器

```bash
./target/release/rpa agent
```
会自动在浏览器中打开 `http://localhost:9921`（`--no-browser` 可禁用，`--port` 可更改端口）。

## 通过 CLI 运行场景

```bash
./target/release/rpa run scenario.yaml
```
