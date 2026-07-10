import { useState } from 'react'
import { ChevronRight, ChevronDown } from 'lucide-react'
import { STEP_TEMPLATES, type StepCategory } from '../utils/stepTemplates'

const CATEGORIES: StepCategory[] = ['フロー', 'アクション', 'ユーザ', '変数', 'データ']

export function NodePalette() {
  const [collapsed, setCollapsed] = useState<Set<StepCategory>>(new Set())

  const toggle = (cat: StepCategory) => {
    setCollapsed((prev) => {
      const next = new Set(prev)
      if (next.has(cat)) next.delete(cat)
      else next.add(cat)
      return next
    })
  }

  return (
    <div className="node-palette">
      <div className="node-palette-header-title">ノード</div>
      {CATEGORIES.map((cat) => {
        const items = STEP_TEMPLATES.filter((t) => t.category === cat)
        const expanded = !collapsed.has(cat)
        return (
          <div key={cat} className="node-palette-category">
            <button className="node-palette-category-toggle" onClick={() => toggle(cat)}>
              {expanded ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
              <span>{cat}</span>
            </button>
            {expanded && (
              <div className="node-palette-items">
                {items.map((t) => (
                  <div
                    key={t.type}
                    className="node-palette-item"
                    draggable
                    onDragStart={(e) => {
                      e.dataTransfer.setData(
                        'application/reactflow',
                        JSON.stringify({ type: t.type, defaults: t.defaults }),
                      )
                      e.dataTransfer.effectAllowed = 'move'
                    }}
                    title={`ドラッグしてキャンバスに配置: ${t.label}`}
                  >
                    {t.label}
                  </div>
                ))}
              </div>
            )}
          </div>
        )
      })}
    </div>
  )
}
