const BASE = ''  // same-origin in production; proxied via Vite in dev

export interface FolderEntry {
  name: string
  scenarios: string[]
}

export interface ScenarioListResult {
  scenarios: string[]
  folders: FolderEntry[]
}

function encodeName(name: string): string {
  return name.split('/').map(encodeURIComponent).join('/')
}

export const api = {
  // ── Scenario CRUD ──────────────────────────────────────────────────────────
  async listScenarios(): Promise<ScenarioListResult> {
    const res = await fetch(`${BASE}/api/scenarios`)
    const data = await res.json() as { scenarios: string[]; folders?: FolderEntry[] }
    return { scenarios: data.scenarios ?? [], folders: data.folders ?? [] }
  },

  async getScenario(name: string): Promise<string> {
    const res = await fetch(`${BASE}/api/scenarios/${encodeName(name)}`)
    const data = await res.json() as { content: string }
    return data.content
  },

  async saveScenario(name: string, content: string): Promise<void> {
    const res = await fetch(`${BASE}/api/scenarios/${encodeName(name)}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ content }),
    })
    if (!res.ok) throw new Error(`Save failed: HTTP ${res.status}`)
  },

  async deleteScenario(name: string): Promise<void> {
    const res = await fetch(`${BASE}/api/scenarios/${encodeName(name)}`, {
      method: 'DELETE',
    })
    if (!res.ok) throw new Error(`Delete failed: HTTP ${res.status}`)
  },

  async moveScenario(from: string, to: string): Promise<void> {
    const content = await this.getScenario(from)
    await this.saveScenario(to, content)
    await this.deleteScenario(from)
  },

  // ── Folder management ──────────────────────────────────────────────────────
  async uploadFile(file: File): Promise<string> {
    const form = new FormData()
    form.append('file', file, file.name)
    const res = await fetch(`${BASE}/api/upload`, { method: 'POST', body: form })
    const data = await res.json() as { ok: boolean; path?: string; message?: string }
    if (!data.ok) throw new Error(data.message ?? 'アップロード失敗')
    return data.path ?? file.name
  },

  async createFolder(name: string): Promise<void> {
    const res = await fetch(`${BASE}/api/folders`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name }),
    })
    if (!res.ok) throw new Error(`Create folder failed: HTTP ${res.status}`)
  },

  async deleteFolder(name: string): Promise<void> {
    const res = await fetch(`${BASE}/api/folders/${encodeURIComponent(name)}`, {
      method: 'DELETE',
    })
    if (!res.ok) throw new Error(`Delete folder failed: HTTP ${res.status}`)
  },

  // ── RPA control ────────────────────────────────────────────────────────────
  async runStep(scenario: string, stepIndex: number): Promise<void> {
    const res = await fetch(`${BASE}/api/run`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ scenario, from: stepIndex, to: stepIndex }),
    })
    if (!res.ok) {
      const body = await res.json().catch(() => ({})) as { message?: string }
      throw new Error(body.message ?? `Run failed: HTTP ${res.status}`)
    }
  },

  async run(scenario: string, options: { from?: number; dry_run?: boolean } = {}): Promise<void> {
    const res = await fetch(`${BASE}/api/run`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ scenario, ...options }),
    })
    if (!res.ok) {
      const body = await res.json().catch(() => ({})) as { message?: string }
      throw new Error(body.message ?? `Run failed: HTTP ${res.status}`)
    }
  },

  async stop(): Promise<void> {
    await fetch(`${BASE}/api/stop`, { method: 'POST' })
  },

  async status(): Promise<{ running: boolean; scenario: string | null }> {
    const res = await fetch(`${BASE}/api/status`)
    return res.json()
  },

  // ── AI Chat ────────────────────────────────────────────────────────────────
  async chat(messages: { role: 'user' | 'assistant'; content: string }[], scenarioYaml?: string): Promise<string> {
    const res = await fetch(`${BASE}/api/chat`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ messages, scenario_yaml: scenarioYaml }),
    })
    if (!res.ok) throw new Error(`Chat API error: ${res.status}`)
    const data = await res.json() as { reply: string }
    return data.reply
  },

  // ── Screenshot ─────────────────────────────────────────────────────────────
  screenshotUrl(): string {
    return `${BASE}/screenshot?t=${Date.now()}`
  },

  // ── SSE progress stream ────────────────────────────────────────────────────
  connectEvents(onEvent: (e: unknown) => void): () => void {
    const es = new EventSource(`${BASE}/events`)
    es.onmessage = (e) => {
      try { onEvent(JSON.parse(e.data)) } catch { /* ignore */ }
    }
    return () => es.close()
  },
}
