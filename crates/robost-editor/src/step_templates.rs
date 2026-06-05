// ---- step templates -------------------------------------------------------

pub(crate) struct StepTemplate {
    pub(crate) category: &'static str,
    pub(crate) display_name: &'static str,
    pub(crate) name: &'static str,
    pub(crate) yaml: &'static str,
}

pub(crate) const STEP_TEMPLATES: &[StepTemplate] = &[
    // ── AI ─────────────────────────────────────────────────────────────────
    StepTemplate { category: "AI", display_name: "AI でステップ作成", name: "ai_create", yaml: "ai_create:\n  prompt: \"\"\n" },
    // ── 制御フロー ─────────────────────────────────────────────────────────
    StepTemplate { category: "制御フロー", display_name: "条件分岐",           name: "if",           yaml: "if:\n  cond: \"{{ var }}\"\nthen:\n  - wait_ms: 100\nelse:\n  - wait_ms: 100\n" },
    StepTemplate { category: "制御フロー", display_name: "繰り返し(foreach)",  name: "foreach",      yaml: "foreach:\n  var: __rows__\n  do:\n    - wait_ms: 100\n" },
    StepTemplate { category: "制御フロー", display_name: "N回繰り返し",        name: "repeat",       yaml: "repeat:\n  count: 3\n  do:\n    - wait_ms: 100\n" },
    StepTemplate { category: "制御フロー", display_name: "前判定ループ",        name: "while",        yaml: "while:\n  cond: \"done == false\"\n  do:\n    - wait_ms: 100\n" },
    StepTemplate { category: "制御フロー", display_name: "後判定ループ",        name: "do_while",     yaml: "do_while:\n  cond: \"done == false\"\n  do:\n    - wait_ms: 100\n" },
    StepTemplate { category: "制御フロー", display_name: "例外処理",           name: "try_catch",    yaml: "try_catch:\n  try:\n    - wait_ms: 100\n  catch:\n    - wait_ms: 100\n" },
    StepTemplate { category: "制御フロー", display_name: "グループ",           name: "group",        yaml: "group:\n  name: グループ1\n  steps:\n    - wait_ms: 100\n" },
    StepTemplate { category: "制御フロー", display_name: "分岐(switch)",       name: "switch",       yaml: "switch:\n  on: \"{{ var }}\"\n  cases:\n    - when: A\n      do:\n        - wait_ms: 100\n" },
    StepTemplate { category: "制御フロー", display_name: "サブシナリオ",       name: "sub_scenario", yaml: "sub_scenario:\n  path: sub.yaml\n" },
    StepTemplate { category: "制御フロー", display_name: "シナリオ呼び出し",   name: "call_scenario",yaml: "call_scenario:\n  path: sub.yaml\n  inputs:\n    key: value\n" },
    StepTemplate { category: "制御フロー", display_name: "シナリオ終了",       name: "exit",         yaml: "exit: ~\n" },
    StepTemplate { category: "制御フロー", display_name: "ループ脱出",         name: "break",        yaml: "break: ~\n" },
    StepTemplate { category: "制御フロー", display_name: "次の繰り返しへ",     name: "continue",     yaml: "continue: ~\n" },
    // ── 画像操作 ───────────────────────────────────────────────────────────
    StepTemplate { category: "画像操作", display_name: "画像待機",             name: "wait_image",     yaml: "wait_image:\n  template: button.png\n  timeout_ms: 5000\n" },
    StepTemplate { category: "画像操作", display_name: "画像クリック",         name: "click_image",    yaml: "click_image:\n  template: button.png\n  timeout_ms: 5000\n" },
    StepTemplate { category: "画像操作", display_name: "画像検索",             name: "find_image",     yaml: "find_image:\n  template: button.png\n  save_as: pos\n  timeout_ms: 5000\n" },
    StepTemplate { category: "画像操作", display_name: "矩形マッチング",       name: "match_rect",     yaml: "match_rect:\n  template: button.png\n  rect:\n    x: 0\n    y: 0\n    width: 100\n    height: 50\n  timeout_ms: 5000\n" },
    StepTemplate { category: "画像操作", display_name: "画像消滅待機",         name: "wait_no_image",  yaml: "wait_no_image:\n  template: button.png\n  timeout_ms: 30000\n" },
    StepTemplate { category: "画像操作", display_name: "画面変化待機",         name: "wait_change",    yaml: "wait_change:\n  timeout_ms: 5000\n" },
    StepTemplate { category: "画像操作", display_name: "OCR マッチング",       name: "ocr_match",      yaml: "ocr_match:\n  contains: \"テキスト\"\n  lang: jpn+eng\n  timeout_ms: 5000\n  save_as: result\n" },
    StepTemplate { category: "画像操作", display_name: "ML 検出",              name: "ml_detect",      yaml: "ml_detect:\n  model: model.onnx\n  threshold: 0.7\n  save_as: detections\n" },
    StepTemplate { category: "画像操作", display_name: "スクリーンショット保存", name: "screenshot_save", yaml: "screenshot_save:\n  path: screenshot.png\n" },
    StepTemplate { category: "画像操作", display_name: "ピクセル色取得",       name: "get_pixel_color", yaml: "get_pixel_color:\n  x: 500\n  y: 300\n  save_as: col\n" },
    StepTemplate { category: "画像操作", display_name: "ピクセル色待機",       name: "wait_color",     yaml: "wait_color:\n  x: 500\n  y: 300\n  color: \"#00FF00\"\n  tolerance: 10\n  timeout_ms: 10000\n" },
    StepTemplate { category: "画像操作", display_name: "ウィンドウ操作",       name: "window_control", yaml: "window_control:\n  title_contains: MyApp\n  action: focus\n" },
    // ── 入力操作 ───────────────────────────────────────────────────────────
    StepTemplate { category: "入力操作", display_name: "文字入力",             name: "type",           yaml: "type: \"hello\"\n" },
    StepTemplate { category: "入力操作", display_name: "キー押下",             name: "press",          yaml: "press: Enter\n" },
    StepTemplate { category: "入力操作", display_name: "キーコンボ",           name: "key_combo",      yaml: "key_combo:\n  keys: [ctrl, c]\n" },
    StepTemplate { category: "入力操作", display_name: "ウィンドウ内クリック", name: "click_in_window",yaml: "click_in_window:\n  window: メモ帳\n  x: 100\n  y: 50\n  action: left\n" },
    // ── 待機 ───────────────────────────────────────────────────────────────
    StepTemplate { category: "待機", display_name: "指定時間待機",             name: "wait_ms",        yaml: "wait_ms: 500\n" },
    StepTemplate { category: "待機", display_name: "ウィンドウ待機",           name: "wait_window",    yaml: "wait_window:\n  title_contains: MyApp\n  state: exists\n  timeout_ms: 10000\n" },
    StepTemplate { category: "待機", display_name: "条件成立まで待機",         name: "wait_until",     yaml: "wait_until:\n  cond: \"status == \\\"done\\\"\"\n  timeout_ms: 30000\n  interval_ms: 500\n" },
    StepTemplate { category: "待機", display_name: "プロセス待機",             name: "wait_process",   yaml: "wait_process:\n  name: notepad.exe\n  state: started\n  timeout_ms: 10000\n" },
    // ── 変数 ───────────────────────────────────────────────────────────────
    StepTemplate { category: "変数", display_name: "変数設定",                 name: "set",            yaml: "set:\n  name: my_var\n  value: \"value\"\n" },
    StepTemplate { category: "変数", display_name: "変数コピー",               name: "copy_var",       yaml: "copy_var:\n  from: src_var\n  to: dst_var\n" },
    StepTemplate { category: "変数", display_name: "日時取得",                 name: "get_datetime",   yaml: "get_datetime:\n  format: \"%Y%m%d\"\n  save_as: today\n" },
    StepTemplate { category: "変数", display_name: "ユーザー名取得",           name: "get_username",   yaml: "get_username:\n  save_as: user\n" },
    StepTemplate { category: "変数", display_name: "計算",                     name: "calc",           yaml: "calc:\n  expr: \"a + b\"\n  save_as: result\n" },
    StepTemplate { category: "変数", display_name: "カウントアップ",           name: "increment",      yaml: "increment:\n  name: counter\n" },
    StepTemplate { category: "変数", display_name: "全角変換",                 name: "to_fullwidth",   yaml: "to_fullwidth:\n  value: \"{{ text }}\"\n  save_as: result\n" },
    StepTemplate { category: "変数", display_name: "半角変換",                 name: "to_halfwidth",   yaml: "to_halfwidth:\n  value: \"{{ text }}\"\n  save_as: result\n" },
    StepTemplate { category: "変数", display_name: "変数インポート(CSV/Excel)", name: "import_vars",   yaml: "import_vars:\n  file: data.xlsx\n  row: 0\n" },
    StepTemplate { category: "変数", display_name: "変数保存(JSON)",           name: "save_vars",      yaml: "save_vars:\n  file: vars.json\n" },
    StepTemplate { category: "変数", display_name: "変数読み込み(JSON)",       name: "load_vars",      yaml: "load_vars:\n  file: vars.json\n" },
    // ── クリップボード ─────────────────────────────────────────────────────
    StepTemplate { category: "クリップボード", display_name: "クリップボード設定", name: "clipboard_set", yaml: "clipboard_set:\n  value: \"text\"\n" },
    StepTemplate { category: "クリップボード", display_name: "クリップボード取得", name: "clipboard_get", yaml: "clipboard_get:\n  save_as: clip\n" },
    // ── ダイアログ ─────────────────────────────────────────────────────────
    StepTemplate { category: "ダイアログ", display_name: "待機ボックス",       name: "dialog_wait",    yaml: "dialog_wait:\n  message: \"確認してください\"\n" },
    StepTemplate { category: "ダイアログ", display_name: "入力ボックス",       name: "dialog_input",   yaml: "dialog_input:\n  message: \"値を入力してください\"\n  save_as: user_input\n" },
    StepTemplate { category: "ダイアログ", display_name: "選択ボックス",       name: "dialog_select",  yaml: "dialog_select:\n  message: \"選択してください\"\n  options: [A, B]\n  save_as: choice\n" },
    // ── スクリプト ─────────────────────────────────────────────────────────
    StepTemplate { category: "スクリプト", display_name: "コマンド実行",       name: "shell",          yaml: "shell:\n  cmd: echo\n  args: [hello]\n  save_as: output\n" },
    StepTemplate { category: "スクリプト", display_name: "スクリプト実行",     name: "script",         yaml: "script:\n  script: |\n    let x = 1 + 1;\n    x\n  save_as: result\n" },
    // ── ライブラリ ─────────────────────────────────────────────────────────
    StepTemplate { category: "ライブラリ", display_name: "ライブラリ呼び出し", name: "library",        yaml: "library:\n  name: plugin.function\n  inputs:\n    arg1: value\n  save_as: out\n" },
    // ── ファイル ───────────────────────────────────────────────────────────
    StepTemplate { category: "ファイル", display_name: "ファイルコピー",       name: "file_copy",      yaml: "file_copy:\n  src: source.txt\n  dst: dest.txt\n" },
    StepTemplate { category: "ファイル", display_name: "ファイル移動",         name: "file_move",      yaml: "file_move:\n  src: source.txt\n  dst: dest.txt\n" },
    StepTemplate { category: "ファイル", display_name: "ファイル削除",         name: "file_delete",    yaml: "file_delete:\n  path: file.txt\n" },
    StepTemplate { category: "ファイル", display_name: "ファイル名変更",       name: "file_rename",    yaml: "file_rename:\n  path: file.txt\n  name: new_name.txt\n" },
    StepTemplate { category: "ファイル", display_name: "ファイル存在確認",     name: "file_exists",    yaml: "file_exists:\n  path: file.txt\n  save_as: exists\n" },
    StepTemplate { category: "ファイル", display_name: "ファイル読み込み",     name: "file_read",      yaml: "file_read:\n  path: file.txt\n  save_as: content\n" },
    StepTemplate { category: "ファイル", display_name: "ファイル書き込み",     name: "file_write",     yaml: "file_write:\n  path: file.txt\n  content: \"{{ text }}\"\n" },
    StepTemplate { category: "ファイル", display_name: "ファイル追記",         name: "file_append",    yaml: "file_append:\n  path: file.txt\n  content: \"{{ text }}\"\n" },
    StepTemplate { category: "ファイル", display_name: "ファイルサイズ取得",   name: "file_size",      yaml: "file_size:\n  path: file.txt\n  save_as: size\n" },
    StepTemplate { category: "ファイル", display_name: "更新日時取得",         name: "file_modified_at",yaml: "file_modified_at:\n  path: file.txt\n  format: \"%Y-%m-%d %H:%M:%S\"\n  save_as: mtime\n" },
    StepTemplate { category: "ファイル", display_name: "ファイル一覧取得",     name: "file_list",      yaml: "file_list:\n  dir: ./data\n  pattern: \"*.csv\"\n  save_as: files\n" },
    StepTemplate { category: "ファイル", display_name: "ログ書き込み",         name: "log_write",      yaml: "log_write:\n  file: robot.log\n  message: \"{{ msg }}\"\n  level: info\n" },
    StepTemplate { category: "ファイル", display_name: "ディレクトリ作成",     name: "dir_create",     yaml: "dir_create:\n  path: output/\n" },
    StepTemplate { category: "ファイル", display_name: "ディレクトリ削除",     name: "dir_delete",     yaml: "dir_delete:\n  path: old_dir\n  recursive: true\n" },
    StepTemplate { category: "ファイル", display_name: "ディレクトリ存在確認", name: "dir_exists",     yaml: "dir_exists:\n  path: ./output\n  save_as: exists\n" },
    StepTemplate { category: "ファイル", display_name: "ZIP 圧縮",             name: "zip_compress",   yaml: "zip_compress:\n  dest: archive.zip\n  files:\n    - file1.txt\n" },
    StepTemplate { category: "ファイル", display_name: "ZIP 展開",             name: "zip_extract",    yaml: "zip_extract:\n  src: archive.zip\n  dest: output/\n" },
    StepTemplate { category: "ファイル", display_name: "ZIP 一覧",             name: "zip_list",       yaml: "zip_list:\n  src: archive.zip\n  save_as: entries\n" },
    StepTemplate { category: "ファイル", display_name: "FTP アップロード",     name: "ftp_upload",     yaml: "ftp_upload:\n  host: ftp.example.com\n  user: user\n  password: \"{{ env.FTP_PASS }}\"\n  local: report.csv\n  remote: /out/report.csv\n" },
    StepTemplate { category: "ファイル", display_name: "FTP ダウンロード",     name: "ftp_download",   yaml: "ftp_download:\n  host: ftp.example.com\n  user: user\n  password: \"{{ env.FTP_PASS }}\"\n  remote: /in/data.csv\n  local: data.csv\n" },
    StepTemplate { category: "ファイル", display_name: "FTP 一覧",             name: "ftp_list",       yaml: "ftp_list:\n  host: ftp.example.com\n  user: user\n  password: \"{{ env.FTP_PASS }}\"\n  remote: /data/\n  save_as: files\n" },
    StepTemplate { category: "ファイル", display_name: "FTP 削除",             name: "ftp_delete",     yaml: "ftp_delete:\n  host: ftp.example.com\n  user: user\n  password: \"{{ env.FTP_PASS }}\"\n  remote: /old/file.csv\n" },
    StepTemplate { category: "ファイル", display_name: "FTP ディレクトリ作成", name: "ftp_mkdir",      yaml: "ftp_mkdir:\n  host: ftp.example.com\n  user: user\n  password: \"{{ env.FTP_PASS }}\"\n  remote: /new_dir\n" },
    // ── Excel ──────────────────────────────────────────────────────────────
    StepTemplate { category: "Excel", display_name: "シート読み込み",          name: "excel_read_sheet",  yaml: "excel_read_sheet:\n  file: data.xlsx\n  sheet: Sheet1\n  has_header: true\n  save_as: rows\n" },
    StepTemplate { category: "Excel", display_name: "範囲読み込み",            name: "excel_read_range",  yaml: "excel_read_range:\n  file: data.xlsx\n  sheet: Sheet1\n  range: \"A1:D100\"\n  save_as: rows\n" },
    StepTemplate { category: "Excel", display_name: "セル読み込み",            name: "excel_read_cell",   yaml: "excel_read_cell:\n  file: data.xlsx\n  sheet: Sheet1\n  cell: A1\n  save_as: value\n" },
    StepTemplate { category: "Excel", display_name: "セル書き込み",            name: "excel_write_cell",  yaml: "excel_write_cell:\n  file: data.xlsx\n  sheet: Sheet1\n  cell: A1\n  value: \"{{ var }}\"\n" },
    StepTemplate { category: "Excel", display_name: "範囲書き込み",            name: "excel_write_range", yaml: "excel_write_range:\n  file: data.xlsx\n  sheet: Sheet1\n  cell: A2\n  data: rows\n" },
    StepTemplate { category: "Excel", display_name: "シートサイズ取得",        name: "excel_get_dims",    yaml: "excel_get_dims:\n  file: data.xlsx\n  sheet: Sheet1\n  save_as: dims\n" },
    StepTemplate { category: "Excel", display_name: "行検索",                  name: "excel_find_row",    yaml: "excel_find_row:\n  file: data.xlsx\n  col: A\n  value: \"{{ search_val }}\"\n  save_as: row_num\n" },
    StepTemplate { category: "Excel", display_name: "シート追加",              name: "excel_add_sheet",   yaml: "excel_add_sheet:\n  file: data.xlsx\n  name: NewSheet\n" },
    StepTemplate { category: "Excel", display_name: "シート削除",              name: "excel_delete_sheet",yaml: "excel_delete_sheet:\n  file: data.xlsx\n  name: OldSheet\n" },
    StepTemplate { category: "Excel", display_name: "シート名変更",            name: "excel_rename_sheet",yaml: "excel_rename_sheet:\n  file: data.xlsx\n  from_name: Sheet1\n  to_name: Data\n" },
    // ── データ ─────────────────────────────────────────────────────────────
    StepTemplate { category: "データ", display_name: "DB クエリ(複数行)",      name: "db_query",          yaml: "db_query:\n  url: \"sqlite://./data.db\"\n  sql: \"SELECT * FROM table\"\n  save_as: rows\n" },
    StepTemplate { category: "データ", display_name: "DB クエリ(1行)",         name: "db_query_one",      yaml: "db_query_one:\n  url: \"sqlite://./data.db\"\n  sql: \"SELECT * FROM table WHERE id=?\"\n  params: [\"{{ id }}\"]\n  save_as: row\n" },
    StepTemplate { category: "データ", display_name: "DB 実行",                name: "db_execute",        yaml: "db_execute:\n  url: \"sqlite://./data.db\"\n  sql: \"UPDATE table SET col=? WHERE id=?\"\n  params: [\"{{ val }}\", \"{{ id }}\"]\n" },
    StepTemplate { category: "データ", display_name: "PDF テキスト抽出",       name: "pdf_extract_text",  yaml: "pdf_extract_text:\n  file: report.pdf\n  save_as: pdf_text\n" },
    StepTemplate { category: "データ", display_name: "PDF ページ数",           name: "pdf_page_count",    yaml: "pdf_page_count:\n  file: report.pdf\n  save_as: n\n" },
    // ── 文字列 ─────────────────────────────────────────────────────────────
    StepTemplate { category: "文字列", display_name: "文字列置換",             name: "string_replace",    yaml: "string_replace:\n  value: \"{{ text }}\"\n  from: old\n  to: new\n  save_as: result\n" },
    StepTemplate { category: "文字列", display_name: "前後の空白除去",         name: "string_trim",       yaml: "string_trim:\n  value: \"{{ text }}\"\n  save_as: result\n" },
    StepTemplate { category: "文字列", display_name: "大文字変換",             name: "string_upper",      yaml: "string_upper:\n  value: \"{{ text }}\"\n  save_as: result\n" },
    StepTemplate { category: "文字列", display_name: "小文字変換",             name: "string_lower",      yaml: "string_lower:\n  value: \"{{ text }}\"\n  save_as: result\n" },
    StepTemplate { category: "文字列", display_name: "部分文字列取得",         name: "string_substring",  yaml: "string_substring:\n  value: \"{{ text }}\"\n  start: 0\n  length: 5\n  save_as: result\n" },
    StepTemplate { category: "文字列", display_name: "文字列長取得",           name: "string_length",     yaml: "string_length:\n  value: \"{{ text }}\"\n  save_as: len\n" },
    StepTemplate { category: "文字列", display_name: "文字列分割",             name: "string_split",      yaml: "string_split:\n  value: \"{{ text }}\"\n  delimiter: \",\"\n  save_as: parts\n" },
    StepTemplate { category: "文字列", display_name: "文字列結合",             name: "string_join",       yaml: "string_join:\n  value: parts\n  separator: \",\"\n  save_as: result\n" },
    StepTemplate { category: "文字列", display_name: "正規表現マッチ",         name: "string_regex",      yaml: "string_regex:\n  value: \"{{ text }}\"\n  pattern: \"\\\\d+\"\n  save_as: match\n" },
    StepTemplate { category: "文字列", display_name: "文字列フォーマット",     name: "string_format",     yaml: "string_format:\n  format: \"Hello, {0}!\"\n  args: [\"{{ name }}\"]\n  save_as: msg\n" },
    StepTemplate { category: "文字列", display_name: "文字列を含むか確認",     name: "string_contains",   yaml: "string_contains:\n  value: \"{{ text }}\"\n  search: keyword\n  save_as: found\n" },
    StepTemplate { category: "文字列", display_name: "前方一致確認",           name: "string_starts_with",yaml: "string_starts_with:\n  value: \"{{ text }}\"\n  search: prefix\n  save_as: result\n" },
    StepTemplate { category: "文字列", display_name: "後方一致確認",           name: "string_ends_with",  yaml: "string_ends_with:\n  value: \"{{ text }}\"\n  search: .pdf\n  save_as: result\n" },
    StepTemplate { category: "文字列", display_name: "文字列検索位置",         name: "string_index_of",   yaml: "string_index_of:\n  value: \"{{ text }}\"\n  search: keyword\n  save_as: pos\n" },
    StepTemplate { category: "文字列", display_name: "出現回数カウント",       name: "string_count",      yaml: "string_count:\n  value: \"{{ text }}\"\n  search: \",\"\n  save_as: count\n" },
    // ── 日付 ───────────────────────────────────────────────────────────────
    StepTemplate { category: "日付", display_name: "日付フォーマット変換",     name: "date_format",       yaml: "date_format:\n  value: \"{{ date }}\"\n  from_format: \"%Y%m%d\"\n  to_format: \"%Y/%m/%d\"\n  save_as: result\n" },
    StepTemplate { category: "日付", display_name: "日付加算",                 name: "date_add",          yaml: "date_add:\n  value: \"{{ date }}\"\n  format: \"%Y-%m-%d\"\n  days: 1\n  save_as: result\n" },
    StepTemplate { category: "日付", display_name: "日付差分",                 name: "date_diff",         yaml: "date_diff:\n  from: \"{{ date1 }}\"\n  to: \"{{ date2 }}\"\n  format: \"%Y-%m-%d\"\n  unit: days\n  save_as: diff\n" },
    // ── JSON ───────────────────────────────────────────────────────────────
    StepTemplate { category: "JSON", display_name: "JSON パース",              name: "json_parse",        yaml: "json_parse:\n  value: \"{{ json_str }}\"\n  save_as: obj\n" },
    StepTemplate { category: "JSON", display_name: "JSON 文字列化",            name: "json_stringify",    yaml: "json_stringify:\n  value: my_var\n  save_as: json_str\n" },
    // ── パス ───────────────────────────────────────────────────────────────
    StepTemplate { category: "パス", display_name: "パス結合",                 name: "path_join",         yaml: "path_join:\n  parts: [dir, sub, file.txt]\n  save_as: path\n" },
    StepTemplate { category: "パス", display_name: "ファイル名取得",           name: "path_basename",     yaml: "path_basename:\n  path: \"{{ filepath }}\"\n  save_as: name\n" },
    StepTemplate { category: "パス", display_name: "ディレクトリ取得",         name: "path_dirname",      yaml: "path_dirname:\n  path: \"{{ filepath }}\"\n  save_as: dir\n" },
    StepTemplate { category: "パス", display_name: "環境変数取得",             name: "env_get",           yaml: "env_get:\n  name: HOME\n  save_as: home\n" },
    // ── マウス ─────────────────────────────────────────────────────────────
    StepTemplate { category: "マウス", display_name: "マウス移動",             name: "mouse_move",        yaml: "mouse_move:\n  x: \"500\"\n  y: \"300\"\n" },
    StepTemplate { category: "マウス", display_name: "座標クリック",           name: "mouse_click_xy",    yaml: "mouse_click_xy:\n  x: \"500\"\n  y: \"300\"\n  action: left\n" },
    StepTemplate { category: "マウス", display_name: "ドラッグ",               name: "mouse_drag",        yaml: "mouse_drag:\n  from_x: \"100\"\n  from_y: \"100\"\n  to_x: \"500\"\n  to_y: \"300\"\n" },
    StepTemplate { category: "マウス", display_name: "スクロール",             name: "mouse_scroll",      yaml: "mouse_scroll:\n  direction: down\n  amount: 3\n" },
    StepTemplate { category: "マウス", display_name: "ホバー",                 name: "mouse_hover",       yaml: "mouse_hover:\n  x: \"500\"\n  y: \"300\"\n  hover_ms: 500\n" },
    // ── プロセス ───────────────────────────────────────────────────────────
    StepTemplate { category: "プロセス", display_name: "プロセス起動",         name: "process_start",     yaml: "process_start:\n  command: notepad.exe\n  wait_ms: 1000\n" },
    StepTemplate { category: "プロセス", display_name: "プロセス終了",         name: "process_kill",      yaml: "process_kill:\n  name: notepad.exe\n" },
    StepTemplate { category: "プロセス", display_name: "プロセス存在確認",     name: "process_exists",    yaml: "process_exists:\n  name: notepad.exe\n  save_as: running\n" },
    // ── HTTP ───────────────────────────────────────────────────────────────
    StepTemplate { category: "HTTP", display_name: "GET リクエスト",           name: "http_get",          yaml: "http_get:\n  url: \"https://api.example.com/data\"\n  save_as: response\n" },
    StepTemplate { category: "HTTP", display_name: "POST リクエスト",          name: "http_post",         yaml: "http_post:\n  url: \"https://api.example.com/data\"\n  body:\n    key: value\n  save_as: response\n" },
    StepTemplate { category: "HTTP", display_name: "PUT リクエスト",           name: "http_put",          yaml: "http_put:\n  url: \"https://api.example.com/data/1\"\n  body:\n    key: value\n  save_as: response\n" },
    StepTemplate { category: "HTTP", display_name: "PATCH リクエスト",         name: "http_patch",        yaml: "http_patch:\n  url: \"https://api.example.com/data/1\"\n  body:\n    key: value\n  save_as: response\n" },
    StepTemplate { category: "HTTP", display_name: "DELETE リクエスト",        name: "http_delete",       yaml: "http_delete:\n  url: \"https://api.example.com/data/1\"\n  save_as: response\n" },
    // ── メール ─────────────────────────────────────────────────────────────
    StepTemplate { category: "メール", display_name: "メール送信",             name: "mail_send",         yaml: "mail_send:\n  host: smtp.example.com\n  user: user@example.com\n  password: \"{{ env.MAIL_PASS }}\"\n  from: user@example.com\n  to: to@example.com\n  subject: 件名\n  body: 本文\n" },
    StepTemplate { category: "メール", display_name: "メール受信(IMAP)",       name: "mail_receive",      yaml: "mail_receive:\n  host: imap.example.com\n  user: user@example.com\n  password: \"{{ env.MAIL_PASS }}\"\n  count: 10\n  only_unseen: true\n  save_as: emails\n" },
    // ── Web ────────────────────────────────────────────────────────────────
    StepTemplate { category: "Web", display_name: "ブラウザを開く",            name: "web_open",          yaml: "web_open:\n  url: \"https://example.com\"\n" },
    StepTemplate { category: "Web", display_name: "要素クリック",              name: "web_click",         yaml: "web_click:\n  selector: \"#submit-btn\"\n" },
    StepTemplate { category: "Web", display_name: "テキスト入力",              name: "web_type",          yaml: "web_type:\n  selector: \"#username\"\n  text: \"{{ user }}\"\n" },
    StepTemplate { category: "Web", display_name: "テキスト/属性取得",         name: "web_get",           yaml: "web_get:\n  selector: .result\n  save_as: text\n" },
    StepTemplate { category: "Web", display_name: "要素待機",                  name: "web_wait",          yaml: "web_wait:\n  selector: \"#content\"\n  timeout_ms: 5000\n" },
    StepTemplate { category: "Web", display_name: "ブラウザスクリーンショット",name: "web_screenshot",    yaml: "web_screenshot:\n  path: web_screen.png\n" },
    StepTemplate { category: "Web", display_name: "ブラウザを閉じる",          name: "web_close",         yaml: "web_close: ~\n" },
    StepTemplate { category: "Web", display_name: "テキスト含有待機",          name: "web_wait_text",     yaml: "web_wait_text:\n  selector: .status\n  text: 完了\n  timeout_ms: 10000\n" },
    StepTemplate { category: "Web", display_name: "ドロップダウン選択",        name: "web_select",        yaml: "web_select:\n  selector: \"#country\"\n  item: Japan\n" },
    StepTemplate { category: "Web", display_name: "JavaScript 実行",           name: "web_execute_js",    yaml: "web_execute_js:\n  script: \"return document.title;\"\n  save_as: title\n" },
    StepTemplate { category: "Web", display_name: "フレーム切り替え",          name: "web_switch_frame",  yaml: "web_switch_frame:\n  selector: \"#iframe1\"\n" },
    StepTemplate { category: "Web", display_name: "スクロール",                name: "web_scroll",        yaml: "web_scroll:\n  y: 300\n" },
    StepTemplate { category: "Web", display_name: "アラート処理",              name: "web_alert",         yaml: "web_alert:\n  action: accept\n" },
    StepTemplate { category: "Web", display_name: "URL 取得",                  name: "web_get_url",       yaml: "web_get_url:\n  save_as: url\n" },
    StepTemplate { category: "Web", display_name: "タイトル取得",              name: "web_get_title",     yaml: "web_get_title:\n  save_as: title\n" },
    StepTemplate { category: "Web", display_name: "全要素テキスト取得",        name: "web_get_all",       yaml: "web_get_all:\n  selector: .item\n  save_as: items\n" },
    // ── UIA ────────────────────────────────────────────────────────────────
    StepTemplate { category: "UIA", display_name: "プロパティ取得",            name: "uia_get",           yaml: "uia_get:\n  by:\n    name: ユーザー名\n  property: value\n  save_as: result\n" },
    StepTemplate { category: "UIA", display_name: "値設定",                    name: "uia_set",           yaml: "uia_set:\n  by:\n    name: 入力欄\n  value: \"{{ text }}\"\n" },
    StepTemplate { category: "UIA", display_name: "要素クリック",              name: "uia_click",         yaml: "uia_click:\n  by:\n    name: OK\n" },
    StepTemplate { category: "UIA", display_name: "要素検索",                  name: "uia_find",          yaml: "uia_find:\n  by:\n    name: ボタン\n  save_as: element\n" },
    StepTemplate { category: "UIA", display_name: "状態待機",                  name: "uia_wait",          yaml: "uia_wait:\n  by:\n    name: OK\n  state: enabled\n  timeout_ms: 10000\n" },
    StepTemplate { category: "UIA", display_name: "項目選択",                  name: "uia_select",        yaml: "uia_select:\n  by:\n    name: Country\n  item: Japan\n" },
    StepTemplate { category: "UIA", display_name: "子要素一覧取得",            name: "uia_get_children",  yaml: "uia_get_children:\n  by:\n    name: Files\n  save_as: items\n" },
    StepTemplate { category: "UIA", display_name: "チェックボックス操作",      name: "uia_check",         yaml: "uia_check:\n  by:\n    name: 同意する\n  checked: true\n" },
    // ── CSV ────────────────────────────────────────────────────────────────
    StepTemplate { category: "CSV", display_name: "CSV 読み込み",              name: "csv_read",          yaml: "csv_read:\n  path: data.csv\n  has_header: true\n  save_as: rows\n" },
    StepTemplate { category: "CSV", display_name: "CSV 書き込み",              name: "csv_write",         yaml: "csv_write:\n  path: output.csv\n  rows: my_rows\n" },
    // ── リスト ─────────────────────────────────────────────────────────────
    StepTemplate { category: "リスト", display_name: "リスト長取得",           name: "list_length",       yaml: "list_length:\n  list: my_list\n  save_as: len\n" },
    StepTemplate { category: "リスト", display_name: "要素取得",               name: "list_get",          yaml: "list_get:\n  list: my_list\n  index: \"0\"\n  save_as: item\n" },
    StepTemplate { category: "リスト", display_name: "要素追加",               name: "list_push",         yaml: "list_push:\n  list: my_list\n  value: \"{{ item }}\"\n" },
    StepTemplate { category: "リスト", display_name: "要素削除",               name: "list_remove",       yaml: "list_remove:\n  list: my_list\n  index: \"0\"\n" },
    StepTemplate { category: "リスト", display_name: "要素の存在確認",         name: "list_contains",     yaml: "list_contains:\n  list: my_list\n  value: target\n  save_as: found\n" },
    // ── ユーティリティ ─────────────────────────────────────────────────────
    StepTemplate { category: "ユーティリティ", display_name: "Base64 エンコード", name: "base64_encode",  yaml: "base64_encode:\n  value: \"{{ text }}\"\n  save_as: encoded\n" },
    StepTemplate { category: "ユーティリティ", display_name: "Base64 デコード",   name: "base64_decode",  yaml: "base64_decode:\n  value: \"{{ encoded }}\"\n  save_as: text\n" },
    StepTemplate { category: "ユーティリティ", display_name: "数値変換",          name: "to_number",      yaml: "to_number:\n  value: \"{{ str_num }}\"\n  save_as: num\n" },
    StepTemplate { category: "ユーティリティ", display_name: "文字列変換",        name: "to_string",      yaml: "to_string:\n  value: my_num\n  save_as: str\n" },
    StepTemplate { category: "ユーティリティ", display_name: "変数の型取得",      name: "var_type",       yaml: "var_type:\n  value: my_var\n  save_as: type_name\n" },
    StepTemplate { category: "ユーティリティ", display_name: "乱数生成",          name: "number_random",  yaml: "number_random:\n  min: 1.0\n  max: 100.0\n  integer: true\n  save_as: rand\n" },
    StepTemplate { category: "ユーティリティ", display_name: "URL を開く",        name: "url_open",       yaml: "url_open:\n  url: \"https://example.com\"\n" },
    StepTemplate { category: "ユーティリティ", display_name: "デスクトップ通知",  name: "notify",         yaml: "notify:\n  title: 完了\n  message: 処理が完了しました\n" },
];

/// Return a Phosphor Regular icon character for the given step key.
pub(crate) fn step_icon(name: &str) -> &'static str {
    use egui_phosphor::regular as ph;
    match name {
        // AI
        "ai_create" => ph::ROBOT,
        // 制御フロー
        "if" => ph::GIT_BRANCH,
        "foreach" => ph::REPEAT,
        "repeat" => ph::ARROWS_CLOCKWISE,
        "while" => ph::ARROW_CLOCKWISE,
        "do_while" => ph::ARROW_ARC_LEFT,
        "try_catch" => ph::SHIELD,
        "group" => ph::FOLDER,
        "switch" => ph::ARROWS_SPLIT,
        "sub_scenario" | "call_scenario" => ph::FLOW_ARROW,
        "exit" => ph::DOOR,
        "break" => ph::STOP,
        "continue" => ph::SKIP_FORWARD,
        // 画像操作
        "wait_image" => ph::EYE,
        "click_image" => ph::CURSOR_CLICK,
        "find_image" => ph::MAGNIFYING_GLASS,
        "match_rect" => ph::SELECTION,
        "wait_no_image" => ph::PROHIBIT,
        "wait_change" => ph::MONITOR,
        "ocr_match" => ph::SCAN,
        "ml_detect" => ph::BRAIN,
        "screenshot_save" => ph::CAMERA,
        "get_pixel_color" | "wait_color" => ph::PAINT_BUCKET,
        "window_control" => ph::APP_WINDOW,
        // 入力操作
        "type" | "press" | "key_combo" => ph::KEYBOARD,
        "click_in_window" => ph::CURSOR,
        // 待機
        "wait_ms" => ph::TIMER,
        "wait_window" => ph::CLOCK,
        "wait_until" | "wait_process" => ph::HOURGLASS,
        // 変数
        "set" => ph::PACKAGE,
        "copy_var" => ph::COPY,
        "get_datetime" => ph::CALENDAR,
        "get_username" => ph::USER,
        "calc" => ph::CALCULATOR,
        "increment" => ph::PLUS_CIRCLE,
        "to_fullwidth" | "to_halfwidth" => ph::TRANSLATE,
        "import_vars" => ph::DOWNLOAD,
        "save_vars" => ph::FLOPPY_DISK,
        "load_vars" => ph::UPLOAD,
        // クリップボード
        "clipboard_set" | "clipboard_get" => ph::CLIPBOARD_TEXT,
        // ダイアログ
        "dialog_wait" => ph::CHAT_TEXT,
        "dialog_input" => ph::NOTE_PENCIL,
        "dialog_select" => ph::LIST_BULLETS,
        // スクリプト
        "shell" | "script" => ph::TERMINAL_WINDOW,
        // ライブラリ
        "library" => ph::PUZZLE_PIECE,
        // ファイル
        "file_copy" => ph::COPY,
        "file_move" => ph::ARROW_SQUARE_OUT,
        "file_delete" | "dir_delete" => ph::TRASH,
        "file_rename" => ph::PENCIL_SIMPLE,
        "file_exists" | "dir_exists" => ph::QUESTION,
        "file_read" => ph::BOOK_OPEN,
        "file_write" | "file_append" | "log_write" => ph::FILE_TEXT,
        "file_size" | "file_modified_at" | "file_list" => ph::FILE_SEARCH,
        "dir_create" => ph::FOLDER,
        "zip_compress" | "zip_extract" | "zip_list" => ph::FILE_PLUS,
        "ftp_upload" => ph::UPLOAD,
        "ftp_download" => ph::DOWNLOAD,
        "ftp_list" | "ftp_delete" | "ftp_mkdir" => ph::LINK_SIMPLE,
        // Excel
        "excel_read_sheet" | "excel_read_range" | "excel_read_cell" => ph::TABLE,
        "excel_write_cell" | "excel_write_range" => ph::PENCIL_SIMPLE,
        "excel_get_dims" | "excel_find_row" => ph::MAGNIFYING_GLASS,
        "excel_add_sheet" | "excel_delete_sheet" | "excel_rename_sheet" => ph::COLUMNS,
        // データ
        "db_query" | "db_query_one" | "db_execute" => ph::DATABASE,
        "pdf_extract_text" | "pdf_page_count" => ph::FILE_TEXT,
        // 文字列
        "string_replace" | "string_trim" | "string_upper" | "string_lower" | "string_substring"
        | "string_length" | "string_split" | "string_join" | "string_regex" | "string_format"
        | "string_contains" | "string_starts_with" | "string_ends_with" | "string_index_of"
        | "string_count" => ph::QUOTES,
        // 日付
        "date_format" | "date_add" | "date_diff" => ph::CALENDAR,
        // JSON
        "json_parse" | "json_stringify" => ph::HASH_STRAIGHT,
        // パス
        "path_join" | "path_basename" | "path_dirname" => ph::PATH,
        "env_get" => ph::LEAF,
        // マウス
        "mouse_move" | "mouse_click_xy" | "mouse_drag" | "mouse_scroll" | "mouse_hover" => {
            ph::MOUSE
        }
        // プロセス
        "process_start" | "process_kill" | "process_exists" => ph::GEAR,
        // HTTP
        "http_get" | "http_post" | "http_put" | "http_patch" | "http_delete" => ph::GLOBE,
        // メール
        "mail_send" | "mail_receive" => ph::ENVELOPE,
        // Web
        "web_open" | "web_close" | "web_get_url" | "web_get_title" | "web_scroll"
        | "web_switch_frame" => ph::GLOBE_SIMPLE,
        "web_click" | "web_select" | "web_alert" => ph::CURSOR_CLICK,
        "web_type" | "web_execute_js" => ph::TERMINAL_WINDOW,
        "web_wait" | "web_wait_text" => ph::TIMER,
        "web_screenshot" => ph::CAMERA,
        "web_get" | "web_get_all" => ph::LIST,
        // UIA
        "uia_get" | "uia_set" | "uia_click" | "uia_find" | "uia_wait" | "uia_select"
        | "uia_get_children" | "uia_check" => ph::SLIDERS,
        // CSV
        "csv_read" | "csv_write" => ph::TABLE,
        // リスト
        "list_length" | "list_get" | "list_push" | "list_remove" | "list_contains" => {
            ph::LIST_NUMBERS
        }
        // ユーティリティ
        "base64_encode" | "base64_decode" => ph::HASH,
        "to_number" | "to_string" | "var_type" | "number_random" => ph::ARROWS_LEFT_RIGHT,
        "url_open" => ph::GLOBE,
        "notify" => ph::BELL,
        _ => ph::SPARKLE,
    }
}
