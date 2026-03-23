import { Routes, Route, Navigate } from 'react-router-dom';
import { useState, useEffect, createContext, useContext } from 'react';
import Layout from './components/layout/Layout';
import Dashboard from './pages/Dashboard';
import AgentChat from './pages/AgentChat';
import Tools from './pages/Tools';
import Cron from './pages/Cron';
import Integrations from './pages/Integrations';
import Memory from './pages/Memory';
import Devices from './pages/Devices';
import Config from './pages/Config';
import Cost from './pages/Cost';
import Logs from './pages/Logs';
import Doctor from './pages/Doctor';
import Permissions from './pages/Permissions';
import { AuthProvider, useAuth } from './hooks/useAuth';
import { setLocale, type Locale } from './lib/i18n';
import {
  AdminContext,
  consumeAdminFromURL,
  getPersistedAdmin,
  persistAdmin,
} from './hooks/useAdminMode';

// Locale context
interface LocaleContextType {
  locale: Locale;
  setAppLocale: (locale: Locale) => void;
}

export const LocaleContext = createContext<LocaleContextType>({
  locale: 'tr',
  setAppLocale: (_locale: Locale) => {},
});

export const useLocaleContext = () => useContext(LocaleContext);

// Pairing dialog component
function PairingDialog({ onPair }: { onPair: (code: string) => Promise<void> }) {
  const [code, setCode] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError('');
    try {
      await onPair(code);
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : 'Pairing failed');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen bg-[#0f172a] flex items-center justify-center px-4">
      <div className="w-full max-w-md rounded-2xl border border-[#334155] bg-[#1e293b] p-8 shadow-xl shadow-black/20">
        <div className="mb-8 text-center">
          <div className="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-xl bg-teal-500/20 text-teal-400">
            <span className="text-xl font-bold">DB</span>
          </div>
          <h1 className="text-2xl font-semibold tracking-tight text-slate-50">DatumBridge</h1>
          <p className="mt-2 text-sm text-slate-400">Enter the 6-digit pairing code from your terminal</p>
        </div>
        <form onSubmit={handleSubmit} className="space-y-4">
          <input
            type="text"
            value={code}
            onChange={(e) => setCode(e.target.value)}
            placeholder="000000"
            className="w-full rounded-lg border border-slate-600 bg-slate-800/50 px-4 py-3.5 text-center text-2xl tracking-[0.4em] text-slate-100 placeholder:text-slate-500 focus:border-teal-500 focus:ring-2 focus:ring-teal-500/20"
            maxLength={6}
            autoFocus
          />
          {error && (
            <p className="text-center text-sm text-rose-400">{error}</p>
          )}
          <button
            type="submit"
            disabled={loading || code.length < 6}
            className="btn-primary w-full py-3 disabled:cursor-not-allowed disabled:opacity-50"
          >
            {loading ? 'Pairing…' : 'Pair device'}
          </button>
        </form>
      </div>
    </div>
  );
}

// Authenticated app shell — only mounted after login so hook count is stable.
function AuthenticatedApp() {
  const [isAdmin, setIsAdmin] = useState(() => {
    if (consumeAdminFromURL()) {
      persistAdmin(true);
      return true;
    }
    return getPersistedAdmin();
  });
  const [locale, setLocaleState] = useState<Locale>('tr');

  const setAdmin = (value: boolean) => {
    persistAdmin(value);
    setIsAdmin(value);
  };

  const setAppLocale = (newLocale: Locale) => {
    setLocaleState(newLocale);
    setLocale(newLocale);
  };

  const adminOrHome = (el: React.ReactNode) =>
    isAdmin ? el : <Navigate to="/" replace />;

  return (
    <AdminContext.Provider value={{ isAdmin, setAdmin }}>
      <LocaleContext.Provider value={{ locale, setAppLocale }}>
        <Routes>
          <Route element={<Layout />}>
            <Route path="/" element={<Dashboard />} />
            <Route path="/agent" element={<AgentChat />} />
            <Route path="/tools" element={adminOrHome(<Tools />)} />
            <Route path="/cron" element={adminOrHome(<Cron />)} />
            <Route path="/integrations" element={adminOrHome(<Integrations />)} />
            <Route path="/memory" element={adminOrHome(<Memory />)} />
            <Route path="/devices" element={adminOrHome(<Devices />)} />
            <Route path="/config" element={adminOrHome(<Config />)} />
            <Route path="/cost" element={adminOrHome(<Cost />)} />
            <Route path="/logs" element={adminOrHome(<Logs />)} />
            <Route path="/doctor" element={adminOrHome(<Doctor />)} />
            <Route path="/permissions" element={adminOrHome(<Permissions />)} />
            <Route path="*" element={<Navigate to="/" replace />} />
          </Route>
        </Routes>
      </LocaleContext.Provider>
    </AdminContext.Provider>
  );
}

function AppContent() {
  const { isAuthenticated, loading, pair, logout } = useAuth();

  // Listen for 401 events to force logout — must run every render (same hook order).
  useEffect(() => {
    const handler = () => {
      logout();
    };
    window.addEventListener('octoclaw-unauthorized', handler);
    return () => window.removeEventListener('octoclaw-unauthorized', handler);
  }, [logout]);

  if (loading) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-[#0f172a]">
        <div className="flex items-center gap-3 text-slate-400">
          <div className="h-5 w-5 animate-spin rounded-full border-2 border-teal-500 border-t-transparent" />
          <span>Connecting…</span>
        </div>
      </div>
    );
  }

  if (!isAuthenticated) {
    return <PairingDialog onPair={pair} />;
  }

  return <AuthenticatedApp />;
}

export default function App() {
  return (
    <AuthProvider>
      <AppContent />
    </AuthProvider>
  );
}
