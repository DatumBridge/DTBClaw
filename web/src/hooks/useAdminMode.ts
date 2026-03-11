import { createContext, useContext } from 'react';

const ADMIN_STORAGE_KEY = 'zeroclaw_admin_mode';

interface AdminContextType {
  isAdmin: boolean;
  setAdmin: (value: boolean) => void;
}

export const AdminContext = createContext<AdminContextType>({
  isAdmin: false,
  setAdmin: () => {},
});

export function useAdminMode(): boolean {
  return useContext(AdminContext).isAdmin;
}

export function useAdminToggle(): (value: boolean) => void {
  return useContext(AdminContext).setAdmin;
}

export function getPersistedAdmin(): boolean {
  return localStorage.getItem(ADMIN_STORAGE_KEY) === 'true';
}

export function persistAdmin(value: boolean): void {
  if (value) {
    localStorage.setItem(ADMIN_STORAGE_KEY, 'true');
  } else {
    localStorage.removeItem(ADMIN_STORAGE_KEY);
  }
}

export function consumeAdminFromURL(): boolean {
  const params = new URLSearchParams(window.location.search);
  if (params.get('admin') === 'true') {
    params.delete('admin');
    const qs = params.toString();
    const newUrl =
      window.location.pathname + (qs ? `?${qs}` : '') + window.location.hash;
    window.history.replaceState(null, '', newUrl);
    return true;
  }
  return false;
}
