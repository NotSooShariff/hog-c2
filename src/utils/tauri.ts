import { invoke } from '@tauri-apps/api/core';
import { sendNotification } from '@tauri-apps/plugin-notification';
import type { AppLimit, AppUsage } from '../types';

export async function getAppStats(): Promise<AppUsage[]> {
  return await invoke<AppUsage[]>('get_app_stats');
}

export async function getAppUsage(appName: string): Promise<number> {
  return await invoke<number>('get_app_usage', { appName });
}

export async function resetStats(): Promise<void> {
  await invoke('reset_stats');
}

export async function showNotification(title: string, body: string): Promise<void> {
  await sendNotification({
    title,
    body,
  });
}

export async function registerClientWithNotion(): Promise<string> {
  return await invoke<string>('register_client_with_notion');
}

export async function syncNotionNow(): Promise<string> {
  return await invoke<string>('sync_notion_now');
}

export async function updateAppLimits(limits: AppLimit[]): Promise<void> {
  await invoke('update_app_limits', { limits });
}

export async function getAppLimits(): Promise<AppLimit[]> {
  return await invoke<AppLimit[]>('get_app_limits');
}

export async function isAutostartEnabled(): Promise<boolean> {
  return await invoke<boolean>('is_autostart_enabled');
}

export async function setAutostartEnabled(enable: boolean): Promise<void> {
  await invoke('set_autostart_enabled', { enable });
}

export async function showWindow(): Promise<void> {
  await invoke('show_window');
}

export async function hideWindow(): Promise<void> {
  await invoke('hide_window');
}

export function formatDuration(seconds: number): string {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = seconds % 60;

  if (hours > 0) {
    return `${hours}h ${minutes}m`;
  } else if (minutes > 0) {
    return `${minutes}m ${secs}s`;
  } else {
    return `${secs}s`;
  }
}
