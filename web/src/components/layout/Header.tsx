import { useLocation } from 'react-router-dom';
import { LogOut, Menu } from 'lucide-react';
import { t } from '@/lib/i18n';
import type { Locale } from '@/lib/i18n';
import { useLocaleContext } from '@/App';
import { useAuth } from '@/hooks/useAuth';

const routeTitles: Record<string, string> = {
  '/': 'nav.dashboard',
  '/agent': 'nav.agent',
  '/tools': 'nav.tools',
  '/cron': 'nav.cron',
  '/integrations': 'nav.integrations',
  '/memory': 'nav.memory',
  '/config': 'nav.config',
  '/cost': 'nav.cost',
  '/logs': 'nav.logs',
  '/doctor': 'nav.doctor',
};

const localeCycle: Locale[] = ['en', 'tr', 'zh-CN'];

interface HeaderProps {
  onToggleSidebar: () => void;
}

export default function Header({ onToggleSidebar }: HeaderProps) {
  const location = useLocation();
  const { logout } = useAuth();
  const { locale, setAppLocale } = useLocaleContext();

  const titleKey = routeTitles[location.pathname] ?? 'nav.dashboard';
  const pageTitle = t(titleKey);

  const toggleLanguage = () => {
    const currentIndex = localeCycle.indexOf(locale);
    const nextLocale = localeCycle[(currentIndex + 1) % localeCycle.length] ?? 'en';
    setAppLocale(nextLocale);
  };

  return (
    <header className="flex h-14 items-center justify-between border-b border-slate-700/80 bg-[#1e293b] px-4 md:px-6">
      <div className="flex items-center gap-3">
        <button
          type="button"
          onClick={onToggleSidebar}
          aria-label="Open navigation"
          className="rounded-lg p-2 text-slate-400 transition-colors hover:bg-slate-700/80 hover:text-slate-100 md:hidden"
        >
          <Menu className="h-5 w-5" />
        </button>
        <h1 className="text-lg font-semibold tracking-tight text-slate-50">{pageTitle}</h1>
      </div>

      <div className="flex items-center gap-2 md:gap-3">
        <button
          type="button"
          onClick={toggleLanguage}
          className="rounded-lg border border-slate-600 bg-slate-800/50 px-3 py-1.5 text-sm font-medium text-slate-300 transition-colors hover:bg-slate-700 hover:text-slate-100"
        >
          {locale === 'en' ? 'EN' : locale === 'tr' ? 'TR' : '中文'}
        </button>

        <button
          type="button"
          onClick={logout}
          className="flex items-center gap-2 rounded-lg px-3 py-1.5 text-sm text-slate-400 transition-colors hover:bg-slate-700/80 hover:text-slate-100"
        >
          <LogOut className="h-4 w-4" />
          <span className="hidden sm:inline">{t('auth.logout')}</span>
        </button>
      </div>
    </header>
  );
}
