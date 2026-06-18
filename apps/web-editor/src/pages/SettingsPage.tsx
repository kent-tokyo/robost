import './SettingsPage.css'

export function SettingsPage() {
  return (
    <div className="settings-page">
      <h2>設定</h2>
      <section className="settings-section">
        <h3>AI アシスタント</h3>
        <p className="settings-note">
          APIキーはエージェントの <code>.env</code> ファイルで設定します。
        </p>
        <pre className="settings-code">{`# .env
ANTHROPIC_API_KEY=sk-ant-...`}</pre>
      </section>
      <section className="settings-section">
        <h3>エージェント</h3>
        <p className="settings-note">現在のエージェント: <code>http://localhost:9921</code></p>
      </section>
    </div>
  )
}
