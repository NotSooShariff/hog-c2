import { useState } from 'react';
import { Plus, Trash2, CheckCircle, Circle, Edit2, X } from 'lucide-react';
import type { Task } from '../types';

interface TaskListProps {
  tasks: Task[];
  onAddTask: (task: Omit<Task, 'id' | 'createdAt'>) => void;
  onUpdateTask: (id: string, updates: Partial<Task>) => void;
  onDeleteTask: (id: string) => void;
}

export default function TaskList({ tasks, onAddTask, onUpdateTask, onDeleteTask }: TaskListProps) {
  const [showNewTask, setShowNewTask] = useState(false);
  const [editingTask, setEditingTask] = useState<string | null>(null);
  const [formData, setFormData] = useState({
    title: '',
    description: '',
    allowedApps: '',
    blockedApps: '',
    allowedSites: '',
    blockedSites: '',
  });

  const resetForm = () => {
    setFormData({
      title: '',
      description: '',
      allowedApps: '',
      blockedApps: '',
      allowedSites: '',
      blockedSites: '',
    });
    setShowNewTask(false);
    setEditingTask(null);
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();

    const taskData = {
      title: formData.title,
      description: formData.description,
      completed: false,
      allowedApps: formData.allowedApps.split(',').map(s => s.trim()).filter(Boolean),
      blockedApps: formData.blockedApps.split(',').map(s => s.trim()).filter(Boolean),
      allowedSites: formData.allowedSites.split(',').map(s => s.trim()).filter(Boolean),
      blockedSites: formData.blockedSites.split(',').map(s => s.trim()).filter(Boolean),
    };

    if (editingTask) {
      onUpdateTask(editingTask, taskData);
    } else {
      onAddTask(taskData);
    }
    resetForm();
  };

  const handleEdit = (task: Task) => {
    setEditingTask(task.id);
    setFormData({
      title: task.title,
      description: task.description,
      allowedApps: task.allowedApps.join(', '),
      blockedApps: task.blockedApps.join(', '),
      allowedSites: task.allowedSites.join(', '),
      blockedSites: task.blockedSites.join(', '),
    });
    setShowNewTask(true);
  };

  const activeTasks = tasks.filter(t => !t.completed);
  const completedTasks = tasks.filter(t => t.completed);

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-3xl font-bold text-gray-800 mb-2">Tasks</h2>
          <p className="text-gray-600">Manage your productivity goals</p>
        </div>
        <button
          onClick={() => setShowNewTask(!showNewTask)}
          className="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors"
        >
          <Plus className="w-5 h-5" />
          New Task
        </button>
      </div>

      {/* New Task Form */}
      {showNewTask && (
        <div className="bg-white rounded-lg shadow p-6 border border-gray-200">
          <div className="flex justify-between items-center mb-4">
            <h3 className="text-xl font-bold text-gray-800">
              {editingTask ? 'Edit Task' : 'Create New Task'}
            </h3>
            <button
              onClick={resetForm}
              className="text-gray-500 hover:text-gray-700 hover:bg-gray-100 p-2 rounded-lg transition-colors"
            >
              <X className="w-5 h-5" />
            </button>
          </div>
          <form onSubmit={handleSubmit} className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Title
              </label>
              <input
                type="text"
                value={formData.title}
                onChange={(e) => setFormData({ ...formData, title: e.target.value })}
                className="w-full px-4 py-2 border border-gray-300 rounded-lg bg-white text-gray-800 focus:ring-2 focus:ring-blue-500 focus:border-transparent focus:outline-none"
                required
                placeholder="e.g., Write blog post"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Description
              </label>
              <textarea
                value={formData.description}
                onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                className="w-full px-4 py-2 border border-gray-300 rounded-lg bg-white text-gray-800 focus:ring-2 focus:ring-blue-500 focus:border-transparent focus:outline-none"
                rows={3}
                placeholder="Describe your task..."
              />
            </div>

            <div className="grid grid-cols-2 gap-3">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Allowed Apps (comma-separated)
                </label>
                <input
                  type="text"
                  value={formData.allowedApps}
                  onChange={(e) => setFormData({ ...formData, allowedApps: e.target.value })}
                  className="w-full px-4 py-2 border border-gray-300 rounded-lg bg-white text-gray-800 focus:ring-2 focus:ring-blue-500 focus:border-transparent focus:outline-none"
                  placeholder="e.g., code.exe, notion.exe"
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Blocked Apps (comma-separated)
                </label>
                <input
                  type="text"
                  value={formData.blockedApps}
                  onChange={(e) => setFormData({ ...formData, blockedApps: e.target.value })}
                  className="w-full px-4 py-2 border border-gray-300 rounded-lg bg-white text-gray-800 focus:ring-2 focus:ring-blue-500 focus:border-transparent focus:outline-none"
                  placeholder="e.g., chrome.exe, discord.exe"
                />
              </div>
            </div>

            <div className="grid grid-cols-2 gap-3">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Allowed Sites (comma-separated)
                </label>
                <input
                  type="text"
                  value={formData.allowedSites}
                  onChange={(e) => setFormData({ ...formData, allowedSites: e.target.value })}
                  className="w-full px-4 py-2 border border-gray-300 rounded-lg bg-white text-gray-800 focus:ring-2 focus:ring-blue-500 focus:border-transparent focus:outline-none"
                  placeholder="e.g., github.com, stackoverflow.com"
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Blocked Sites (comma-separated)
                </label>
                <input
                  type="text"
                  value={formData.blockedSites}
                  onChange={(e) => setFormData({ ...formData, blockedSites: e.target.value })}
                  className="w-full px-4 py-2 border border-gray-300 rounded-lg bg-white text-gray-800 focus:ring-2 focus:ring-blue-500 focus:border-transparent focus:outline-none"
                  placeholder="e.g., youtube.com, facebook.com"
                />
              </div>
            </div>

            <div className="flex gap-3">
              <button
                type="submit"
                className="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors font-medium"
              >
                {editingTask ? 'Update Task' : 'Create Task'}
              </button>
              <button
                type="button"
                onClick={resetForm}
                className="px-4 py-2 bg-gray-200 hover:bg-gray-300 text-gray-800 rounded-lg transition-colors font-medium"
              >
                Cancel
              </button>
            </div>
          </form>
        </div>
      )}

      {/* Active Tasks */}
      <div className="bg-white rounded-lg shadow p-6 border border-gray-200">
        <h3 className="text-xl font-bold text-gray-800 mb-4">Active Tasks</h3>
        {activeTasks.length === 0 ? (
          <p className="text-gray-500 text-center py-8">
            No active tasks. Create one to get started!
          </p>
        ) : (
          <div className="space-y-3">
            {activeTasks.map((task) => (
              <div
                key={task.id}
                className="p-4 bg-gray-50 rounded-lg border border-gray-200"
              >
                <div className="flex items-start gap-3">
                  <button
                    onClick={() => onUpdateTask(task.id, { completed: true })}
                    className="mt-1 text-gray-400 hover:text-green-500 transition-colors"
                  >
                    <Circle className="w-5 h-5" />
                  </button>
                  <div className="flex-1">
                    <h4 className="font-semibold text-gray-800 mb-1">{task.title}</h4>
                    {task.description && (
                      <p className="text-sm text-gray-600 mb-2">{task.description}</p>
                    )}
                    <div className="flex flex-wrap gap-2">
                      {task.allowedApps.length > 0 && (
                        <span className="text-xs px-2 py-1 bg-green-100 text-green-700 rounded">
                          Allowed: {task.allowedApps.join(', ')}
                        </span>
                      )}
                      {task.blockedApps.length > 0 && (
                        <span className="text-xs px-2 py-1 bg-red-100 text-red-700 rounded">
                          Blocked: {task.blockedApps.join(', ')}
                        </span>
                      )}
                    </div>
                  </div>
                  <div className="flex gap-2">
                    <button
                      onClick={() => handleEdit(task)}
                      className="text-gray-500 hover:text-blue-500 transition-colors"
                    >
                      <Edit2 className="w-5 h-5" />
                    </button>
                    <button
                      onClick={() => onDeleteTask(task.id)}
                      className="text-gray-500 hover:text-red-500 transition-colors"
                    >
                      <Trash2 className="w-5 h-5" />
                    </button>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Completed Tasks */}
      {completedTasks.length > 0 && (
        <div className="bg-white rounded-lg shadow p-6 border border-gray-200">
          <h3 className="text-xl font-bold text-gray-800 mb-4">Completed Tasks</h3>
          <div className="space-y-3">
            {completedTasks.map((task) => (
              <div
                key={task.id}
                className="p-4 bg-gray-50 rounded-lg border border-gray-200 opacity-75"
              >
                <div className="flex items-start gap-3">
                  <button
                    onClick={() => onUpdateTask(task.id, { completed: false })}
                    className="mt-1 text-green-500 hover:text-gray-400 transition-colors"
                  >
                    <CheckCircle className="w-5 h-5" />
                  </button>
                  <div className="flex-1">
                    <h4 className="font-semibold text-gray-800 line-through mb-1">
                      {task.title}
                    </h4>
                    {task.description && (
                      <p className="text-sm text-gray-600">{task.description}</p>
                    )}
                  </div>
                  <button
                    onClick={() => onDeleteTask(task.id)}
                    className="text-gray-500 hover:text-red-500 transition-colors"
                  >
                    <Trash2 className="w-5 h-5" />
                  </button>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
