#[cfg(target_os = "windows")]
use windows::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};
#[cfg(target_os = "windows")]
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};

/// Terminate a process by name on Windows
#[cfg(target_os = "windows")]
pub fn terminate_process_by_name(process_name: &str) -> Result<(), String> {
    unsafe {
        // Create snapshot of all processes
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)
            .map_err(|e| format!("Failed to create process snapshot: {}", e))?;

        let mut process_entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };

        // Get first process
        if Process32FirstW(snapshot, &mut process_entry).is_err() {
            return Err("Failed to get first process".to_string());
        }

        let mut terminated_count = 0;

        // Iterate through processes
        loop {
            // Convert process name from wide string
            let current_name = String::from_utf16_lossy(
                &process_entry.szExeFile
                    .iter()
                    .take_while(|&&c| c != 0)
                    .copied()
                    .collect::<Vec<u16>>(),
            );

            // Check if this is the target process
            if current_name.eq_ignore_ascii_case(process_name) {
                log::info!("Found process {} with PID {}", current_name, process_entry.th32ProcessID);

                // Open the process with terminate permission
                match OpenProcess(PROCESS_TERMINATE, false, process_entry.th32ProcessID) {
                    Ok(handle) => {
                        match TerminateProcess(handle, 1) {
                            Ok(_) => {
                                log::info!("Successfully terminated process {} (PID: {})", current_name, process_entry.th32ProcessID);
                                terminated_count += 1;
                            }
                            Err(e) => {
                                log::warn!("Failed to terminate process {}: {}", current_name, e);
                            }
                        }
                    }
                    Err(e) => {
                        log::warn!("Failed to open process {}: {}", current_name, e);
                    }
                }
            }

            // Move to next process
            if Process32NextW(snapshot, &mut process_entry).is_err() {
                break;
            }
        }

        if terminated_count > 0 {
            Ok(())
        } else {
            Err(format!("Process '{}' not found", process_name))
        }
    }
}

/// Terminate a process by name (no-op on non-Windows platforms)
#[cfg(not(target_os = "windows"))]
pub fn terminate_process_by_name(_process_name: &str) -> Result<(), String> {
    Err("Process termination not supported on this platform".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "windows")]
    fn test_terminate_nonexistent_process() {
        let result = terminate_process_by_name("nonexistent_app_12345.exe");
        assert!(result.is_err());
    }
}
