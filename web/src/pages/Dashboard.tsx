import { useState, useEffect } from 'react';
import {
  Cpu,
  Clock,
  Globe,
  Database,
  Activity,
  DollarSign,
  Radio,
} from 'lucide-react';
import type { StatusResponse, CostSummary } from '@/types/api';
import { getStatus, getCost } from '@/lib/api';

function formatUptime(seconds: number): string {
  const d = Math.floor(seconds / 86400);
  const h = Math.floor((seconds % 86400) / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  if (d > 0) return `${d}d ${h}h ${m}m`;
  if (h > 0) return `${h}h ${m}m`;
  return `${m}m`;
}

function formatUSD(value: number): string {
  return `$${value.toFixed(4)}`;
}

function healthColor(status: string): string {
  switch (status.toLowerCase()) {
    case 'ok':
    case 'healthy':
      return 'bg-emerald-500';
    case 'warn':
    case 'warning':
    case 'degraded':
      return 'bg-amber-500';
    default:
      return 'bg-rose-500';
  }
}

function healthBorder(status: string): string {
  switch (status.toLowerCase()) {
    case 'ok':
    case 'healthy':
      return 'border-emerald-500/30';
    case 'warn':
    case 'warning':
    case 'degraded':
      return 'border-amber-500/30';
    default:
      return 'border-rose-500/30';
  }
}

export default function Dashboard() {
  const [status, setStatus] = useState<StatusResponse | null>(null);
  const [cost, setCost] = useState<CostSummary | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    Promise.all([getStatus(), getCost()])
      .then(([s, c]) => {
        setStatus(s);
        setCost(c);
      })
      .catch((err) => setError(err.message));
  }, []);

  if (error) {
    return (
      <div className="p-6">
        <div className="rounded-xl border border-rose-500/30 bg-rose-500/10 px-4 py-3 text-rose-300">
          Failed to load dashboard: {error}
        </div>
      </div>
    );
  }

  if (!status || !cost) {
    return (
      <div className="flex h-64 items-center justify-center">
        <div className="h-8 w-8 animate-spin rounded-full border-2 border-teal-500 border-t-transparent" />
      </div>
    );
  }

  const maxCost = Math.max(cost.session_cost_usd, cost.daily_cost_usd, cost.monthly_cost_usd, 0.001);

  return (
    <div className="space-y-6 p-6">
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <div className="rounded-xl border border-slate-700/80 bg-slate-800/50 p-5 shadow-sm">
          <div className="mb-3 flex items-center gap-3">
            <div className="rounded-lg bg-teal-500/15 p-2">
              <Cpu className="h-5 w-5 text-teal-400" />
            </div>
            <span className="text-sm text-slate-400">Provider / Model</span>
          </div>
          <p className="truncate text-lg font-semibold text-slate-50">
            {status.provider ?? 'Unknown'}
          </p>
          <p className="truncate text-sm text-slate-400">{status.model}</p>
        </div>

        <div className="rounded-xl border border-slate-700/80 bg-slate-800/50 p-5 shadow-sm">
          <div className="mb-3 flex items-center gap-3">
            <div className="rounded-lg bg-emerald-500/15 p-2">
              <Clock className="h-5 w-5 text-emerald-400" />
            </div>
            <span className="text-sm text-slate-400">Uptime</span>
          </div>
          <p className="text-lg font-semibold text-slate-50">
            {formatUptime(status.uptime_seconds)}
          </p>
          <p className="text-sm text-slate-400">Since last restart</p>
        </div>

        <div className="rounded-xl border border-slate-700/80 bg-slate-800/50 p-5 shadow-sm">
          <div className="mb-3 flex items-center gap-3">
            <div className="rounded-lg bg-sky-500/15 p-2">
              <Globe className="h-5 w-5 text-sky-400" />
            </div>
            <span className="text-sm text-slate-400">Gateway Port</span>
          </div>
          <p className="text-lg font-semibold text-slate-50">
            :{status.gateway_port}
          </p>
          <p className="text-sm text-slate-400">Locale: {status.locale}</p>
        </div>

        <div className="rounded-xl border border-slate-700/80 bg-slate-800/50 p-5 shadow-sm">
          <div className="mb-3 flex items-center gap-3">
            <div className="rounded-lg bg-amber-500/15 p-2">
              <Database className="h-5 w-5 text-amber-400" />
            </div>
            <span className="text-sm text-slate-400">Memory Backend</span>
          </div>
          <p className="capitalize text-lg font-semibold text-slate-50">
            {status.memory_backend}
          </p>
          <p className="text-sm text-slate-400">
            Paired: {status.paired ? 'Yes' : 'No'}
          </p>
        </div>
      </div>

      <div className="grid grid-cols-1 gap-6 lg:grid-cols-3">
        <div className="rounded-xl border border-slate-700/80 bg-slate-800/50 p-5 shadow-sm">
          <div className="mb-4 flex items-center gap-2">
            <DollarSign className="h-5 w-5 text-teal-400" />
            <h2 className="text-base font-semibold text-slate-50">Cost Overview</h2>
          </div>
          <div className="space-y-4">
            {[
              { label: 'Session', value: cost.session_cost_usd, color: 'bg-teal-500' },
              { label: 'Daily', value: cost.daily_cost_usd, color: 'bg-emerald-500' },
              { label: 'Monthly', value: cost.monthly_cost_usd, color: 'bg-sky-500' },
            ].map(({ label, value, color }) => (
              <div key={label}>
                <div className="mb-1 flex justify-between text-sm">
                  <span className="text-slate-400">{label}</span>
                  <span className="font-medium text-slate-100">{formatUSD(value)}</span>
                </div>
                <div className="h-2 w-full overflow-hidden rounded-full bg-slate-700">
                  <div
                    className={`h-full rounded-full ${color}`}
                    style={{ width: `${Math.max((value / maxCost) * 100, 2)}%` }}
                  />
                </div>
              </div>
            ))}
          </div>
          <div className="mt-4 flex justify-between border-t border-slate-700/80 pt-3 text-sm">
            <span className="text-slate-400">Total Tokens</span>
            <span className="text-slate-100">{cost.total_tokens.toLocaleString()}</span>
          </div>
          <div className="mt-1 flex justify-between text-sm">
            <span className="text-slate-400">Requests</span>
            <span className="text-slate-100">{cost.request_count.toLocaleString()}</span>
          </div>
        </div>

        <div className="rounded-xl border border-slate-700/80 bg-slate-800/50 p-5 shadow-sm">
          <div className="mb-4 flex items-center gap-2">
            <Radio className="h-5 w-5 text-teal-400" />
            <h2 className="text-base font-semibold text-slate-50">Active Channels</h2>
          </div>
          <div className="space-y-2">
            {Object.entries(status.channels).length === 0 ? (
              <p className="text-sm text-slate-500">No channels configured</p>
            ) : (
              Object.entries(status.channels).map(([name, active]) => (
                <div
                  key={name}
                  className="flex items-center justify-between rounded-lg bg-slate-700/40 px-3 py-2"
                >
                  <span className="capitalize text-sm text-slate-100">{name}</span>
                  <div className="flex items-center gap-2">
                    <span
                      className={`inline-block h-2.5 w-2.5 rounded-full ${
                        active ? 'bg-emerald-500' : 'bg-slate-500'
                      }`}
                    />
                    <span className="text-xs text-slate-400">
                      {active ? 'Active' : 'Inactive'}
                    </span>
                  </div>
                </div>
              ))
            )}
          </div>
        </div>

        <div className="rounded-xl border border-slate-700/80 bg-slate-800/50 p-5 shadow-sm">
          <div className="mb-4 flex items-center gap-2">
            <Activity className="h-5 w-5 text-teal-400" />
            <h2 className="text-base font-semibold text-slate-50">Component Health</h2>
          </div>
          <div className="grid grid-cols-2 gap-3">
            {Object.entries(status.health.components).length === 0 ? (
              <p className="col-span-2 text-sm text-slate-500">No components reporting</p>
            ) : (
              Object.entries(status.health.components).map(([name, comp]) => (
                <div
                  key={name}
                  className={`rounded-lg border p-3 ${healthBorder(comp.status)} bg-slate-700/40`}
                >
                  <div className="mb-1 flex items-center gap-2">
                    <span className={`inline-block h-2 w-2 rounded-full ${healthColor(comp.status)}`} />
                    <span className="truncate text-sm font-medium capitalize text-slate-100">
                      {name}
                    </span>
                  </div>
                  <p className="capitalize text-xs text-slate-400">{comp.status}</p>
                  {comp.restart_count > 0 && (
                    <p className="mt-1 text-xs text-amber-400">
                      Restarts: {comp.restart_count}
                    </p>
                  )}
                </div>
              ))
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
