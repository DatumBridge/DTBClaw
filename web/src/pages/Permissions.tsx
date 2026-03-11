import { useState, useEffect } from 'react';
import {
  ShieldCheck,
  FolderOpen,
  FolderX,
  Terminal,
  Plus,
  Trash2,
  Save,
  Loader2,
  CheckCircle2,
  AlertCircle,
  ToggleLeft,
  ToggleRight,
} from 'lucide-react';
import type { Permissions as PermissionsType } from '@/types/api';
import { getPermissions, putPermissions } from '@/lib/api';
import { t } from '@/lib/i18n';

function PathListSection({
  icon: Icon,
  iconColor,
  titleKey,
  descKey,
  items,
  placeholder,
  onAdd,
  onRemove,
}: {
  icon: React.ComponentType<{ className?: string }>;
  iconColor: string;
  titleKey: string;
  descKey: string;
  items: string[];
  placeholder: string;
  onAdd: (value: string) => void;
  onRemove: (index: number) => void;
}) {
  const [input, setInput] = useState('');

  const handleAdd = () => {
    const trimmed = input.trim();
    if (trimmed && !items.includes(trimmed)) {
      onAdd(trimmed);
      setInput('');
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      handleAdd();
    }
  };

  return (
    <div className="bg-gray-900 rounded-xl border border-gray-800 p-5">
      <div className="flex items-center gap-2 mb-1">
        <Icon className={`h-5 w-5 ${iconColor}`} />
        <h3 className="text-sm font-semibold text-white">{t(titleKey)}</h3>
      </div>
      <p className="text-xs text-gray-400 mb-4">{t(descKey)}</p>

      <div className="flex gap-2 mb-3">
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={placeholder}
          className="flex-1 bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-sm text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
        />
        <button
          type="button"
          onClick={handleAdd}
          disabled={!input.trim()}
          className="flex items-center gap-1.5 px-3 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-700 disabled:text-gray-500 text-white rounded-lg text-sm font-medium transition-colors"
        >
          <Plus className="h-4 w-4" />
          {t('permissions.add')}
        </button>
      </div>

      {items.length === 0 ? (
        <p className="text-xs text-gray-500 italic py-2">No entries configured.</p>
      ) : (
        <ul className="space-y-1.5">
          {items.map((item, idx) => (
            <li
              key={`${item}-${idx}`}
              className="flex items-center justify-between bg-gray-800/60 rounded-lg px-3 py-2 group"
            >
              <code className="text-sm text-gray-300 font-mono truncate">
                {item}
              </code>
              <button
                type="button"
                onClick={() => onRemove(idx)}
                className="p-1 rounded text-gray-500 hover:text-red-400 hover:bg-red-900/20 opacity-0 group-hover:opacity-100 transition-all"
                title={t('permissions.remove')}
              >
                <Trash2 className="h-3.5 w-3.5" />
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

export default function Permissions() {
  const [permissions, setPermissions] = useState<PermissionsType | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [toast, setToast] = useState<{ type: 'success' | 'error'; message: string } | null>(null);
  const [dirty, setDirty] = useState(false);

  useEffect(() => {
    getPermissions()
      .then((data) => setPermissions(data))
      .catch((err) => setError(err.message))
      .finally(() => setLoading(false));
  }, []);

  useEffect(() => {
    if (toast) {
      const timer = setTimeout(() => setToast(null), 3000);
      return () => clearTimeout(timer);
    }
  }, [toast]);

  const update = (patch: Partial<PermissionsType>) => {
    if (!permissions) return;
    setPermissions({ ...permissions, ...patch });
    setDirty(true);
  };

  const handleSave = async () => {
    if (!permissions) return;
    setSaving(true);
    try {
      await putPermissions(permissions);
      setToast({ type: 'success', message: t('permissions.saved') });
      setDirty(false);
    } catch (err) {
      setToast({ type: 'error', message: t('permissions.error') });
    } finally {
      setSaving(false);
    }
  };

  if (error) {
    return (
      <div className="p-6">
        <div className="rounded-lg bg-red-900/30 border border-red-700 p-4 text-red-300">
          Failed to load permissions: {error}
        </div>
      </div>
    );
  }

  if (loading || !permissions) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-2 border-blue-500 border-t-transparent" />
      </div>
    );
  }

  return (
    <div className="p-6 space-y-6 max-w-4xl">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <ShieldCheck className="h-6 w-6 text-blue-400" />
          <h1 className="text-xl font-bold text-white">{t('permissions.title')}</h1>
        </div>
        <button
          type="button"
          onClick={handleSave}
          disabled={saving || !dirty}
          className="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-700 disabled:text-gray-500 text-white rounded-lg text-sm font-medium transition-colors"
        >
          {saving ? (
            <Loader2 className="h-4 w-4 animate-spin" />
          ) : (
            <Save className="h-4 w-4" />
          )}
          {t('permissions.save')}
        </button>
      </div>

      {/* Toast */}
      {toast && (
        <div
          className={[
            'flex items-center gap-2 rounded-lg px-4 py-3 text-sm',
            toast.type === 'success'
              ? 'bg-green-900/30 border border-green-700 text-green-300'
              : 'bg-red-900/30 border border-red-700 text-red-300',
          ].join(' ')}
        >
          {toast.type === 'success' ? (
            <CheckCircle2 className="h-4 w-4 flex-shrink-0" />
          ) : (
            <AlertCircle className="h-4 w-4 flex-shrink-0" />
          )}
          {toast.message}
        </div>
      )}

      {/* Workspace Only Toggle */}
      <div className="bg-gray-900 rounded-xl border border-gray-800 p-5">
        <div className="flex items-center justify-between">
          <div>
            <div className="flex items-center gap-2 mb-1">
              <FolderOpen className="h-5 w-5 text-amber-400" />
              <h3 className="text-sm font-semibold text-white">
                {t('permissions.workspace_only')}
              </h3>
            </div>
            <p className="text-xs text-gray-400">
              {t('permissions.workspace_only_desc')}
            </p>
          </div>
          <button
            type="button"
            onClick={() => update({ workspace_only: !permissions.workspace_only })}
            className="flex-shrink-0 ml-4"
          >
            {permissions.workspace_only ? (
              <ToggleRight className="h-8 w-8 text-blue-400" />
            ) : (
              <ToggleLeft className="h-8 w-8 text-gray-500" />
            )}
          </button>
        </div>
      </div>

      {/* Allowed Roots */}
      <PathListSection
        icon={FolderOpen}
        iconColor="text-green-400"
        titleKey="permissions.allowed_roots"
        descKey="permissions.allowed_roots_desc"
        items={permissions.allowed_roots}
        placeholder={t('permissions.placeholder_path')}
        onAdd={(val) =>
          update({ allowed_roots: [...permissions.allowed_roots, val] })
        }
        onRemove={(idx) =>
          update({
            allowed_roots: permissions.allowed_roots.filter((_, i) => i !== idx),
          })
        }
      />

      {/* Forbidden Paths */}
      <PathListSection
        icon={FolderX}
        iconColor="text-red-400"
        titleKey="permissions.forbidden_paths"
        descKey="permissions.forbidden_paths_desc"
        items={permissions.forbidden_paths}
        placeholder={t('permissions.placeholder_path')}
        onAdd={(val) =>
          update({ forbidden_paths: [...permissions.forbidden_paths, val] })
        }
        onRemove={(idx) =>
          update({
            forbidden_paths: permissions.forbidden_paths.filter(
              (_, i) => i !== idx,
            ),
          })
        }
      />

      {/* Allowed Commands */}
      <PathListSection
        icon={Terminal}
        iconColor="text-purple-400"
        titleKey="permissions.allowed_commands"
        descKey="permissions.allowed_commands_desc"
        items={permissions.allowed_commands}
        placeholder={t('permissions.placeholder_command')}
        onAdd={(val) =>
          update({ allowed_commands: [...permissions.allowed_commands, val] })
        }
        onRemove={(idx) =>
          update({
            allowed_commands: permissions.allowed_commands.filter(
              (_, i) => i !== idx,
            ),
          })
        }
      />
    </div>
  );
}
