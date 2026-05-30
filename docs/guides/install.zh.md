# 安装

## 前提条件

- Rust stable（≥ 1.75）
- Windows：Visual Studio Build Tools
- macOS：Xcode 命令行工具
- OCR 功能：Tesseract（`brew install tesseract` / `apt install tesseract-ocr`）

## 从源码构建

```bash
git clone https://github.com/kent-tokyo/robost
cd robost
cargo build --release
```

二进制文件位于 `target/release/`：

| 二进制文件 | 说明 |
|-----------|------|
| `robost-editor` | 可视化场景编辑器 |
| `robost-cli` | 命令行运行器 |
| `robost-snip` | 模板采集工具（托盘应用） |

## 启动编辑器

```bash
cargo run -p robost-editor
```

## 通过 CLI 运行场景

```bash
cargo run -p robost-cli -- run scenario.yaml
```
