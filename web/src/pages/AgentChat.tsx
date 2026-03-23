import { useState, useEffect, useRef } from 'react';
import { Send, Bot, User, AlertCircle } from 'lucide-react';
import type { WsMessage } from '@/types/api';
import { WebSocketClient } from '@/lib/ws';

interface ChatMessage {
  id: string;
  role: 'user' | 'agent';
  content: string;
  timestamp: Date;
}

let fallbackMessageIdCounter = 0;
const EMPTY_DONE_FALLBACK =
  'Tool execution completed, but no final response text was returned.';

function makeMessageId(): string {
  const uuid = globalThis.crypto?.randomUUID?.();
  if (uuid) return uuid;

  fallbackMessageIdCounter += 1;
  return `msg_${Date.now().toString(36)}_${fallbackMessageIdCounter.toString(36)}_${Math.random()
    .toString(36)
    .slice(2, 10)}`;
}

export default function AgentChat() {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [input, setInput] = useState('');
  const [typing, setTyping] = useState(false);
  const [connected, setConnected] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const wsRef = useRef<WebSocketClient | null>(null);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const pendingContentRef = useRef('');

  useEffect(() => {
    const ws = new WebSocketClient();

    ws.onOpen = () => {
      setConnected(true);
      setError(null);
    };

    ws.onClose = () => {
      setConnected(false);
    };

    ws.onError = () => {
      setError('Connection error. Attempting to reconnect...');
    };

    ws.onMessage = (msg: WsMessage) => {
      switch (msg.type) {
        case 'history': {
          const restored: ChatMessage[] = (msg.messages ?? [])
            .filter((entry) => entry.content?.trim())
            .map((entry): ChatMessage => ({
              id: makeMessageId(),
              role: entry.role === 'user' ? 'user' : 'agent',
              content: entry.content.trim(),
              timestamp: new Date(),
            }));

          setMessages(restored);
          setTyping(false);
          pendingContentRef.current = '';
          break;
        }

        case 'chunk':
          setTyping(true);
          pendingContentRef.current += msg.content ?? '';
          break;

        case 'message':
        case 'done': {
          const content = (msg.full_response ?? msg.content ?? pendingContentRef.current ?? '').trim();
          const finalContent = content || EMPTY_DONE_FALLBACK;

          setMessages((prev) => [
            ...prev,
            {
              id: makeMessageId(),
              role: 'agent',
              content: finalContent,
              timestamp: new Date(),
            },
          ]);

          pendingContentRef.current = '';
          setTyping(false);
          break;
        }

        case 'tool_call':
          setMessages((prev) => [
            ...prev,
            {
              id: makeMessageId(),
              role: 'agent',
              content: `[Tool Call] ${msg.name ?? 'unknown'}(${JSON.stringify(msg.args ?? {})})`,
              timestamp: new Date(),
            },
          ]);
          break;

        case 'tool_result':
          setMessages((prev) => [
            ...prev,
            {
              id: makeMessageId(),
              role: 'agent',
              content: `[Tool Result] ${msg.output ?? ''}`,
              timestamp: new Date(),
            },
          ]);
          break;

        case 'error':
          setMessages((prev) => [
            ...prev,
            {
              id: makeMessageId(),
              role: 'agent',
              content: `[Error] ${msg.message ?? 'Unknown error'}`,
              timestamp: new Date(),
            },
          ]);
          setTyping(false);
          pendingContentRef.current = '';
          break;
      }
    };

    ws.connect();
    wsRef.current = ws;

    return () => {
      ws.disconnect();
    };
  }, []);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages, typing]);

  const handleSend = () => {
    const trimmed = input.trim();
    if (!trimmed || !wsRef.current?.connected) return;

    setMessages((prev) => [
      ...prev,
      {
        id: makeMessageId(),
        role: 'user',
        content: trimmed,
        timestamp: new Date(),
      },
    ]);

    try {
      wsRef.current.sendMessage(trimmed);
      setTyping(true);
      pendingContentRef.current = '';
    } catch {
      setError('Failed to send message. Please try again.');
    }

    setInput('');
    inputRef.current?.focus();
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  return (
    <div className="flex flex-col h-[calc(100vh-3.5rem)]">
      {/* Connection status bar */}
      {error && (
        <div className="px-4 py-2 bg-rose-500/15 border-b border-rose-500/40 flex items-center gap-2 text-sm text-rose-300">
          <AlertCircle className="h-4 w-4 flex-shrink-0" />
          {error}
        </div>
      )}

      {/* Messages area */}
      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        {messages.length === 0 && (
          <div className="flex flex-col items-center justify-center h-full text-slate-500">
            <Bot className="h-12 w-12 mb-3 text-slate-600" />
            <p className="text-lg font-medium">DatumBridge Agent</p>
            <p className="text-sm mt-1">Send a message to start the conversation</p>
          </div>
        )}

        {messages.map((msg) => (
          <div
            key={msg.id}
            className={`flex items-start gap-3 ${
              msg.role === 'user' ? 'flex-row-reverse' : ''
            }`}
          >
            <div
              className={`flex-shrink-0 w-8 h-8 rounded-full flex items-center justify-center ${
                msg.role === 'user'
                  ? 'bg-teal-500'
                  : 'bg-slate-700'
              }`}
            >
              {msg.role === 'user' ? (
                <User className="h-4 w-4 text-white" />
              ) : (
                <Bot className="h-4 w-4 text-white" />
              )}
            </div>
            <div
              className={`max-w-[75%] rounded-xl px-4 py-3 ${
                msg.role === 'user'
                  ? 'bg-teal-500 text-white'
                  : 'bg-slate-800 text-slate-100 border border-slate-700'
              }`}
            >
              <p className="text-sm whitespace-pre-wrap break-words">{msg.content}</p>
              <p
                className={`text-xs mt-1 ${
                  msg.role === 'user' ? 'text-teal-200' : 'text-slate-500'
                }`}
              >
                {msg.timestamp.toLocaleTimeString()}
              </p>
            </div>
          </div>
        ))}

        {typing && (
          <div className="flex items-start gap-3">
            <div className="flex-shrink-0 w-8 h-8 rounded-full bg-slate-700 flex items-center justify-center">
              <Bot className="h-4 w-4 text-white" />
            </div>
            <div className="bg-slate-800 border border-slate-700 rounded-xl px-4 py-3">
              <div className="flex items-center gap-1">
                <span className="w-2 h-2 bg-slate-400 rounded-full animate-bounce" style={{ animationDelay: '0ms' }} />
                <span className="w-2 h-2 bg-slate-400 rounded-full animate-bounce" style={{ animationDelay: '150ms' }} />
                <span className="w-2 h-2 bg-slate-400 rounded-full animate-bounce" style={{ animationDelay: '300ms' }} />
              </div>
              <p className="text-xs text-slate-500 mt-1">Typing...</p>
            </div>
          </div>
        )}

        <div ref={messagesEndRef} />
      </div>

      {/* Input area */}
      <div className="border-t border-slate-700/80 bg-slate-800/50 p-4">
        <div className="flex items-center gap-3 max-w-4xl mx-auto">
          <div className="flex-1 relative">
            <input
              ref={inputRef}
              type="text"
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder={connected ? 'Type a message...' : 'Connecting...'}
              disabled={!connected}
              className="w-full bg-slate-800 border border-slate-600 rounded-xl px-4 py-3 text-sm text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-teal-500 focus:border-transparent disabled:opacity-50"
            />
          </div>
          <button
            onClick={handleSend}
            disabled={!connected || !input.trim()}
            className="flex-shrink-0 btn-primary rounded-xl p-3 disabled:bg-slate-700 disabled:text-slate-500"
          >
            <Send className="h-5 w-5" />
          </button>
        </div>
        <div className="flex items-center justify-center mt-2 gap-2">
          <span
            className={`inline-block h-2 w-2 rounded-full ${
              connected ? 'bg-emerald-500' : 'bg-rose-500'
            }`}
          />
          <span className="text-xs text-slate-500">
            {connected ? 'Connected' : 'Disconnected'}
          </span>
        </div>
      </div>
    </div>
  );
}
