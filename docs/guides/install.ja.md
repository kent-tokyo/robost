# インストール

## Windows: インストーラー or ポータブル ZIP（ビルド不要）

[README のダウンロード節](https://github.com/kent-tokyo/robost#download) を参照してください — `robost-setup.exe` またはポータブル ZIP、Rust ツールチェーンは不要です。

## ソースからビルド

前提条件:
- Rust stable (≥ 1.75)
- Windows の場合: Visual Studio Build Tools
- macOS の場合: Xcode コマンドラインツール
- OCR を使用する場合: Tesseract (`brew install tesseract` / `apt install tesseract-ocr`)

```bash
git clone https://github.com/kent-tokyo/robost
cd robost
cargo build --release --features embed-editor
```

ビルドされたバイナリは `target/release/` に配置されます:

| バイナリ | 説明 |
|--------|-------------|
| `rpa` | CLI + エージェント — `agent` でブラウザにビジュアルエディタが開く |
| `robost-snip` | テンプレート採取ツール (トレイアプリ) |

## エディタを起動する

```bash
./target/release/rpa agent
```
`http://localhost:9921` が自動的にブラウザで開きます（`--no-browser` で無効化、`--port` でポート変更）。

## CLI でシナリオを実行する

```bash
./target/release/rpa run scenario.yaml
```
