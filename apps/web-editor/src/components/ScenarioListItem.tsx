import { useState } from 'react'
import './ScenarioListItem.css'

// ── SVG アイコン ──────────────────────────────────────────────────────────────

function IconDuplicate() {
  return (
    <svg width="14" height="14" viewBox="0 0 16 16" fill="none" aria-hidden="true">
      <rect x="5" y="5" width="9" height="9" rx="1.5" stroke="currentColor" strokeWidth="1.5"/>
      <path d="M3 11V3.5A1.5 1.5 0 0 1 4.5 2H12" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round"/>
    </svg>
  )
}

function IconTrash() {
  return (
    <svg width="14" height="14" viewBox="0 0 16 16" fill="none" aria-hidden="true">
      <path d="M2.5 4h11M6 4V2.5a.5.5 0 0 1 .5-.5h3a.5.5 0 0 1 .5.5V4M5 4l.5 9h5L11 4" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"/>
    </svg>
  )
}

// ── コンポーネント ────────────────────────────────────────────────────────────

interface Props {
  name: string
  active: boolean
  onOpen: () => void
  onDuplicate: () => Promise<void>
  onDelete: () => Promise<void>
}

export function ScenarioListItem({ name, active, onOpen, onDuplicate, onDelete }: Props) {
  const [confirming, setConfirming] = useState(false)
  const [busy, setBusy] = useState(false)

  const handleDuplicate = async (e: React.MouseEvent) => {
    e.stopPropagation()
    setBusy(true)
    try { await onDuplicate() } finally { setBusy(false) }
  }

  const handleDeleteClick = (e: React.MouseEvent) => {
    e.stopPropagation()
    setConfirming(true)
  }

  const handleDeleteConfirm = async (e: React.MouseEvent) => {
    e.stopPropagation()
    setBusy(true)
    try { await onDelete() } finally { setBusy(false); setConfirming(false) }
  }

  const handleDeleteCancel = (e: React.MouseEvent) => {
    e.stopPropagation()
    setConfirming(false)
  }

  if (confirming) {
    return (
      <div className={`sli-row sli-confirming ${active ? 'active' : ''}`}>
        <span className="sli-name sli-name-sm">削除しますか？</span>
        <div className="sli-actions">
          <button
            className="sli-btn sli-btn-danger"
            onClick={handleDeleteConfirm}
            disabled={busy}
            title="削除する"
          >
            <IconTrash />
          </button>
          <button
            className="sli-btn"
            onClick={handleDeleteCancel}
            title="キャンセル"
          >
            ✕
          </button>
        </div>
      </div>
    )
  }

  return (
    <div
      className={`sli-row ${active ? 'active' : ''}`}
      onClick={onOpen}
      role="button"
      tabIndex={0}
      onKeyDown={(e) => e.key === 'Enter' && onOpen()}
    >
      <span className="sli-name" title={name}>{name}</span>
      <div className="sli-actions">
        <button
          className="sli-btn"
          onClick={handleDuplicate}
          disabled={busy}
          title="複製"
        >
          <IconDuplicate />
        </button>
        <button
          className="sli-btn sli-btn-danger"
          onClick={handleDeleteClick}
          disabled={busy}
          title="削除"
        >
          <IconTrash />
        </button>
      </div>
    </div>
  )
}
