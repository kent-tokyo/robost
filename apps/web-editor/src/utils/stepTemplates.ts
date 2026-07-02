export type StepCategory = 'フロー' | 'アクション' | 'ユーザ' | '変数' | 'データ'

export interface StepTemplate {
  type: string
  label: string
  category: StepCategory
  defaults: Record<string, unknown>
}

export const STEP_TEMPLATES: StepTemplate[] = [
  // ── フロー ──────────────────────────────────────────────────────────────
  { type: 'group',           label: 'グループ',               category: 'フロー',     defaults: { do: [] } },
  { type: 'if',              label: '分岐 (if)',              category: 'フロー',     defaults: { cond: '', then: [], else: [] } },
  { type: 'switch',          label: '多分岐 (switch)',        category: 'フロー',     defaults: { on: '', cases: [], default: [] } },
  { type: 'foreach',         label: '繰り返し (foreach)',     category: 'フロー',     defaults: { var: 'rows', do: [] } },
  { type: 'repeat',          label: '繰り返し (repeat)',      category: 'フロー',     defaults: { count: 1, do: [] } },
  { type: 'while',           label: '繰り返し (while)',       category: 'フロー',     defaults: { cond: '', do: [] } },
  { type: 'do_while',        label: '後判定繰返 (do_while)',  category: 'フロー',     defaults: { cond: '', do: [] } },
  { type: 'try_catch',       label: '例外処理',               category: 'フロー',     defaults: { try: [], catch: [], finally: [] } },
  { type: 'call_scenario',   label: 'シナリオ呼び出し',       category: 'フロー',     defaults: { path: '', inputs: {} } },
  { type: 'sub_scenario',    label: 'サブルーチン呼び出し',   category: 'フロー',     defaults: { path: '', inputs: {} } },

  // ── アクション ──────────────────────────────────────────────────────────
  { type: 'click_image',     label: '画像クリック',           category: 'アクション', defaults: { template: '', threshold: 0.87 } },
  { type: 'wait_image',      label: '画像待機',               category: 'アクション', defaults: { template: '' } },
  { type: 'match_rect',      label: '矩形マッチング',         category: 'アクション', defaults: { template: '', rect: { x: 0, y: 0, width: 100, height: 100 } } },
  { type: 'wait_window',     label: 'ウィンドウ状態待機',     category: 'アクション', defaults: { title_contains: '' } },
  { type: 'wait_ms',         label: '指定時間待機',           category: 'アクション', defaults: { ms: 1000 } },
  { type: 'type',            label: '文字列送信',             category: 'アクション', defaults: { text: '' } },
  { type: 'shell',           label: 'コマンド実行',           category: 'アクション', defaults: { cmd: '' } },
  { type: 'script',          label: 'スクリプト実行',         category: 'アクション', defaults: { code: '' } },
  { type: 'window_control',  label: 'ウィンドウ前面化',       category: 'アクション', defaults: { title_contains: '', action: 'focus' } },
  { type: 'click_text',      label: '文字クリック',           category: 'アクション', defaults: { text: '', lang: 'jpn', action: 'left' } },
  { type: 'move_to_text',    label: 'テキストへ移動',         category: 'アクション', defaults: { text: '', lang: 'jpn' } },

  // ── ユーザ ──────────────────────────────────────────────────────────────
  { type: 'dialog_wait',     label: '待機ボックス',           category: 'ユーザ',     defaults: { message: '' } },
  { type: 'dialog_input',    label: 'インプットボックス',     category: 'ユーザ',     defaults: { message: '', save_as: 'input_value' } },
  { type: 'dialog_select',   label: '選択ボックス',           category: 'ユーザ',     defaults: { message: '', options: [], save_as: 'selected' } },

  // ── 変数 ────────────────────────────────────────────────────────────────
  { type: 'set',             label: '変数値設定',             category: '変数',       defaults: { name: '', value: '' } },
  { type: 'copy_var',        label: '変数値コピー',           category: '変数',       defaults: { from: '', to: '' } },
  { type: 'get_datetime',    label: '日時取得',               category: '変数',       defaults: { save_as: 'now' } },
  { type: 'calc',            label: '四則演算',               category: '変数',       defaults: { expr: '', save_as: 'result' } },
  { type: 'increment',       label: 'カウントアップ',         category: '変数',       defaults: { name: '' } },

  // ── データ ──────────────────────────────────────────────────────────────
  { type: 'excel_read_sheet', label: 'Excel/CSV読み込み',    category: 'データ',     defaults: { file: '', has_header: true, save_as: 'rows' } },
  { type: 'csv_read',         label: 'CSV読み込み',           category: 'データ',     defaults: { path: '', has_header: true, save_as: 'rows' } },
  { type: 'import_vars',      label: '変数インポート (1行)',  category: 'データ',     defaults: { file: '', row: 0, prefix: '' } },
]
