import { Sidebar } from './components/Sidebar'
import { ScenarioPage } from './pages/ScenarioPage'
import { SettingsPage } from './pages/SettingsPage'
import { useAppStore } from './store/appStore'
import './App.css'

export function App() {
  const { page } = useAppStore()

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
