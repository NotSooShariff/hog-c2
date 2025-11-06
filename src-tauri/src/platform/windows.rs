#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId,
};
#[cfg(target_os = "windows")]
use windows::Win32::System::Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION};

/// Get information about the currently active window on Windows
#[cfg(target_os = "windows")]
pub fn get_active_window_info() -> Option<(String, String)> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return None;
        }

        // Get window title
        let mut title_buf = [0u16; 512];
        let title_len = GetWindowTextW(hwnd, &mut title_buf);
        let window_title = if title_len > 0 {
            String::from_utf16_lossy(&title_buf[..title_len as usize])
        } else {
            String::new()
        };

        // Get process name
        let mut process_id: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id as *mut u32));

        if process_id == 0 {
            return Some((String::from("Unknown"), window_title));
        }

        let process_handle = match OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION,
            false,
            process_id,
        ) {
            Ok(handle) => handle,
            Err(_) => return Some((String::from("Unknown"), window_title)),
        };

        let mut exe_path = [0u16; 512];
        let mut size = exe_path.len() as u32;

        match QueryFullProcessImageNameW(
            process_handle,
            Default::default(),
            windows::core::PWSTR(exe_path.as_mut_ptr()),
            &mut size,
        ) {
            Ok(_) => {
                let path = String::from_utf16_lossy(&exe_path[..size as usize]);
                let app_name = path.split('\\').last().unwrap_or("Unknown").to_string();
                Some((app_name, window_title))
            }
            Err(_) => Some((String::from("Unknown"), window_title)),
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn get_active_window_info() -> Option<(String, String)> {
    None
}
