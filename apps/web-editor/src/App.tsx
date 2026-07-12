import { useEffect } from 'react'
import { Sidebar } from './components/Sidebar'
import { ScenarioPage } from './pages/ScenarioPage'
import { SettingsPage } from './pages/SettingsPage'
import { useAppStore } from './store/appStore'
import './App.css'

export function App() {
  const { page, theme } = useAppStore()

  useEffect(() => {
    document.documentElement.dataset.theme = theme
  }, [theme])

  return (
    <div className="app">
      <Sidebar />
      <main className="app-main">
        {page === 'scenario' && <ScenarioPage />}
        {page === 'settings' && <SettingsPage />}
      </main>
    </div>
  )
}
