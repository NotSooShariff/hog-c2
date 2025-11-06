import { useState, useEffect } from 'react';
import { Clock, ListTodo, Settings as SettingsIcon, BarChart3 } from 'lucide-react';
import Dashboard from './components/Dashboard';
import TaskList from './components/TaskList';
import Settings from './components/Settings';
import type { Task, AppLimit, Settings as SettingsType } from './types';
import { load, type Store } from '@tauri-apps/plugin-store';
import { registerClientWithNotion, updateAppLimits } from './utils/tauri';

type Tab = 'dashboard' | 'tasks' | 'settings';

function App() {
  const [activeTab, setActiveTab] = useState<Tab>('dashboard');
  const [tasks, setTasks] = useState<Task[]>([]);
  const [settings, setSettings] = useState<SettingsType>({
    limits: [],
    enableNotifications: true,
    trackingEnabled: true,
    nudgeInterval: 30,
  });
  const [store, setStore] = useState<Store | null>(null);

  // Initialize store
  useEffect(() => {
    const initStore = async () => {
      const storeInstance = await load('productivity-tracker.json');
      setStore(storeInstance);
    };
    initStore();
  }, []);

  // Register client with Notion on startup
  useEffect(() => {
    const registerClient = async () => {
      try {
        const result = await registerClientWithNotion();
        console.log('Notion registration:', result);
      } catch (error) {
        console.error('Failed to register with Notion:', error);
      }
    };
    registerClient();
  }, []);

  // Load data from store when it's ready
  useEffect(() => {
    if (!store) return;

    const loadData = async () => {
      const savedTasks = await store.get<Task[]>('tasks');
      const savedSettings = await store.get<SettingsType>('settings');

      if (savedTasks) setTasks(savedTasks);
      if (savedSettings) setSettings(savedSettings);
    };
    loadData();
  }, [store]);

  // Save tasks when they change
  useEffect(() => {
    if (!store) return;
    store.set('tasks', tasks);
    store.save();
  }, [tasks, store]);

  // Save settings when they change
  useEffect(() => {
    if (!store) return;
    store.set('settings', settings);
    store.save();
  }, [settings, store]);

  // Sync limits with backend whenever they change
  useEffect(() => {
    const syncLimits = async () => {
      try {
        await updateAppLimits(settings.limits);
        console.log('Synced limits with backend:', settings.limits.length);
      } catch (error) {
        console.error('Failed to sync limits with backend:', error);
      }
    };
    syncLimits();
  }, [settings.limits]);

  const addTask = (task: Omit<Task, 'id' | 'createdAt'>) => {
    const newTask: Task = {
      ...task,
      id: crypto.randomUUID(),
      createdAt: new Date().toISOString(),
    };
    setTasks([...tasks, newTask]);
  };

  const updateTask = (id: string, updates: Partial<Task>) => {
    setTasks(tasks.map(task =>
      task.id === id ? { ...task, ...updates } : task
    ));
  };

  const deleteTask = (id: string) => {
    setTasks(tasks.filter(task => task.id !== id));
  };

  const addLimit = (limit: AppLimit) => {
    setSettings({
      ...settings,
      limits: [...settings.limits, limit],
    });
  };

  const updateLimit = (appName: string, updates: Partial<AppLimit>) => {
    setSettings({
      ...settings,
      limits: settings.limits.map(limit =>
        limit.app_name === appName ? { ...limit, ...updates } : limit
      ),
    });
  };

  const deleteLimit = (appName: string) => {
    setSettings({
      ...settings,
      limits: settings.limits.filter(limit => limit.app_name !== appName),
    });
  };

  return (
    <div className="min-h-screen bg-white">
      <div className="flex h-screen">
        {/* Sidebar */}
        <aside className="w-64 bg-gray-50 border-r border-gray-200">
          <div className="p-6">
            <div className="flex items-center gap-3 mb-8">
              <div className="p-2 bg-blue-600 rounded-lg">
                <BarChart3 className="w-6 h-6 text-white" />
              </div>
              <div>
                <h1 className="text-xl font-semibold text-gray-900">
                  FocusForge
                </h1>
                <p className="text-xs text-gray-500">Productivity Tracker</p>
              </div>
            </div>
            <nav className="space-y-1">
              <button
                onClick={() => setActiveTab('dashboard')}
                className={`w-full flex items-center gap-3 px-4 py-3 rounded-lg transition-colors ${
                  activeTab === 'dashboard'
                    ? 'bg-blue-600 text-white'
                    : 'text-gray-700 hover:bg-gray-100'
                }`}
              >
                <Clock className="w-5 h-5" />
                <span className="font-medium">Dashboard</span>
              </button>
              <button
                onClick={() => setActiveTab('tasks')}
                className={`w-full flex items-center gap-3 px-4 py-3 rounded-lg transition-colors ${
                  activeTab === 'tasks'
                    ? 'bg-blue-600 text-white'
                    : 'text-gray-700 hover:bg-gray-100'
                }`}
              >
                <ListTodo className="w-5 h-5" />
                <span className="font-medium">Tasks</span>
              </button>
              <button
                onClick={() => setActiveTab('settings')}
                className={`w-full flex items-center gap-3 px-4 py-3 rounded-lg transition-colors ${
                  activeTab === 'settings'
                    ? 'bg-blue-600 text-white'
                    : 'text-gray-700 hover:bg-gray-100'
                }`}
              >
                <SettingsIcon className="w-5 h-5" />
                <span className="font-medium">Settings</span>
              </button>
            </nav>
          </div>
        </aside>

        {/* Main Content */}
        <main className="flex-1 overflow-auto bg-white">
          <div className="p-8">
            {activeTab === 'dashboard' && <Dashboard settings={settings} tasks={tasks} />}
            {activeTab === 'tasks' && (
              <TaskList
                tasks={tasks}
                onAddTask={addTask}
                onUpdateTask={updateTask}
                onDeleteTask={deleteTask}
              />
            )}
            {activeTab === 'settings' && (
              <Settings
                settings={settings}
                onUpdateSettings={setSettings}
                onAddLimit={addLimit}
                onUpdateLimit={updateLimit}
                onDeleteLimit={deleteLimit}
              />
            )}
          </div>
        </main>
      </div>
    </div>
  );
}

export default App;
