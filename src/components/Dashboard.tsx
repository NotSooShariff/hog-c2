import { useState, useEffect } from 'react';
import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, PieChart, Pie, Cell } from 'recharts';
import { Clock, Activity, AlertCircle } from 'lucide-react';
import type { AppUsage, Settings, Task } from '../types';
import { getAppStats, formatDuration, showNotification } from '../utils/tauri';

interface DashboardProps {
  settings: Settings;
  tasks: Task[];
}

const COLORS = ['#0ea5e9', '#8b5cf6', '#10b981', '#f59e0b', '#ef4444', '#ec4899'];

export default function Dashboard({ settings, tasks }: DashboardProps) {
  const [appStats, setAppStats] = useState<AppUsage[]>([]);
  const [lastNotified, setLastNotified] = useState<Record<string, number>>({});

  useEffect(() => {
    const fetchStats = async () => {
      const stats = await getAppStats();
      setAppStats(stats.sort((a, b) => b.duration_seconds - a.duration_seconds));
    };

    fetchStats();
    const interval = setInterval(fetchStats, 2000);

    return () => clearInterval(interval);
  }, []);

  // Check for limits and send notifications
  useEffect(() => {
    if (!settings.enableNotifications) return;

    const now = Date.now();
    appStats.forEach(stat => {
      const limit = settings.limits.find(l => l.app_name === stat.app_name && l.enabled);
      if (!limit) return;

      const durationMinutes = stat.duration_seconds / 60;
      const timeSinceLastNotif = (now - (lastNotified[stat.app_name] || 0)) / 1000 / 60;

      if (
        durationMinutes >= limit.notification_threshold_minutes &&
        timeSinceLastNotif >= settings.nudgeInterval
      ) {
        showNotification(
          'Productivity Reminder',
          `You've been on ${stat.app_name} for ${formatDuration(stat.duration_seconds)}. Time for a break?`
        );
        setLastNotified({ ...lastNotified, [stat.app_name]: now });
      }
    });
  }, [appStats, settings, lastNotified]);

  const totalTime = appStats.reduce((sum, app) => sum + app.duration_seconds, 0);
  const chartData = appStats.slice(0, 6).map(app => ({
    name: app.app_name.replace('.exe', ''),
    duration: app.duration_seconds / 60,
  }));

  const pieData = appStats.slice(0, 5).map((app, idx) => ({
    name: app.app_name.replace('.exe', ''),
    value: app.duration_seconds,
    color: COLORS[idx % COLORS.length],
  }));

  const activeTasks = tasks.filter(t => !t.completed);

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-3xl font-bold text-gray-800 mb-2">Dashboard</h2>
        <p className="text-gray-600">Track your productivity and time usage</p>
      </div>

      {/* Stats Cards */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div className="bg-white rounded-lg shadow p-6 border border-gray-200">
          <div className="flex items-center gap-4">
            <div className="p-3 bg-blue-600 rounded-lg">
              <Clock className="w-6 h-6 text-white" />
            </div>
            <div>
              <p className="text-sm font-medium text-gray-600 mb-1">Total Time</p>
              <p className="text-2xl font-bold text-gray-800">{formatDuration(totalTime)}</p>
            </div>
          </div>
        </div>

        <div className="bg-white rounded-lg shadow p-6 border border-gray-200">
          <div className="flex items-center gap-4">
            <div className="p-3 bg-green-600 rounded-lg">
              <Activity className="w-6 h-6 text-white" />
            </div>
            <div>
              <p className="text-sm font-medium text-gray-600 mb-1">Apps Tracked</p>
              <p className="text-2xl font-bold text-gray-800">{appStats.length}</p>
            </div>
          </div>
        </div>

        <div className="bg-white rounded-lg shadow p-6 border border-gray-200">
          <div className="flex items-center gap-4">
            <div className="p-3 bg-purple-600 rounded-lg">
              <AlertCircle className="w-6 h-6 text-white" />
            </div>
            <div>
              <p className="text-sm font-medium text-gray-600 mb-1">Active Tasks</p>
              <p className="text-2xl font-bold text-gray-800">{activeTasks.length}</p>
            </div>
          </div>
        </div>
      </div>

      {/* Charts */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <div className="bg-white rounded-lg shadow p-6 border border-gray-200">
          <h3 className="text-xl font-bold text-gray-800 mb-4">Time by Application</h3>
          <ResponsiveContainer width="100%" height={300}>
            <BarChart data={chartData}>
              <CartesianGrid strokeDasharray="3 3" stroke="#e5e7eb" />
              <XAxis dataKey="name" stroke="#6b7280" />
              <YAxis stroke="#6b7280" label={{ value: 'Minutes', angle: -90, position: 'insideLeft' }} />
              <Tooltip
                contentStyle={{ backgroundColor: 'white', border: '1px solid #e5e7eb', borderRadius: '8px', padding: '8px' }}
                formatter={(value: number) => [`${value.toFixed(1)} min`, 'Duration']}
              />
              <Bar dataKey="duration" fill="#3b82f6" radius={[8, 8, 0, 0]} />
            </BarChart>
          </ResponsiveContainer>
        </div>

        <div className="bg-white rounded-lg shadow p-6 border border-gray-200">
          <h3 className="text-xl font-bold text-gray-800 mb-4">Usage Distribution</h3>
          <ResponsiveContainer width="100%" height={300}>
            <PieChart>
              <Pie
                data={pieData}
                cx="50%"
                cy="50%"
                labelLine={false}
                label
                outerRadius={100}
                fill="#8884d8"
                dataKey="value"
              >
                {pieData.map((entry, index) => (
                  <Cell key={`cell-${index}`} fill={entry.color} />
                ))}
              </Pie>
              <Tooltip
                formatter={(value) => formatDuration(value as number)}
                contentStyle={{ backgroundColor: 'white', border: '1px solid #e5e7eb', borderRadius: '8px', padding: '8px' }}
              />
            </PieChart>
          </ResponsiveContainer>
        </div>
      </div>

      {/* App List */}
      <div className="bg-white rounded-lg shadow p-6 border border-gray-200">
        <h3 className="text-xl font-bold text-gray-800 mb-4">Application Usage</h3>
        <div className="space-y-3">
          {appStats.length === 0 ? (
            <p className="text-gray-500 text-center py-8">
              No activity tracked yet. Start using apps to see statistics.
            </p>
          ) : (
            appStats.map((app, idx) => {
              const limit = settings.limits.find(l => l.app_name === app.app_name && l.enabled);
              const percentage = limit
                ? Math.min((app.duration_seconds / 60 / limit.max_duration_minutes) * 100, 100)
                : 0;
              const isOverLimit = limit && app.duration_seconds / 60 >= limit.max_duration_minutes;

              return (
                <div key={idx} className="p-4 bg-gray-50 rounded-lg border border-gray-200">
                  <div className="flex justify-between items-center mb-2">
                    <span className="font-semibold text-gray-800">{app.app_name}</span>
                    <span className={`font-bold ${isOverLimit ? 'text-red-500' : 'text-blue-600'}`}>
                      {formatDuration(app.duration_seconds)}
                    </span>
                  </div>
                  <p className="text-sm text-gray-600 truncate mb-2">{app.window_title}</p>
                  {limit && (
                    <div className="mt-2">
                      <div className="w-full bg-gray-200 rounded-full h-2 overflow-hidden">
                        <div
                          className={`h-2 rounded-full transition-all ${
                            isOverLimit ? 'bg-red-500' : 'bg-blue-600'
                          }`}
                          style={{ width: `${percentage}%` }}
                        />
                      </div>
                      <p className="text-xs font-medium text-gray-500 mt-1">
                        Limit: {limit.max_duration_minutes} minutes
                      </p>
                    </div>
                  )}
                </div>
              );
            })
          )}
        </div>
      </div>
    </div>
  );
}
