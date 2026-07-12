import { useState, useRef, useEffect } from 'react'
import { MessageCircle, X, Send, Loader2, Bot, User } from 'lucide-react'
import { api } from '../api/client'
import { useScenarioStore } from '../store/scenarioStore'
import './AiChat.css'

interface Message {
  role: 'user' | 'assistant'
  content: string
}

export function AiChat() {
  const [open, setOpen] = useState(false)
  const [messages, setMessages] = useState<Message[]>([])
  const [input, setInput] = useState('')
  const [loading, setLoading] = useState(false)
  const bottomRef = useRef<HTMLDivElement>(null)
  const { yaml, setYaml, createScenarioFromYaml } = useScenarioStore()

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [messages])

  const send = async () => {
    const text = input.trim()
    if (!text || loading) return

    const next: Message[] = [...messages, { role: 'user', content: text }]
    setMessages(next)
    setInput('')
    setLoading(true)

    try {
      const reply = await api.chat(next, yaml || undefined)
      setMessages([...next, { role: 'assistant', content: reply }])

      // YAMLコードブロックが返ってきたら自動反映（未選択なら新規シナリオとして開く）
      const match = reply.match(/```ya?ml[ \t]*\r?\n([\s\S]*?)```/)
      if (match) {
        const generatedYaml = match[1].trim()
        // 応答待ち中に別シナリオが開かれた場合に備え、判定直前の最新状態を読む
        if (useScenarioStore.getState().activeScenario) {
          setYaml(generatedYaml)
        } else {
          await createScenarioFromYaml(generatedYaml)
        }
      }
    } catch (err) {
      setMessages([
        ...next,
        { role: 'assistant', content: `エラーが発生しました: ${String(err)}` },
      ])
    } finally {
      setLoading(false)
    }
  }

  return (
    <>
      {/* 右下トグルボタン */}
      {!open && (
        <button className="ai-chat-fab" onClick={() => setOpen(true)} title="AIアシスタント">
          <MessageCircle size={22} />
          <span>AIアシスタント</span>
        </button>
      )}

      {/* チャットパネル */}
      {open && (
        <div className="ai-chat-panel">
          <div className="ai-chat-header">
            <div className="ai-chat-title">
              <Bot size={16} />
              <span>AIアシスタント</span>
            </div>
            <button className="ai-chat-close" onClick={() => setOpen(false)}>
              <X size={16} />
            </button>
          </div>

          <div className="ai-chat-messages">
            {messages.length === 0 && (
              <div className="ai-chat-empty">
                <Bot size={32} />
                <p>シナリオ作成をお手伝いします。<br />やりたい自動化を日本語で教えてください。</p>
              </div>
            )}
            {messages.map((m, i) => (
              <div key={i} className={`ai-chat-message ${m.role}`}>
                <span className="ai-chat-avatar">
                  {m.role === 'user' ? <User size={14} /> : <Bot size={14} />}
                </span>
                <pre className="ai-chat-content">{m.content}</pre>
              </div>
            ))}
            {loading && (
              <div className="ai-chat-message assistant">
                <span className="ai-chat-avatar"><Bot size={14} /></span>
                <span className="ai-chat-typing"><Loader2 size={14} className="spin" /> 考え中...</span>
              </div>
            )}
            <div ref={bottomRef} />
          </div>

          <div className="ai-chat-input-row">
            <textarea
              className="ai-chat-input"
              placeholder="メッセージを入力"
              value={input}
              onChange={(e) => setInput(e.target.value)}
              rows={2}
            />
            <button
              className="ai-chat-send"
              onClick={send}
              disabled={loading || !input.trim()}
            >
              <Send size={16} />
            </button>
          </div>
        </div>
      )}
    </>
  )
}
