# robost

**robost** = **robo**t (ロボット) + **ro**bu**st** (堅牢) + **Rust** (言語)

Rust 製 OSS デスクトップ自動化 (RPA) ツール。

[English](README.md) | [中文](README_zh.md) | [詳細マニュアル](https://kent-tokyo.github.io/robost/)

## スクリーンショット

| シナリオ YAML | 実行出力 |
|:---:|:---:|
| ![Scenario YAML](assets/screenshots/scenario_yaml.png) | ![Run Output](assets/screenshots/scenario_run.png) |

## フローエディタ (開発中)

| 概要 | ノード設定 |
|:---:|:---:|
| ![Flow Editor Overview](assets/screenshots/editor_overview.png) | ![Node Config](assets/screenshots/editor_node_config.png) |

| ループ / 分岐フロー | テンプレートピッカー |
|:---:|:---:|
| ![Loop Flow](assets/screenshots/editor_flow_loop.png) | ![Template Picker](assets/screenshots/editor_template_picker.png) |

## ダウンロード

> **最新リリース**: [GitHub Releases](https://github.com/kent-tokyo/robost/releases/latest)

### robost-editor — ビジュアルシナリオエディタ

| プラットフォーム | ダウンロード |
|---|---|
| macOS (Apple Silicon) | [robost-editor-aarch64-apple-darwin.tar.gz](https://github.com/kent-tokyo/robost/releases/latest/download/robost-editor-aarch64-apple-darwin.tar.gz) |
| macOS (Intel) | [robost-editor-x86_64-apple-darwin.tar.gz](https://github.com/kent-tokyo/robost/releases/latest/download/robost-editor-x86_64-apple-darwin.tar.gz) |
| Windows | [robost-editor-x86_64-windows.zip](https://github.com/kent-tokyo/robost/releases/latest/download/robost-editor-x86_64-windows.zip) |

### rpa — CLI ランナー

| プラットフォーム | ダウンロード |
|---|---|
| macOS (Apple Silicon) | [rpa-aarch64-apple-darwin.tar.gz](https://github.com/kent-tokyo/robost/releases/latest/download/rpa-aarch64-apple-darwin.tar.gz) |
| macOS (Intel) | [rpa-x86_64-apple-darwin.tar.gz](https://github.com/kent-tokyo/robost/releases/latest/download/rpa-x86_64-apple-darwin.tar.gz) |
| Windows | [rpa-x86_64-windows.zip](https://github.com/kent-tokyo/robost/releases/latest/download/rpa-x86_64-windows.zip) |

## 特徴

- **画像認識** — マルチスケール NCC テンプレマッチ、Tesseract OCR、ONNX ML 検出
- **リモートデスクトップ対応** — RDP/Citrix/VNC のウィンドウをローカルでキャプチャ。対象マシンへのエージェント不要
- **transient UI のキャプチャ** — ホットキーで画面をフリーズするため、消えてしまうドロップダウンやツールチップも採取できる
- **WASM プラグイン** — サンドボックス内で実行。プラグインがクラッシュしてもランナーには影響しない
- **プレーン YAML シナリオ** — 変数・ループ・分岐・Rhai インラインスクリプト・サブシナリオ・データソース対応
- **ビジュアルエディタ** — リスト＆Canvas ビュー (自由配置・ズーム/パン・ミニマップ・スナップ)・自然言語からの AI ステップ生成・AI シナリオアシスタント (Anthropic/OpenAI)・EN/JA/ZH UI

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
| SAP GUI 自動化 | — Phase 2 | Yes | Yes | No | No | No |
| シナリオ記録 | — Phase 2 | Yes | Yes | No | No | No |
| オーケストレーター (中央管理) | — Phase 3 | Yes (限定的) | Yes | No | No | No |
| 瞬間 UI キャプチャ (ドロップダウン等) | Yes — フリーズ + オーバーレイ | Yes | 部分的 | No | No | No |
| マルチスケール DPI 対応 (125%/150%) | Yes — 内蔵 | 部分的 | 部分的 | No | No | No |
| WASM プラグインサンドボックス | Yes — メモリ安全 | No | No | No | No | No |
| インラインスクリプト | Yes — Rhai (サンドボックス) | 部分的 | VB.NET / C# | Python 本体 | Jython | Python |
| シナリオのバージョン管理 | Yes — プレーン YAML | No | 部分的 | Yes — Python | 部分的 | Yes — プレーンテキスト |
| 起動オーバーヘッド | 約 10 ms (ネイティブバイナリ) | 数秒 | 数秒 | Python 起動 | JVM 起動 (約 2 秒) | Python 起動 |
| OCR サポート | Yes (Tesseract、オプション) | Yes | Yes | No | 部分的 | No (プラグイン経由) |

## robost を選ぶ理由

PyAutoGUI や SikuliX との一番の違いは **RDP/Citrix をエージェントなしで操作できる** 点です。ローカル側で RDP ウィンドウをキャプチャして enigo で入力を送るため、対象側の環境に依存しません。マルチスケール NCC マッチングで DPI スケーリング (100/125/150%) による座標ズレも自動で吸収します。

シナリオ形式は WinActor に近いノード語彙 (click_image、wait_image、foreach、dialog_input など) を持つため、既存の自動化処理からの移行がしやすいです。プレーン YAML なのでテキストエディタで読めますし、git で差分管理できます。独自バイナリ形式のツールは不要です。

プラグインは WASM サンドボックスで実行されます。権限は `plugin.toml` で宣言したものだけが許可され、プラグインがパニックしてもランナーは継続します。Rust・AssemblyScript・Go・C など `.wasm` にコンパイルできる言語であれば、コアをフォークせずに組み込めます。

## アーキテクチャ

```
crates/
├── robost-capture/      # 画面/ウィンドウキャプチャ (xcap、DPI 対応)
├── robost-input/        # マウス/キー入力 + ウィンドウ前面化 (enigo)
├── robost-vision/       # テンプレマッチ (NCC)、OCR、ML 検出
├── robost-backend/      # Backend trait: ローカル / RDP / VNC を統一
├── robost-core/         # シナリオエンジン: YAML パース、ステップ実行、リトライ、フロー制御
├── robost-snip/         # テンプレート採取 GUI (tray app、ホットキー、オーバーレイ、日本語 UI)
├── robost-editor/       # ビジュアルシナリオエディタ (リスト + Canvas ビュー、AI ステップ生成、AI チャット、多言語)
├── robost-template/     # 共有座標・ジオメトリ型
├── robost-plugin-api/   # プラグイン作者向け公開 API (crates.io 公開候補)
├── robost-plugin-host/  # wasmtime ベースの WASM プラグインランナー (epoch タイムアウト付き)
├── robost-script/       # Rhai インラインスクリプト (サンドボックス)
├── robost-stdlib/       # ビルトインシナリオノード群
└── robost-cli/          # CLI バイナリ
```

## クイックスタート

```bash
cargo build --workspace
cargo run -p robost-cli -- run scenario.yaml
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
  - file_exists:      { path: data.csv, save_as: exists }
  - file_copy:        { src: a.txt, dst: b.txt }
  - file_move:        { src: tmp.txt, dst: archive/tmp.txt }
  - file_delete:      { path: old.txt }
  - file_rename:      { path: a.txt, name: b.txt }
  - file_list:        { dir: "logs", pattern: "*.log", save_as: files }
  - file_read:        { path: notes.txt, save_as: content }
  - file_write:       { path: out.txt, content: "{{ result }}", mode: overwrite }  # overwrite|append
  - file_append:      { path: out.txt, content: "{{ line }}\n" }
  - file_size:        { path: data.xlsx, save_as: size_bytes }
  - file_modified_at: { path: data.xlsx, format: "%Y-%m-%d %H:%M:%S", save_as: mtime }

  # ディレクトリ操作
  - dir_create: { path: "output/logs" }
  - dir_delete: { path: "tmp", recursive: true, ignore_missing: true }
  - dir_exists: { path: "output", save_as: exists }

  # プロセス操作
  - process_start:  { command: notepad.exe, wait_ms: 500 }
  - process_kill:   { name: notepad.exe }
  - process_exists: { name: notepad.exe, save_as: running }
  - wait_process:   { name: notepad.exe, state: started, timeout_ms: 10000 }  # started|exited

  # 日付操作
  - date_format: { value: "{{ today }}", format: "%Y/%m/%d", save_as: formatted }
  - date_add:    { value: "{{ today }}", days: 7, save_as: next_week }
  - date_diff:   { from: "{{ start }}", to: "{{ end }}", unit: days, save_as: elapsed }

  # 文字列操作
  - string_replace:   { value: "{{ text }}", from: "旧", to: "新", save_as: result }
  - string_trim:      { value: "  hello  ", save_as: trimmed }
  - string_upper:     { value: "{{ text }}", save_as: upper }
  - string_lower:     { value: "{{ text }}", save_as: lower }
  - string_substring: { value: "{{ text }}", start: 0, length: 5, save_as: sub }
  - string_length:    { value: "{{ text }}", save_as: len }
  - string_split:     { value: "a,b,c", delimiter: ",", save_as: parts }
  - string_join:      { value: parts, separator: "、", save_as: joined }
  - string_regex:     { value: "{{ text }}", pattern: "\\d+", save_as: match }

  # 文字列クエリ
  - string_contains:    { value: "{{ text }}", search: "hello", save_as: found }
  - string_starts_with: { value: "{{ text }}", search: "http", save_as: found }
  - string_ends_with:   { value: "{{ text }}", search: ".xlsx", save_as: found }
  - string_index_of:    { value: "{{ text }}", search: ":", save_as: pos }  # 0-based; 未発見は -1
  - string_count:       { value: "hello world hello", search: "hello", save_as: n }

  # 文字列フォーマット・Base64
  - string_format: { format: "こんにちは {0}! ({1} 件)", args: [name, count], save_as: msg }
  - base64_encode: { value: "{{ content }}", save_as: encoded }
  - base64_decode: { value: "{{ encoded }}", save_as: decoded }

  # JSON / パス / 環境変数
  - json_parse:     { value: "{\"k\":1}", save_as: obj }
  - json_stringify: { value: "{{ obj }}", save_as: json_str }
  - path_join:      { parts: ["dir", "sub", "file.txt"], save_as: full_path }
  - path_basename:  { path: "/dir/file.txt", save_as: name }
  - path_dirname:   { path: "/dir/file.txt", save_as: dir }
  - env_get:        { name: HOME, save_as: home_dir }

  # マウス座標操作
  - mouse_move:      { x: 500, y: 300 }
  - mouse_click_xy:  { x: 500, y: 300, button: left }  # left|right|double
  - mouse_drag:      { from_x: 100, from_y: 100, to_x: 400, to_y: 400, hold_ms: 100 }
  - mouse_scroll:    { direction: down, amount: 3 }    # up|down|left|right
  - mouse_hover:     { x: 500, y: 300, hover_ms: 500 }
  - click_in_window: { window: "メモ帳", x: 100, y: 50, action: left }  # left|right|double

  # キーコンビネーション
  - key_combo: { keys: [ctrl, c] }           # Ctrl+C (コピー)
  - key_combo: { keys: [ctrl, shift, tab] }  # Ctrl+Shift+Tab

  # CSV 操作
  - csv_read:  { path: data.csv, has_header: true, save_as: rows }
  - csv_write: { path: out.csv, rows: "{{ rows }}", mode: overwrite }  # overwrite|append

  # HTTP (feature = "http" が必要)
  - http_get:    { url: "https://api.example.com/items", save_as: resp }
  - http_post:   { url: "https://api.example.com/items", body: "{{ payload }}", save_as: resp }
  - http_put:    { url: "https://api.example.com/items/1", body: "{{ payload }}", save_as: resp }
  - http_delete: { url: "https://api.example.com/items/1", save_as: resp }
  - http_patch:  { url: "https://api.example.com/items/1", body: "{{ patch }}", save_as: resp }
  # 認証付き
  - http_get:    { url: "https://api.example.com/secure", auth: { basic: { user: "u", password: "p" } }, save_as: resp }
  - http_post:   { url: "https://api.example.com/secure", body: "{{ payload }}", auth: { bearer: { token: "{{ tok }}" } }, save_as: resp }

  # Excel セル / 範囲 (書き込みは feature = "excel-write" が必要)
  - excel_read_cell:   { file: data.xlsx, sheet: Sheet1, cell: A2, save_as: cell_val }
  - excel_read_range:  { file: data.xlsx, sheet: Sheet1, range: "A2:Z10", save_as: table }
  - excel_write_cell:  { file: data.xlsx, sheet: Sheet1, cell: A2, value: "{{ result }}" }
  - excel_write_range: { file: data.xlsx, sheet: Sheet1, cell: A2, data: "{{ rows }}" }

  # Excel シート管理 (feature = "excel-write" が必要)
  - excel_add_sheet:    { file: data.xlsx, name: NewSheet }
  - excel_delete_sheet: { file: data.xlsx, name: OldSheet }
  - excel_rename_sheet: { file: data.xlsx, from_name: Sheet1, to_name: Data }
  - excel_read_sheet:   { file: data.xlsx, sheet: Sheet1, has_header: true, save_as: rows }
  - excel_get_dims:     { file: data.xlsx, sheet: Sheet1, save_as: dims }  # {rows, cols}
  - excel_find_row:     { file: data.xlsx, col: A, value: "{{ search }}", save_as: row_num }  # 1-based or -1

  # メール (IMAP 受信 / SMTP 送信)
  - mail_receive:
      host: "imap.example.com"
      user: "{{ env_user }}"
      password: "{{ env_pass }}"
      folder: INBOX
      count: 10
      only_unseen: true
      save_as: emails   # [{subject, from, date, body, seen}]
  - mail_send:
      host: "smtp.example.com"
      user: "{{ env_user }}"
      password: "{{ env_pass }}"
      from: "bot@example.com"
      to: "user@example.com"
      subject: "週次報告"
      body: "{{ report }}"

  # Webhook 通知
  - notify_slack: { url: "{{ SLACK_WEBHOOK }}", message: "{{ count }} 件処理しました" }
  - notify_teams: { url: "{{ TEAMS_WEBHOOK }}", title: "完了", message: "{{ count }} 件処理しました" }

  # OS キーチェーン (macOS Keychain / Windows 資格情報マネージャー / Linux Secret Service)
  - keychain_set:    { service: myapp, account: api_key, value: "{{ secret }}" }
  - keychain_get:    { service: myapp, account: api_key, save_as: secret }
  - keychain_delete: { service: myapp, account: api_key }

  # スケジューラー (rpa schedule CLI 参照)
  # シナリオは cron でトリガーされる — ステップ内記述は不要

  # ピクセル / 色
  - get_pixel_color: { x: 500, y: 300, save_as: col }       # {r, g, b, hex}
  - wait_color:      { x: 500, y: 300, color: "#00FF00", tolerance: 10, timeout_ms: 10000 }

  # UI Automation (Windows のみ)
  - uia_get:          { by: { name: "ユーザー名" }, property: value, save_as: text }  # name|value|class|rect
  - uia_set:          { by: { name: "ユーザー名" }, value: "user@example.com" }
  - uia_click:        { by: { name: "OK" } }
  - uia_find:         { by: { id: "btnSubmit" }, save_as: elem }   # {x, y, width, height, name}
  - uia_wait:         { by: { name: "OK" }, state: enabled, timeout_ms: 10000 }  # exists|enabled|visible
  - uia_select:       { by: { name: "国" }, item: "Japan" }
  - uia_get_children: { by: { name: "ファイル" }, save_as: items }  # [{name, value, class}]
  - uia_check:        { by: { name: "同意する" }, checked: true }

  # Web ブラウザ自動化 (feature = "web" が必要; chromedriver/geckodriver を先に起動)
  - web_open:             { url: "https://example.com", driver: "http://localhost:4444" }
  - web_close: ~
  - web_click:            { selector: "#submit", timeout_ms: 5000 }
  - web_type:             { selector: "#username", text: "user", clear: true }
  - web_get:              { selector: ".result", save_as: text }
  - web_get:              { selector: ".result", attr: "href", save_as: link }
  - web_wait:             { selector: "#spinner", timeout_ms: 10000 }
  - web_wait_text:        { selector: "#status", text: "完了", timeout_ms: 10000 }
  - web_screenshot:       { path: "screens/page.png" }
  - web_select:           { selector: "#country", item: "Japan" }
  - web_execute_js:       { script: "return document.title;", save_as: title }
  - web_switch_frame:     { selector: "#iframe1" }
  - web_switch_frame:     { index: 0 }
  - web_switch_frame: ~                                              # トップに戻る
  - web_scroll:           { y: 300 }                                 # ウィンドウスクロール
  - web_scroll:           { selector: "#list", y: 100 }             # 要素スクロール
  - web_alert:            { action: accept }                         # accept|dismiss|get_text
  - web_navigate_back: ~
  - web_navigate_forward: ~
  - web_get_url:          { save_as: current_url }
  - web_get_title:        { save_as: page_title }
  - web_get_all:          { selector: ".item", save_as: items }      # 全要素のテキスト
  - web_get_all:          { selector: "a", attr: "href", save_as: links }  # 全リンクの href

  # 型変換
  - to_number: { value: "42.5", save_as: n }
  - to_string: { value: "{{ count }}", save_as: s }
  - var_type:  { value: "{{ obj }}", save_as: type_name }  # "string"|"number"|"bool"|"array"|"object"|"null"

  # リスト操作
  - list_length:   { list: items, save_as: len }
  - list_get:      { list: items, index: "0", save_as: first }
  - list_push:     { list: items, value: "{{ new_item }}" }
  - list_remove:   { list: items, index: "0" }
  - list_contains: { list: items, value: "target", save_as: found }

  # 乱数
  - number_random: { min: 1, max: 100, integer: true, save_as: n }

  # foreach インデックス変数
  - foreach:
      var: rows
      index_var: i   # 省略可能な 0-based カウンタ
      do:
        - log_write: { file: run.log, message: "{{ i }}: {{ item }}" }

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
cargo run -p robost-cli -- run scenario.yaml --export result.xlsx
```

## テンプレート採取 (robost-snip)

1. `cargo run -p robost-snip` を起動 — tray app として常駐 (ウィンドウなし、フォーカス奪わない)
2. 対象 UI (ドロップダウン、ダイアログ、ツールチップ等) を手動で表示
3. **Ctrl+Shift+C** を押下 (またはトレイメニューから「キャプチャ開始」) — 画面が凍結されフルスクリーンオーバーレイ表示
4. 矩形選択でテンプレート範囲を決定
5. 必要に応じて **アンカーポイント** (クリック基準点) と **マスク領域** (タイムスタンプ等の動的部分を除外) を追加
6. マッチング確認 ボタンで現在の凍結画面でのマッチを検証
7. **💾 保存** — PNG + メタデータ YAML が `templates/` に保存; 125%/150% の自動マルチスケール生成

## プラグイン

プラグインは `.wasm` + `plugin.toml` のペア配布。WASM サンドボックス内で実行され、権限を宣言する必要がある。

```bash
# プラグインをビルド (別 workspace)
cargo build -p my-plugin --target wasm32-wasip2

# 権限確認付きインストール
cargo run -p robost-cli -- plugin install ./my-plugin.wasm

# 確認スキップ
cargo run -p robost-cli -- plugin install ./my-plugin.wasm -y

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

rpa schedule add --cron "<expr>" --scenario <path.yaml> [--name <name>]
rpa schedule list
rpa schedule remove <id|name>
rpa schedule run           # スケジューラーデーモンを起動
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
cargo build --features robost-core/ocr
```

## 開発コマンド

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all

cargo run -p robost-snip          # テンプレート採取ツール起動
cargo run -p robost-editor        # ビジュアルシナリオエディタ起動
```

## 公開クレート

全クレートが [crates.io](https://crates.io/) に v0.1.0 で公開されています。

| クレート | 説明 |
|---|---|
| [robost](https://crates.io/crates/robost) | メタクレート |
| [robost-vision](https://crates.io/crates/robost-vision) | マルチスケール NCC テンプレートマッチ・OCR・ML 検出 |
| [robost-capture](https://crates.io/crates/robost-capture) | クロスプラットフォーム画面/ウィンドウキャプチャ |
| [robost-input](https://crates.io/crates/robost-input) | マウス/キーボード入力エミュレーション |
| [robost-template](https://crates.io/crates/robost-template) | 座標・テンプレート共有型 |
| [robost-backend](https://crates.io/crates/robost-backend) | バックエンド統一 trait (Local/RDP/VNC) |
| [robost-core](https://crates.io/crates/robost-core) | YAML シナリオエンジン |
| [robost-stdlib](https://crates.io/crates/robost-stdlib) | 組み込みシナリオノードライブラリ |
| [robost-script](https://crates.io/crates/robost-script) | Rhai インラインスクリプト |
| [robost-plugin-api](https://crates.io/crates/robost-plugin-api) | プラグイン作者向け API |
| [robost-plugin-host](https://crates.io/crates/robost-plugin-host) | WASM プラグインランナー (wasmtime) |
| [robost-uia](https://crates.io/crates/robost-uia) | Windows UI Automation |
| [robost-web](https://crates.io/crates/robost-web) | WebDriver ブラウザ自動化 |
| [robost-editor](https://crates.io/crates/robost-editor) | ビジュアルシナリオエディタ |
| [robost-snip](https://crates.io/crates/robost-snip) | テンプレート採取トレイアプリ |
| [robost-cli](https://crates.io/crates/robost-cli) | CLI ランナー (`rpa` バイナリ) |

## ライセンス

MIT OR Apache-2.0

## ロードマップ

| フェーズ | 状態 | 主要内容 |
|---|---|---|
| **Phase 1** | 完了 | 200+ シナリオノード · CLI · ビジュアルエディタ (AI チャット・DnD・多言語) · snip · Web/UIA/Excel/Mail/OCR/スケジューラー · 全クレート crates.io 公開 |
| **Phase 2** | 計画中 | シナリオ記録 · Word/SFTP/ML 検出/並列実行/レジストリ/M365 |
| **Phase 3** | 将来 | オーケストレーター · キューモデル · AI 支援シナリオ生成 |
