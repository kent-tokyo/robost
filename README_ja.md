# robost

**robost** = **robo**t (ロボット) + **ro**bu**st** (堅牢) + **Rust** (言語)

Rust 製 OSS デスクトップ自動化 (RPA) ツール。

[English](README.md) | [中文](README_zh.md) | [詳細マニュアル](https://kent-tokyo.github.io/robost/)

## ビジュアルシナリオエディタ

| Canvas ビュー — ステップのフローチャート | YAML エディタ — 直接編集とライブキャンバス |
|:---:|:---:|
| ![Canvas View](assets/screenshots/editor_canvas_new.png) | ![YAML Editor](assets/screenshots/editor_yaml_new.png) |

| AI アシスタント — 自動化を自然言語で説明 | CLI ヘルプ |
|:---:|:---:|
| ![AI Assistant](assets/screenshots/editor_ai_new.png) | ![CLI Help](assets/screenshots/cli_help.png) |

## ダウンロード

> **最新リリース**: [GitHub Releases](https://github.com/kent-tokyo/robost/releases/latest)

### Windows — インストーラー（推奨）

**[⬇ robost-setup.exe](https://github.com/kent-tokyo/robost/releases/latest/download/robost-setup.exe)** — ダブルクリックでインストール。追加の依存関係は不要です。

- `Program Files\robost` にインストール。スタートメニューとデスクトップにショートカットを作成
- ショートカットから起動するとブラウザが自動的に開き、ビジュアルエディタが使えます
- Windows の「設定 → アプリ」から綺麗にアンインストール可能

> **SmartScreen 警告について**: インストーラーはコード署名されていないため、「Windows によって PC が保護されました」と表示される場合があります。
> **「詳細情報」→「実行」** をクリックしてください。これはコード署名証明書を持たないオープンソースソフトウェアでは一般的な表示です。

### macOS

| プラットフォーム | ダウンロード |
|---|---|
| macOS (Apple Silicon) | [rpa-aarch64-apple-darwin.tar.gz](https://github.com/kent-tokyo/robost/releases/latest/download/rpa-aarch64-apple-darwin.tar.gz) |

### Windows — ポータブル ZIP

| プラットフォーム | ダウンロード |
|---|---|
| Windows | [rpa-x86_64-windows.zip](https://github.com/kent-tokyo/robost/releases/latest/download/rpa-x86_64-windows.zip) |

- 展開して **`rpa.exe` をダブルクリック** — ブラウザが自動で開きビジュアルエディタが使えます
- インストール不要。どのフォルダからでも実行できます

## 特徴

- **画像認識** — マルチスケール NCC テンプレマッチ、Tesseract OCR または Windows 組み込み WinRT OCR (インストール不要)
- **リモートデスクトップ対応** — RDP/Citrix/VNC のウィンドウをローカルでキャプチャ。対象マシンへのエージェント不要
- **transient UI のキャプチャ** — ホットキーで画面をフリーズするため、消えてしまうドロップダウンやツールチップも採取できる
- **WASM プラグイン** — サンドボックス内で実行。プラグインがクラッシュしてもランナーには影響しない
- **プレーン YAML シナリオ** — 変数・ループ・分岐・Rhai インラインスクリプト・サブシナリオ・データソース対応
- **ビジュアルエディタ** — リスト＆Canvas ビュー・自然言語からの AI ステップ生成・AI シナリオアシスタント (Anthropic/OpenAI)・完全多言語対応 (EN/JA/ZH)

## 自動化ツール比較

| 機能 | **robost** | WinActor | UiPath | PyAutoGUI | SikuliX | Robot Framework |
|---|---|---|---|---|---|---|
| ライセンス | MIT / Apache-2.0 | 商用 | 商用 | MIT | MIT | Apache-2.0 |
| 言語 | Rust (YAML シナリオ) | 独自 GUI | 独自 GUI | Python | Java (Jython) | Python |
| オープンソース | Yes | No | No | Yes | Yes | Yes |
| リモートデスクトップ (RDP/Citrix/VNC) | Yes — エージェント不要 | Yes | Yes (エージェント必要) | No | No | No |
| 画像認識 | Yes — マルチスケール NCC | Yes | Yes — AI 支援 | No | Yes — ピクセル完全一致 | No (プラグイン経由) |
| Web ブラウザ自動化 | Yes — WebDriver | Yes | Yes | No | No | Yes (SeleniumLibrary) |
| Excel 自動化 | Yes — セル/シート/数式 | Yes | Yes | No | No | No (プラグイン経由) |
| Word / PowerPoint | — Phase 2 | Yes | Yes | No | No | No |
| シナリオ記録 | — Phase 2 | Yes | Yes | No | No | No |
| 瞬間 UI キャプチャ (ドロップダウン等) | Yes — フリーズ + オーバーレイ | Yes | 部分的 | No | No | No |
| マルチスケール DPI 対応 (125%/150%) | Yes — 内蔵 | 部分的 | 部分的 | No | No | No |
| WASM プラグインサンドボックス | Yes — メモリ安全 | No | No | No | No | No |
| インラインスクリプト | Yes — Rhai (サンドボックス) | 部分的 | VB.NET / C# | Python 本体 | Jython | Python |
| シナリオのバージョン管理 | Yes — プレーン YAML | No | 部分的 | Yes — Python | 部分的 | Yes — プレーンテキスト |
| 起動オーバーヘッド | 約 10 ms (ネイティブバイナリ) | 数秒 | 数秒 | Python 起動 | JVM 起動 (約 2 秒) | Python 起動 |
| OCR サポート | Yes (Tesseract または Windows 組み込み WinRT、オプション) | Yes | Yes | No | 部分的 | No (プラグイン経由) |

## robost を選ぶ理由

PyAutoGUI や SikuliX との一番の違いは **RDP/Citrix をエージェントなしで操作できる** 点です。ローカル側で RDP ウィンドウをキャプチャして enigo で入力を送るため、対象側の環境に依存しません。マルチスケール NCC マッチングで DPI スケーリング (100/125/150%) による座標ズレも自動で吸収します。

シナリオ形式は WinActor に近いノード語彙 (`click_image`、`wait_image`、`foreach`、`dialog_input` など) を持つため、既存の自動化処理からの移行がしやすいです。プレーン YAML なのでテキストエディタで読めますし、git で差分管理できます。

プラグインは WASM サンドボックスで実行されます。権限は `plugin.toml` で宣言したものだけが許可され、プラグインがパニックしてもランナーは継続します。Rust・AssemblyScript・Go・C など `.wasm` にコンパイルできる言語であれば組み込めます。

## クイックスタート

### Windows — ビルドなし・インストールなし

1. [GitHub Releases](https://github.com/kent-tokyo/robost/releases/latest) から **`rpa-x86_64-windows.zip`** をダウンロード
2. 任意のフォルダに展開（例: `C:\Tools\robost`）
3. **`rpa.exe` をダブルクリック** → ブラウザにビジュアルエディタが開く

Rust・Cargo・Visual Studio Build Tools・Python・Node.js は不要。

```
rpa run examples\windows\notepad.yaml
rpa run examples\windows\notepad.yaml --dry-run   # 実行せずプレビュー
```

### シナリオを書く

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

```
rpa run scenario.yaml
rpa run scenario.yaml --data data.xlsx
```

> **開発者向け（ソースからビルドする場合）:** → [開発](#開発) セクションを参照

全ステップリファレンス: [詳細マニュアル → ステップリファレンス](https://kent-tokyo.github.io/robost/)

## テンプレート採取 (robost-snip)

1. `robost-snip.exe`（インストール版）または開発者向け: `cargo run -p robost-snip` — トレイアプリとして起動
2. 対象 UI を表示（ドロップダウン・ツールチップなど）
3. **Ctrl+Shift+C** — 画面をフルスクリーンオーバーレイでフリーズ
4. テンプレート領域をドラッグ選択; **マッチテスト** で確認
5. **保存** — PNG + メタデータ YAML が `templates/` に書き出し; マルチスケール版も自動生成

## プラグインシステム

プラグインは `.wasm` + `plugin.toml` のペアで WASM サンドボックス内で動作します。

```bash
cargo build -p my-plugin --target wasm32-wasip2
rpa plugin install ./my-plugin.wasm   # 権限を確認してインストール
# 使い方: - library: { name: "my-plugin.function", inputs: { key: value }, save_as: result }
```

## 開発

```bash
cargo build --workspace
cargo test --workspace
cargo run -p robost-snip     # テンプレート採取ツール
cargo run -p robost-editor   # ビジュアルシナリオエディタ
```

全クレートは [crates.io](https://crates.io/search?q=robost) で公開中 (v0.1.2)。

## ロードマップ

| フェーズ | 状態 | 主要内容 |
|---|---|---|
| **Phase 1** | ✅ 完了 | 200+ シナリオノード · CLI · ビジュアルエディタ (AI チャット・DnD・多言語) · snip · Web/UIA/Excel/Mail/OCR/スケジューラー · 全クレート crates.io 公開 |
| **Phase 2** | 🔜 計画中 | シナリオ記録 · Word/SFTP/ML 検出/並列実行/レジストリ/M365 |

## コントリビュート

Issue・PR 歓迎です。大きな変更は先に Issue で相談してください。

## ライセンス

MIT OR Apache-2.0
