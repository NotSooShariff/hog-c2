import { Minimize, Maximize, X } from 'lucide-react';
import { getCurrentWindow } from '@tauri-apps/api/window';

export default function TitleBar() {
  const appWindow = getCurrentWindow();

  const minimizeWindow = () => {
    appWindow.minimize();
  };

  const maximizeWindow = () => {
    appWindow.toggleMaximize();
  };

  const closeWindow = () => {
    appWindow.hide();
  };

  return (
    <div
      data-tauri-drag-region
      className="fixed top-0 left-0 right-0 h-12 bg-gradient-to-r from-blue-600/10 via-purple-600/10 to-pink-600/10 backdrop-blur-xl border-b border-gray-200/20 dark:border-gray-700/20 flex items-center justify-between px-4 z-50 select-none"
    >
      {/* Logo and Title */}
      <div className="flex items-center gap-3" data-tauri-drag-region>
        <img src="/logo.svg" alt="FocusForge" className="w-7 h-7" />
        <span className="font-bold text-lg bg-gradient-to-r from-blue-600 via-purple-600 to-pink-600 bg-clip-text text-transparent">
          FocusForge
        </span>
      </div>

      {/* Window Controls */}
      <div className="flex items-center gap-1">
        <button
          onClick={minimizeWindow}
          className="w-10 h-10 rounded-lg hover:bg-gray-200/50 dark:hover:bg-gray-700/50 flex items-center justify-center transition-colors group"
          aria-label="Minimize"
        >
          <Minimize className="w-4 h-4 text-gray-600 dark:text-gray-400 group-hover:text-gray-900 dark:group-hover:text-white transition-colors" />
        </button>
        <button
          onClick={maximizeWindow}
          className="w-10 h-10 rounded-lg hover:bg-gray-200/50 dark:hover:bg-gray-700/50 flex items-center justify-center transition-colors group"
          aria-label="Maximize"
        >
          <Maximize className="w-4 h-4 text-gray-600 dark:text-gray-400 group-hover:text-gray-900 dark:group-hover:text-white transition-colors" />
        </button>
        <button
          onClick={closeWindow}
          className="w-10 h-10 rounded-lg hover:bg-red-500/20 dark:hover:bg-red-500/20 flex items-center justify-center transition-colors group"
          aria-label="Close"
        >
          <X className="w-4 h-4 text-gray-600 dark:text-gray-400 group-hover:text-red-600 dark:group-hover:text-red-400 transition-colors" />
        </button>
      </div>
    </div>
  );
}
