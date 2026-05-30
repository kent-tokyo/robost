# インストール

## 前提条件

- Rust stable (≥ 1.75)
- Windows の場合: Visual Studio Build Tools
- macOS の場合: Xcode コマンドラインツール
- OCR を使用する場合: Tesseract (`brew install tesseract` / `apt install tesseract-ocr`)

## ソースからビルド

```bash
git clone https://github.com/kent-tokyo/robost
cd robost
cargo build --release
```

ビルドされたバイナリは `target/release/` に配置されます:

| バイナリ | 説明 |
|--------|-------------|
| `robost-editor` | ビジュアルシナリオエディタ |
| `robost-cli` | コマンドラインランナー |
| `robost-snip` | テンプレート採取ツール (トレイアプリ) |

## エディタを起動する

```bash
cargo run -p robost-editor
```

## CLI でシナリオを実行する

```bash
cargo run -p robost-cli -- run scenario.yaml
```
