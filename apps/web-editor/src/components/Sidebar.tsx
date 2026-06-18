import { FileText, Settings, Bot } from 'lucide-react'
import { useAppStore, type Page } from '../store/appStore'
import './Sidebar.css'

const items: { page: Page; icon: React.ReactNode; label: string }[] = [
  { page: 'scenario', icon: <FileText size={20} />, label: 'シナリオ作成' },
  { page: 'settings', icon: <Settings size={20} />, label: '設定' },
]

export function Sidebar() {
  const { page, setPage } = useAppStore()

  return (
    <aside className="sidebar">
      <div className="sidebar-logo">
        <Bot size={24} />
        <span>robost</span>
      </div>
      <nav className="sidebar-nav">
        {items.map((item) => (
          <button
            key={item.page}
            className={`sidebar-item ${page === item.page ? 'active' : ''}`}
            onClick={() => setPage(item.page)}
          >
            {item.icon}
            <span>{item.label}</span>
          </button>
        ))}
      </nav>
    </aside>
  )
}
