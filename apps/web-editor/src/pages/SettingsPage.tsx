import { useEffect, useState } from 'react'
import { api } from '../api/client'
import { useAppStore, type Theme } from '../store/appStore'
import './SettingsPage.css'

const THEMES: { value: Theme; label: string }[] = [
  { value: 'purple', label: 'パープル（デフォルト）' },
  { value: 'blue', label: 'ブルー' },
  { value: 'green', label: 'ダークグリーン' },
]

export function SettingsPage() {
  const { appTitle, setAppTitle, theme, setTheme } = useAppStore()
  const [titleDraft, setTitleDraft] = useState(appTitle)
  const [hasApiKey, setHasApiKey] = useState(false)
  const [apiKey, setApiKey] = useState('')
  const [status, setStatus] = useState<string | null>(null)
  const [saving, setSaving] = useState(false)

  useEffect(() => {
    api.getSettings().then((s) => setHasApiKey(s.hasApiKey)).catch(() => {})
  }, [])

  const persist = async (key: string) => {
    setSaving(true)
    setStatus(null)
    try {
      const { message } = await api.saveSettings(key)
      const settings = await api.getSettings().catch(() => null)
      setHasApiKey(settings?.hasApiKey ?? key.trim() !== '')
      setApiKey('')
      setStatus(message ?? (key.trim() === '' ? 'APIキーを削除しました。' : 'APIキーを保存しました。'))
    } catch (err) {
      setStatus(`エラー: ${String(err)}`)
    } finally {
      setSaving(false)
    }
  }

  const save = () => persist(apiKey)
  const clear = () => persist('')

  return (
    <div className="settings-page">
      <h2>設定</h2>
      <section className="settings-section">
        <h3>表示</h3>
        <p className="settings-note">アプリ名（左上に表示）</p>
        <div className="settings-input-row">
          <input
            className="settings-input"
            placeholder="robost"
            value={titleDraft}
            onChange={(e) => setTitleDraft(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && setAppTitle(titleDraft)}
          />
          <button className="settings-button" onClick={() => setAppTitle(titleDraft)}>
            保存
          </button>
        </div>
        <p className="settings-note">カラーテーマ</p>
        <select
          className="settings-input"
          value={theme}
          onChange={(e) => setTheme(e.target.value as Theme)}
        >
          {THEMES.map((t) => (
            <option key={t.value} value={t.value}>{t.label}</option>
          ))}
        </select>
      </section>
      <section className="settings-section">
        <h3>AI アシスタント</h3>
        <p className="settings-note">
          {hasApiKey
            ? 'APIキーはOSのキーチェーンに保存されています。'
            : 'APIキーが設定されていません。'}
        </p>
        <div className="settings-input-row">
          <input
            type="password"
            className="settings-input"
            placeholder="sk-ant-..."
            value={apiKey}
            onChange={(e) => setApiKey(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && !saving && apiKey.trim() && save()}
          />
          <button className="settings-button" onClick={save} disabled={saving || !apiKey.trim()}>
            保存
          </button>
          {hasApiKey && (
            <button className="settings-button settings-button-secondary" onClick={clear} disabled={saving}>
              削除
            </button>
          )}
        </div>
        {status && <p className="settings-note">{status}</p>}
        <p className="settings-note">
          環境変数 <code>ANTHROPIC_API_KEY</code> が設定されている場合はそちらが優先されます。
        </p>
      </section>
      <section className="settings-section">
        <h3>エージェント</h3>
        <p className="settings-note">現在のエージェント: <code>http://localhost:9921</code></p>
      </section>
    </div>
  )
}
