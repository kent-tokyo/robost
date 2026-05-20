# rust_rpa

Rust 製 OSS デスクトップ自動化 (RPA) ツール。

## 差別化ポイント

- **画像認識ベースの自動化** — マルチスケール NCC テンプレマッチ、OCR (Tesseract)、ML 検出
- **リモートデスクトップ対応** — RDP/Citrix/VNC を外部キャプチャ方式で操作
- **エンタープライズ水準のテンプレート採取 UX** — ドロップダウン等の transient UI をホットキーフリーズで採取; アンカー/マスク/マルチスケール対応
- **WASM プラグインによる拡張性** — 権限宣言付きサンドボックスコミュニティプラグイン
- **リッチな YAML シナリオ形式** — 変数・フロー制御・データソース・インラインスクリプト・サブシナリオ対応

## オープンソース自動化ツールとの比較

| 機能 | **rust_rpa** | PyAutoGUI | SikuliX | Robot Framework |
|---|---|---|---|---|
| ライセンス | MIT / Apache-2.0 | MIT | MIT | Apache-2.0 |
| 言語 | Rust (YAML シナリオ) | Python | Java (Jython スクリプト) | Python |
| リモートデスクトップ (RDP/Citrix/VNC) | 対応 — エージェント不要 | 非対応 | 非対応 | 非対応 |
| 画像認識 | 対応 — マルチスケール NCC | 非対応 | 対応 — ピクセル完全一致 | 非対応 (プラグイン経由) |
| Transient UI キャプチャ (ドロップダウン・ツールチップ) | 対応 — フリーズ + オーバーレイ | 非対応 | 非対応 | 非対応 |
| マルチスケール DPI 耐性 (125%/150%) | 対応 — ビルトイン | 非対応 | 非対応 | 非対応 |
| プラグインサンドボックス | 対応 — WASM (メモリ安全) | 非対応 | 非対応 | 非対応 |
| クロスプラットフォーム開発 | 対応 — macOS/Linux/Windows | 対応 | 対応 | 対応 |
| シナリオのバージョン管理 | 対応 — プレーン YAML | 対応 — Python | 部分対応 — `.sikuli` ディレクトリ | 対応 — プレーンテキスト |
| 起動オーバーヘッド | 約 10 ms (ネイティブバイナリ) | Python 起動コスト | JVM 起動 (約 2 秒) | Python 起動コスト |
| インラインスクリプト | 対応 — Rhai (サンドボックス) | Python 本体 | Jython | Python |
| OCR サポート | 対応 (Tesseract、オプション) | 非対応 | 部分対応 | 非対応 (プラグイン経由) |

## rust_rpa を使うメリット

**商用 RPA ツールから移行するチーム向け**
rust_rpa は主要な商用 RPA 製品と共通するノード語彙 (click_image、wait_image、foreach、dialog_input など) を採用しているため、既存シナリオの移行が容易です。シナリオはプレーン YAML で管理でき、PR でレビュー可能、`git diff` で変更点が一目瞭然です。独自バイナリ形式のツールは不要です。

**RDP / リモートデスクトップ自動化向け**
ターゲットマシンへのエージェントインストールは不要です。ローカルマシン上で RDP ウィンドウをキャプチャし enigo で入力を送る方式のため、Citrix・VNC・任意のウィンドウセッションにも同じ仕組みで対応します。マルチスケール NCC マッチングで DPI スケーリング (100/125/150%) による座標ズレを自動補正します。

**エンジニアリングチーム向け**
- **ライセンス費用ゼロ** — ボット数・ユーザー数に関わらず無償。並列ワーカーを制限なく実行できます。
- **Git ネイティブ** — YAML シナリオはテキストファイル。`git diff` で実行間の差分を確認できます。
- **コンポーザブル** — サブシナリオ・変数・Rhai インラインスクリプト・WASM プラグインを統一した呼び出し構文で利用できます。
- **デフォルトで安全** — WASM プラグインはサンドボックス内で動作し、クラッシュしてもランナープロセスへの影響はありません。
- **高速起動** — Rust ネイティブバイナリはミリ秒単位で起動します。JVM や .NET ランタイムのウォームアップは不要です。

**オープンソースコントリビューター向け**
WASM プラグインインタフェース (`rpa-plugin-api`) によりランナー本体とノード実装が分離されています。Rust、AssemblyScript、Go、C でビルドした `.wasm` をコアのフォークなしに統合できます。権限は `plugin.toml` で宣言され、ランタイムで強制されます。ドキュメントに書くだけで終わりません。

## アーキテクチャ

```
crates/
├── rpa-capture/      # 画面/ウィンドウキャプチャ (xcap、DPI 対応)
├── rpa-input/        # マウス/キー入力 + ウィンドウ前面化 (enigo)
├── rpa-vision/       # テンプレマッチ (NCC)、OCR、ML 検出
├── rpa-backend/      # Backend trait: ローカル / RDP / VNC を統一
├── rpa-core/         # シナリオエンジン: YAML パース、ステップ実行、リトライ、フロー制御
├── rpa-snip/         # テンプレート採取 GUI (tray app、ホットキー、オーバーレイ、日本語 UI)
├── rpa-editor/       # ビジュアルシナリオエディタ (リストパネル + YAML、ダークテーマ、ログパネル)
├── rpa-template/     # 共有座標・ジオメトリ型
├── rpa-plugin-api/   # プラグイン作者向け公開 API (crates.io 公開候補)
├── rpa-plugin-host/  # wasmtime ベースの WASM プラグインランナー (epoch タイムアウト付き)
├── rpa-script/       # Rhai インラインスクリプト (サンドボックス)
├── rpa-stdlib/       # ビルトインシナリオノード群
└── rpa-cli/          # CLI バイナリ
```

## クイックスタート

```bash
cargo build --workspace
cargo run -p rpa-cli -- run scenario.yaml
```

## シナリオ形式

```yaml
name: "example"
target:
  kind: window
  title_contains: "MyApp"
variables:
  retry_count: 0
steps:
  # 画像操作
  - wait_image:  { template: login_button.png, timeout_ms: 5000 }
  - click_image: { template: login_button.png, action: left, offset_x: 0, offset_y: 0 }
  - find_image:  { template: icon.png, save_as: pos }   # {found, x, y, score}
  - match_rect:
      template: badge.png
      rect: { x: 100, y: 200, width: 300, height: 100 }
      save_as: result

  # OCR (Tesseract + --features ocr が必要)
  - ocr_match:
      contains: "ログイン"
      lang: "jpn+eng"
      timeout_ms: 5000
      save_as: ocr_result   # {found, text}

  # 入力操作
  - type: "username"
  - type: { secret_env: PASSWORD }
  - press: Tab

  # 変数操作
  - set:          { name: count, value: 0 }
  - increment:    { name: count, by: 1 }
  - copy_var:     { from: src, to: dst }
  - get_datetime: { format: "%Y%m%d", save_as: today }
  - get_username: { save_as: user }
  - calc:         { expr: "count * 2", save_as: doubled }
  - to_fullwidth: { value: "abc", save_as: full }
  - to_halfwidth: { value: "ａｂｃ", save_as: half }

  # クリップボード
  - clipboard_set: { value: "{{ text }}" }
  - clipboard_get: { save_as: copied }

  # シェル実行
  - shell: { cmd: python3, args: [script.py], save_as: output, timeout_ms: 30000 }

  # フロー制御
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
  - foreach: { var: __rows__, do: [ { type: "{{ 氏名 }}" } ] }
  - try_catch:
      try:     [ { click_image: { template: btn.png } } ]
      catch:   [ { set: { name: _error, value: "failed" } } ]
      finally: [ { wait_ms: 100 } ]
  - group:   { name: "ログインブロック", do: [ { type: "user" } ] }
  - break
  - continue
  - exit

  # ユーザーインタラクション (CLI: stdin 入力; --silent: デフォルト値で自動スキップ)
  - dialog_wait:   { message: "画面を確認して Enter を押してください。", title: "待機" }
  - dialog_input:  { message: "ファイル名を入力:", default: "output.xlsx", save_as: fname }
  - dialog_select: { message: "操作を選択:", options: [保存, スキップ, 中止], save_as: choice }

  # スクリーンショット・観測
  - screenshot_save: { path: "caps/{{ today }}.png" }                    # 全画面
  - screenshot_save: { path: "caps/win.png", window: "MyApp" }           # 特定ウィンドウ
  - wait_no_image:   { template: spinner.png, timeout_ms: 30000 }        # 画像が消えるまで待機

  # システム統合
  - url_open: { url: "https://example.com/report" }
  - notify:   { title: "完了", message: "{{ count }} 件処理しました" }

  # ウィンドウ
  - wait_window:    { title_contains: "MyApp", state: exists, timeout_ms: 10000 }
  - window_control: { title_contains: "メモ帳", action: focus }  # focus|maximize|minimize|close

  # ログ出力
  - log_write: { file: run.log, message: "ステップ {{ count }} 完了", level: info }  # info|warn|error|debug

  # ファイル操作
  - file_exists:  { path: data.csv, save_as: exists }
  - file_copy:    { src: a.txt, dst: b.txt }
  - file_move:    { src: tmp.txt, dst: archive/tmp.txt }
  - file_delete:  { path: old.txt }
  - file_rename:  { path: a.txt, new_name: b.txt }
  - file_list:    { pattern: "logs/*.log", save_as: files }
  - file_read:    { path: notes.txt, save_as: content }
  - file_write:   { path: out.txt, content: "{{ result }}", mode: overwrite }  # overwrite|append
  - file_append:  { path: out.txt, content: "{{ line }}\n" }

  # プロセス操作
  - process_start:  { name: notepad.exe, wait_ms: 500 }
  - process_kill:   { name: notepad.exe }
  - process_exists: { name: notepad.exe, save_as: running }

  # 日付操作
  - date_format: { value: "{{ today }}", format: "%Y/%m/%d", save_as: formatted }
  - date_add:    { value: "{{ today }}", days: 7, save_as: next_week }
  - date_diff:   { from: "{{ start }}", to: "{{ end }}", unit: days, save_as: elapsed }

  # 文字列操作
  - string_replace:   { value: "{{ text }}", from: "旧", to: "新", save_as: result }
  - string_trim:      { value: "  hello  ", save_as: trimmed }
  - string_upper:     { value: "{{ text }}", save_as: upper }
  - string_lower:     { value: "{{ text }}", save_as: lower }
  - string_substring: { value: "{{ text }}", start: 0, end: 5, save_as: sub }
  - string_length:    { value: "{{ text }}", save_as: len }
  - string_split:     { value: "a,b,c", sep: ",", save_as: parts }
  - string_join:      { values: "{{ parts }}", sep: "、", save_as: joined }
  - string_regex:     { value: "{{ text }}", pattern: "\\d+", save_as: match }

  # JSON / パス / 環境変数
  - json_parse:     { value: "{\"k\":1}", save_as: obj }
  - json_stringify: { value: "{{ obj }}", save_as: json_str }
  - path_join:      { parts: ["dir", "sub", "file.txt"], save_as: full_path }
  - path_basename:  { path: "/dir/file.txt", save_as: name }
  - path_dirname:   { path: "/dir/file.txt", save_as: dir }
  - env_get:        { name: HOME, save_as: home_dir }

  # マウス座標操作
  - mouse_move:     { x: 500, y: 300 }
  - mouse_click_xy: { x: 500, y: 300, button: left }  # left|right|double
  - mouse_drag:     { from_x: 100, from_y: 100, to_x: 400, to_y: 400, hold_ms: 100 }
  - mouse_scroll:   { direction: down, amount: 3 }    # up|down|left|right

  # キーコンビネーション
  - key_combo: { keys: [ctrl, c] }           # Ctrl+C (コピー)
  - key_combo: { keys: [ctrl, shift, tab] }  # Ctrl+Shift+Tab

  # CSV 操作
  - csv_read:  { path: data.csv, has_header: true, save_as: rows }
  - csv_write: { path: out.csv, rows: "{{ rows }}", mode: overwrite }  # overwrite|append

  # HTTP (feature = "http" が必要)
  - http_get:  { url: "https://api.example.com/items", save_as: resp }
  - http_post: { url: "https://api.example.com/items", body: "{{ payload }}", save_as: resp }
  - http_put:  { url: "https://api.example.com/items/1", body: "{{ payload }}", save_as: resp }

  # Excel (feature = "excel-write" が必要)
  - excel_read_cell:  { path: data.xlsx, sheet: Sheet1, row: 2, col: 1, save_as: cell_val }
  - excel_read_range: { path: data.xlsx, sheet: Sheet1, start_row: 2, end_row: 10, save_as: range }
  - excel_write_cell: { path: data.xlsx, sheet: Sheet1, row: 2, col: 1, value: "{{ result }}" }

  # 変数永続化
  - import_vars: { path: params.xlsx, row: 2 }
  - save_vars:   { path: state.json, vars: [count, status] }
  - load_vars:   { path: state.json }

  # サブシナリオ・スクリプト
  - sub_scenario:  { path: sub/login.yaml, inputs: { user: "{{ user }}" } }
  - call_scenario: { path: "{{ path }}", save_as: result }
  - script:        { script: "let d = now(); d.format(\"%Y%m%d\")", save_as: date }
  - library:       { name: "excel-reader.read_sheet", inputs: { path: data.xlsx }, save_as: rows }
```

## データ一覧

Excel/CSV を行ごとに読み込み、列ヘッダを変数名として自動マッピング:

```yaml
data_source:
  file: data.xlsx
  sheet: Sheet1
steps:
  - foreach: { var: __rows__, do: [ { type: "{{ 氏名 }}" } ] }
```

実行後のエクスポート:

```bash
cargo run -p rpa-cli -- run scenario.yaml --export result.xlsx
```

## テンプレート採取 (rpa-snip)

1. `cargo run -p rpa-snip` を起動 — tray app として常駐 (ウィンドウなし、フォーカス奪わない)
2. 対象 UI (ドロップダウン、ダイアログ、ツールチップ等) を手動で表示
3. **Ctrl+Shift+C** を押下 (またはトレイメニューから「キャプチャ開始」) — 画面が凍結されフルスクリーンオーバーレイ表示
4. 矩形選択でテンプレート範囲を決定
5. 必要に応じて **アンカーポイント** (クリック基準点) と **マスク領域** (タイムスタンプ等の動的部分を除外) を追加
6. **▶ マッチング確認** ボタンで現在の凍結画面でのマッチを検証
7. **💾 保存** — PNG + メタデータ YAML が `templates/` に保存; 125%/150% の自動マルチスケール生成

## プラグイン

プラグインは `.wasm` + `plugin.toml` のペア配布。WASM サンドボックス内で実行され、権限を宣言する必要がある。

```bash
# プラグインをビルド (別 workspace)
cargo build -p my-plugin --target wasm32-wasip2

# 権限確認付きインストール
cargo run -p rpa-cli -- plugin install ./my-plugin.wasm

# 確認スキップ
cargo run -p rpa-cli -- plugin install ./my-plugin.wasm -y

# シナリオでの使用
# - library: { name: "my-plugin.function", inputs: { key: value }, save_as: result }
```

## CLI リファレンス

```
rpa run <scenario.yaml> [オプション]

  --from <N>         N 番目のステップから実行 (0-based)
  --steps <S..E>     範囲指定実行 (例: "2..5")
  --data <path>      data_source ファイルを上書き指定
  --export <path>    実行後に __rows__ をエクスポート (.csv / .xlsx)
  --silent           全ダイアログをデフォルト値で自動スキップ
  --wait-ms <ms>     N ミリ秒待機してから実行開始
  --exit             完了後にプロセス終了

rpa plugin install <path.wasm> [-y]
rpa plugin list
```

## OCR 機能

OCR は実行ホストに Tesseract のインストールが必要:

```bash
# macOS
brew install tesseract tesseract-lang

# Ubuntu / Debian
sudo apt install tesseract-ocr tesseract-ocr-jpn tesseract-ocr-eng

# Windows: https://github.com/UB-Mannheim/tesseract/wiki
```

`ocr` feature を有効にしてビルド:

```bash
cargo build --features rpa-core/ocr
```

## 開発コマンド

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all

cargo run -p rpa-snip          # テンプレート採取ツール起動
cargo run -p rpa-editor        # ビジュアルシナリオエディタ起動
```

## 公開済み Crate

| Crate | バージョン | 説明 |
|---|---|---|
| [rpa-vision](https://crates.io/crates/rpa-vision) | 0.1.0 | デスクトップ自動化向けマルチスケール NCC テンプレマッチ + OCR |

## ライセンス

MIT OR Apache-2.0
