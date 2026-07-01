import { useState, useRef, useEffect } from 'react'
import { X, Save, Play, Plus, ChevronDown, ChevronRight, Trash2 } from 'lucide-react'
import { type ScenarioStep, normalizeStep, denormalizeStep } from '../utils/yamlFlow'
import { useScenarioStore } from '../store/scenarioStore'
import './StepEditor.css'

interface Props {
  step: ScenarioStep
  stepIndex: number
  onUpdate: (patch: Partial<ScenarioStep>) => void
  onClose: () => void
  onSave?: () => void
  onRunStep?: () => void
}

// ── 汎用フィールドコンポーネント ──────────────────────────────────────────

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="se-field">
      <label className="se-label">{label}</label>
      {children}
    </div>
  )
}

function TextInput({
  value,
  onChange,
  placeholder,
}: {
  value: string
  onChange: (v: string) => void
  placeholder?: string
}) {
  return (
    <input
      className="se-input"
      type="text"
      value={value}
      onChange={(e) => onChange(e.target.value)}
      placeholder={placeholder}
    />
  )
}

function Select({
  value,
  options,
  onChange,
}: {
  value: string
  options: { value: string; label: string }[]
  onChange: (v: string) => void
}) {
  return (
    <select className="se-select" value={value} onChange={(e) => onChange(e.target.value)}>
      {options.map((o) => (
        <option key={o.value} value={o.value}>
          {o.label}
        </option>
      ))}
    </select>
  )
}

function NumberInput({
  value,
  onChange,
  min,
}: {
  value: number
  onChange: (v: number) => void
  min?: number
}) {
  return (
    <input
      className="se-input"
      type="number"
      value={value}
      min={min}
      onChange={(e) => onChange(Number(e.target.value))}
    />
  )
}

// ── ステップタイプ別エディタ ──────────────────────────────────────────────

function WindowControlEditor({ step, onUpdate }: { step: ScenarioStep; onUpdate: (p: Partial<ScenarioStep>) => void }) {
  return (
    <>
      <Field label="ウィンドウタイトル（含む）">
        <TextInput
          value={(step.title_contains as string) ?? ''}
          onChange={(v) => onUpdate({ title_contains: v })}
          placeholder="例: メモ帳"
        />
      </Field>
      <Field label="アクション">
        <Select
          value={(step.action as string) ?? 'focus'}
          options={[
            { value: 'focus',    label: '前面にする (Focus)' },
            { value: 'maximize', label: '最大化 (Maximize)' },
            { value: 'minimize', label: '最小化 (Minimize)' },
            { value: 'close',    label: '閉じる (Close)' },
          ]}
          onChange={(v) => onUpdate({ action: v })}
        />
      </Field>
    </>
  )
}

function WaitEditor({ step, onUpdate }: { step: ScenarioStep; onUpdate: (p: Partial<ScenarioStep>) => void }) {
  return (
    <Field label="待機時間 (ms)">
      <NumberInput
        value={(step.ms as number) ?? 1000}
        min={0}
        onChange={(v) => onUpdate({ ms: v })}
      />
    </Field>
  )
}

function TypeTextEditor({ step, onUpdate }: { step: ScenarioStep; onUpdate: (p: Partial<ScenarioStep>) => void }) {
  return (
    <Field label="入力テキスト">
      <TextInput
        value={(step.text as string) ?? ''}
        onChange={(v) => onUpdate({ text: v })}
        placeholder="例: Hello"
      />
    </Field>
  )
}

function ClickImageEditor({ step, onUpdate }: { step: ScenarioStep; onUpdate: (p: Partial<ScenarioStep>) => void }) {
  return (
    <>
      <Field label="テンプレート画像パス">
        <TextInput
          value={(step.template as string) ?? ''}
          onChange={(v) => onUpdate({ template: v })}
          placeholder="例: templates/button.png"
        />
      </Field>
      <Field label="マッチング閾値 (0〜1)">
        <input
          className="se-input"
          type="number"
          step="0.01"
          min={0}
          max={1}
          value={(step.threshold as number) ?? 0.87}
          onChange={(e) => onUpdate({ threshold: parseFloat(e.target.value) })}
        />
      </Field>
    </>
  )
}

function ClickTextEditor({ step, onUpdate }: { step: ScenarioStep; onUpdate: (p: Partial<ScenarioStep>) => void }) {
  return (
    <>
      <Field label="クリックするテキスト">
        <TextInput
          value={(step.text as string) ?? ''}
          onChange={(v) => onUpdate({ text: v })}
          placeholder="例: 送信"
        />
      </Field>
      <Field label="言語">
        <Select
          value={(step.lang as string) ?? 'jpn'}
          options={[
            { value: 'jpn',     label: '日本語 (jpn)' },
            { value: 'eng',     label: '英語 (eng)' },
            { value: 'jpn+eng', label: '日本語+英語 (jpn+eng)' },
          ]}
          onChange={(v) => onUpdate({ lang: v })}
        />
      </Field>
      <Field label="クリック種別">
        <Select
          value={(step.action as string) ?? 'left'}
          options={[
            { value: 'left',   label: '左クリック' },
            { value: 'right',  label: '右クリック' },
            { value: 'double', label: 'ダブルクリック' },
          ]}
          onChange={(v) => onUpdate({ action: v })}
        />
      </Field>
      <Field label="X オフセット (px)  ← 右が正">
        <NumberInput
          value={(step.offset_x as number) ?? 0}
          onChange={(v) => onUpdate({ offset_x: v })}
        />
      </Field>
      <Field label="Y オフセット (px)  ↓ 下が正">
        <NumberInput
          value={(step.offset_y as number) ?? 0}
          onChange={(v) => onUpdate({ offset_y: v })}
        />
      </Field>
    </>
  )
}

function MoveToTextEditor({ step, onUpdate }: { step: ScenarioStep; onUpdate: (p: Partial<ScenarioStep>) => void }) {
  return (
    <>
      <Field label="移動先のテキスト">
        <TextInput
          value={(step.text as string) ?? ''}
          onChange={(v) => onUpdate({ text: v })}
          placeholder="例: ファイル"
        />
      </Field>
      <Field label="言語">
        <Select
          value={(step.lang as string) ?? 'jpn'}
          options={[
            { value: 'jpn',     label: '日本語 (jpn)' },
            { value: 'eng',     label: '英語 (eng)' },
            { value: 'jpn+eng', label: '日本語+英語 (jpn+eng)' },
          ]}
          onChange={(v) => onUpdate({ lang: v })}
        />
      </Field>
      <Field label="X オフセット (px)  ← 右が正">
        <NumberInput
          value={(step.offset_x as number) ?? 0}
          onChange={(v) => onUpdate({ offset_x: v })}
        />
      </Field>
      <Field label="Y オフセット (px)  ↓ 下が正">
        <NumberInput
          value={(step.offset_y as number) ?? 0}
          onChange={(v) => onUpdate({ offset_y: v })}
        />
      </Field>
    </>
  )
}

function ExcelReadSheetEditor({ step, onUpdate }: { step: ScenarioStep; onUpdate: (p: Partial<ScenarioStep>) => void }) {
  return (
    <>
      <Field label="ファイルパス (.xlsx / .csv)">
        <TextInput
          value={(step.file as string) ?? ''}
          onChange={(v) => onUpdate({ file: v })}
          placeholder="例: data.xlsx"
        />
      </Field>
      <Field label="シート名 (Excelのみ、省略可)">
        <TextInput
          value={(step.sheet as string) ?? ''}
          onChange={(v) => onUpdate({ sheet: v || undefined })}
          placeholder="例: Sheet1"
        />
      </Field>
      <Field label="1行目を見出し行として使う">
        <label style={{ display: 'flex', alignItems: 'center', gap: 8, cursor: 'pointer' }}>
          <input
            type="checkbox"
            checked={(step.has_header as boolean) ?? true}
            onChange={(e) => onUpdate({ has_header: e.target.checked })}
          />
          <span style={{ fontSize: 13, color: '#9090b0' }}>有効（チェック ON で列名→変数キー）</span>
        </label>
      </Field>
      <Field label="保存する変数名">
        <TextInput
          value={(step.save_as as string) ?? ''}
          onChange={(v) => onUpdate({ save_as: v })}
          placeholder="例: rows"
        />
      </Field>
    </>
  )
}

function CsvReadEditor({ step, onUpdate }: { step: ScenarioStep; onUpdate: (p: Partial<ScenarioStep>) => void }) {
  return (
    <>
      <Field label="CSVファイルパス">
        <TextInput
          value={(step.path as string) ?? ''}
          onChange={(v) => onUpdate({ path: v })}
          placeholder="例: data.csv"
        />
      </Field>
      <Field label="1行目を見出し行として使う">
        <label style={{ display: 'flex', alignItems: 'center', gap: 8, cursor: 'pointer' }}>
          <input
            type="checkbox"
            checked={(step.has_header as boolean) ?? true}
            onChange={(e) => onUpdate({ has_header: e.target.checked })}
          />
          <span style={{ fontSize: 13, color: '#9090b0' }}>有効（チェック ON で列名→変数キー）</span>
        </label>
      </Field>
      <Field label="保存する変数名">
        <TextInput
          value={(step.save_as as string) ?? ''}
          onChange={(v) => onUpdate({ save_as: v })}
          placeholder="例: rows"
        />
      </Field>
    </>
  )
}

function ImportVarsEditor({ step, onUpdate }: { step: ScenarioStep; onUpdate: (p: Partial<ScenarioStep>) => void }) {
  return (
    <>
      <Field label="ファイルパス (.xlsx / .csv)">
        <TextInput
          value={(step.file as string) ?? ''}
          onChange={(v) => onUpdate({ file: v })}
          placeholder="例: config.xlsx"
        />
      </Field>
      <Field label="シート名 (省略可)">
        <TextInput
          value={(step.sheet as string) ?? ''}
          onChange={(v) => onUpdate({ sheet: v || undefined })}
          placeholder="例: Sheet1"
        />
      </Field>
      <Field label="行番号 (0ベース、ヘッダー除く)">
        <NumberInput
          value={(step.row as number) ?? 0}
          min={0}
          onChange={(v) => onUpdate({ row: v })}
        />
      </Field>
      <Field label="変数名プレフィックス (省略可)">
        <TextInput
          value={(step.prefix as string) ?? ''}
          onChange={(v) => onUpdate({ prefix: v })}
          placeholder="例: cfg_ → cfg_列名"
        />
      </Field>
    </>
  )
}

// ── シナリオ呼び出し ──────────────────────────────────────────────────────

function CallScenarioEditor({
  step,
  onUpdate,
}: {
  step: ScenarioStep
  onUpdate: (p: Partial<ScenarioStep>) => void
}) {
  const { scenarios, folders } = useScenarioStore()

  // Flatten all available scenario paths
  const allScenarios = [
    ...scenarios,
    ...folders.flatMap((f) => f.scenarios.map((s) => `${f.name}/${s}`)),
  ]

  // inputs as array of [key, value] pairs for editing
  const inputsObj = (step.inputs as Record<string, string> | undefined) ?? {}
  const inputPairs: [string, string][] = Object.entries(inputsObj)

  const setInputPairs = (pairs: [string, string][]) => {
    const obj: Record<string, string> = {}
    for (const [k, v] of pairs) if (k) obj[k] = v
    onUpdate({ inputs: obj })
  }

  const addInput = () => setInputPairs([...inputPairs, ['', '']])
  const removeInput = (i: number) => setInputPairs(inputPairs.filter((_, idx) => idx !== i))
  const updateInput = (i: number, key: string, val: string) => {
    setInputPairs(inputPairs.map((pair, idx) => idx === i ? [key, val] : pair))
  }

  return (
    <>
      <Field label="呼び出すシナリオ">
        {allScenarios.length > 0 ? (
          <select
            className="se-select"
            value={(step.path as string) ?? ''}
            onChange={(e) => onUpdate({ path: e.target.value })}
          >
            <option value="">— 選択してください —</option>
            {allScenarios.map((s) => (
              <option key={s} value={s}>{s}</option>
            ))}
          </select>
        ) : (
          <TextInput
            value={(step.path as string) ?? ''}
            onChange={(v) => onUpdate({ path: v })}
            placeholder="例: sub_scenario.yaml"
          />
        )}
      </Field>

      {/* Input variables */}
      <div className="se-call-inputs">
        <div className="se-call-inputs-header">
          <span>渡す変数</span>
          <button className="foreach-add-btn" onClick={addInput}>
            <Plus size={11} /> 追加
          </button>
        </div>
        {inputPairs.length === 0 && (
          <p className="se-call-inputs-empty">変数なし（サブシナリオはデフォルト変数で実行）</p>
        )}
        {inputPairs.map(([k, v], i) => (
          <div key={i} className="se-call-input-row">
            <input
              className="se-input se-call-input-key"
              placeholder="変数名"
              value={k}
              onChange={(e) => updateInput(i, e.target.value, v)}
            />
            <span className="se-call-input-eq">→</span>
            <input
              className="se-input se-call-input-val"
              placeholder={'例: {{item.列名}}'}
              value={v}
              onChange={(e) => updateInput(i, k, e.target.value)}
            />
            <button className="foreach-child-del" style={{ opacity: 1 }} onClick={() => removeInput(i)}>
              <Trash2 size={11} />
            </button>
          </div>
        ))}
        {inputPairs.length > 0 && (
          <p className="se-call-inputs-hint">変数は <code>{'{{item.列名}}'}</code> のように指定できます</p>
        )}
      </div>

      <Field label="結果を保存する変数名 (省略可)">
        <TextInput
          value={(step.save_as as string) ?? ''}
          onChange={(v) => onUpdate({ save_as: v || undefined })}
          placeholder="省略可"
        />
      </Field>
    </>
  )
}

// ── ループ内ステップ追加ドロップダウン ───────────────────────────────────

const CHILD_TEMPLATES = [
  { type: 'call_scenario', label: '別シナリオを呼び出す', defaults: { path: '', inputs: {} } },
  { type: 'type',           label: 'テキスト入力',   defaults: { text: '' } },
  { type: 'click_image',    label: '画像クリック',   defaults: { template: '', threshold: 0.87 } },
  { type: 'click_text',     label: '文字クリック',   defaults: { text: '', lang: 'jpn', action: 'left' } },
  { type: 'wait_ms',        label: '待機',           defaults: { ms: 1000 } },
  { type: 'window_control', label: 'ウィンドウ操作', defaults: { title_contains: '', action: 'focus' } },
  { type: 'set',            label: '変数セット',      defaults: { name: '', value: '' } },
  { type: 'script',         label: 'スクリプト',     defaults: { code: '' } },
  { type: 'press',          label: 'キー押下',       defaults: 'Enter' },
  { type: 'shell',          label: 'シェル実行',     defaults: { cmd: '' } },
]

function AddChildButton({ onAdd }: { onAdd: (type: string, defaults: unknown) => void }) {
  const [open, setOpen] = useState(false)
  const ref = useRef<HTMLDivElement>(null)
  useEffect(() => {
    if (!open) return
    const h = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Element)) setOpen(false)
    }
    document.addEventListener('mousedown', h)
    return () => document.removeEventListener('mousedown', h)
  }, [open])
  return (
    <div ref={ref} style={{ position: 'relative' }}>
      <button className="foreach-add-btn" onClick={() => setOpen(v => !v)}>
        <Plus size={12} /> ステップ追加
      </button>
      {open && (
        <div className="foreach-add-menu">
          {CHILD_TEMPLATES.map(t => (
            <button
              key={t.type}
              className="foreach-add-item"
              onClick={() => { onAdd(t.type, t.defaults); setOpen(false) }}
            >
              {t.label}
            </button>
          ))}
        </div>
      )}
    </div>
  )
}

// ── foreach コンテナエディタ ───────────────────────────────────────────────

function ForeachChildStepsEditor({
  step,
  onUpdate,
}: {
  step: ScenarioStep
  onUpdate: (p: Partial<ScenarioStep>) => void
}) {
  const rawDo = (step.do as unknown[] | undefined) ?? []
  const childSteps: ScenarioStep[] = rawDo.map(s =>
    normalizeStep(s as Record<string, unknown>)
  )
  const [openIdx, setOpenIdx] = useState<number | null>(null)

  const rebuildDo = (updated: ScenarioStep[]) =>
    onUpdate({ do: updated.map(denormalizeStep) })

  const updateChild = (i: number, patch: Partial<ScenarioStep>) => {
    const next = childSteps.map((s, idx) => idx === i ? { ...s, ...patch } : s)
    rebuildDo(next)
  }

  const deleteChild = (i: number) => {
    rebuildDo(childSteps.filter((_, idx) => idx !== i))
    if (openIdx === i) setOpenIdx(null)
    else if (openIdx !== null && openIdx > i) setOpenIdx(openIdx - 1)
  }

  const addChild = (type: string, defaults: unknown) => {
    const data = typeof defaults === 'object' && defaults !== null && !Array.isArray(defaults)
      ? (defaults as Record<string, unknown>)
      : {}
    const newStep: ScenarioStep = { type, ...data }
    const next = [...childSteps, newStep]
    rebuildDo(next)
    setOpenIdx(next.length - 1)
  }

  return (
    <>
      <Field label="ループする配列変数名">
        <TextInput
          value={(step.var as string) ?? ''}
          onChange={v => onUpdate({ var: v })}
          placeholder="例: __rows__"
        />
      </Field>
      <Field label="ループインデックス変数名 (省略可)">
        <TextInput
          value={(step.index_var as string) ?? ''}
          onChange={v => onUpdate({ index_var: v || undefined })}
          placeholder="例: i"
        />
      </Field>

      {/* ── ループ内ステップ ───────────────────────────── */}
      <div className="foreach-children">
        <div className="foreach-children-header">
          <span className="foreach-children-title">ループ内ステップ（{childSteps.length}）</span>
          <AddChildButton onAdd={addChild} />
        </div>

        {childSteps.length === 0 && (
          <p className="foreach-empty">「ステップ追加」でループ内の処理を追加してください<br />各イテレーションで <code>{'{{item.列名}}'}</code> が使えます</p>
        )}

        {childSteps.map((child, i) => {
          const isOpen = openIdx === i
          const label = STEP_LABELS[child.type ?? ''] ?? child.type ?? `Step ${i + 1}`
          return (
            <div key={i} className={`foreach-child${isOpen ? ' open' : ''}`}>
              <div className="foreach-child-row" onClick={() => setOpenIdx(isOpen ? null : i)}>
                {isOpen ? <ChevronDown size={12} /> : <ChevronRight size={12} />}
                <span className="foreach-child-badge">{child.type ?? '?'}</span>
                <span className="foreach-child-name">{child.name ?? label}</span>
                <button
                  className="foreach-child-del"
                  onClick={e => { e.stopPropagation(); deleteChild(i) }}
                  title="削除"
                >
                  <Trash2 size={11} />
                </button>
              </div>
              {isOpen && (
                <div className="foreach-child-body">
                  <Field label="ステップ名">
                    <TextInput
                      value={(child.name as string) ?? ''}
                      onChange={v => updateChild(i, { name: v })}
                      placeholder="省略可"
                    />
                  </Field>
                  <TypeSpecificEditor step={child} onUpdate={p => updateChild(i, p)} />
                </div>
              )}
            </div>
          )
        })}
      </div>
    </>
  )
}

// ── メインコンポーネント ──────────────────────────────────────────────────

const STEP_LABELS: Record<string, string> = {
  window_control:   'ウィンドウ操作',
  click_text:       '文字クリック',
  move_to_text:     'テキストへ移動',
  wait_ms:          '待機',
  type:             'テキスト入力',
  click_image:      '画像クリック',
  wait_image:       '画像待機',
  find_image:       '画像検索',
  key_combo:        'キー操作',
  mouse_move:       'マウス移動',
  mouse_scroll:     'スクロール',
  mouse_drag:       'ドラッグ',
  excel_read_sheet: 'Excel/CSV読み込み',
  excel_read_range: 'Excelセル範囲読み込み',
  csv_read:         'CSV読み込み',
  import_vars:      '変数インポート',
  foreach:          'ループ (foreach)',
  call_scenario:    'シナリオ呼び出し',
}

function typeLabel(t?: string) {
  return t ? (STEP_LABELS[t] ?? t) : '不明'
}

function TypeSpecificEditor({
  step,
  onUpdate,
}: {
  step: ScenarioStep
  onUpdate: (p: Partial<ScenarioStep>) => void
}) {
  switch (step.type) {
    case 'window_control':   return <WindowControlEditor step={step} onUpdate={onUpdate} />
    case 'click_text':       return <ClickTextEditor step={step} onUpdate={onUpdate} />
    case 'move_to_text':     return <MoveToTextEditor step={step} onUpdate={onUpdate} />
    case 'wait_ms':          return <WaitEditor step={step} onUpdate={onUpdate} />
    case 'type':             return <TypeTextEditor step={step} onUpdate={onUpdate} />
    case 'click_image':      return <ClickImageEditor step={step} onUpdate={onUpdate} />
    case 'excel_read_sheet': return <ExcelReadSheetEditor step={step} onUpdate={onUpdate} />
    case 'csv_read':         return <CsvReadEditor step={step} onUpdate={onUpdate} />
    case 'import_vars':      return <ImportVarsEditor step={step} onUpdate={onUpdate} />
    case 'call_scenario':    return <CallScenarioEditor step={step} onUpdate={onUpdate} />
    case 'foreach':          return <ForeachChildStepsEditor step={step} onUpdate={onUpdate} />
    default:
      return (
        <p className="se-no-editor">
          このステップタイプのビジュアルエディタは未実装です。<br />
          YAMLを直接編集してください。
        </p>
      )
  }
}

export function StepEditor({ step, stepIndex, onUpdate, onClose, onSave, onRunStep }: Props) {
  return (
    <div className="step-editor">
      <div className="se-header">
        <div className="se-header-info">
          <span className="se-type-badge">{typeLabel(step.type)}</span>
          <span className="se-index">Step {stepIndex + 1}</span>
        </div>
        <div className="se-header-actions">
          {onRunStep && (
            <button className="se-action-btn run" onClick={onRunStep} title="このステップだけ実行">
              <Play size={13} />
            </button>
          )}
          {onSave && (
            <button className="se-action-btn save" onClick={onSave} title="保存">
              <Save size={13} />
            </button>
          )}
          <button className="se-close" onClick={onClose} title="閉じる"><X size={15} /></button>
        </div>
      </div>

      <div className="se-body">
        {/* 全ステップ共通: 名前 */}
        <Field label="ステップ名">
          <TextInput
            value={(step.name as string) ?? ''}
            onChange={(v) => onUpdate({ name: v })}
            placeholder="例: ウィンドウをフォーカス"
          />
        </Field>

        {/* 全ステップ共通: 有効/無効(スキップ) */}
        <Field label="有効">
          <label style={{ display: 'flex', alignItems: 'center', gap: 8, cursor: 'pointer' }}>
            <input
              type="checkbox"
              checked={(step.enabled as boolean) ?? true}
              onChange={(e) => onUpdate({ enabled: e.target.checked })}
            />
            <span style={{ fontSize: 13, color: '#9090b0' }}>チェック OFF でこのステップを実行時にスキップ</span>
          </label>
        </Field>

        <div className="se-divider" />

        {/* タイプ別フィールド */}
        <TypeSpecificEditor step={step} onUpdate={onUpdate} />

        <div className="se-divider" />

        {/* 変数の使い方ヒント */}
        <div className="se-var-hint">
          <div className="se-var-hint-title">変数の使い方</div>
          <div className="se-var-hint-row">
            <code>{'{{変数名}}'}</code>
            <span>変数の値に展開</span>
          </div>
          <div className="se-var-hint-row">
            <code>{'{{item.列名}}'}</code>
            <span>foreach ループの現在行</span>
          </div>
          <div className="se-var-hint-row">
            <code>{'{{__rows__}}'}</code>
            <span>data_source の全行配列</span>
          </div>
          <div className="se-var-hint-row">
            <code>{'{{i}}'}</code>
            <span>ループインデックス</span>
          </div>
        </div>
      </div>
    </div>
  )
}
