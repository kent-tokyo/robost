# インストール

> **最新リリース**: [GitHub Releases](https://github.com/kent-tokyo/robost/releases/latest)

## Windows — インストーラー（推奨）

**[⬇ robost-setup.exe](https://github.com/kent-tokyo/robost/releases/latest/download/robost-setup.exe)** — ダブルクリックでインストール。追加の依存関係は不要です。

- `Program Files\robost` にインストール。スタートメニューとデスクトップにショートカットを作成
- ショートカットから起動するとブラウザが自動的に開き、ビジュアルエディタが使えます
- Windows の「設定 → アプリ」から綺麗にアンインストール可能

!!! warning "SmartScreen 警告について"
    インストーラーはコード署名されていないため、「Windows によって PC が保護されました」と表示される場合があります。
    **「詳細情報」→「実行」** をクリックしてください。これはコード署名証明書を持たないオープンソースソフトウェアでは一般的な表示です。

## Windows — ポータブル ZIP

| プラットフォーム | ダウンロード |
|---|---|
| Windows | [rpa-x86_64-windows.zip](https://github.com/kent-tokyo/robost/releases/latest/download/rpa-x86_64-windows.zip) |

- 展開して **`rpa.exe` をダブルクリック** — ブラウザが自動で開きビジュアルエディタが使えます
- インストール不要。どのフォルダからでも実行できます

## macOS

| プラットフォーム | ダウンロード |
|---|---|
| macOS (Apple Silicon) | [rpa-aarch64-apple-darwin.tar.gz](https://github.com/kent-tokyo/robost/releases/latest/download/rpa-aarch64-apple-darwin.tar.gz) |

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
