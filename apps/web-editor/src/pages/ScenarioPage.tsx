import { useEffect, useState, useCallback, useRef } from 'react'
import {
  ReactFlow,
  Background,
  Controls,
  MiniMap,
  addEdge,
  useNodesState,
  useEdgesState,
  Handle,
  Position,
  type Node,
  type Edge,
  type Connection,
  type NodeMouseHandler,
  type NodeProps,
  type ReactFlowInstance,
  BackgroundVariant,
} from '@xyflow/react'
import '@xyflow/react/dist/style.css'
import {
  Play, Square, Plus, Save, FolderOpen, Loader2,
  ChevronRight, ChevronDown, Folder, FolderPlus,
  Trash2, Settings, FileSpreadsheet, Upload, Code,
} from 'lucide-react'
import { useScenarioStore } from '../store/scenarioStore'
import { api } from '../api/client'
import { useRunStore } from '../store/runStore'
import { AiChat } from '../components/AiChat'
import { StepEditor } from '../components/StepEditor'
import { ScenarioListItem } from '../components/ScenarioListItem'
import { NodePalette } from '../components/NodePalette'
import { yamlToNodes, nodesToYaml, updateStepInYaml, getDataSourceFromYaml, type ScenarioStep } from '../utils/yamlFlow'
import { STEP_TEMPLATES } from '../utils/stepTemplates'
import './ScenarioPage.css'

// ── カスタムステップノード ────────────────────────────────────────────────

function StepNode({ data }: NodeProps) {
  const stepIndex = data.stepIndex as number
  const label = data.label as string
  const stepType = data.type as string | undefined

  const deleteStep = useScenarioStore((s) => s.deleteStep)
  const save = useScenarioStore((s) => s.save)
  const activeScenario = useScenarioStore((s) => s.activeScenario)
  const startRunStep = useRunStore((s) => s.startRunStep)
  const status = useRunStore((s) => s.status)
  const isRunning = status === 'running'
  const currentStep = useRunStore((s) => s.currentStep)
  const isActive = isRunning && stepIndex === currentStep
  const isFailed = status === 'failed' && stepIndex === currentStep

  const handleDelete = (e: React.MouseEvent) => {
    e.stopPropagation()
    deleteStep(stepIndex)
  }

  const handleRunStep = async (e: React.MouseEvent) => {
    e.stopPropagation()
    if (!activeScenario || isRunning) return
    await save()
    startRunStep(activeScenario, stepIndex)
  }

  return (
    <div className={`step-node${isActive ? ' running' : ''}${isFailed ? ' error' : ''}`}>
      <Handle type="target" position={Position.Top} />
      <div className="step-node-body">
        <span className="step-node-label">{label}</span>
        {stepType && <span className="step-node-type-badge">{stepType}</span>}
      </div>
      <div className="step-node-actions">
        <button
          className="step-node-btn run"
          onClick={handleRunStep}
          disabled={isRunning}
          title="このステップだけ実行"
        >
          <Play size={11} />
        </button>
        <button className="step-node-btn delete" onClick={handleDelete} title="削除">
          <Trash2 size={11} />
        </button>
      </div>
      <Handle type="source" position={Position.Bottom} />
    </div>
  )
}

const nodeTypes = { stepNode: StepNode }

// ── ステップ追加ドロップダウン ────────────────────────────────────────────

function AddStepDropdown({
  onAdd,
  disabled,
}: {
  onAdd: (type: string, defaults: Record<string, unknown>) => void
  disabled?: boolean
}) {
  const [open, setOpen] = useState(false)
  const ref = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!open) return
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Element)) setOpen(false)
    }
    document.addEventListener('mousedown', handler)
    return () => document.removeEventListener('mousedown', handler)
  }, [open])

  return (
    <div ref={ref} style={{ position: 'relative' }}>
      <button
        className="toolbar-btn"
        onClick={() => setOpen((v) => !v)}
        disabled={disabled}
        title="ステップを追加"
      >
        <Plus size={15} /> ステップ
      </button>
      {open && (
        <div className="step-dropdown">
          {STEP_TEMPLATES.map((t) => (
            <button
              key={t.type}
              className="step-dropdown-item"
              onClick={() => {
                onAdd(t.type, t.defaults)
                setOpen(false)
              }}
            >
              {t.label}
            </button>
          ))}
        </div>
      )}
    </div>
  )
}

// ── フォルダ行 ────────────────────────────────────────────────────────────

function FolderRow({
  name, scenarios, activeScenario, expanded,
  onToggle, onOpen, onDuplicate, onDelete,
  onNewInFolder, onDeleteFolder,
  isDragOver, onDragOver, onDragLeave, onDrop,
  draggingScenario, onChildDragStart, onChildDragEnd,
}: {
  name: string
  scenarios: string[]
  activeScenario: string | null
  expanded: boolean
  onToggle: () => void
  onOpen: (full: string) => void
  onDuplicate: (full: string) => Promise<void>
  onDelete: (full: string) => Promise<void>
  onNewInFolder: () => void
  onDeleteFolder: () => void
  isDragOver?: boolean
  onDragOver?: (e: React.DragEvent) => void
  onDragLeave?: () => void
  onDrop?: (e: React.DragEvent) => void
  draggingScenario?: string | null
  onChildDragStart?: (full: string) => void
  onChildDragEnd?: () => void
}) {
  return (
    <>
      <div
        className={`folder-row${isDragOver ? ' drag-over-folder' : ''}`}
        onDragOver={onDragOver}
        onDragLeave={onDragLeave}
        onDrop={onDrop}
      >
        <button className="folder-toggle" onClick={onToggle}>
          {expanded ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
          <Folder size={14} />
          <span className="folder-name">{name}</span>
          {isDragOver && <span className="folder-drop-hint">ここへ移動</span>}
        </button>
        <div className="folder-actions">
          <button className="icon-btn" onClick={onNewInFolder} title="このフォルダに新規シナリオ">
            <Plus size={13} />
          </button>
          <button className="icon-btn danger-hover" onClick={onDeleteFolder} title="フォルダを削除">
            ×
          </button>
        </div>
      </div>
      {expanded && (
        <div className="folder-children">
          {scenarios.map((s) => {
            const full = `${name}/${s}`
            return (
              <div
                key={full}
                draggable
                onDragStart={(e) => { onChildDragStart?.(full); e.dataTransfer.effectAllowed = 'move' }}
                onDragEnd={onChildDragEnd}
                className={draggingScenario === full ? 'dragging-item' : ''}
              >
                <ScenarioListItem
                  name={s}
                  active={full === activeScenario}
                  onOpen={() => onOpen(full)}
                  onDuplicate={() => onDuplicate(full)}
                  onDelete={() => onDelete(full)}
                />
              </div>
            )
          })}
          {scenarios.length === 0 && (
            <p className="scenario-list-empty" style={{ paddingLeft: '1.5rem' }}>空のフォルダ</p>
          )}
        </div>
      )}
    </>
  )
}

// ── メインページ ──────────────────────────────────────────────────────────

export function ScenarioPage() {
  const {
    scenarios, folders, activeScenario, yaml, dirty, loading,
    loadList, openScenario, setYaml, save, newScenario,
    duplicateScenario, deleteScenario, addStep, setDataSource,
    moveScenario, createFolder, deleteFolder,
  } = useScenarioStore()
  const { status, logs, startRun, startRunStep, stopRun } = useRunStore()

  const [nodes, setNodes, onNodesChange] = useNodesState<Node>([])
  const [edges, setEdges, onEdgesChange] = useEdgesState<Edge>([])
  const [newName, setNewName] = useState('')
  const [showNewModal, setShowNewModal] = useState(false)
  const [newInFolder, setNewInFolder] = useState<string | undefined>()
  const [showNewFolderModal, setShowNewFolderModal] = useState(false)
  const [newFolderName, setNewFolderName] = useState('')
  const [expandedFolders, setExpandedFolders] = useState<Set<string>>(new Set())
  const [selectedStep, setSelectedStep] = useState<{ step: ScenarioStep; index: number } | null>(null)
  const [draggingScenario, setDraggingScenario] = useState<string | null>(null)
  const [dropTarget, setDropTarget] = useState<string | null>(null) // folder name or 'root'
  const [showYamlEditor, setShowYamlEditor] = useState(false)
  const [yamlDraft, setYamlDraft] = useState('')
  const [showDataSourcePanel, setShowDataSourcePanel] = useState(false)
  const [dsFile, setDsFile] = useState('')
  const [dsSheet, setDsSheet] = useState('')
  const [uploading, setUploading] = useState(false)
  const fileInputRef = useRef<HTMLInputElement>(null)
  const rfInstance = useRef<ReactFlowInstance | null>(null)

  useEffect(() => { loadList() }, [loadList])

  // Sync data_source fields when panel opens or active scenario changes
  useEffect(() => {
    if (!showDataSourcePanel) return
    const { file, sheet } = getDataSourceFromYaml(yaml)
    setDsFile(file)
    setDsSheet(sheet)
  }, [showDataSourcePanel, yaml])

  // Sync YAML draft when editor opens
  useEffect(() => {
    if (showYamlEditor) setYamlDraft(yaml)
  }, [showYamlEditor]) // eslint-disable-line react-hooks/exhaustive-deps

  useEffect(() => {
    if (!yaml) return
    const { nodes: n, edges: e } = yamlToNodes(yaml)
    setNodes(n.map((node) => ({ ...node, type: 'stepNode' })))
    setEdges(e)
    setSelectedStep((prev) => {
      if (!prev) return null
      const freshStep = n[prev.index]?.data?.step as ScenarioStep | undefined
      return freshStep ? { step: freshStep, index: prev.index } : null
    })
  }, [yaml, setNodes, setEdges])

  const onNodesChangeWithSync = useCallback(
    (changes: Parameters<typeof onNodesChange>[0]) => { onNodesChange(changes) },
    [onNodesChange],
  )

  const onConnect = useCallback(
    (params: Connection) => setEdges((eds) => addEdge(params, eds)),
    [setEdges],
  )

  const onPaletteDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    e.dataTransfer.dropEffect = 'move'
  }, [])

  const onPaletteDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault()
      if (!activeScenario) return
      const raw = e.dataTransfer.getData('application/reactflow')
      if (!raw) return
      const { type, defaults } = JSON.parse(raw) as { type: string; defaults: Record<string, unknown> }
      const dropPos = rfInstance.current?.screenToFlowPosition({ x: e.clientX, y: e.clientY })
      const atIndex = dropPos ? nodes.filter((n) => n.position.y < dropPos.y).length : undefined
      addStep(type, defaults, atIndex)
    },
    [activeScenario, nodes, addStep],
  )

  const onNodeClick: NodeMouseHandler = useCallback((_event, node) => {
    const step = node.data?.step as ScenarioStep | undefined
    const index = node.data?.stepIndex as number | undefined
    if (step !== undefined && index !== undefined) {
      setSelectedStep({ step, index })
    }
  }, [])

  const handleStepUpdate = useCallback(
    (patch: Partial<ScenarioStep>) => {
      if (!selectedStep) return
      const newYaml = updateStepInYaml(yaml, selectedStep.index, patch)
      setYaml(newYaml)
      setSelectedStep((prev) => prev ? { ...prev, step: { ...prev.step, ...patch } } : null)
    },
    [selectedStep, yaml, setYaml],
  )

  const handleSave = async () => {
    const updatedYaml = nodesToYaml(nodes, edges, yaml)
    setYaml(updatedYaml)
    await save()
  }

  const handleNew = async () => {
    if (!newName.trim()) return
    const name = newName.trim().endsWith('.yaml') ? newName.trim() : `${newName.trim()}.yaml`
    await newScenario(name, newInFolder)
    setShowNewModal(false)
    setNewName('')
    setNewInFolder(undefined)
  }

  const handleDrop = useCallback(async (targetFolder: string | null) => {
    if (!draggingScenario) return
    setDropTarget(null)
    setDraggingScenario(null)
    const fileName = draggingScenario.split('/').pop() ?? draggingScenario
    const toName = targetFolder ? `${targetFolder}/${fileName}` : fileName
    if (draggingScenario !== toName) {
      await moveScenario(draggingScenario, toName)
    }
  }, [draggingScenario, moveScenario])

  const handleRunStep = useCallback(async (stepIndex: number) => {
    if (!activeScenario || status === 'running') return
    await save()
    startRunStep(activeScenario, stepIndex)
  }, [activeScenario, status, save, startRunStep])

  const handleNewFolder = async () => {
    if (!newFolderName.trim()) return
    await createFolder(newFolderName.trim())
    setExpandedFolders((prev) => new Set([...prev, newFolderName.trim()]))
    setShowNewFolderModal(false)
    setNewFolderName('')
  }

  const toggleFolder = (name: string) =>
    setExpandedFolders((prev) => {
      const next = new Set(prev)
      next.has(name) ? next.delete(name) : next.add(name)
      return next
    })

  const isRunning = status === 'running'
  const hasContent = scenarios.length > 0 || folders.length > 0

  return (
    <div className="scenario-page">
      {/* Scenario list panel */}
      <div className="scenario-list-panel">
        <div className="scenario-list-header">
          <span>シナリオ</span>
          <div style={{ display: 'flex', gap: '4px' }}>
            <button className="icon-btn" onClick={() => { setNewInFolder(undefined); setShowNewModal(true) }} title="新規シナリオ">
              <Plus size={16} />
            </button>
            <button className="icon-btn" onClick={() => setShowNewFolderModal(true)} title="新規フォルダ">
              <FolderPlus size={16} />
            </button>
          </div>
        </div>

        {/* Root drop zone (when dragging a scenario out of a folder) */}
        <div
          className={`root-drop-zone ${dropTarget === 'root' ? 'drag-over' : ''} ${draggingScenario ? 'active' : ''}`}
          onDragOver={(e) => { e.preventDefault(); setDropTarget('root') }}
          onDragLeave={() => setDropTarget(null)}
          onDrop={(e) => { e.preventDefault(); handleDrop(null) }}
        >
          {draggingScenario && dropTarget === 'root' && (
            <span className="drop-hint">ここでドロップ → トップレベルに移動</span>
          )}
        </div>

        {/* Top-level scenarios */}
        {scenarios.map((name) => (
          <div
            key={name}
            draggable
            onDragStart={(e) => {
              setDraggingScenario(name)
              e.dataTransfer.effectAllowed = 'move'
            }}
            onDragEnd={() => { setDraggingScenario(null); setDropTarget(null) }}
            className={draggingScenario === name ? 'dragging-item' : ''}
          >
            <ScenarioListItem
              name={name}
              active={name === activeScenario}
              onOpen={() => openScenario(name)}
              onDuplicate={() => duplicateScenario(name)}
              onDelete={() => deleteScenario(name)}
            />
          </div>
        ))}

        {/* Folders */}
        {folders.map((folder) => (
          <FolderRow
            key={folder.name}
            name={folder.name}
            scenarios={folder.scenarios}
            activeScenario={activeScenario}
            expanded={expandedFolders.has(folder.name)}
            onToggle={() => toggleFolder(folder.name)}
            onOpen={(full) => openScenario(full)}
            onDuplicate={(full) => duplicateScenario(full)}
            onDelete={(full) => deleteScenario(full)}
            onNewInFolder={() => {
              setNewInFolder(folder.name)
              setExpandedFolders((prev) => new Set([...prev, folder.name]))
              setShowNewModal(true)
            }}
            onDeleteFolder={() => deleteFolder(folder.name)}
            isDragOver={dropTarget === folder.name}
            onDragOver={(e) => { e.preventDefault(); setDropTarget(folder.name) }}
            onDragLeave={() => setDropTarget(null)}
            onDrop={(e) => { e.preventDefault(); handleDrop(folder.name) }}
            draggingScenario={draggingScenario}
            onChildDragStart={(full) => setDraggingScenario(full)}
            onChildDragEnd={() => { setDraggingScenario(null); setDropTarget(null) }}
          />
        ))}

        {!hasContent && (
          <p className="scenario-list-empty">シナリオがありません</p>
        )}
      </div>

      {/* Node palette panel */}
      <NodePalette />

      {/* Main canvas area */}
      <div className="scenario-canvas-area">
        {/* Toolbar */}
        <div className="scenario-toolbar">
          <div className="toolbar-left">
            <span className="scenario-title">
              {activeScenario ?? '—'}
              {dirty && <span className="dirty-dot">●</span>}
            </span>
          </div>
          <div className="toolbar-right">
            <button
              className={`toolbar-btn${showYamlEditor ? ' active' : ''}`}
              onClick={() => { setShowYamlEditor((v) => !v); setShowDataSourcePanel(false) }}
              disabled={!activeScenario}
              title="YAMLを直接編集"
            >
              <Code size={15} /> YAML
            </button>
            <button
              className={`toolbar-btn${showDataSourcePanel ? ' active' : ''}`}
              onClick={() => setShowDataSourcePanel((v) => !v)}
              disabled={!activeScenario}
              title="データソース設定 (Excel/CSV)"
            >
              <FileSpreadsheet size={15} /> データソース
            </button>
            <AddStepDropdown onAdd={addStep} disabled={!activeScenario} />
            <button className="toolbar-btn" onClick={handleSave} disabled={!activeScenario || !dirty}>
              <Save size={15} /> 保存
            </button>
            {isRunning ? (
              <button className="toolbar-btn danger" onClick={stopRun}>
                <Square size={15} /> 停止
              </button>
            ) : (
              <button
                className="toolbar-btn primary"
                onClick={() => activeScenario && startRun(activeScenario)}
                disabled={!activeScenario}
              >
                {loading ? <Loader2 size={15} className="spin" /> : <Play size={15} />}
                実行
              </button>
            )}
          </div>
        </div>

        {/* YAML editor panel */}
        {showYamlEditor && activeScenario && (
          <div className="yaml-editor-panel">
            <div className="yaml-editor-header">
              <Code size={14} />
              <span>YAML 直接編集</span>
              <span className="yaml-editor-hint">foreach の <code>do:</code> など、ビジュアルエディタ未対応の項目はここで編集できます</span>
              <button className="se-close" onClick={() => setShowYamlEditor(false)}><Code size={13} /></button>
            </div>
            <textarea
              className="yaml-textarea"
              value={yamlDraft}
              onChange={(e) => setYamlDraft(e.target.value)}
              spellCheck={false}
            />
            <div className="yaml-editor-footer">
              <button
                className="toolbar-btn primary"
                onClick={() => {
                  setYaml(yamlDraft)
                  setShowYamlEditor(false)
                }}
              >
                <Save size={13} /> 適用
              </button>
              <button className="toolbar-btn" onClick={() => setShowYamlEditor(false)}>
                キャンセル
              </button>
              <span className="yaml-foreach-tip">
                💡 foreach 例:
                <code>{'- foreach:\n    var: __rows__\n    do:\n      - type:\n          text: "{{item.名前}}"\n      - wait_ms: 500'}</code>
              </span>
            </div>
          </div>
        )}

        {/* Data source panel */}
        {showDataSourcePanel && activeScenario && (
          <div className="datasource-panel">
            <div className="datasource-panel-header">
              <FileSpreadsheet size={14} />
              <span>データソース — 全行を <code>__rows__</code> に自動ロード（Excel / CSV）</span>
              <button className="se-close" onClick={() => setShowDataSourcePanel(false)}><Settings size={13} /></button>
            </div>
            <div className="datasource-panel-body">
              {/* Hidden file input */}
              <input
                ref={fileInputRef}
                type="file"
                accept=".xlsx,.xls,.csv"
                style={{ display: 'none' }}
                onChange={async (e) => {
                  const file = e.target.files?.[0]
                  if (!file) return
                  setUploading(true)
                  try {
                    const path = await api.uploadFile(file)
                    setDsFile(path)
                  } catch (err) {
                    alert(`アップロード失敗: ${err}`)
                  } finally {
                    setUploading(false)
                    if (fileInputRef.current) fileInputRef.current.value = ''
                  }
                }}
              />
              <div className="ds-field">
                <label>ファイルパス (.xlsx / .csv)</label>
                <div style={{ display: 'flex', gap: 6 }}>
                  <input
                    className="ds-input"
                    style={{ flex: 1 }}
                    placeholder="例: data.xlsx または ../data/input.csv"
                    value={dsFile}
                    onChange={(e) => setDsFile(e.target.value)}
                  />
                  <button
                    className="toolbar-btn"
                    onClick={() => fileInputRef.current?.click()}
                    disabled={uploading}
                    title="ファイルを選択してサーバーにアップロード"
                    style={{ whiteSpace: 'nowrap', flexShrink: 0 }}
                  >
                    {uploading
                      ? <Loader2 size={14} className="spin" />
                      : <Upload size={14} />}
                    {uploading ? 'アップロード中...' : 'ファイル読込'}
                  </button>
                </div>
              </div>
              <div className="ds-field">
                <label>シート名（Excelのみ・省略で最初のシート）</label>
                <input
                  className="ds-input"
                  placeholder="例: Sheet1"
                  value={dsSheet}
                  onChange={(e) => setDsSheet(e.target.value)}
                />
              </div>
              <div className="ds-actions">
                <button
                  className="toolbar-btn primary"
                  onClick={() => {
                    setDataSource(dsFile, dsSheet || undefined)
                    setShowDataSourcePanel(false)
                  }}
                >
                  <Save size={13} /> 設定を反映
                </button>
                {dsFile && (
                  <button
                    className="toolbar-btn"
                    onClick={() => {
                      setDsFile('')
                      setDsSheet('')
                      setDataSource('', undefined)
                      setShowDataSourcePanel(false)
                    }}
                  >
                    解除
                  </button>
                )}
              </div>
              <p className="ds-hint">
                ヘッダー行が列名になります。ループには <strong>foreach: {'{'} var: __rows__ {'}'}</strong> を使用してください。<br />
                各行は <code>item</code> として参照でき、列の値は <code>{'{'}{'{'}item.列名{'}'}{'}'}</code> でアクセスできます。
              </p>
            </div>
          </div>
        )}

        {/* ReactFlow canvas */}
        <div className="flow-container" onDragOver={onPaletteDragOver} onDrop={onPaletteDrop}>
          {!activeScenario ? (
            <div className="flow-empty">
              <FolderOpen size={48} />
              <p>左のリストからシナリオを選択するか、<br />新しいシナリオを作成してください。</p>
            </div>
          ) : (
            <ReactFlow
              nodes={nodes}
              edges={edges}
              nodeTypes={nodeTypes}
              onNodesChange={onNodesChangeWithSync}
              onEdgesChange={onEdgesChange}
              onConnect={onConnect}
              onNodeClick={onNodeClick}
              onInit={(instance) => { rfInstance.current = instance }}
              fitView
            >
              <Background variant={BackgroundVariant.Dots} color="#2d2d4e" />
              <Controls />
              <MiniMap nodeColor="#7c6af7" maskColor="rgba(10,10,20,0.7)" />
            </ReactFlow>
          )}
        </div>

        {/* Log panel */}
        {logs.length > 0 && (
          <div className="log-panel">
            {logs.slice(-50).map((l, i) => (
              <div key={i} className={`log-line ${l.level}`}>
                <span className="log-time">{new Date(l.time).toLocaleTimeString()}</span>
                <span>{l.message}</span>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Step editor panel */}
      {selectedStep && (
        <StepEditor
          step={selectedStep.step}
          stepIndex={selectedStep.index}
          onUpdate={handleStepUpdate}
          onClose={() => setSelectedStep(null)}
          onSave={handleSave}
          onRunStep={() => handleRunStep(selectedStep.index)}
        />
      )}

      {/* AI Chat assistant */}
      <AiChat />

      {/* New scenario modal */}
      {showNewModal && (
        <div className="modal-overlay" onClick={() => setShowNewModal(false)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <h3>{newInFolder ? `新しいシナリオ (${newInFolder}/)` : '新しいシナリオ'}</h3>
            <input
              autoFocus
              className="modal-input"
              placeholder="ファイル名 (例: login.yaml)"
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleNew()}
            />
            <div className="modal-actions">
              <button className="modal-btn" onClick={() => { setShowNewModal(false); setNewInFolder(undefined) }}>キャンセル</button>
              <button className="modal-btn primary" onClick={handleNew}>作成</button>
            </div>
          </div>
        </div>
      )}

      {/* New folder modal */}
      {showNewFolderModal && (
        <div className="modal-overlay" onClick={() => setShowNewFolderModal(false)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <h3>新しいフォルダ</h3>
            <input
              autoFocus
              className="modal-input"
              placeholder="フォルダ名 (例: ログイン)"
              value={newFolderName}
              onChange={(e) => setNewFolderName(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleNewFolder()}
            />
            <div className="modal-actions">
              <button className="modal-btn" onClick={() => setShowNewFolderModal(false)}>キャンセル</button>
              <button className="modal-btn primary" onClick={handleNewFolder}>作成</button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
