import { useState, useRef, useEffect } from 'react';
import ReactMarkdown from 'react-markdown';
import { Send, TerminalSquare, CheckCircle2, XCircle, Bot, User, Loader2 } from 'lucide-react';

type SSEEvent = {
  type: string;
  content: string;
};

type AppMessage = {
  role: "user" | "assistant" | "system";
  content: string;
  events?: SSEEvent[]; // Stores tool calls & statuses safely
};

export default function App() {
  const [messages, setMessages] = useState<AppMessage[]>([]);
  const [input, setInput] = useState("");
  const [isProcessing, setIsProcessing] = useState(false);
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!input.trim() || isProcessing) return;

    const userMessage: AppMessage = { role: "user", content: input };
    const newMessages = [...messages, userMessage];
    setMessages(newMessages);
    setInput("");
    setIsProcessing(true);

    try {
      const response = await fetch("http://127.0.0.1:3000/chat", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ messages: newMessages.map(m => ({ role: m.role, content: m.content })) })
      });

      if (!response.body) throw new Error("No response body");

      const reader = response.body.getReader();
      const decoder = new TextDecoder();

      setMessages(prev => [...prev, { role: "assistant", content: "", events: [] }]);

      let assistantContent = "";
      let events: SSEEvent[] = [];

      while (true) {
        const { value, done } = await reader.read();
        if (done) break;
        
        const chunk = decoder.decode(value);
        const lines = chunk.split('\n');
        
        for (const line of lines) {
          if (line.startsWith('data: ')) {
            try {
              const data = JSON.parse(line.substring(6));
              
              if (data.type === "text") {
                assistantContent += data.content;
              } else if (data.type !== "done") {
                events.push(data);
              }

              setMessages(prev => {
                const updated = [...prev];
                updated[updated.length - 1] = {
                  role: "assistant",
                  content: assistantContent,
                  events: [...events]
                };
                return updated;
              });

            } catch (err) {
              console.error("SSE parse error:", err);
            }
          }
        }
      }

    } catch (e) {
      console.error(e);
    } finally {
      setIsProcessing(false);
    }
  };

  return (
    <div className="flex flex-col h-screen bg-zinc-950 text-slate-200">
      {/* Header */}
      <div className="flex-none p-4 bg-zinc-900/60 backdrop-blur-md border-b border-white/5 flex items-center justify-between">
        <div className="flex items-center gap-3">
            <div className="bg-indigo-500/20 p-2 rounded-lg">
                <TerminalSquare className="w-6 h-6 text-indigo-400" />
            </div>
            <div>
                <h1 className="text-xl font-bold tracking-tight text-white">Helix Agent</h1>
                <p className="text-xs text-indigo-400 font-medium">Rust + Vite Hybrid Stack</p>
            </div>
        </div>
        <div className="flex items-center gap-2">
            <div className="w-2 h-2 rounded-full bg-emerald-500 shadow-[0_0_10px_theme(colors.emerald.500)] animate-pulse"></div>
            <span className="text-sm font-medium text-emerald-500">Autonomous</span>
        </div>
      </div>

      {/* Chat Area */}
      <div className="flex-1 overflow-y-auto p-4 md:p-8">
        <div className="max-w-4xl mx-auto space-y-6">
          {messages.length === 0 && (
            <div className="flex flex-col items-center justify-center h-full text-center text-zinc-500 pt-20">
                <Bot className="w-12 h-12 mb-4 opacity-50" />
                <h2 className="text-2xl font-semibold mb-2 text-zinc-400">System Ready</h2>
                <p>Agent is online and standing by. Multi-line pastes supported.</p>
            </div>
          )}

          {messages.map((msg, i) => (
            <div key={i} className={`flex gap-4 ${msg.role === 'user' ? 'justify-end' : 'justify-start'}`}>
              {msg.role === 'assistant' && (
                <div className="flex-none mt-1">
                  <div className="w-8 h-8 rounded-full bg-indigo-500/20 border border-indigo-500/30 flex items-center justify-center">
                    <Bot className="w-5 h-5 text-indigo-400" />
                  </div>
                </div>
              )}
              
              <div className={`max-w-[85%] ${msg.role === 'user' ? 'order-1' : 'order-2'}`}>
                {msg.role === 'user' ? (
                  <div className="bg-indigo-600 text-white rounded-2xl rounded-tr-sm px-5 py-3 shadow-lg">
                    <pre className="whitespace-pre-wrap font-sans text-sm">{msg.content}</pre>
                  </div>
                ) : (
                  <div className="space-y-3">
                    {/* Tool Events */}
                    {msg.events && msg.events.length > 0 && (
                      <div className="space-y-2 mb-4 bg-zinc-900/40 rounded-xl p-4 border border-white/5">
                        {msg.events.map((evt, j) => (
                           <div key={j} className="flex gap-3 text-sm items-start font-mono">
                               {evt.type === 'tool_start' && <Loader2 className="w-4 h-4 text-amber-500 animate-spin mt-0.5 shrink-0" />}
                               {evt.type === 'tool_result' && (evt.content.includes('Success') ? <CheckCircle2 className="w-4 h-4 text-emerald-500 mt-0.5 shrink-0" /> : <XCircle className="w-4 h-4 text-rose-500 mt-0.5 shrink-0" />)}
                               {evt.type === 'system' && <TerminalSquare className="w-4 h-4 text-blue-400 mt-0.5 shrink-0" />}
                               {evt.type === 'error' && <XCircle className="w-4 h-4 text-rose-500 mt-0.5 shrink-0" />}
                               <span className={evt.type === 'error' || evt.type.includes('Fail') ? 'text-rose-400' : 'text-zinc-400'}>
                                   {evt.content}
                               </span>
                           </div>
                        ))}
                      </div>
                    )}
                    
                    {/* Final Output */}
                    {msg.content && (
                      <div className="bg-zinc-900 border border-zinc-800 rounded-2xl px-6 py-4 shadow-sm text-zinc-300 prose prose-invert prose-indigo max-w-none">
                        <ReactMarkdown>{msg.content}</ReactMarkdown>
                      </div>
                    )}
                  </div>
                )}
              </div>

              {msg.role === 'user' && (
                <div className="flex-none mt-1 order-2">
                  <div className="w-8 h-8 rounded-full bg-zinc-800 border border-zinc-700 flex items-center justify-center">
                    <User className="w-5 h-5 text-zinc-400" />
                  </div>
                </div>
              )}
            </div>
          ))}
          <div ref={bottomRef} className="h-4" />
        </div>
      </div>

      {/* Input Area */}
      <div className="flex-none p-4 md:p-6 bg-zinc-950 border-t border-white/5">
        <form onSubmit={handleSubmit} className="max-w-4xl mx-auto relative glass-panel rounded-2xl overflow-hidden group focus-within:border-indigo-500/50 transition-colors">
          <textarea
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault();
                handleSubmit(e);
              }
            }}
            placeholder="Instruct the agent... (Shift+Enter for new line)"
            className="w-full bg-transparent text-white p-4 pr-14 resize-none outline-none min-h-[60px] max-h-[300px]"
            rows={Math.min(10, input.split('\n').length || 1)}
            disabled={isProcessing}
          />
          <button
            type="submit"
            disabled={!input.trim() || isProcessing}
            className="absolute right-3 bottom-3 p-2 rounded-xl bg-indigo-500 hover:bg-indigo-600 disabled:opacity-50 disabled:hover:bg-indigo-500 text-white transition-all shadow-md group-focus-within:shadow-indigo-500/20"
          >
            {isProcessing ? <Loader2 className="w-5 h-5 animate-spin" /> : <Send className="w-5 h-5" />}
          </button>
        </form>
      </div>
    </div>
  );
}
