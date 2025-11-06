export interface AppUsage {
  app_name: string;
  window_title: string;
  duration_seconds: number;
  last_active: string;
}

export interface Task {
  id: string;
  title: string;
  description: string;
  completed: boolean;
  allowedApps: string[];
  blockedApps: string[];
  allowedSites: string[];
  blockedSites: string[];
  createdAt: string;
}

export interface AppLimit {
  app_name: string;
  max_duration_minutes: number;
  notification_threshold_minutes: number;
  enabled: boolean;
}

export interface Settings {
  limits: AppLimit[];
  enableNotifications: boolean;
  trackingEnabled: boolean;
  nudgeInterval: number; // minutes
}
