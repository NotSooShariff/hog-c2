// Platform-specific implementations for window tracking

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "macos")]
pub mod macos;

mod process_control;

// Export unified interface
#[cfg(target_os = "windows")]
pub fn get_active_window_info() -> Option<(String, String)> {
    windows::get_active_window_info()
}

#[cfg(target_os = "linux")]
pub fn get_active_window_info() -> Option<(String, String)> {
    linux::get_active_window_info()
}

#[cfg(target_os = "macos")]
pub fn get_active_window_info() -> Option<(String, String)> {
    macos::get_active_window_info()
}

// Fallback for unsupported platforms
#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
pub fn get_active_window_info() -> Option<(String, String)> {
    None
}

// Export process control
pub use process_control::terminate_process_by_name;
