# 安装

## Windows：安装程序或便携版 ZIP（无需构建）

参见 [README 的下载部分](https://github.com/kent-tokyo/robost#download) — `robost-setup.exe` 或便携版 ZIP，无需 Rust 工具链。

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
