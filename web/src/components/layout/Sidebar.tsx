import { NavLink } from 'react-router-dom';
import {
  LayoutDashboard,
  MessageSquare,
  Wrench,
  Clock,
  Puzzle,
  Brain,
  Smartphone,
  Settings,
  DollarSign,
  Activity,
  Stethoscope,
  X,
  Shield,
  ShieldCheck,
} from 'lucide-react';
import { t } from '@/lib/i18n';
import { useAdminMode, useAdminToggle } from '@/hooks/useAdminMode';

const mainNavItems = [
  { to: '/', icon: LayoutDashboard, labelKey: 'nav.dashboard' },
  { to: '/agent', icon: MessageSquare, labelKey: 'nav.agent' },
];

const adminNavItems = [
  { to: '/tools', icon: Wrench, labelKey: 'nav.tools' },
  { to: '/cron', icon: Clock, labelKey: 'nav.cron' },
  { to: '/integrations', icon: Puzzle, labelKey: 'nav.integrations' },
  { to: '/memory', icon: Brain, labelKey: 'nav.memory' },
  { to: '/devices', icon: Smartphone, labelKey: 'nav.devices' },
  { to: '/config', icon: Settings, labelKey: 'nav.config' },
  { to: '/cost', icon: DollarSign, labelKey: 'nav.cost' },
  { to: '/logs', icon: Activity, labelKey: 'nav.logs' },
  { to: '/doctor', icon: Stethoscope, labelKey: 'nav.doctor' },
  { to: '/permissions', icon: ShieldCheck, labelKey: 'nav.permissions' },
];

interface SidebarProps {
  isOpen: boolean;
  onClose: () => void;
}

function NavItem({
  to,
  icon: Icon,
  labelKey,
  onClose,
}: {
  to: string;
  icon: React.ComponentType<{ className?: string }>;
  labelKey: string;
  onClose: () => void;
}) {
  return (
    <NavLink
      to={to}
      end={to === '/'}
      onClick={onClose}
      className={({ isActive }) =>
        [
          'flex items-center gap-3 rounded-lg px-3 py-2.5 text-sm font-medium transition-colors',
          isActive
            ? 'bg-teal-500/15 text-teal-400'
            : 'text-slate-400 hover:bg-slate-700/60 hover:text-slate-100',
        ].join(' ')
      }
    >
      <Icon className="h-5 w-5 flex-shrink-0" />
      <span>{t(labelKey)}</span>
    </NavLink>
  );
}

export default function Sidebar({ isOpen, onClose }: SidebarProps) {
  const isAdmin = useAdminMode();
  const setAdmin = useAdminToggle();

  return (
    <>
      <button
        type="button"
        aria-label="Close navigation"
        onClick={onClose}
        className={[
          'fixed inset-0 z-30 bg-black/40 transition-opacity md:hidden',
          isOpen ? 'opacity-100' : 'pointer-events-none opacity-0',
        ].join(' ')}
      />
      <aside
        className={[
          'fixed top-0 left-0 z-40 flex h-screen w-64 flex-col border-r border-slate-700/80 bg-[#1e293b]',
          'transform transition-transform duration-200 ease-out',
          isOpen ? 'translate-x-0' : '-translate-x-full',
          'md:translate-x-0',
        ].join(' ')}
      >
        <div className="flex items-center gap-3 border-b border-slate-700/80 px-5 py-4">
          <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-teal-500 text-sm font-semibold text-white shadow-sm">
            DB
          </div>
          <span className="text-base font-semibold tracking-tight text-slate-50">
            DatumBridge
          </span>
          <button
            type="button"
            onClick={onClose}
            aria-label="Close navigation"
            className="md:hidden rounded-lg p-2 text-slate-400 transition-colors hover:bg-slate-700/80 hover:text-slate-100"
          >
            <X className="h-4 w-4" />
          </button>
        </div>

        <nav className="flex-1 space-y-1 overflow-y-auto px-3 py-4">
          {mainNavItems.map((item) => (
            <NavItem key={item.to} {...item} onClose={onClose} />
          ))}

          {isAdmin && (
            <>
              <div className="flex items-center gap-2 px-3 pt-5 pb-1">
                <Shield className="h-3.5 w-3.5 text-slate-500" />
                <span className="text-xs font-medium uppercase tracking-wider text-slate-500">
                  {t('nav.section_admin')}
                </span>
              </div>
              {adminNavItems.map((item) => (
                <NavItem key={item.to} {...item} onClose={onClose} />
              ))}
            </>
          )}
        </nav>

        {isAdmin && (
          <div className="border-t border-slate-700/80 px-4 py-3">
            <button
              type="button"
              onClick={() => setAdmin(false)}
              className="flex w-full items-center gap-2 text-xs text-slate-500 transition-colors hover:text-slate-300 group"
              title="Click to deactivate Admin Mode"
            >
              <Shield className="h-3 w-3" />
              <span>{t('nav.admin_mode')}</span>
              <X className="ml-auto h-3 w-3 opacity-0 transition-opacity group-hover:opacity-100" />
            </button>
          </div>
        )}
      </aside>
    </>
  );
}
