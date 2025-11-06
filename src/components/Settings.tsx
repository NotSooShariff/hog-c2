import { useState } from 'react';
import { Plus, Trash2, Bell, Clock, X } from 'lucide-react';
import type { AppLimit, Settings as SettingsType } from '../types';

interface SettingsProps {
  settings: SettingsType;
  onUpdateSettings: (settings: SettingsType) => void;
  onAddLimit: (limit: AppLimit) => void;
  onUpdateLimit: (appName: string, updates: Partial<AppLimit>) => void;
  onDeleteLimit: (appName: string) => void;
}

export default function Settings({
  settings,
  onUpdateSettings,
  onAddLimit,
  onUpdateLimit,
  onDeleteLimit,
}: SettingsProps) {
  const [showNewLimit, setShowNewLimit] = useState(false);
  const [limitForm, setLimitForm] = useState({
    appName: '',
    maxDuration: '',
    threshold: '',
  });

  const handleAddLimit = (e: React.FormEvent) => {
    e.preventDefault();
    onAddLimit({
      app_name: limitForm.appName,
      max_duration_minutes: parseInt(limitForm.maxDuration),
      notification_threshold_minutes: parseInt(limitForm.threshold),
      enabled: true,
    });
    setLimitForm({ appName: '', maxDuration: '', threshold: '' });
    setShowNewLimit(false);
  };

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-3xl font-bold text-gray-800 mb-2">Settings</h2>
        <p className="text-gray-600">Configure your productivity preferences</p>
      </div>

      {/* General Settings */}
      <div className="bg-white rounded-lg shadow p-6 border border-gray-200">
        <h3 className="text-xl font-bold text-gray-800 mb-4">General Settings</h3>
        <div className="space-y-4">
          <div className="flex items-center justify-between p-4 bg-gray-50 rounded-lg">
            <div className="flex items-center gap-3">
              <Bell className="w-5 h-5 text-blue-600" />
              <div>
                <p className="font-medium text-gray-800">Enable Notifications</p>
                <p className="text-sm text-gray-600">
                  Get gentle reminders when you exceed time limits
                </p>
              </div>
            </div>
            <button
              onClick={() =>
                onUpdateSettings({ ...settings, enableNotifications: !settings.enableNotifications })
              }
              className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
                settings.enableNotifications ? 'bg-blue-600' : 'bg-gray-300'
              }`}
            >
              <span
                className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                  settings.enableNotifications ? 'translate-x-6' : 'translate-x-1'
                }`}
              />
            </button>
          </div>

          <div className="flex items-center justify-between p-4 bg-gray-50 rounded-lg">
            <div className="flex items-center gap-3">
              <Clock className="w-5 h-5 text-blue-600" />
              <div>
                <p className="font-medium text-gray-800">Enable Tracking</p>
                <p className="text-sm text-gray-600">
                  Track time spent on applications
                </p>
              </div>
            </div>
            <button
              onClick={() =>
                onUpdateSettings({ ...settings, trackingEnabled: !settings.trackingEnabled })
              }
              className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
                settings.trackingEnabled ? 'bg-blue-600' : 'bg-gray-300'
              }`}
            >
              <span
                className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                  settings.trackingEnabled ? 'translate-x-6' : 'translate-x-1'
                }`}
              />
            </button>
          </div>

          <div className="p-4 bg-gray-50 rounded-lg">
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Nudge Interval (minutes)
            </label>
            <p className="text-xs text-gray-500 mb-2">
              How often to send reminders when you exceed limits
            </p>
            <input
              type="number"
              value={settings.nudgeInterval}
              onChange={(e) =>
                onUpdateSettings({ ...settings, nudgeInterval: parseInt(e.target.value) || 30 })
              }
              min="1"
              max="120"
              className="w-full px-4 py-2 border border-gray-300 rounded-lg bg-white text-gray-800 focus:ring-2 focus:ring-blue-500 focus:outline-none"
            />
          </div>
        </div>
      </div>

      {/* App Limits */}
      <div className="bg-white rounded-lg shadow p-6 border border-gray-200">
        <div className="flex justify-between items-center mb-4">
          <h3 className="text-xl font-bold text-gray-800">Application Limits</h3>
          <button
            onClick={() => setShowNewLimit(!showNewLimit)}
            className="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors"
          >
            <Plus className="w-5 h-5" />
            Add Limit
          </button>
        </div>

        {showNewLimit && (
          <form onSubmit={handleAddLimit} className="mb-6 p-4 bg-gray-50 rounded-lg">
            <div className="flex justify-between items-center mb-4">
              <h4 className="font-medium text-gray-800">New App Limit</h4>
              <button
                type="button"
                onClick={() => setShowNewLimit(false)}
                className="text-gray-500 hover:text-gray-700"
              >
                <X className="w-5 h-5" />
              </button>
            </div>
            <div className="space-y-3">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Application Name
                </label>
                <input
                  type="text"
                  value={limitForm.appName}
                  onChange={(e) => setLimitForm({ ...limitForm, appName: e.target.value })}
                  placeholder="e.g., chrome.exe, youtube.com"
                  className="w-full px-4 py-2 border border-gray-300 rounded-lg bg-white text-gray-800 focus:ring-2 focus:ring-blue-500 focus:outline-none"
                  required
                />
              </div>
              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Max Duration (minutes)
                  </label>
                  <input
                    type="number"
                    value={limitForm.maxDuration}
                    onChange={(e) => setLimitForm({ ...limitForm, maxDuration: e.target.value })}
                    placeholder="60"
                    min="1"
                    className="w-full px-4 py-2 border border-gray-300 rounded-lg bg-white text-gray-800 focus:ring-2 focus:ring-blue-500 focus:outline-none"
                    required
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Alert At (minutes)
                  </label>
                  <input
                    type="number"
                    value={limitForm.threshold}
                    onChange={(e) => setLimitForm({ ...limitForm, threshold: e.target.value })}
                    placeholder="45"
                    min="1"
                    className="w-full px-4 py-2 border border-gray-300 rounded-lg bg-white text-gray-800 focus:ring-2 focus:ring-blue-500 focus:outline-none"
                    required
                  />
                </div>
              </div>
              <button
                type="submit"
                className="w-full px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors"
              >
                Add Limit
              </button>
            </div>
          </form>
        )}

        <div className="space-y-3">
          {settings.limits.length === 0 ? (
            <p className="text-gray-500 text-center py-8">
              No app limits configured. Add one to get gentle nudges!
            </p>
          ) : (
            settings.limits.map((limit) => (
              <div
                key={limit.app_name}
                className="p-4 bg-gray-50 rounded-lg"
              >
                <div className="flex items-center justify-between">
                  <div className="flex-1">
                    <h4 className="font-semibold text-gray-800 mb-1">
                      {limit.app_name}
                    </h4>
                    <p className="text-sm text-gray-600">
                      Max: {limit.max_duration_minutes} min • Alert at: {limit.notification_threshold_minutes} min
                    </p>
                  </div>
                  <div className="flex items-center gap-3">
                    <button
                      onClick={() => onUpdateLimit(limit.app_name, { enabled: !limit.enabled })}
                      className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
                        limit.enabled ? 'bg-blue-600' : 'bg-gray-300'
                      }`}
                    >
                      <span
                        className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                          limit.enabled ? 'translate-x-6' : 'translate-x-1'
                        }`}
                      />
                    </button>
                    <button
                      onClick={() => onDeleteLimit(limit.app_name)}
                      className="text-gray-500 hover:text-red-500 transition-colors"
                    >
                      <Trash2 className="w-5 h-5" />
                    </button>
                  </div>
                </div>
              </div>
            ))
          )}
        </div>
      </div>

      {/* Info Section */}
      <div className="bg-blue-50 rounded-lg p-6 border border-blue-200">
        <h4 className="font-semibold text-blue-900 mb-2">Tips for Better Productivity</h4>
        <ul className="space-y-2 text-sm text-blue-800">
          <li>• Set realistic time limits for distracting apps like social media</li>
          <li>• Use task-based tracking to associate apps with specific goals</li>
          <li>• Review your daily statistics to identify time-wasting patterns</li>
          <li>• Take regular breaks to maintain focus and avoid burnout</li>
        </ul>
      </div>
    </div>
  );
}
